use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::{self, Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = env::temp_dir().join(format!("pipe-logger-test-{}-{timestamp}", process::id()));

        fs::create_dir(&path).unwrap();

        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn copies_stdin_bytes_to_stdout_and_log_file() {
    let temp_dir = TempDir::new();
    let log_path = temp_dir.path().join("output.log");
    let expected: &[u8] = b"first\nsecond-line\nnon-UTF-8: \xFF\n";

    let mut child = Command::new(env!("CARGO_BIN_EXE_pipe-logger"))
        .arg("--log-path")
        .arg(&log_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child.stdin.take().unwrap().write_all(expected).unwrap();

    let output = child.wait_with_output().unwrap();

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(expected, output.stdout.as_slice());
    assert_eq!(expected, fs::read(log_path).unwrap().as_slice());
}

#[test]
fn rotates_log_file_when_size_exceeds_threshold() {
    let temp_dir = TempDir::new();
    let log_path = temp_dir.path().join("output.log");
    // 17 bytes exceed the 10-byte rotation threshold, so exactly one rotation happens.
    let input: &[u8] = b"0123456789abcdef\n";

    let mut child = Command::new(env!("CARGO_BIN_EXE_pipe-logger"))
        .arg("--log-path")
        .arg(&log_path)
        .arg("-r")
        .arg("10")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child.stdin.take().unwrap().write_all(input).unwrap();

    let output = child.wait_with_output().unwrap();

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(input, output.stdout.as_slice());
    // After rotation the active file is renamed away and a fresh empty one takes its place.
    assert_eq!(0, fs::read(&log_path).unwrap().len());

    let rotated: Vec<_> = fs::read_dir(temp_dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .filter(|name| {
            let name = name.to_string_lossy();

            name != "output.log" && name.starts_with("output-") && name.ends_with(".log")
        })
        .collect();

    assert_eq!(1, rotated.len());
    assert_eq!(input, fs::read(temp_dir.path().join(&rotated[0])).unwrap().as_slice());
}

#[test]
fn re_outputs_through_stderr_with_err_flag() {
    let temp_dir = TempDir::new();
    let log_path = temp_dir.path().join("output.log");
    let input: &[u8] = b"log line\n";

    let mut child = Command::new(env!("CARGO_BIN_EXE_pipe-logger"))
        .arg("--log-path")
        .arg(&log_path)
        .arg("--err")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child.stdin.take().unwrap().write_all(input).unwrap();

    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    // With --err the logs are re-output through stderr, so stdout stays empty.
    assert_eq!(input, output.stderr.as_slice());
    assert_eq!(0, output.stdout.len());
    assert_eq!(input, fs::read(log_path).unwrap().as_slice());
}
