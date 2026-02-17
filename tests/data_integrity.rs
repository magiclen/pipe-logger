/// Integration tests for piped data integrity.
///
/// Reproduces data corruption bug magiclen/pipe-logger#4:
/// `read_line` appends to the buffer, but it was never cleared,
/// so `write_all(&input.as_bytes()[..c])` wrote stale data from
/// the start of the growing buffer instead of the newly-read line.
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

fn pipe_logger_bin() -> std::path::PathBuf {
    // cargo sets this env var for integration tests
    env!("CARGO_BIN_EXE_pipe-logger").into()
}

/// Pipe `lines` into pipe-logger, return the log file contents.
fn run_pipe_logger(lines: &[String]) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    let log_path = dir.path().join("test.log");

    let mut child = Command::new(pipe_logger_bin())
        .arg("--log-path")
        .arg(&log_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn pipe-logger");

    {
        let stdin = child.stdin.as_mut().expect("open stdin");
        for line in lines {
            writeln!(stdin, "{}", line).expect("write line");
        }
    } // stdin dropped here → EOF

    let status = child.wait().expect("wait");
    assert!(status.success(), "pipe-logger exited with {status}");

    fs::read_to_string(&log_path).expect("read log")
}

#[test]
#[ignore] // Reproduces data corruption bug magiclen/pipe-logger#4 — remove after fix
fn sequential_numbers_integrity() {
    let lines: Vec<String> = (1..=500).map(|n| n.to_string()).collect();
    let output = run_pipe_logger(&lines);

    let output_lines: Vec<&str> = output.lines().collect();
    assert_eq!(
        output_lines.len(),
        500,
        "expected 500 lines, got {}",
        output_lines.len()
    );

    for (i, line) in output_lines.iter().enumerate() {
        let expected = (i + 1).to_string();
        assert_eq!(
            *line, expected,
            "line {} mismatch: expected {:?}, got {:?}",
            i + 1,
            expected,
            line
        );
    }
}

#[test]
#[ignore] // Reproduces data corruption bug magiclen/pipe-logger#4 — remove after fix
fn long_jsonl_lines_integrity() {
    let lines: Vec<String> = (1..=200)
        .map(|n| {
            format!(
                r#"{{"id":{},"event":"tool_use","data":"{}"}}"#,
                n,
                "x".repeat(200)
            )
        })
        .collect();

    let output = run_pipe_logger(&lines);
    let output_lines: Vec<&str> = output.lines().collect();

    assert_eq!(
        output_lines.len(),
        lines.len(),
        "line count mismatch: expected {}, got {}",
        lines.len(),
        output_lines.len()
    );

    for (i, (got, expected)) in output_lines.iter().zip(lines.iter()).enumerate() {
        assert_eq!(
            *got, expected,
            "JSONL line {} mismatch",
            i + 1
        );
    }
}
