//! S16a3 parser and exit contract, not the qualified S16b report model.
use irig106_time_cli::run;
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
#[derive(Default)]
struct FailedOutput {
    writes: usize,
    flush_only: bool,
}
impl Write for FailedOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.writes += 1;
        if self.flush_only {
            Ok(bytes.len())
        } else {
            Err(io::Error::other("injected output"))
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::other("injected flush"))
    }
}
fn invoke(args: &[&str]) -> u8 {
    run(args, &mut Vec::new(), &mut Vec::new()) as u8
}
fn on(input: &Input, options: &[&str]) -> u8 {
    let mut args = vec!["summary", input.path()];
    args.extend_from_slice(options);
    invoke(&args)
}
fn reference() -> Vec<u8> {
    // Fig11-14: DATE1/LY1, 2024-02-29 13:45:27.35. Header word sum FC55.
    vec![
        0x25, 0xeb, 0, 0, 36, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0x11, 0, 0, 0, 0, 0, 0, 0x55, 0xfc, 0,
        3, 0, 0, 0x35, 0x27, 0x45, 0x13, 0x29, 2, 0x24, 0x20,
    ]
}
#[test]
fn exit_zero_success() {
    assert_eq!(invoke(&["--version"]), 0);
}
#[test]
fn exit_one_uses_inclusive_severity_and_never() {
    let warning = Input::new(&[0; 3]);
    let mut padding = reference();
    padding.resize(padding.len() + 512, 0);
    let info = Input::new(&padding);
    let mut malformed = reference();
    malformed[28] = 0xfa; // Invalid BCD digit.
    let error = Input::new(&malformed);
    let mut reserved = reference();
    reserved[29] |= 0x80; // TF-005, not generic decode error.
    let reserved = Input::new(&reserved);
    for (input, expected) in [
        (&warning, [0, 1, 1, 0]),
        (&info, [0, 0, 1, 0]),
        (&error, [1, 1, 1, 0]),
        (&reserved, [0, 1, 1, 0]),
    ] {
        assert_eq!(on(input, &[]), expected[0]);
        for (level, code) in ["error", "warning", "info", "never"]
            .into_iter()
            .zip(expected)
        {
            assert_eq!(on(input, &["--fail-on", level]), code, "{level}");
        }
    }
    for command in [
        "summary",
        "channels",
        "jumps",
        "timeline",
        "csv",
        "correlate",
    ] {
        let mut args = vec![command, error.path()];
        if command == "correlate" {
            args.push("0");
        }
        assert_eq!(invoke(&args), 1, "{command} must retain its error finding");
        args.extend(["--fail-on", "never"]);
        assert_eq!(invoke(&args), 0);
    }
}
#[test]
fn exit_two_usage_precedes_input_and_output() {
    let missing = Input::new(&[]);
    std::fs::remove_file(&missing.0).unwrap();
    for tail in [
        vec!["--fail-on"],
        vec!["--fail-on", "ERROR"],
        vec!["--fail-on", "information"],
        vec!["--fail-on", "bogus"],
        vec!["--fail-on", "error", "--fail-on", "never"],
        vec!["--fail-on=error"],
    ] {
        let mut args = vec!["summary", missing.path()];
        args.extend(tail);
        assert_eq!(invoke(&args), 2);
        assert_eq!(
            run(
                &args,
                &mut FailedOutput::default(),
                &mut FailedOutput::default()
            ) as u8,
            2
        );
    }
}
#[test]
fn exit_three_first_input_failure_survives_later_output_failure() {
    let invalid = Input::new(&[0; 24]);
    assert_eq!(on(&invalid, &["--fail-on", "never"]), 3);
    let mut diagnostic = Vec::new();
    assert_eq!(
        run(
            &["summary", invalid.path()],
            &mut Vec::new(),
            &mut diagnostic
        ) as u8,
        3
    );
    assert!(
        String::from_utf8(diagnostic)
            .unwrap()
            .contains("invalid packet sync")
    );
    let (mut out, mut err) = (FailedOutput::default(), FailedOutput::default());
    assert_eq!(
        run(&["summary", invalid.path()], &mut out, &mut err) as u8,
        3
    );
    assert_eq!(out.writes, 0);
    assert!(err.writes > 0); // Diagnosis attempted after input failed.
}
#[test]
fn exit_four_output_and_flush_failures_precede_findings() {
    let warning = Input::new(&[0; 3]);
    let args = ["summary", warning.path(), "--fail-on", "warning"];
    let (mut out, mut err) = (FailedOutput::default(), FailedOutput::default());
    assert_eq!(run(&args, &mut out, &mut err) as u8, 4);
    assert!(out.writes > 0 && err.writes > 0); // First output failed; later diagnosis attempted.
    for fail_stdout in [true, false] {
        let mut failed = FailedOutput {
            flush_only: true,
            ..Default::default()
        };
        let code = if fail_stdout {
            run(&args, &mut failed, &mut Vec::new())
        } else {
            run(&args, &mut Vec::new(), &mut failed)
        };
        assert_eq!(code as u8, 4);
    }
}
fn parity(args: &[&str], expected: i32) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    assert_eq!(run(args, &mut out, &mut err) as i32, expected);
    let binary = Command::new(env!("CARGO_BIN_EXE_irigtime"))
        .args(args)
        .output()
        .unwrap();
    assert_eq!(binary.status.code(), Some(expected));
    assert_eq!(binary.stdout, out);
    assert_eq!(binary.stderr, err);
}
#[test]
fn every_exit_class_has_host_standalone_parity() {
    let warning = Input::new(&[0; 3]);
    let invalid = Input::new(&[0; 24]);
    let absent_output = warning.0.join("missing.csv");
    parity(&["--version", "--fail-on", "info"], 0);
    parity(&["summary", warning.path(), "--fail-on", "warning"], 1);
    parity(&["summary", invalid.path(), "--fail-on", "ERROR"], 2);
    parity(&["summary", invalid.path(), "--fail-on", "never"], 3);
    parity(
        &[
            "csv",
            warning.path(),
            "--output",
            absent_output.to_str().unwrap(),
            "--fail-on",
            "warning",
        ],
        4,
    );
}
#[test]
fn shared_option_is_validated_even_without_findings_and_edges_are_unchanged() {
    let valid = Input::new(&reference());
    for command in [
        "summary",
        "channels",
        "jumps",
        "timeline",
        "csv",
        "correlate",
    ] {
        let mut args = vec![command, valid.path()];
        if command == "correlate" {
            args.push("0");
        }
        args.extend(["--fail-on", "info"]);
        assert_eq!(invoke(&args), 0);
        assert_eq!(invoke(&[command, "--help", "--fail-on", "warning"]), 0);
    }
    for command in ["--help", "--version"] {
        for level in ["error", "warning", "info", "never"] {
            assert_eq!(invoke(&[command, "--fail-on", level]), 0);
        }
        for bad in [
            vec![command, "--fail-on", "Error"],
            vec![command, "--fail-on", "warning", "--fail-on", "info"],
            vec![command, "--fail-on"],
        ] {
            assert_eq!(invoke(&bad), 2);
        }
    }
    assert!(irig106_time_cli::args::parse(&["csv", "input", "--output", "--limit"]).is_ok());
    assert!(irig106_time_cli::args::parse(&["summary", "input", "--help"]).is_err());
}
#[test]
fn recovered_sync_cut_packet_context_and_jumps_remain_warning() {
    let mut ntp = reference();
    ntp[2..4].copy_from_slice(&2u16.to_le_bytes());
    ntp[15] = 0x12;
    ntp[22..24].copy_from_slice(&0xfd57u16.to_le_bytes());
    ntp[24..].fill(0);
    ntp[24] = 1; // Figs11-15/16: NTF0 TS1.
    ntp[28..32].copy_from_slice(&2_208_988_800u32.to_le_bytes()); // RFC5905 Fig4.
    let mut cases = vec![
        [vec![0xaa, 0xbb, 0xcc], reference()].concat(),
        reference()[..30].to_vec(),
        [reference(), ntp.clone()].concat(),
    ];
    for word in [0, 2, 0x31] {
        // Invalid TS, reserved TS and reserved NTF: TF-003/006 warning.
        let mut invalid = ntp.clone();
        invalid[24] = word;
        cases.push([reference(), invalid].concat());
    }
    for bytes in cases {
        let input = Input::new(&bytes);
        assert_eq!(on(&input, &[]), 0);
        assert_eq!(on(&input, &["--fail-on", "warning"]), 1);
    }
    let mut next = reference();
    next[29] = 0x30; // Same RTC, +3 seconds absolute.
    let input = Input::new(&[reference(), next].concat());
    for command in ["summary", "jumps"] {
        assert_eq!(invoke(&[command, input.path(), "--fail-on", "error"]), 0);
        assert_eq!(invoke(&[command, input.path(), "--fail-on", "warning"]), 1);
    }
}
/// Requirements: L1-CLI-007, L2-CLI-107, L3-CLI-107
#[test]
fn complete_five_class_threshold_precedence_and_host_contract() {
    exit_zero_success();
    exit_one_uses_inclusive_severity_and_never();
    exit_two_usage_precedes_input_and_output();
    exit_three_first_input_failure_survives_later_output_failure();
    exit_four_output_and_flush_failures_precede_findings();
    every_exit_class_has_host_standalone_parity();
    shared_option_is_validated_even_without_findings_and_edges_are_unchanged();
    recovered_sync_cut_packet_context_and_jumps_remain_warning();
    review_short_f1_doy_body_is_an_error_finding();
    review_no_time_packets_correlate_is_a_warning();
}

