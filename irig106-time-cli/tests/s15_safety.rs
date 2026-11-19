//! S15 CLI failure controls; partial CLI-105/ERR-101, not whole markers.
use std::{
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};
struct Input {
    dir: PathBuf,
    file: PathBuf,
}
impl Input {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "irigtime-s15-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&dir).unwrap();
        let file = dir.join("input.ch10");
        std::fs::write(&file, []).unwrap();
        Self { dir, file }
    }
}
impl Drop for Input {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}
fn check(args: &[&str], code: i32) {
    let output = Command::new(env!("CARGO_BIN_EXE_ch10time"))
        .args(args)
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(code),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"));
}
#[test]
fn malformed_rtc_is_a_usage_error_not_a_panic() {
    let input = Input::new();
    check(&["correlate", input.file.to_str().unwrap(), "not-hex"], 2);
}
#[test]
fn threshold_overflow_is_a_usage_error_not_a_panic() {
    let input = Input::new();
    check(
        &[
            "jumps",
            input.file.to_str().unwrap(),
            "--threshold-ms",
            "18446744073709551615",
        ],
        2,
    );
}
#[test]
fn csv_create_failure_is_an_output_error_not_a_panic() {
    let input = Input::new();
    let missing = input.dir.join("missing/output.csv");
    check(
        &[
            "csv",
            input.file.to_str().unwrap(),
            "--output",
            missing.to_str().unwrap(),
        ],
        4,
    );
}
#[test]
fn unreadable_input_is_an_input_error() {
    let input = Input::new();
    check(
        &["summary", input.dir.join("absent.ch10").to_str().unwrap()],
        3,
    );
}

#[cfg(target_os = "linux")]
#[test]
fn csv_flush_failure_is_an_output_error_not_silent_success() {
    let input = Input::new();
    // BufWriter accepts this small header; /dev/full fails when flushed.
    check(
        &["csv", input.file.to_str().unwrap(), "--output", "/dev/full"],
        4,
    );
}

// S16a1 layout/name clause only; host runner and exit contract follow later.
#[test]
fn standalone_identity_matches_lockstep_version() {
    let version = Command::new(env!("CARGO_BIN_EXE_ch10time"))
        .arg("--version")
        .output()
        .unwrap();
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8(version.stdout).unwrap(),
        concat!("irigtime ", env!("CARGO_PKG_VERSION"), "\n")
    );
    let help = Command::new(env!("CARGO_BIN_EXE_ch10time"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(help.status.success());
    let help = String::from_utf8(help.stdout).unwrap();
    assert!(help.contains("irigtime summary"));
    assert!(!help.contains("ch10time"));
}
