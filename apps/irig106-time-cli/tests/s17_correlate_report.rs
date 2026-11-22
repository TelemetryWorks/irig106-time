//! Partial CLI-106 correlate report parity; no whole-requirement marker.
use irig106_time_cli::{ExitClass, run};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
struct Input(PathBuf);
impl Input {
    fn new(packets: &[(u16, u64, u32)]) -> Self {
        Self::bytes(
            &packets
                .iter()
                .flat_map(|&(ch, rtc, sec)| packet(ch, rtc, sec))
                .collect::<Vec<_>>(),
        )
    }
    fn bytes(bytes: &[u8]) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "s17-query-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, bytes).unwrap();
        Self(path)
    }
    fn json_at(&self, seconds: u64) -> (ExitClass, serde_json::Value, String) {
        let target = format!("{:X}", seconds * 10_000_000);
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let exit = run(
            &[
                "correlate",
                self.0.to_str().unwrap(),
                &target,
                "--format",
                "json",
                "--fail-on",
                "warning",
            ],
            &mut out,
            &mut err,
        );
        assert_ne!(exit, ExitClass::Usage, "{}", String::from_utf8_lossy(&err));
        assert!(out.is_ascii());
        (
            exit,
            serde_json::from_slice(&out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }
    fn query_at(&self, seconds: u64) -> (ExitClass, String, String) {
        let target = format!("{:X}", seconds * 10_000_000);
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let exit = run(
            &[
                "correlate",
                self.0.to_str().unwrap(),
                &target,
                "--fail-on",
                "warning",
            ],
            &mut out,
            &mut err,
        );
        (
            exit,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }
    fn query(&self) -> (String, String) {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert_eq!(
            run(
                &[
                    "correlate",
                    self.0.to_str().unwrap(),
                    "2FAF080",
                    "--fail-on",
                    "never"
                ],
                &mut out,
                &mut err
            ),
            ExitClass::Success
        );
        (
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }
}
impl Drop for Input {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
fn packet(ch: u16, rtc: u64, sec: u32) -> Vec<u8> {
    // 106-24R1 §11.2.1.1, Figures11-12/13: FMT4/DATE0, DOY100,
    // specified seconds after 12:00, RTC100ns ticks; Data Length10 plus filler2.
    let bcd = |v: u32| ((v / 10) * 16 + v % 10) as u8;
    let mut p = vec![0; 36];
    p[..2].copy_from_slice(&[0x25, 0xeb]);
    p[2..4].copy_from_slice(&ch.to_le_bytes());
    p[4] = 36;
    p[8] = 10;
    p[15] = 0x11;
    p[16..22].copy_from_slice(&(rtc * 10_000_000).to_le_bytes()[..6]);
    p[24..34].copy_from_slice(&[
        0x40,
        0,
        0,
        0,
        0,
        bcd(sec % 60),
        bcd(sec / 60 % 60),
        bcd(12 + sec / 3600),
        0,
        1,
    ]);
    let sum = p[..22].chunks_exact(2).fold(0u16, |s, w| {
        s.wrapping_add(u16::from_le_bytes([w[0], w[1]]))
    });
    p[22..24].copy_from_slice(&sum.to_le_bytes());
    p
}
#[test]
fn in_order_query_keeps_any_and_each_channel_text_exact() {
    let (out, err) = Input::new(&[(7, 0, 0), (8, 0, 3600), (7, 40, 40), (8, 41, 3641)]).query();
    assert_eq!(
        out,
        "Resolving RTC 0x000002FAF080 (50000000 ticks, 5000000000 ns)\n\n  Any channel  → 100:12:00:05.000000000 UTC (stated FMT)\n  Channel   7  → 100:12:00:05.000000000 UTC (stated FMT)\n  Channel   8  → 100:13:00:05.000000000 UTC (stated FMT)\n"
    );
    assert_eq!(err, "");
}
#[test]
fn reset_query_keeps_both_segment_answers_and_other_channel_exact() {
    // >30s backward step40->1 starts segment2; RTC5 lies in both7 ranges.
    let (out, err) = Input::new(&[
        (7, 0, 0),
        (8, 0, 3600),
        (7, 40, 40),
        (7, 1, 61),
        (7, 10, 70),
        (8, 41, 3641),
    ])
    .query();
    assert_eq!(
        out,
        "Resolving RTC 0x000002FAF080 (50000000 ticks, 5000000000 ns)\n\n  Channel   7 Segment   1  → 100:12:00:05.000000000 UTC (stated FMT)\n  Channel   7 Segment   2  → 100:12:01:05.000000000 UTC (stated FMT)\n  Channel   8  → 100:13:00:05.000000000 UTC (stated FMT)\n"
    );
    assert!(err.contains("warning=2"), "{err}");
    assert!(err.contains("RTC is ambiguous across reset segments on channel 7"));
    assert_eq!(
        out,
        Input::new(&[
            (7, 0, 0),
            (8, 0, 3600),
            (7, 40, 40),
            (7, 1, 61),
            (7, 10, 70),
            (8, 41, 3641)
        ])
        .query()
        .0
    );
}

#[test]
fn correlate_report_writer_failure_still_overrides_findings() {
    struct Broken;
    impl std::io::Write for Broken {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("query writer probe"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let input = Input::new(&[(7, 0, 0), (7, 40, 40), (7, 1, 61), (7, 10, 70)]);
    assert_eq!(
        run(
            &[
                "correlate",
                input.0.to_str().unwrap(),
                "2FAF080",
                "--fail-on",
                "warning"
            ],
            &mut Broken,
            &mut Vec::new()
        ),
        ExitClass::Output
    );
}

#[test]
fn reset_findings_precede_answer_body_in_merged_output() {
    use std::{
        cell::RefCell,
        io::{self, Write},
    };
    struct Shared<'a>(&'a RefCell<Vec<u8>>);
    impl Write for Shared<'_> {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0.borrow_mut().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let input = Input::new(&[(7, 0, 0), (7, 40, 40), (7, 1, 61), (7, 10, 70)]);
    let bytes = RefCell::new(Vec::new());
    assert_eq!(
        run(
            &[
                "correlate",
                input.0.to_str().unwrap(),
                "2FAF080",
                "--fail-on",
                "never"
            ],
            &mut Shared(&bytes),
            &mut Shared(&bytes)
        ),
        ExitClass::Success
    );
    let text = String::from_utf8(bytes.into_inner()).unwrap();
    assert!(text.find("Resolving RTC").unwrap() < text.find("Findings:").unwrap());
    assert!(
        text.find("Findings:").unwrap() < text.find("Channel   7 Segment   1").unwrap(),
        "{text}"
    );
}

fn year_boundary_packet(rtc: u64, day: u16, hour: u8, minute: u8, second: u8) -> Vec<u8> {
    // Fig11-13: sec/min/hour BCD; low day byte tens/units, next byte hundreds.
    // Change payload only: §11.2.1.1 primary header checksum remains unchanged.
    let bcd = |n: u16| ((n / 10) * 16 + n % 10) as u8;
    let mut p = packet(7, rtc, 0);
    p[29] = bcd(u16::from(second));
    p[30] = bcd(u16::from(minute));
    p[31] = bcd(u16::from(hour));
    p[32] = bcd(day % 100);
    p[33] = (day / 100) as u8;
    p
}

#[test]
fn any_channel_calendar_context_error_is_counted_and_trips_warning_threshold() {
    // Twenty 1Hz references end at unknown-year DOY365 23:59:49 (RTC19s).
    // Query RTC60s requires +41s, crossing the unknown year; both the any-channel
    // and per-channel queries must independently report their context warning.
    let bytes = (0..20)
        .flat_map(|rtc| year_boundary_packet(rtc, 365, 23, 59, 30 + rtc as u8))
        .collect::<Vec<_>>();
    let (exit, out, err) = Input::bytes(&bytes).query_at(60);
    assert_eq!(exit, ExitClass::Findings);
    assert!(
        out.lines().any(|line| line
            == "  Any channel  → Warning: time needs context: unknown year at calendar boundary"),
        "{out}"
    );
    assert!(
        err.contains("Findings: error=0, warning=2, info=0 (total=2)"),
        "{err}"
    );
    assert_eq!(err.matches("Finding: Warning: Correlation query: time needs context: unknown year at calendar boundary").count(),2,"{err}");
}

#[test]
fn reset_segment_calendar_context_error_is_rendered_unresolved() {
    let mut bytes = (0..6)
        .flat_map(|rtc| year_boundary_packet(rtc, 365, 23, 59, 50 + rtc as u8))
        .collect::<Vec<_>>();
    for rtc in [20, 21] {
        bytes.extend(year_boundary_packet(rtc, 1, 0, 0, (rtc - 10) as u8));
    }
    // Accepted short-reset rule: three increasing references below RTC21s,
    // beginning at RTC0s, establish segment2. Its range does not cover RTC10..12.
    for rtc in 0..3 {
        bytes.extend(year_boundary_packet(rtc, 1, 0, 1, rtc as u8));
    }
    let input = Input::bytes(&bytes);
    for target in 10..=12 {
        let (exit, out, err) = input.query_at(target);
        assert_eq!(exit, ExitClass::Findings);
        assert!(out.lines().any(|line|line=="  Channel   7 Segment   1  → unresolved: time needs context: unknown year at calendar boundary"),"{out}");
        assert!(err.contains("Finding: Warning: Segment query: time needs context: unknown year at calendar boundary"),"{err}");
    }
}

fn query_clock(hour: u8, minute: u8, second: u8) -> serde_json::Value {
    // DOY100 starts after99 days. Expected native count; UTC is stated by fixture FMT4, not a year inference.
    let ns = (99u64 * 86400 + u64::from(hour) * 3600 + u64::from(minute) * 60 + u64::from(second))
        * 1_000_000_000;
    serde_json::json!({"year":null,"day":100,"hour":hour,"minute":minute,"second":second,"nanosecond":0,
        "clock_count_ns":ns.to_string(),"scale":"UTC","basis":"stated FMT","year_offset":"0"})
}
#[test]
fn correlate_json_matches_native_answers_with_all_query_fields() {
    use serde_json::json;
    let input = Input::new(&[(7, 0, 0), (8, 0, 3600), (7, 40, 40), (8, 41, 3641)]);
    let (exit, v, err) = input.json_at(5);
    assert_eq!(exit, ExitClass::Success);
    assert_eq!(err, input.query_at(5).2);
    assert_eq!(v["schema_revision"], 12);
    assert_eq!(v["options"]["query_rtc"], "50000000");
    assert!(v["options"]["limit"].is_null());
    assert!(v["options"]["threshold_ms"].is_null());
    assert!(v["report"]["rows"].is_null());
    assert!(v["report"]["jumps"].is_null());
    assert_eq!(
        v["report"]["correlation"],
        json!({
        "any":{"network_time":null,"clock":query_clock(12,0,5),"issue":null},
        "answers":[
            {"channel":7,"segment":"1","labelled":false,"network_time":null,"clock":query_clock(12,0,5),"issue":null},
            {"channel":8,"segment":"1","labelled":false,"network_time":null,"clock":query_clock(13,0,5),"issue":null}]})
    );
    let schema: serde_json::Value =
        serde_json::from_str(include_str!("../schema/irigtime-report-v1.schema.json")).unwrap();
    for (value, definition) in [
        (&v["report"]["correlation"], "query_report"),
        (&v["report"]["correlation"]["any"], "query_value"),
        (&v["report"]["correlation"]["answers"][0], "query_answer"),
    ] {
        assert_eq!(
            value.as_object().unwrap().keys().collect::<Vec<_>>(),
            schema["$defs"][definition]["properties"]
                .as_object()
                .unwrap()
                .keys()
                .collect::<Vec<_>>()
        );
    }
    assert_eq!(v, input.json_at(5).1);
}
#[test]
fn correlate_json_reset_golden_keeps_each_segment_and_unreset_channel() {
    let input = Input::new(&[
        (7, 0, 0),
        (8, 0, 3600),
        (7, 40, 40),
        (7, 1, 61),
        (7, 10, 70),
        (8, 41, 3641),
    ]);
    let (exit, v, err) = input.json_at(5);
    assert_eq!(exit, ExitClass::Findings);
    assert_eq!(err, input.query_at(5).2);
    assert_eq!(v["findings"]["warning"], "2");
    assert_eq!(
        v["report"]["correlation"],
        serde_json::json!({
        "any":{"network_time":null,"clock":null,"issue":"Any-channel answer withheld across RTC resets; use labelled segment answers"},
        "answers":[
            {"channel":7,"segment":"1","labelled":true,"network_time":null,"clock":query_clock(12,0,5),"issue":null},
            {"channel":7,"segment":"2","labelled":true,"network_time":null,"clock":query_clock(12,1,5),"issue":null},
            {"channel":8,"segment":"1","labelled":false,"network_time":null,"clock":query_clock(13,0,5),"issue":null}]})
    );
    assert_eq!(
        v["report"]["channels"][0]["segments"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}
#[test]
fn correlate_json_preserves_context_errors_and_no_reference_reason() {
    let bytes = (0..20)
        .flat_map(|rtc| year_boundary_packet(rtc, 365, 23, 59, 30 + rtc as u8))
        .collect::<Vec<_>>();
    let input = Input::bytes(&bytes);
    let (exit, v, err) = input.json_at(60);
    assert_eq!(exit, ExitClass::Findings);
    assert_eq!(err, input.query_at(60).2);
    assert_eq!(v["findings"]["warning"], "2");
    assert_eq!(
        v["report"]["correlation"],
        serde_json::json!({"any":{"network_time":null,"clock":null,"issue":"time needs context: unknown year at calendar boundary"},"answers":[]})
    );
    let mut bytes = (0..6)
        .flat_map(|rtc| year_boundary_packet(rtc, 365, 23, 59, 50 + rtc as u8))
        .collect::<Vec<_>>();
    for rtc in [20, 21] {
        bytes.extend(year_boundary_packet(rtc, 1, 0, 0, (rtc - 10) as u8));
    }
    for rtc in 0..3 {
        bytes.extend(year_boundary_packet(rtc, 1, 0, 1, rtc as u8));
    }
    let (_, v, _) = Input::bytes(&bytes).json_at(12);
    assert_eq!(
        v["report"]["correlation"]["answers"],
        serde_json::json!([{"channel":7,"segment":"1","labelled":true,"network_time":null,"clock":null,"issue":"time needs context: unknown year at calendar boundary"}])
    );
    // §11.2.1.1: valid PCM header, zero Data Length, checksum EB25+18+900=F43D.
    let (_, v, _) = Input::bytes(&[
        0x25, 0xeb, 0, 0, 24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 0, 0, 0, 0, 0x3d, 0xf4,
    ])
    .json_at(0);
    assert_eq!(
        v["report"]["correlation"],
        serde_json::json!({"any":{"network_time":null,"clock":null,"issue":"No time reference points found in file"},"answers":[]})
    );
}
#[test]
fn correlate_json_usage_and_writer_failure_keep_exit_classes() {
    struct Broken;
    impl std::io::Write for Broken {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("query JSON writer"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let input = Input::new(&[(7, 0, 0), (7, 40, 40)]);
    assert_eq!(
        run(
            &[
                "correlate",
                input.0.to_str().unwrap(),
                "2FAF080",
                "--format",
                "json"
            ],
            &mut Broken,
            &mut Vec::new()
        ),
        ExitClass::Output
    );
    for tail in [
        vec!["--format", "JSON"],
        vec!["--format", "json", "--format", "text"],
        vec!["--format"],
    ] {
        let mut args = vec!["correlate", "/missing-input", "0"];
        args.extend(tail);
        assert_eq!(
            run(&args, &mut Vec::new(), &mut Vec::new()),
            ExitClass::Usage
        );
    }
}