#[test]
fn review_short_f1_doy_body_is_an_error_finding() {
    // Fig11-13 requires six DOY body bytes. This complete32-byte frame declares
    // CSDW4 + body4, so the time body is short without invalid packet bounds.
    let mut bytes = reference();
    bytes.truncate(32);
    bytes[4..8].copy_from_slice(&32u32.to_le_bytes());
    bytes[8..12].copy_from_slice(&8u32.to_le_bytes());
    bytes[22..24].copy_from_slice(&0xfc4du16.to_le_bytes()); // EB25 +20 +8 +1100.
    bytes[24..28].fill(0); // DATE0: DOY, not DMY.
    let input = Input::new(&bytes);
    let mut report = Vec::new();
    assert_eq!(
        run(&["summary", input.path()], &mut report, &mut Vec::new()) as u8,
        1
    );
    assert!(
        String::from_utf8(report)
            .unwrap()
            .contains("buffer too short for DOY BCD")
    );
    assert_eq!(on(&input, &["--fail-on", "never"]), 0);
}

#[test]
fn review_no_time_packets_correlate_is_a_warning() {
    // A complete non-time transport frame, as in the native-path control.
    let mut bytes = reference();
    bytes.truncate(24);
    bytes[4..8].copy_from_slice(&24u32.to_le_bytes());
    bytes[8..12].fill(0);
    bytes[15] = 0x09;
    bytes[22..24].copy_from_slice(&0xf43du16.to_le_bytes()); // EB25 +18 +900.
    let input = Input::new(&bytes);
    let mut diagnostic = Vec::new();
    assert_eq!(
        run(
            &["correlate", input.path(), "0"],
            &mut Vec::new(),
            &mut diagnostic
        ) as u8,
        0
    );
    assert!(
        String::from_utf8(diagnostic)
            .unwrap()
            .contains("Warning: No time reference points found")
    );
    assert_eq!(
        invoke(&["correlate", input.path(), "0", "--fail-on", "warning"]),
        1
    );
}
