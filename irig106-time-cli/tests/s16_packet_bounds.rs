//! Host input-boundary regressions; partial CLI-102/107 and ERR-101 clauses.
use irig106_time_cli::{ExitClass, run};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Input(PathBuf);
impl Input {
    fn new(bytes: &[u8]) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "irigtime-bounds-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, bytes).unwrap();
        Self(path)
    }
}
impl Drop for Input {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
fn packet(kind: u8, flags: u8, length: usize, data_length: u32) -> Vec<u8> {
    let mut bytes = vec![0; length];
    bytes[..2].copy_from_slice(&0xEB25u16.to_le_bytes());
    bytes[4..8].copy_from_slice(&(length as u32).to_le_bytes());
    bytes[8..12].copy_from_slice(&data_length.to_le_bytes());
    bytes[14] = flags;
    bytes[15] = kind;
    bytes
}
fn input_error(bytes: &[u8]) {
    let input = Input::new(bytes);
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        run(
            &["summary", input.0.to_str().unwrap()],
            &mut Vec::new(),
            &mut Vec::new(),
        )
    }));
    assert_eq!(
        outcome.expect("run must not panic into the host"),
        ExitClass::Input
    );
}
fn content_finding(bytes: &[u8]) {
    let input = Input::new(bytes);
    let (mut out, mut err) = (Vec::new(), Vec::new());
    assert_eq!(
        run(
            &["summary", input.0.to_str().unwrap(), "--fail-on", "never"],
            &mut out,
            &mut err
        ),
        ExitClass::Success
    );
    assert!(String::from_utf8(out).unwrap().contains("Errors ("));
}
#[test]
fn f2_missing_secondary_header_is_an_input_error_not_host_panic() {
    input_error(&packet(0x12, 0x04, 24, 0));
}
#[test]
fn structural_lengths_are_fatal_but_short_time_bodies_are_findings() {
    for kind in [0x11, 0x12] {
        for flags in [0, 4] {
            for length in 24..48 {
                let start = if flags == 4 { 36 } else { 24 };
                let minimum = if kind == 0x11 { 10 } else { 12 };
                if length < start + minimum {
                    let bytes = packet(kind, flags, length, (length.saturating_sub(start)) as u32);
                    if length < start || length % 4 != 0 {
                        input_error(&bytes);
                    } else {
                        content_finding(&bytes);
                    }
                }
            }
        }
    }
    for length in 1..24 {
        content_finding(&vec![0; length]);
    }
    input_error(&[0; 24]); // A full header-sized region with no recoverable sync.
    let mut broken = packet(0x09, 0, 24, 0);
    broken[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
    input_error(&broken);
    broken[4..8].copy_from_slice(&0u32.to_le_bytes());
    input_error(&broken);
}
#[test]
fn declared_data_length_bounds_f1_and_f2_even_with_trailer_bytes() {
    for (kind, body) in [
        // Figure 11-13: CSDW=0, DOY001 00:00:00.00 (six BCD bytes).
        (0x11, vec![0, 0, 0, 0, 0, 0, 0, 0, 1, 0]),
        // Figures 11-15/17: NTF1/TS1; 2017-01-01 TAI seconds, zero ns.
        (
            0x12,
            [
                vec![0x11, 0, 0, 0],
                1_483_228_837u32.to_le_bytes().to_vec(),
                vec![0; 4],
            ]
            .concat(),
        ),
    ] {
        let mut bytes = packet(kind, 0, 48, body.len() as u32);
        bytes[24..24 + body.len()].copy_from_slice(&body);
        let valid = Input::new(&bytes);
        assert_eq!(
            run(
                &["summary", valid.0.to_str().unwrap()],
                &mut Vec::new(),
                &mut Vec::new()
            ),
            ExitClass::Success
        );
        for length in [0u32, u32::MAX] {
            bytes[8..12].copy_from_slice(&length.to_le_bytes());
            if length == 0 {
                content_finding(&bytes);
            } else {
                input_error(&bytes);
            }
        }
    }
}

#[cfg(unix)]
mod native_argv {
    use super::*;
    use std::{ffi::OsString, os::unix::ffi::OsStringExt, process::Command};
    fn native_path(path: &std::path::Path, byte: u8) -> PathBuf {
        let mut name = path.as_os_str().to_os_string();
        name.push(OsString::from_vec(vec![byte]));
        PathBuf::from(name)
    }
    #[test]
    fn non_utf8_input_and_output_paths_work_in_host_and_binary() {
        let mut input = Input::new(&packet(0x09, 0, 24, 0));
        let native = native_path(&input.0, 0xff);
        std::fs::rename(&input.0, &native).unwrap();
        input.0 = native;
        let output = Input(native_path(&input.0, 0xfe));
        let args = [
            OsString::from("csv"),
            input.0.as_os_str().to_owned(),
            OsString::from("--output"),
            output.0.as_os_str().to_owned(),
        ];
        assert_eq!(
            run(&args, &mut Vec::new(), &mut Vec::new()),
            ExitClass::Success
        );
        let bytes = std::fs::read(&output.0).unwrap();
        assert!(bytes.starts_with(b"packet_num,"));
        assert_eq!(bytes.iter().filter(|b| **b == b'\n').count(), 2);
        let status = Command::new(env!("CARGO_BIN_EXE_irigtime"))
            .args(&args)
            .output()
            .unwrap();
        assert!(status.status.success(), "{:?}", status);
        assert_eq!(std::fs::read(&output.0).unwrap(), bytes);
    }
    #[test]
    fn non_utf8_command_and_option_are_usage_errors_not_panics() {
        let input = Input::new(&packet(0x09, 0, 24, 0));
        let bad = OsString::from_vec(vec![b'-', b'-', 0xff]);
        for args in [
            vec![bad.clone()],
            vec![OsString::from("summary"), bad.clone()],
            vec![
                OsString::from("summary"),
                input.0.as_os_str().to_owned(),
                bad.clone(),
            ],
            vec![
                OsString::from("summary"),
                input.0.with_extension("missing").into_os_string(),
                bad,
            ],
        ] {
            let result = catch_unwind(AssertUnwindSafe(|| {
                run(&args, &mut Vec::new(), &mut Vec::new())
            }));
            assert_eq!(result.expect("host argv must not panic"), ExitClass::Usage);
            let status = Command::new(env!("CARGO_BIN_EXE_irigtime"))
                .args(&args)
                .output()
                .unwrap();
            assert_eq!(status.status.code(), Some(2));
            assert!(!String::from_utf8_lossy(&status.stderr).contains("panicked"));
        }
    }
}

fn f1_reference() -> Vec<u8> {
    // Figure11-14: DATE1/LY1, 2024-02-29 13:45:27.35; 12 data bytes.
    let mut bytes = packet(0x11, 0, 36, 12);
    bytes[24..].copy_from_slice(&[0, 3, 0, 0, 0x35, 0x27, 0x45, 0x13, 0x29, 2, 0x24, 0x20]);
    // Primary header word sum: EB25 + 24 + C + 1100 = FC55.
    bytes[22..24].copy_from_slice(&0xfc55u16.to_le_bytes());
    bytes
}
fn success_with_finding(bytes: &[u8], finding: &str, count: usize) {
    let input = Input::new(bytes);
    for command in [
        "summary",
        "channels",
        "jumps",
        "timeline",
        "csv",
        "correlate",
    ] {
        let mut args = vec![command, input.0.to_str().unwrap()];
        if command == "correlate" {
            args.push("0");
        }
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let result = catch_unwind(AssertUnwindSafe(|| run(&args, &mut out, &mut err)));
        assert_eq!(
            result.expect("run must not panic"),
            ExitClass::Success,
            "{command}: {}",
            String::from_utf8_lossy(&err)
        );
        let report = String::from_utf8(out).unwrap();
        let messages = format!("{report}\n{}", String::from_utf8_lossy(&err));
        assert!(messages.contains(finding), "{command}: {messages}");
        if command == "summary" {
            assert!(report.contains(&format!("Total Packets      : {count}")));
            assert!(report.contains("Data Type Breakdown"));
            assert!(report.contains("Correlation Refs   : 2"));
            assert!(report.contains("2024"));
        }
        if command == "csv" {
            assert_eq!(report.lines().count(), count + 1);
        }
    }
}
#[test]
fn ntp_context_error_between_valid_f1_packets_is_a_reported_finding() {
    // Figs11-15/16: NTF0/TS1, RFC5905 Fig4's 1970 seconds, fraction0.
    let mut ntp = packet(0x12, 0, 36, 12);
    ntp[2..4].copy_from_slice(&2u16.to_le_bytes());
    ntp[22..24].copy_from_slice(&0xfd57u16.to_le_bytes());
    ntp[24] = 1;
    ntp[28..32].copy_from_slice(&2_208_988_800u32.to_le_bytes());
    success_with_finding(
        &[f1_reference(), ntp, f1_reference()].concat(),
        "NTP era/basis required",
        3,
    );
}
#[test]
fn recovered_sync_loss_is_a_reported_finding() {
    success_with_finding(
        &[vec![0xaa, 0xbb, 0xcc], f1_reference(), f1_reference()].concat(),
        "Sync lost",
        2,
    );
}
#[test]
fn power_loss_tail_keeps_the_full_preceding_analysis() {
    for tail in [vec![0x25, 0xeb, 0], f1_reference()[..30].to_vec()] {
        success_with_finding(
            &[f1_reference(), f1_reference(), tail].concat(),
            "truncated final",
            2,
        );
    }
}
#[test]
fn reserved_bcd_bit_is_a_reported_finding() {
    let mut reserved = f1_reference();
    reserved[29] |= 0x80;
    success_with_finding(
        &[f1_reference(), reserved, f1_reference()].concat(),
        "reserved bit",
        3,
    );
}

#[test]
fn trailing_zero_block_padding_preserves_analysis() {
    success_with_finding(
        &[f1_reference(), f1_reference(), vec![0; 512]].concat(),
        "512 trailing bytes without sync at offset 0x48",
        2,
    );
}
#[test]
fn trailing_ff_block_padding_preserves_analysis() {
    success_with_finding(
        &[f1_reference(), f1_reference(), vec![0xff; 4096]].concat(),
        "4096 trailing bytes without sync at offset 0x48",
        2,
    );
}
#[test]
fn no_sync_without_established_stream_is_still_input_error() {
    for bytes in [vec![0; 24], vec![0; 512], vec![0xff; 4096]] {
        input_error(&bytes);
    }
}
