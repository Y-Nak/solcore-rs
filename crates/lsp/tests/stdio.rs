#![cfg(feature = "native")]

//! Native protocol fixtures exercise the actual server binary, filesystem loading,
//! and workspace notifications. Sources live beside each case as `.sol.txt` files.
//! Request parameters and expectations use `$uri:path` for temporary file URIs.

use std::{
    collections::BTreeMap,
    fs,
    io::{self, BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use dir_test::{Fixture, dir_test};
use lsp_types::Url;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    documents: Vec<String>,
    workspace_folders: Vec<String>,
    steps: Vec<Step>,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Step {
    Open {
        document: String,
    },
    Diagnostics {
        document: String,
        count: Option<usize>,
        #[serde(default)]
        contains: Vec<ExpectedDiagnostic>,
    },
    Request {
        method: String,
        params: Value,
        expected: Value,
    },
    /// Use the actual published diagnostic as the editor's code-action context.
    CodeActions {
        document: String,
        expected: Value,
    },
    Notify {
        method: String,
        params: Value,
    },
    RemoveFile {
        document: String,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedDiagnostic {
    severity: Option<u64>,
    message_contains: Option<String>,
}

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new(fixture_root: &Path, documents: &[String], folders: &[String]) -> Self {
        static NEXT_WORKSPACE: AtomicUsize = AtomicUsize::new(0);
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let workspace = Self {
            root: std::env::temp_dir().join(format!(
                "solcore-lsp-stdio-{}-{nonce}-{}",
                std::process::id(),
                NEXT_WORKSPACE.fetch_add(1, Ordering::Relaxed)
            )),
        };
        for folder in folders {
            fs::create_dir_all(workspace.root.join(folder)).expect("create workspace folder");
        }
        for document in documents {
            let target = workspace.root.join(document);
            fs::create_dir_all(target.parent().unwrap()).expect("create document parent");
            fs::copy(fixture_root.join(format!("{document}.txt")), target)
                .expect("copy fixture document");
        }
        workspace
    }

    fn uri(&self, path: &str) -> Url {
        Url::from_file_path(self.root.join(path)).expect("fixture URI")
    }

    fn expand_uris(&self, value: &mut Value) {
        match value {
            Value::String(text) => {
                if let Some(path) = text.strip_prefix("$uri:") {
                    *text = self.uri(path).into();
                }
            }
            Value::Array(values) => {
                for value in values {
                    self.expand_uris(value);
                }
            }
            Value::Object(values) => {
                for value in values.values_mut() {
                    self.expand_uris(value);
                }
            }
            _ => {}
        }
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/native", glob: "**/case.json")]
fn native_stdio_fixture(fixture: Fixture<&str>) {
    let case: Case = serde_json::from_str(fixture.content()).expect("native fixture schema");
    assert!(!case.workspace_folders.is_empty(), "fixture has no roots");
    assert!(!case.steps.is_empty(), "fixture has no steps");
    let fixture_root = Path::new(fixture.path()).parent().unwrap();
    let workspace = TestWorkspace::new(fixture_root, &case.documents, &case.workspace_folders);
    let mut child = Command::new(env!("CARGO_BIN_EXE_solcore-lsp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn solcore-lsp");

    let mut stdin = child.stdin.take().expect("child stdin");
    let stdout = child.stdout.take().expect("child stdout");
    let stderr = child.stderr.take().expect("child stderr");
    let (messages_tx, messages_rx) = mpsc::channel();
    let reader = thread::spawn(move || {
        let mut stdout = BufReader::new(stdout);
        while let Ok(message) = read_message(&mut stdout) {
            if messages_tx.send(message).is_err() {
                break;
            }
        }
    });
    let stderr_reader = thread::spawn(move || {
        let mut output = String::new();
        let _ = BufReader::new(stderr).read_to_string(&mut output);
        output
    });

    let result = run_case(&mut stdin, &messages_rx, &workspace, case);
    let shutdown_result = shutdown_child(&mut child, stdin, &messages_rx);
    if shutdown_result.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    let _ = reader.join();
    let stderr = stderr_reader.join().unwrap_or_default();
    if let Err(error) = result.and(shutdown_result) {
        panic!("{}: {error}\nchild stderr:\n{stderr}", fixture.path());
    }
}

fn run_case(
    stdin: &mut ChildStdin,
    messages_rx: &mpsc::Receiver<Value>,
    workspace: &TestWorkspace,
    case: Case,
) -> Result<(), String> {
    let folders: Vec<_> = case
        .workspace_folders
        .iter()
        .map(|name| json!({ "uri": workspace.uri(name), "name": name }))
        .collect();
    let initialize = request_result(
        stdin,
        messages_rx,
        0,
        "initialize",
        json!({
            "rootUri": folders[0]["uri"],
            "workspaceFolders": folders,
            "capabilities": {}
        }),
    )?;
    if initialize.get("capabilities").is_none() {
        return Err(format!("initialize omitted capabilities: {initialize}"));
    }
    notify(stdin, "initialized", json!({}))?;
    let mut published = BTreeMap::<String, Value>::new();
    for (index, step) in case.steps.into_iter().enumerate() {
        let id = index as i64 + 1;
        match step {
            Step::Open { document } => {
                let text = fs::read_to_string(workspace.root.join(&document))
                    .map_err(|error| format!("read {document}: {error}"))?;
                notify(
                    stdin,
                    "textDocument/didOpen",
                    json!({ "textDocument": {
                        "uri": workspace.uri(&document),
                        "languageId": "solcore", "version": 1, "text": text
                    }}),
                )?;
            }
            Step::Diagnostics {
                document,
                count,
                contains,
            } => {
                let uri = workspace.uri(&document);
                let message = recv_until(messages_rx, |message| {
                    message["method"] == "textDocument/publishDiagnostics"
                        && message["params"]["uri"] == uri.as_str()
                })?;
                let diagnostics = message["params"]["diagnostics"]
                    .as_array()
                    .ok_or_else(|| format!("missing diagnostics: {message}"))?;
                if count.is_some_and(|count| diagnostics.len() != count) {
                    return Err(format!("expected {count:?} diagnostics: {message}"));
                }
                for expected in contains {
                    if !diagnostics.iter().any(|diagnostic| {
                        expected.severity.is_none_or(|severity| {
                            diagnostic["severity"].as_u64() == Some(severity)
                        }) && expected.message_contains.as_ref().is_none_or(|text| {
                            diagnostic["message"]
                                .as_str()
                                .is_some_and(|message| message.contains(text))
                        })
                    }) {
                        return Err(format!("missing expected diagnostic: {message}"));
                    }
                }
                published.insert(document, message["params"]["diagnostics"].clone());
            }
            Step::Request {
                method,
                mut params,
                mut expected,
            } => {
                workspace.expand_uris(&mut params);
                workspace.expand_uris(&mut expected);
                let actual = request_result(stdin, messages_rx, id, &method, params)?;
                expect_result(&method, &actual, &expected)?;
            }
            Step::CodeActions {
                document,
                mut expected,
            } => {
                let diagnostic = published
                    .get(&document)
                    .and_then(|diagnostics| diagnostics.get(0))
                    .ok_or_else(|| format!("no published diagnostic for {document}"))?;
                let actual = request_result(
                    stdin,
                    messages_rx,
                    id,
                    "textDocument/codeAction",
                    json!({
                        "textDocument": { "uri": workspace.uri(&document) },
                        "range": diagnostic["range"],
                        "context": { "diagnostics": [diagnostic] }
                    }),
                )?;
                workspace.expand_uris(&mut expected);
                expect_result("textDocument/codeAction", &actual, &expected)?;
            }
            Step::Notify { method, mut params } => {
                workspace.expand_uris(&mut params);
                notify(stdin, &method, params)?;
            }
            Step::RemoveFile { document } => {
                fs::remove_file(workspace.root.join(&document))
                    .map_err(|error| format!("remove {document}: {error}"))?;
            }
        }
    }
    Ok(())
}

fn expect_result(method: &str, actual: &Value, expected: &Value) -> Result<(), String> {
    if actual != expected {
        return Err(format!(
            "{method} result mismatch\nexpected: {expected}\nactual: {actual}"
        ));
    }
    Ok(())
}

fn notify(stdin: &mut ChildStdin, method: &str, params: Value) -> Result<(), String> {
    send_message(
        stdin,
        &json!({ "jsonrpc": "2.0", "method": method, "params": params }),
    )
}
fn request_result(
    stdin: &mut ChildStdin,
    messages_rx: &mpsc::Receiver<Value>,
    id: i64,
    method: &str,
    params: Value,
) -> Result<Value, String> {
    send_message(
        stdin,
        &json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        }),
    )?;
    let response = recv_until(messages_rx, |message| {
        message.get("id").and_then(Value::as_i64) == Some(id)
    })?;
    response
        .get("result")
        .cloned()
        .ok_or_else(|| format!("{method} response had no result: {response}"))
}

fn shutdown_child(
    child: &mut Child,
    mut stdin: ChildStdin,
    messages_rx: &mpsc::Receiver<Value>,
) -> Result<(), String> {
    send_message(
        &mut stdin,
        &json!({
            "jsonrpc": "2.0",
            "id": i64::MAX,
            "method": "shutdown",
            "params": null
        }),
    )?;

    let _ = recv_until(messages_rx, |message| {
        message.get("id").and_then(Value::as_i64) == Some(i64::MAX)
    })?;

    send_message(
        &mut stdin,
        &json!({
            "jsonrpc": "2.0",
            "method": "exit",
            "params": null
        }),
    )?;
    drop(stdin);

    for _ in 0..20 {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(()),
            Ok(Some(status)) => {
                return Err(format!("solcore-lsp exited with status {status}"));
            }
            Ok(None) => thread::sleep(Duration::from_millis(100)),
            Err(error) => return Err(format!("failed waiting for solcore-lsp: {error}")),
        }
    }

    let _ = child.kill();
    let _ = child.wait();
    Err("solcore-lsp did not exit after shutdown/exit".to_owned())
}

fn recv_until(
    messages_rx: &mpsc::Receiver<Value>,
    mut predicate: impl FnMut(&Value) -> bool,
) -> Result<Value, String> {
    for _ in 0..50 {
        let message = messages_rx
            .recv_timeout(Duration::from_secs(20))
            .map_err(|error| format!("timed out waiting for LSP message: {error}"))?;
        if predicate(&message) {
            return Ok(message);
        }
    }

    Err("did not receive expected LSP message within 50 messages".to_owned())
}

fn send_message(stdin: &mut ChildStdin, message: &Value) -> Result<(), String> {
    let body = message.to_string();
    write!(stdin, "Content-Length: {}\r\n\r\n{body}", body.len())
        .map_err(|error| format!("failed to write LSP message header/body: {error}"))?;
    stdin
        .flush()
        .map_err(|error| format!("failed to flush LSP message: {error}"))
}

fn read_message(reader: &mut impl BufRead) -> io::Result<Value> {
    let mut content_length = None;
    loop {
        let mut line = String::new();
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "stdout closed while reading LSP header",
            ));
        }

        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            break;
        }

        if let Some(value) = line.strip_prefix("Content-Length:") {
            content_length = Some(value.trim().parse::<usize>().map_err(|error| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("invalid content length: {error}"),
                )
            })?);
        }
    }

    let content_length = content_length.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "missing Content-Length in LSP header",
        )
    })?;

    let mut body = vec![0; content_length];
    reader.read_exact(&mut body)?;
    serde_json::from_slice(&body).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}
