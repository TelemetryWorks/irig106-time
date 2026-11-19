//! Partial CLI-104 transport integration; qualified reports remain S16b.
mod common;
use irig106_time_cli::{ExitClass, run};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
struct Input(PathBuf);
impl Input {
    fn new(bytes: &[u8]) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "irigtime-join-{}-{}",
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
fn reference(secondary: bool) -> Vec<u8> {
    // Fig11-14: DATE1/LY1, 2024-02-29 13:45:27.35; §11.2.1.1 bit7.
    let mut b = vec![0; if secondary { 48 } else { 36 }];
    b[..2].copy_from_slice(&[0x25, 0xEB]);
    b[4] = b.len() as u8;
    b[8] = 12;
    b[14] = if secondary { 0x80 } else { 0 };
    b[15] = 0x11;
    let sum = if secondary { 0xFCE1u16 } else { 0xFC55 };
    // EB25 + 30 + C + 1180 = FCE1; no secondary: EB25+24+C+1100=FC55.
    b[22..24].copy_from_slice(&sum.to_le_bytes());
    let start = if secondary { 36 } else { 24 };
    b[start..].copy_from_slice(&[0, 3, 0, 0, 0x35, 0x27, 0x45, 0x13, 0x29, 2, 0x24, 0x20]);
    b
}
fn commands(bytes: &[u8], expect: ExitClass, required: &str) {
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
        assert_eq!(
            run(&args, &mut out, &mut err),
            expect,
            "{command}: {}",
            String::from_utf8_lossy(&err)
        );
        let text = format!(
            "{}{}",
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap()
        );
        let required = if command == "jumps" && required == "2024" {
            "1 channel(s)"
        } else {
            required
        };
        assert!(text.contains(required), "{command}: {text}");
    }
}
#[test]
fn every_command_uses_bit7_secondary_presence() {
    commands(&reference(true), ExitClass::Success, "2024");
}
#[test]
fn every_command_reports_expected_offset_checksum_loss() {
    let good = reference(false);
    let mut bad = good.clone();
    bad[22] ^= 1;
    commands(
        &[good.clone(), bad, good].concat(),
        ExitClass::Success,
        "36 bytes skipped",
    );
}
#[test]
fn no_command_accepts_a_file_without_any_valid_header() {
    for bytes in [vec![], vec![0; 3], vec![0; 24]] {
        commands(&bytes, ExitClass::Input, "valid");
    }
}

#[test]
fn reserved_ntf_has_a_dedicated_error_not_a_field_name_convention() {
    use irig106_time::{
        TimeError,
        network_time::{TimeF2Csdw, parse_time_f2_payload},
    };
    for ntf in 3..16 {
        let word = TimeF2Csdw::from_raw((ntf << 4) | 1);
        let mut payload = [0; 12];
        payload[..4].copy_from_slice(&word.as_raw().to_le_bytes());
        let expected = TimeError::ReservedNetworkTimeFormat { value: ntf as u8 };
        assert_eq!(word.to_le_bytes(), Err(expected.clone()));
        assert_eq!(parse_time_f2_payload(&payload), Err(expected));
    }
}

#[test]
fn timeline_limit_does_not_hide_later_structural_input_or_render_early() {
    let mut broken = reference(false);
    broken[4] = 26;
    broken[8] = 0;
    // EB25 + 001A + 1100 = FC3F: valid checksum, invalid alignment only.
    broken[22..24].copy_from_slice(&[0x3F, 0xFC]);
    let input = Input::new(&[reference(false), broken].concat());
    let mut out = Vec::new();
    assert_eq!(
        run(
            &["timeline", input.0.to_str().unwrap(), "--limit", "1"],
            &mut out,
            &mut Vec::new()
        ),
        ExitClass::Input
    );
    assert!(out.is_empty());
}

#[test]
fn valid_no_time_file_reports_tf001_on_every_command() {
    commands(&common::NO_TIME, ExitClass::Success, "TF-001");
}
