//! Partial CLI-104 transport integration; qualified reports remain S16b.
#[path = "common/mod.rs"]
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

#[test]
fn lone_truncated_packet_with_valid_header_is_input_on_every_command() {
    // §11.2.1.1: FC55 still validates the complete primary header; six of
    // the declared twelve data bytes are missing. No complete packet exists.
    commands(&reference(false)[..30], ExitClass::Input, "valid");
}

#[test]
fn tf001_warning_threshold_is_preserved_on_every_command() {
    let input = Input::new(&common::NO_TIME);
    for command in [
        "summary",
        "channels",
        "jumps",
        "timeline",
        "csv",
        "correlate",
    ] {
        for (level, expected) in [
            ("error", ExitClass::Success),
            ("warning", ExitClass::Findings),
            ("info", ExitClass::Findings),
            ("never", ExitClass::Success),
        ] {
            let mut args = vec![command, input.0.to_str().unwrap()];
            if command == "correlate" {
                args.push("0");
            }
            args.extend(["--fail-on", level]);
            let (mut out, mut err) = (Vec::new(), Vec::new());
            assert_eq!(
                run(&args, &mut out, &mut err),
                expected,
                "{command} {level}"
            );
            let messages = format!(
                "{}{}",
                String::from_utf8(out).unwrap(),
                String::from_utf8(err).unwrap()
            );
            assert!(
                messages.contains("Warning: TF-001"),
                "{command}: {messages}"
            );
        }
    }
}

#[test]
fn timeline_limit_one_on_two_time_packets_emits_exactly_one_row() {
    let input = Input::new(&[reference(false), reference(false)].concat());
    let mut out = Vec::new();
    assert_eq!(
        run(
            &["timeline", input.0.to_str().unwrap(), "--limit", "1"],
            &mut out,
            &mut Vec::new()
        ),
        ExitClass::Success
    );
    let report = String::from_utf8(out).unwrap();
    let rows: Vec<usize> = report
        .lines()
        .filter_map(|line| line.split_whitespace().next()?.parse().ok())
        .collect();
    assert_eq!(rows, [0], "{report}");
    assert!(report.contains("showing first 1 of 2 packets"), "{report}");
}

// Literal Fig11-14 body plus TF-005 and malformed-digit variants; partial
// CLI-104/107 only. Header checksum is unchanged by body mutations.
#[test]
fn every_command_caps_samples_without_losing_counts_or_late_error_threshold() {
    let mut warning = reference(false);
    warning[29] |= 0x80; // Reserved bit15 in first BCD word: TF-005 warning.
    let mut error = reference(false);
    error[28] = 0xFA; // Decimal digit A: malformed, error.
    let bytes = [reference(false), warning.repeat(1003), error, vec![0; 40]].concat();
    let input = Input::new(&bytes);
    for command in [
        "summary",
        "channels",
        "jumps",
        "timeline",
        "csv",
        "correlate",
    ] {
        for (level, expected) in [
            ("error", ExitClass::Findings),
            ("never", ExitClass::Success),
        ] {
            let mut args = vec![command, input.0.to_str().unwrap()];
            if command == "correlate" {
                args.push("0");
            }
            args.extend(["--fail-on", level]);
            let (mut out, mut err) = (Vec::new(), Vec::new());
            assert_eq!(
                run(&args, &mut out, &mut err),
                expected,
                "{command} {level}"
            );
            let (out, err) = (
                String::from_utf8(out).unwrap(),
                String::from_utf8(err).unwrap(),
            );
            let diagnostics = if command == "summary" {
                out.rsplit_once("Errors (").unwrap().1
            } else {
                &err
            };
            assert!(
                diagnostics.contains("Findings: error=1, warning=1003, info=1 (total=1005)"),
                "{command}: {diagnostics}"
            );
            assert!(
                diagnostics.contains("985 more findings"),
                "{command}: {diagnostics}"
            );
            assert!(
                diagnostics.lines().count() <= 24,
                "{command}: {} lines",
                diagnostics.lines().count()
            );
            assert!(!diagnostics.contains("invalid BCD digit")); // late error is not sampled
        }
    }
}

