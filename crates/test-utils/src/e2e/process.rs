//! Owned subprocesses with deadline-independent output capture.
//!
//! Anonymous pipes make EOF depend on every inherited writer, including
//! grandchildren. Temporary files instead let us read a finite snapshot after
//! the child exits or is stopped, without joining blocking reader threads.

use std::{
    io::Read,
    path::Path,
    process::{Child, Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant},
};

use tempfile::NamedTempFile;

const ANVIL_START_TIMEOUT: Duration = Duration::from_secs(15);
const ANVIL_READY_TIMEOUT: Duration = Duration::from_secs(10);

pub struct CommandOutput {
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

struct CapturedProcess {
    child: Child,
    stdout: NamedTempFile,
    stderr: NamedTempFile,
    stopped: bool,
}

impl CapturedProcess {
    fn spawn(command: &mut Command) -> Result<Self, String> {
        let stdout = NamedTempFile::new().map_err(|error| error.to_string())?;
        let stderr = NamedTempFile::new().map_err(|error| error.to_string())?;
        // Reopening, rather than cloning, gives the child its own file cursor.
        // Reading the log must never move the offset used by a concurrent writer.
        command.stdout(Stdio::from(
            stdout.reopen().map_err(|error| error.to_string())?,
        ));
        command.stderr(Stdio::from(
            stderr.reopen().map_err(|error| error.to_string())?,
        ));
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        let child = command.spawn().map_err(|error| error.to_string())?;
        Ok(Self {
            child,
            stdout,
            stderr,
            stopped: false,
        })
    }

    fn stop(&mut self) {
        if self.stopped {
            return;
        }
        #[cfg(unix)]
        {
            // SAFETY: spawn put this child in a new process group whose ID is
            // the child's PID. A negative ID addresses only that owned group.
            unsafe {
                libc::kill(-(self.child.id() as libc::pid_t), libc::SIGKILL);
            }
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.stopped = true;
    }

    fn output(&self) -> Result<(Vec<u8>, Vec<u8>), String> {
        Ok((snapshot(&self.stdout)?, snapshot(&self.stderr)?))
    }

    fn logs(&self) -> String {
        match self.output() {
            Ok((stdout, stderr)) => format!(
                "{}{}",
                String::from_utf8_lossy(&stdout),
                String::from_utf8_lossy(&stderr)
            ),
            Err(error) => format!("failed to read process logs: {error}"),
        }
    }
}

impl Drop for CapturedProcess {
    fn drop(&mut self) {
        self.stop();
    }
}

fn snapshot(file: &NamedTempFile) -> Result<Vec<u8>, String> {
    // Bound the read even if a detached descendant continues writing.
    let file = file.reopen().map_err(|error| error.to_string())?;
    let length = file.metadata().map_err(|error| error.to_string())?.len();
    let mut output = Vec::new();
    file.take(length)
        .read_to_end(&mut output)
        .map_err(|error| error.to_string())?;
    Ok(output)
}

pub fn run_command(
    command: &Path,
    args: &[&str],
    path_args: &[&Path],
    timeout: Duration,
) -> Result<CommandOutput, String> {
    let mut cmd = Command::new(command);
    cmd.args(args).args(path_args);
    let mut process = CapturedProcess::spawn(&mut cmd)
        .map_err(|error| format!("failed to run {}: {error}", command.display()))?;
    let start = Instant::now();
    let status = loop {
        if let Some(status) = process
            .child
            .try_wait()
            .map_err(|error| format!("failed to poll {}: {error}", command.display()))?
        {
            break status;
        }
        if start.elapsed() >= timeout {
            process.stop();
            let (stdout, stderr) = process.output()?;
            return Err(format!(
                "{} timed out after {timeout:?}\nstdout:\n{}\nstderr:\n{}",
                command.display(),
                String::from_utf8_lossy(&stdout),
                String::from_utf8_lossy(&stderr)
            ));
        }
        thread::sleep(Duration::from_millis(10).min(timeout.saturating_sub(start.elapsed())));
    };
    // The command owns any children it launched, even when it exits before them.
    process.stop();
    let (stdout, stderr) = process.output()?;
    Ok(CommandOutput {
        status,
        stdout,
        stderr,
    })
}

pub(super) struct Anvil {
    process: CapturedProcess,
    url: String,
}

impl Anvil {
    pub(super) fn spawn(anvil: &Path, cast: &Path, hardfork: &str) -> Result<Self, String> {
        let mut command = Command::new(anvil);
        command.args(["--host", "127.0.0.1", "--port", "0", "--hardfork", hardfork]);
        let mut process = CapturedProcess::spawn(&mut command)
            .map_err(|error| format!("failed to start {}: {error}", anvil.display()))?;
        let port = wait_for_anvil_port(&mut process, ANVIL_START_TIMEOUT)?;
        let anvil = Self {
            process,
            url: format!("http://127.0.0.1:{port}"),
        };
        anvil.wait_until_ready(cast)?;
        Ok(anvil)
    }

