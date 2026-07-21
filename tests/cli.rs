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