#[test]
fn jump_findings_are_counted_individually_not_as_an_aggregate_flag() {
    let mut next = reference(false);
    next[29] = 0x30; // +3 seconds at the same RTC: first jump.
    let mut last = reference(false);
    last[29] = 0x33; // +3 seconds again: second jump.
    let input = Input::new(&[reference(false), next, last].concat());
    for command in ["summary", "jumps"] {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert_eq!(
            run(
                &[command, input.0.to_str().unwrap(), "--fail-on", "warning"],
                &mut out,
                &mut err
            ),
            ExitClass::Findings
        );
        let text = format!(
            "{}{}",
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap()
        );
        assert!(
            text.contains("Findings: error=0, warning=2, info=0 (total=2)"),
            "{command}: {text}"
        );
    }
}

#[test]
fn query_context_finding_is_included_in_the_final_counts() {
    let input = Input::new(&common::NO_TIME);
    let mut err = Vec::new();
    assert_eq!(
        run(
            &["correlate", input.0.to_str().unwrap(), "0"],
            &mut Vec::new(),
            &mut err
        ),
        ExitClass::Success
    );
    let text = String::from_utf8(err).unwrap();
    assert!(
        text.contains("Findings: error=0, warning=2, info=0 (total=2)"),
        "{text}"
    );
    assert!(text.contains("TF-001") && text.contains("No time reference points"));
}

#[test]
fn retained_reference_count_and_old_query_finding() {
    let mut bytes = Vec::new();
    for i in 0..1100u64 {
        let mut p = reference(false);
        p[16..22].copy_from_slice(&(i * 10_000_000).to_le_bytes()[..6]);
        let sum = p[..22]
            .chunks_exact(2)
            .map(|w| u16::from_le_bytes([w[0], w[1]]))
            .fold(0u16, u16::wrapping_add);
        p[22..24].copy_from_slice(&sum.to_le_bytes());
        bytes.extend(p);
    }
    let input = Input::new(&bytes);
    let mut out = Vec::new();
    assert_eq!(
        run(
            &["summary", input.0.to_str().unwrap()],
            &mut out,
            &mut Vec::new()
        ),
        ExitClass::Success
    );
    let text = String::from_utf8(out).unwrap();
    let retained: usize = text
        .lines()
        .find_map(|l| l.strip_prefix("Correlation Refs   : "))
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(
        retained, 31,
        "30-second inclusive window within the 1024 cap"
    );
    for level in ["error", "warning"] {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert_eq!(
            run(
                &[
                    "correlate",
                    input.0.to_str().unwrap(),
                    "0",
                    "--fail-on",
                    level
                ],
                &mut out,
                &mut err
            ),
            ExitClass::Success
        );
        assert!(String::from_utf8(out).unwrap().contains("13:45:27.350.000"));
        assert!(err.is_empty());
    }
}

#[test]
fn jump_details_are_sampled_but_counts_remain_exact() {
    let mut bytes = Vec::new();
    for i in 0..201 {
        let mut p = reference(false);
        p[29] = if i % 2 == 0 { 0x20 } else { 0x25 }; // Fig11-14 seconds BCD, +/-5s.
        bytes.extend(p);
    }
    let input = Input::new(&bytes);
    for command in ["summary", "jumps"] {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert_eq!(
            run(
                &[command, input.0.to_str().unwrap(), "--fail-on", "warning"],
                &mut out,
                &mut err
            ),
            ExitClass::Findings
        );
        let text = String::from_utf8(out).unwrap();
        assert!(
            text.lines().filter(|l| l.contains("Delta:")).count() <= 20,
            "{command}"
        );
        assert!(text.contains("180 more jump details"), "{text}");
        let diagnostics = format!("{text}{}", String::from_utf8(err).unwrap());
        assert!(diagnostics.contains("warning=200"), "{diagnostics}");
    }
}

#[test]
fn counter_only_row_is_an_approved_warning_not_a_decode_error() {
    let input = Input::new(&common::NO_TIME);
    for command in ["csv", "timeline"] {
        for (level, exit) in [
            ("error", ExitClass::Success),
            ("warning", ExitClass::Findings),
        ] {
            let (mut out, mut err) = (Vec::new(), Vec::new());
            assert_eq!(
                run(
                    &[command, input.0.to_str().unwrap(), "--fail-on", level],
                    &mut out,
                    &mut err
                ),
                exit
            );
            let text = String::from_utf8(err).unwrap();
            assert!(
                text.contains("Warning: Correlation query: no time reference point"),
                "{text}"
            );
            assert!(text.contains("error=0, warning=2"), "{text}"); // row + TF-001
        }
    }
}