    pub(super) fn url(&self) -> &str {
        &self.url
    }

    pub(super) fn logs(&self) -> String {
        self.process.logs()
    }

    fn wait_until_ready(&self, cast: &Path) -> Result<(), String> {
        let start = Instant::now();
        while start.elapsed() < ANVIL_READY_TIMEOUT {
            let output = run_command(
                cast,
                &["block-number", "--rpc-url", &self.url],
                &[],
                Duration::from_secs(2),
            );
            if output.is_ok_and(|output| output.status.success()) {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(100));
        }
        Err(format!(
            "anvil did not become ready at {}\nlogs:\n{}",
            self.url,
            self.logs()
        ))
    }
}

fn wait_for_anvil_port(process: &mut CapturedProcess, timeout: Duration) -> Result<u16, String> {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if let Some(status) = process
            .child
            .try_wait()
            .map_err(|error| format!("failed to poll anvil: {error}"))?
        {
            return Err(format!(
                "anvil exited before printing a port: {status}\nlogs:\n{}",
                process.logs()
            ));
        }
        let (stdout, stderr) = process.output()?;
        for output in [stdout, stderr] {
            let output = String::from_utf8_lossy(&output);
            // A snapshot can end halfway through the port number. Wait for a
            // complete line, just as a line reader on a pipe would do.
            for line in output
                .split_inclusive('\n')
                .filter(|line| line.ends_with('\n'))
            {
                if let Some(port) = parse_anvil_port(line) {
                    return Ok(port);
                }
            }
        }
        thread::sleep(Duration::from_millis(20));
    }
    Err(format!(
        "anvil did not print a listening port\nlogs:\n{}",
        process.logs()
    ))
}

fn parse_anvil_port(line: &str) -> Option<u16> {
    for marker in ["127.0.0.1:", "localhost:"] {
        let Some(start) = line.find(marker).map(|index| index + marker.len()) else {
            continue;
        };
        let digits = line[start..]
            .chars()
            .take_while(|character| character.is_ascii_digit())
            .collect::<String>();
        if let Ok(port) = digits.parse() {
            return Some(port);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[dir_test::dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/anvil_address", glob: "*.txt")]
    fn parses_anvil_listening_address(fixture: dir_test::Fixture<&str>) {
        let name = std::path::Path::new(fixture.path())
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap();
        let expected = match name {
            "http" => Some(49152),
            "ipv4" => Some(8545),
            "unrelated" => None,
            _ => panic!("missing expected port for {name}"),
        };
        assert_eq!(parse_anvil_port(fixture.content().trim_end()), expected);
    }

    #[cfg(unix)]
    mod unix {
        use super::*;
        use crate::e2e::{E2eFailure, FailureKind, with_evm_harness};
        use std::{fs, os::unix::fs::PermissionsExt, panic::AssertUnwindSafe};

        fn shell(script: &str) -> Command {
            let mut command = Command::new("/bin/sh");
            command.args(["-c", script]);
            command
        }

        fn assert_stopped(pid: u32) {
            for _ in 0..100 {
                // SAFETY: signal zero only tests whether this PID exists.
                if unsafe { libc::kill(pid as libc::pid_t, 0) } == -1
                    && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
                {
                    return;
                }
                // Linux containers do not always reap adopted zombies promptly.
                // A zombie has terminated and cannot keep any writers open.
                #[cfg(target_os = "linux")]
                if fs::read_to_string(format!("/proc/{pid}/stat")).is_ok_and(|stat| {
                    stat.rsplit_once(')')
                        .is_some_and(|(_, fields)| fields.split_whitespace().next() == Some("Z"))
                }) {
                    return;
                }
                thread::sleep(Duration::from_millis(10));
            }
            panic!("subprocess {pid} is still running");
        }

        #[test]
        fn captures_both_streams_and_nonzero_status() {
            let output = run_command(
                Path::new("/bin/sh"),
                &["-c", "printf out; printf err >&2; exit 7"],
                &[],
                Duration::from_secs(1),
            )
            .unwrap();
            assert_eq!(output.status.code(), Some(7));
            assert_eq!(output.stdout, b"out");
            assert_eq!(output.stderr, b"err");
        }

        #[test]
        fn exiting_parent_does_not_wait_for_inherited_output_writers() {
            let start = Instant::now();
            let output = run_command(
                Path::new("/bin/sh"),
                &["-c", "sleep 10 & echo $!; exit 0"],
                &[],
                Duration::from_millis(100),
            )
            .unwrap();
            assert!(start.elapsed() < Duration::from_secs(2));
            assert!(output.status.success());
            let pid = String::from_utf8(output.stdout)
                .unwrap()
                .trim()
                .parse()
                .unwrap();
            assert_stopped(pid);
        }

        #[test]
        fn timeout_stops_the_process_tree_without_waiting_for_pipe_eof() {
            let temp = tempfile::tempdir().unwrap();
            let pid_file = temp.path().join("pid");
            let start = Instant::now();
            let error = run_command(
                Path::new("/bin/sh"),
                &[
                    "-c",
                    "sleep 10 & echo $! > \"$1\"; echo before-timeout; wait",
                    "test",
                ],
                &[&pid_file],
                Duration::from_millis(100),
            )
            .err()
            .expect("timeout");
            assert!(start.elapsed() < Duration::from_secs(2), "{error}");
            assert!(error.contains("timed out"), "{error}");
            assert!(error.contains("before-timeout"), "{error}");
            assert_stopped(
                fs::read_to_string(pid_file)
                    .unwrap()
                    .trim()
                    .parse()
                    .unwrap(),
            );
        }

        #[test]
        fn startup_error_and_unwinding_drop_owned_processes() {
            let pid;
            {
                let mut process = CapturedProcess::spawn(&mut shell("exec sleep 10")).unwrap();
                pid = process.child.id();
                let error =
                    wait_for_anvil_port(&mut process, Duration::from_millis(30)).unwrap_err();
                assert!(error.contains("did not print a listening port"));
            }
            assert_stopped(pid);

            let mut pid = 0;
            let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
                let process = CapturedProcess::spawn(&mut shell("exec sleep 10")).unwrap();
                pid = process.child.id();
                panic!("simulate an E2E test panic");
            }));
            assert!(result.is_err());
            assert_stopped(pid);
        }

        #[test]
        fn startup_waits_for_a_complete_port_line() {
            let mut process = CapturedProcess::spawn(&mut shell(
                "printf 'Listening on 127.0.0.1:8'; sleep 0.05; printf '545\\n'; exec sleep 10",
            ))
            .unwrap();
            assert_eq!(
                wait_for_anvil_port(&mut process, Duration::from_secs(1)).unwrap(),
                8545
            );
        }

        // Invoked in an isolated subprocess by the next test. Its own process
        // exit is essential: static destructors do not run at that boundary.
        #[test]
        fn harness_scope_child() {
            let Ok(mode) = std::env::var("SOLCORE_HARNESS_SCOPE_TEST") else {
                return;
            };
            match mode.as_str() {
                "success" => with_evm_harness(|harness| {
                    assert!(harness.is_some());
                    Ok(())
                })
                .unwrap(),
                "error" => {
                    let result: Result<(), E2eFailure> = with_evm_harness(|harness| {
                        assert!(harness.is_some());
                        Err(E2eFailure::new(FailureKind::Call, "expected test failure"))
                    });
                    assert!(result.is_err());
                }
                "panic" => {
                    assert!(
                        std::panic::catch_unwind(|| {
                            let _: Result<(), E2eFailure> = with_evm_harness(|harness| {
                                assert!(harness.is_some());
                                panic!("expected test panic");
                            });
                        })
                        .is_err()
                    );
                    // The resource-free execution lock remains usable.
                    with_evm_harness(|harness| {
                        assert!(harness.is_some());
                        Ok(())
                    })
                    .unwrap();
                }
                _ => panic!("unknown test mode"),
            }
        }

        #[test]
        fn harness_processes_end_with_the_scope_on_success_error_and_panic() {
            let temp = tempfile::tempdir().unwrap();
            let anvil = temp.path().join("anvil");
            let cast = temp.path().join("cast");
            fs::write(&anvil, "#!/bin/sh\nif [ \"$1\" = --version ]; then exit 0; fi\necho $$ >> \"$SOLCORE_HARNESS_PID_FILE\"\necho 'Listening on 127.0.0.1:8545'\nexec sleep 10\n").unwrap();
            fs::write(&cast, "#!/bin/sh\nexit 0\n").unwrap();
            for path in [&anvil, &cast] {
                fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
            }
            for mode in ["success", "error", "panic"] {
                let pid_file = temp.path().join(format!("{mode}.pid"));
                // Do not use run_command here: its process-group cleanup would
                // hide a harness leak in the subprocess under test.
                let output = Command::new(std::env::current_exe().unwrap())
                    .args([
                        "--exact",
                        "e2e::process::tests::unix::harness_scope_child",
                        "--nocapture",
                    ])
                    .env("SOLCORE_HARNESS_SCOPE_TEST", mode)
                    .env("SOLCORE_HARNESS_PID_FILE", &pid_file)
                    .env("ANVIL", &anvil)
                    .env("CAST", &cast)
                    .env("E2E", "1")
                    .env("E2E_REQUIRED", "1")
                    .env_remove("E2E_PIPELINE_ONLY")
                    .output()
                    .unwrap();
                assert!(
                    output.status.success(),
                    "{mode}: {}\n{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                for pid in fs::read_to_string(pid_file).unwrap().lines() {
                    assert_stopped(pid.parse().unwrap());
                }
            }
        }
    }
}
