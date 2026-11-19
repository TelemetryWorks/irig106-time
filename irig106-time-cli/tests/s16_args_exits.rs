//! S16a3 parser and exit contract, not the qualified S16b report model.
use irig106_time_cli::{ExitClass, run};
use std::{
    io::{self, Write},
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};
struct Input(PathBuf);
impl Input {
    fn new(bytes: &[u8]) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "irigtime-exits-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, bytes).unwrap();
        Self(path)
    }
    fn path(&self) -> &str {
        self.0.to_str().unwrap()
    }
}
impl Drop for Input {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
struct FailedOutput;
impl Write for FailedOutput {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("injected output"))
    }
    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::other("injected flush"))
    }
}
fn invoke(args: &[&str]) -> u8 {
    run(args, &mut Vec::new(), &mut Vec::new()) as u8
}
fn on(input: &Input, command: &str, options: &[&str]) -> u8 {
    let mut args = vec![command, input.path()];
    args.extend_from_slice(options);
    invoke(&args)
}
#[test]
fn exit_zero_success() {
    assert_eq!(invoke(&["--version"]), 0);
}
#[test]
fn exit_one_is_selected_severity_not_a_fatal_input_failure() {
    let input = Input::new(&[0; 3]); // Cut final header: warning reader finding.
    assert_eq!(on(&input, "summary", &[]), 0);
    for (level, code) in [
        ("info", 1),
        ("warning", 1),
        ("error", 0),
        ("never", 0),
    ] {
        assert_eq!(on(&input, "summary", &["--fail-on", level]), code);
    }
    let mut packet = vec![0; 24];
    packet[..2].copy_from_slice(&0xeb25u16.to_le_bytes());
    packet[4..8].copy_from_slice(&24u32.to_le_bytes());
    packet[15] = 0x11; // Empty F1 body: error finding, not bad framing.
    let malformed = Input::new(&packet);
    assert_eq!(on(&malformed, "summary", &[]), 1);
    assert_eq!(on(&malformed, "summary", &["--fail-on", "never"]), 0);
}
#[test]
fn exit_two_validates_all_usage_before_opening_input() {
    let missing = Input::new(&[]);
    std::fs::remove_file(&missing.0).unwrap();
    for case in [
        "jumps --threshold-ms bad",
        "jumps --threshold-ms -1",
        "jumps --threshold-ms 18446744073709551615",
        "jumps --threshold-ms",
        "jumps --threshold-ms 1 --threshold-ms 2",
        "timeline --limit bad",
        "timeline --limit 0",
        "timeline --limit -1",
        "timeline --limit +1",
        "timeline --limit 18446744073709551616",
        "timeline --limit",
        "timeline --limit 1 --limit 2",
        "summary --unknown",
        "summary extra",
        "summary --limit 1",
        "csv --output",
        "csv --output a --output b",
        "correlate 1000000000000",
        "summary --fail-on bogus",
        "summary --fail-on",
        "summary --fail-on error --fail-on never",
    ] {
        let mut args: Vec<_> = case.split_ascii_whitespace().collect();
        args.insert(1, missing.path());
        assert_eq!(invoke(&args), 2, "{case}");
    }
    assert_eq!(invoke(&["--version", "extra"]), 2);
}
#[test]
fn exit_three_input_precedes_findings_and_diagnostic_output_failure() {
    let invalid = Input::new(&[0; 24]);
    assert_eq!(on(&invalid, "summary", &["--fail-on", "never"]), 3);
    assert_eq!(
        run(
            &["summary", invalid.path()],
            &mut FailedOutput,
            &mut FailedOutput
        ) as u8,
        3
    );
}
#[test]
fn exit_four_output_precedes_findings() {
    let input = Input::new(&[0; 3]);
    assert_eq!(
        run(
            &["summary", input.path(), "--fail-on", "warning"],
            &mut FailedOutput,
            &mut FailedOutput
        ) as u8,
        4
    );
    let absent = input.0.join("missing.csv");
    assert_eq!(
        on(
            &input,
            "csv",
            &["--fail-on", "warning", "--output", absent.to_str().unwrap()]
        ),
        4
    );
}
#[test]
fn strict_args_help_terminator_and_host_standalone_agree() {
    use irig106_time_cli::args::parse;
    for args in [
        vec!["summary", "--", "-capture.ch10"],
        vec!["correlate", "input", "FFFFFFFFFFFF"],
        vec!["jumps", "input", "--threshold-ms", "18446744073709"],
        vec!["timeline", "input", "--limit", "1"],
    ] {
        assert!(parse(&args).is_ok());
    }
    assert!(parse(&["summary"]).is_err());
    for command in [
        "summary",
        "channels",
        "jumps",
        "timeline",
        "csv",
        "correlate",
    ] {
        assert_eq!(invoke(&[command, "--help"]), 0);
    }
    let input = Input::new(&[0; 3]);
    let args = ["summary", input.path(), "--fail-on", "warning"];
    let (mut out, mut err) = (Vec::new(), Vec::new());
    assert_eq!(run(&args, &mut out, &mut err), ExitClass::Findings);
    let standalone = Command::new(env!("CARGO_BIN_EXE_irigtime"))
        .args(args)
        .output()
        .unwrap();
    assert_eq!(standalone.status.code(), Some(1));
    assert_eq!(standalone.stdout, out);
    assert_eq!(standalone.stderr, err);
}
/// Requirements: L1-CLI-007, L2-CLI-107, L3-CLI-107
#[test]
fn complete_five_class_threshold_precedence_and_host_contract() {
    exit_zero_success();
    exit_one_is_selected_severity_not_a_fatal_input_failure();
    exit_two_validates_all_usage_before_opening_input();
    exit_three_input_precedes_findings_and_diagnostic_output_failure();
    exit_four_output_precedes_findings();
    strict_args_help_terminator_and_host_standalone_agree();
}
