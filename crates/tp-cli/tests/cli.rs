use std::{path::PathBuf, process::Command};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn tp() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tp"))
}

#[test]
fn check_returns_zero_for_valid_program() {
    let output = tp()
        .arg("check")
        .arg(fixture("hello.tp"))
        .output()
        .expect("tp should launch");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn check_returns_one_and_human_diagnostic_for_invalid_program() {
    let output = tp()
        .arg("check")
        .arg(fixture("invalid.tp"))
        .output()
        .expect("tp should launch");
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error[TP-E0200]"), "{stderr}");
    assert!(stderr.contains("unknown name `missing`"), "{stderr}");
}

#[test]
fn run_executes_program_and_writes_program_output_to_stdout() {
    let output = tp()
        .arg("run")
        .arg(fixture("hello.tp"))
        .output()
        .expect("tp should launch");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "hello from TP\n");
}

#[test]
fn json_diagnostics_are_machine_readable_objects() {
    let output = tp()
        .args(["check", "--diagnostic-format", "json"])
        .arg(fixture("invalid.tp"))
        .output()
        .expect("tp should launch");
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("\"schema\":1"), "{stderr}");
    assert!(stderr.contains("\"code\":\"TP-E0200\""), "{stderr}");
    assert!(stderr.contains("\"severity\":\"error\""), "{stderr}");
    assert!(stderr.contains("\"message\":\"unknown name `missing`\""), "{stderr}");
}