fn doy_packet(channel: u16, rtc_seconds: u64, day: u16, seconds_of_day: u32) -> Vec<u8> {
    // Fig11-13 (§11.2.3.2): fractions, seconds, minutes, hours, BCD DOY.
    // 10 data bytes; the last two bytes are filler and must not be decoded.
    let bcd = |n: u32| ((n / 10) << 4 | (n % 10)) as u8;
    let mut p = reference(false);
    p[2..4].copy_from_slice(&channel.to_le_bytes());
    p[8..12].copy_from_slice(&10u32.to_le_bytes());
    p[16..22].copy_from_slice(&(rtc_seconds * 10_000_000).to_le_bytes()[..6]);
    p[24..].fill(0);
    p[29] = bcd(seconds_of_day % 60);
    p[30] = bcd(seconds_of_day / 60 % 60);
    p[31] = bcd(seconds_of_day / 3600);
    p[32] = bcd(u32::from(day % 100));
    p[33] = (day / 100) as u8;
    p[34..].fill(0xDD);
    let sum = p[..22]
        .chunks_exact(2)
        .map(|w| u16::from_le_bytes([w[0], w[1]]))
        .fold(0u16, u16::wrapping_add);
    p[22..24].copy_from_slice(&sum.to_le_bytes());
    p
}

#[test]
fn late_references_skip_jump_and_gap_pairs_inside_and_outside_window() {
    for latest in [2, 40] {
        let bytes = [
            doy_packet(1, 0, 100, 43200),
            doy_packet(1, latest, 100, 43200 + latest as u32),
            common::NO_TIME.to_vec(),
            doy_packet(1, 1, 100, 43201),
        ]
        .concat();
        let input = Input::new(&bytes);
        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert_eq!(
            run(
                &["summary", input.0.to_str().unwrap(), "--fail-on", "warning"],
                &mut out,
                &mut err
            ),
            ExitClass::Findings
        );
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("None detected"), "{text}");
        assert!(text.contains("warning=1"), "{text}");
        assert!(
            text.contains(&format!("max={} ns", latest * 1_000_000_000)),
            "{text}"
        );
        assert!(
            text.contains(&format!(
                "Out-of-order time reference: channel 1 RTC {:#X} -> {:#X} at packet 3",
                latest * 10_000_000,
                10_000_000
            )),
            "{text}"
        );
        assert!(!text.contains("Delta:"));
        if latest == 40 {
            let mut err = Vec::new();
            assert_eq!(
                run(
                    &[
                        "timeline",
                        input.0.to_str().unwrap(),
                        "--fail-on",
                        "warning"
                    ],
                    &mut Vec::new(),
                    &mut err
                ),
                ExitClass::Findings
            );
            assert!(
                String::from_utf8(err).unwrap().contains(
                    "Warning: Correlation query: query precedes retained reference window"
                )
            );
        }
    }
}

#[test]
fn old_target_at_five_seconds_matches_main_on_one_and_two_channels() {
    for channels in [1, 2] {
        let mut bytes = Vec::new();
        for second in 1..=120 {
            for channel in 1..=channels {
                bytes.extend(doy_packet(channel, second, 100, 43200 + second as u32 - 1));
            }
        }
        let input = Input::new(&bytes);
        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert_eq!(
            run(
                &[
                    "correlate",
                    input.0.to_str().unwrap(),
                    "2FAF080",
                    "--fail-on",
                    "warning"
                ],
                &mut out,
                &mut err
            ),
            ExitClass::Success
        );
        let text = String::from_utf8(out).unwrap();
        assert_eq!(
            text.matches("Day 100 12:00:04.000.000").count(),
            usize::from(channels + 1),
            "{text}"
        );
        assert!(err.is_empty(), "{}", String::from_utf8_lossy(&err));
    }
}

#[test]
fn midnight_crossing_is_not_a_negative_day_jump() {
    let bytes = [doy_packet(1, 0, 100, 86399), doy_packet(1, 1, 101, 0)].concat();
    let input = Input::new(&bytes);
    for command in ["summary", "jumps"] {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert_eq!(
            run(
                &[command, input.0.to_str().unwrap(), "--fail-on", "warning"],
                &mut out,
                &mut err
            ),
            ExitClass::Success
        );
        assert!(!String::from_utf8(out).unwrap().contains("Delta:"));
        assert!(err.is_empty());
    }
}

fn one_hz_file_with_late_reference(late_time_offset: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    for second in 0..=20 {
        bytes.extend(doy_packet(1, second, 100, 43200 + second as u32));
    }
    bytes.extend(doy_packet(1, 8, 100, 43208 + late_time_offset));
    bytes.extend(doy_packet(1, 21, 100, 43221));
    bytes
}

