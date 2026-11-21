//! Partial CLI-106 packet-row projection; qualified joining/CSV/correlate remain open.
#[path = "common/mod.rs"]
mod common;
use irig106_time_cli::{ExitClass, run};
use serde_json::{Value, json};
use std::{
    io,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
struct Input(PathBuf);
impl Input {
    fn new() -> Self {
        // §11.2.1.1: checksum-valid PCM0, F1 at RTC1s, PCM2 at RTC2s.
        // Fig11-12/13: FMT4 DATE0, day100 12:00:04.12 UTC, unknown year.
        let mut p = vec![0; 36];
        p[..2].copy_from_slice(&[0x25, 0xeb]);
        p[2] = 7;
        p[4] = 36;
        p[8] = 10;
        p[15] = 0x11;
        p[16..22].copy_from_slice(&10_000_000u64.to_le_bytes()[..6]);
        p[24..34].copy_from_slice(&[0x40, 0, 0, 0, 0x12, 4, 0, 0x12, 0, 1]);
        checksum(&mut p);
        let mut data = common::NO_TIME;
        data[16..22].copy_from_slice(&20_000_000u64.to_le_bytes()[..6]);
        checksum(&mut data);
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "s17-row-json-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, [common::NO_TIME.as_slice(), &p, &data].concat()).unwrap();
        Self(path)
    }
    fn output(&self, format: &str, limit: &str) -> (ExitClass, Vec<u8>, Vec<u8>) {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let exit = run(
            &[
                "timeline",
                self.0.to_str().unwrap(),
                "--format",
                format,
                "--limit",
                limit,
                "--fail-on",
                "warning",
            ],
            &mut out,
            &mut err,
        );
        (exit, out, err)
    }
}
impl Drop for Input {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
fn checksum(p: &mut [u8]) {
    let sum = p[..22].chunks_exact(2).fold(0u16, |s, w| {
        s.wrapping_add(u16::from_le_bytes([w[0], w[1]]))
    });
    p[22..24].copy_from_slice(&sum.to_le_bytes());
}
#[test]
fn timeline_rows_golden_pins_counter_and_clock_without_invented_association() {
    let input = Input::new();
    let (exit, out, err) = input.output("json", "3");
    assert_eq!(exit, ExitClass::Findings);
    assert!(out.is_ascii());
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["schema_revision"], 3);
    assert_eq!(v["options"]["limit"], "3");
    assert!(v["options"]["threshold_ms"].is_null());
    assert!(v["report"]["jumps"].is_null());
    assert_eq!(v["report"]["total_packets"], "3");
    assert_eq!(
        v["report"]["rows"],
        json!([
            {"kind":"counter-only", "packet":"0", "channel":0, "rtc":"0", "offset_bytes":"0", "data_type":9,
             "association":"association_unestablished", "clock":null, "candidates":[], "issue":"No resolved clock; see command findings"},
            {"kind":"absolute", "packet":"1", "channel":7, "rtc":"10000000", "offset_bytes":"24", "data_type":17,
             "association":"association_unestablished", "candidates":[], "issue":null,
             "clock":{"year":null,"day":100,"hour":12,"minute":0,"second":4,"nanosecond":120000000,
                      "clock_count_ns":"8596804120000000","scale":"unknown","basis":"unresolved","year_offset":"0"}},
            {"kind":"absolute", "packet":"2", "channel":0, "rtc":"20000000", "offset_bytes":"60", "data_type":9,
             "association":"association_unestablished", "candidates":[], "issue":null,
             "clock":{"year":null,"day":100,"hour":12,"minute":0,"second":5,"nanosecond":120000000,
                      "clock_count_ns":"8596805120000000","scale":"unknown","basis":"unresolved","year_offset":"0"}}
        ])
    );
    let (text_exit, text, text_err) = input.output("text", "3");
    assert_eq!(text_exit, exit);
    assert_eq!(text_err, err);
    let text = String::from_utf8(text).unwrap();
    assert!(text.contains("N/A"));
    assert!(text.contains("100:12:00:04.120000000"));
    assert!(text.contains("100:12:00:05.120000000"));
    assert_eq!(out, input.output("json", "3").1);
}
#[test]
fn row_limit_does_not_stop_input_analysis() {
    let input = Input::new();
    let (exit, out, _) = input.output("json", "1");
    assert_eq!(exit, ExitClass::Findings);
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["report"]["rows"].as_array().unwrap().len(), 1);
    assert_eq!(v["options"]["limit"], "1");
    assert_eq!(v["report"]["total_packets"], "3");
    assert_eq!(v["report"]["time_packets"], "1");
    assert_eq!(
        v["report"]["channels"][0]["segments"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}
#[test]
fn timeline_json_writer_failure_is_output_and_format_validation_precedes_input() {
    struct Broken;
    impl io::Write for Broken {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("writer probe"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let input = Input::new();
    assert_eq!(
        run(
            &["timeline", input.0.to_str().unwrap(), "--format", "json"],
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
        let mut args = vec!["timeline", "/missing-input"];
        args.extend(tail);
        assert_eq!(
            run(&args, &mut Vec::new(), &mut Vec::new()),
            ExitClass::Usage
        );
    }
}
