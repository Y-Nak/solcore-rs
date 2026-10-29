//! Exercise the same installed compiler entry point used by editor extensions.
use std::{
    io::{BufRead, BufReader, Read, Write},
    process::{Child, Command, Stdio},
    sync::mpsc::{Receiver, channel},
    thread,
    time::{Duration, Instant},
};

use serde_json::{Value, json};

#[test]
fn compiler_exposes_bundled_lsp_and_version() {
    let help = Command::new(env!("CARGO_BIN_EXE_solcore"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("lsp [--stdio]"));

    let version = Command::new(env!("CARGO_BIN_EXE_solcore"))
        .arg("--version")
        .output()
        .unwrap();
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8_lossy(&version.stdout),
        format!("solcore {}\n", env!("CARGO_PKG_VERSION"))
    );

    let lsp_version = Command::new(env!("CARGO_BIN_EXE_solcore"))
        .args(["lsp", "--version"])
        .output()
        .unwrap();
    assert!(lsp_version.status.success());
    assert_eq!(
        String::from_utf8_lossy(&lsp_version.stdout),
        format!("solcore-lsp {}\n", env!("CARGO_PKG_VERSION"))
    );

    let invalid = Command::new(env!("CARGO_BIN_EXE_solcore"))
        .args(["lsp", "--unknown"])
        .output()
        .unwrap();
    assert_eq!(invalid.status.code(), Some(2));
    assert!(invalid.stdout.is_empty());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("solcore lsp --help"));
}

#[test]
fn bundled_lsp_speaks_stdio_without_a_startup_banner() {
    for arguments in [vec!["lsp"], vec!["lsp", "--stdio"]] {
        let mut server = Server::start(&arguments);
        server.send(json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": { "capabilities": {}, "processId": null, "rootUri": null }
        }));
        let initialize = server.response(1);
        assert_eq!(
            initialize["result"]["serverInfo"]["version"],
            env!("CARGO_PKG_VERSION")
        );
        assert_eq!(initialize["result"]["capabilities"]["textDocumentSync"], 1);
        server.send(json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}));
        server.send(json!({"jsonrpc": "2.0", "id": 2, "method": "shutdown"}));
        assert!(server.response(2)["result"].is_null());
        server.send(json!({"jsonrpc": "2.0", "method": "exit"}));
        server.child.stdin.take();
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(status) = server.child.try_wait().unwrap() {
                assert!(status.success(), "language server exit: {status}");
                break;
            }
            assert!(Instant::now() < deadline, "language server did not exit");
            thread::sleep(Duration::from_millis(10));
        }
    }
}

struct Server {
    child: Child,
    messages: Receiver<Value>,
}

impl Server {
    fn start(args: &[&str]) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_solcore"))
            .args(args)
            // The LSP embeds std; it must work outside a development checkout.
            .current_dir(std::env::temp_dir())
            .env_remove("SOLCORE_STD")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let (sender, messages) = channel();
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut header = String::new();
                if reader.read_line(&mut header).unwrap() == 0 {
                    break;
                }
                let length: usize = header
                    .strip_prefix("Content-Length: ")
                    .expect("stdout must contain only LSP frames")
                    .trim()
                    .parse()
                    .unwrap();
                loop {
                    header.clear();
                    assert!(reader.read_line(&mut header).unwrap() > 0);
                    if header == "\r\n" {
                        break;
                    }
                }
                let mut body = vec![0; length];
                reader.read_exact(&mut body).unwrap();
                if sender.send(serde_json::from_slice(&body).unwrap()).is_err() {
                    break;
                }
            }
        });
        Self { child, messages }
    }

    fn send(&mut self, message: Value) {
        let body = serde_json::to_vec(&message).unwrap();
        let input = self.child.stdin.as_mut().unwrap();
        write!(input, "Content-Length: {}\r\n\r\n", body.len()).unwrap();
        input.write_all(&body).unwrap();
        input.flush().unwrap();
    }

    fn response(&self, id: u64) -> Value {
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let message = self.messages.recv_timeout(remaining).expect("LSP response");
            if message["id"] == id {
                assert!(message.get("error").is_none(), "{message}");
                return message;
            }
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