#[test]
fn late_reference_does_not_inflate_next_gap() {
    // Fig11-13 clock labels at 1 Hz; late RTC8 follows RTC20, then RTC21.
    let input = Input::new(&one_hz_file_with_late_reference(0));
    let (mut out, mut err) = (Vec::new(), Vec::new());
    assert_eq!(
        run(
            &["summary", input.0.to_str().unwrap(), "--fail-on", "warning"],
            &mut out,
            &mut err
        ),
        ExitClass::Findings
    );
    let text = String::from_utf8(out).unwrap();
    assert!(
        text.contains("Observed RTC gaps: min=1000000000 ns max=1000000000 ns"),
        "{text}"
    );
    assert!(text.contains("warning=1"), "{text}");
    assert!(!text.contains("Delta:"), "{text}");
}

#[test]
fn inconsistent_late_reference_does_not_poison_next_jump() {
    let input = Input::new(&one_hz_file_with_late_reference(300));
    for command in ["summary", "jumps"] {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert_eq!(
            run(
                &[command, input.0.to_str().unwrap(), "--fail-on", "warning"],
                &mut out,
                &mut err
            ),
            ExitClass::Findings
        );
        let text = format!(
            "{}{}",
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap()
        );
        assert!(!text.contains("Delta:"), "{text}");
        assert!(text.contains("warning=1"), "{text}");
        assert!(text.contains("Out-of-order time reference"));
    }
}

#[test]
fn consecutive_late_references_compare_against_pair_baseline() {
    // Fig11-13 DOY clock labels: RTC15 -> 3 -> 4 -> 16 seconds.
    let bytes: Vec<u8> = [15, 3, 4, 16]
        .into_iter()
        .flat_map(|second| doy_packet(1, second, 100, 43200 + second as u32))
        .collect();
    let input = Input::new(&bytes);
    for command in ["summary", "jumps"] {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert_eq!(
            run(
                &[command, input.0.to_str().unwrap(), "--fail-on", "warning"],
                &mut out,
                &mut err
            ),
            ExitClass::Findings
        );
        let text = format!(
            "{}{}",
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap()
        );
        assert!(
            text.contains("Findings: error=0, warning=2, info=0 (total=2)"),
            "{text}"
        );
        assert_eq!(
            text.matches("Out-of-order time reference").count(),
            2,
            "{text}"
        );
        assert!(text.contains("at packet 1"));
        assert!(text.contains("at packet 2"));
        assert!(!text.contains("Delta:"), "{text}");
        if command == "summary" {
            assert!(
                text.contains("Observed RTC gaps: min=1000000000 ns max=1000000000 ns"),
                "{text}"
            );
        }
    }
}

#[test]
fn rtc_reset_single_step_and_three_reference_run_keep_real_jump_only() {
    for last in [20u64, 40] {
        let mut bytes = Vec::new();
        for seconds in 0..=last {
            bytes.extend(doy_packet(1, seconds, 100, 43200 + seconds as u32));
        }
        for seconds in 1..=4 {
            // New epoch clock starts +100s; only its RTC4 point adds a +5s step.
            bytes.extend(doy_packet(
                1,
                seconds,
                100,
                43300 + seconds as u32 + if seconds == 4 { 5 } else { 0 },
            ));
        }
        let input = Input::new(&bytes);
        for command in ["summary", "jumps"] {
            let (mut out, mut err) = (Vec::new(), Vec::new());
            assert_eq!(
                run(
                    &[command, input.0.to_str().unwrap(), "--fail-on", "warning"],
                    &mut out,
                    &mut err
                ),
                ExitClass::Findings
            );
            let text = format!(
                "{}{}",
                String::from_utf8(out).unwrap(),
                String::from_utf8(err).unwrap()
            );
            assert_eq!(text.matches("RTC reset on channel 1").count(), 1, "{text}");
            assert!(
                text.contains(&format!(
                    "at packet {}: {:#X} -> {:#X}",
                    last + 1,
                    last * 10_000_000,
                    10_000_000
                )),
                "{text}"
            );
            assert!(!text.contains("Out-of-order"), "{text}");
            assert_eq!(text.matches("Delta:").count(), 1, "{text}");
            assert!(text.contains("Delta: +5000.000 ms"), "{text}");
            assert!(text.contains("warning=2"), "{text}");
            if command == "summary" {
                assert!(text.contains("max=1000000000 ns"), "{text}");
            }
        }
    }
}
