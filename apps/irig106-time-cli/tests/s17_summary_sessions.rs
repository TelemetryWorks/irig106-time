//! Native session span clause of CLI-104/106 only; quality remains separate.
use irig106_time_cli::{ExitClass, run};
use serde_json::{Value, json};
use std::{
    io::{self, Write},
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
struct Input(PathBuf);
impl Drop for Input {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
fn packet(channel: u16, kind: u8, ticks: u64) -> Vec<u8> {
    // 106-24R1 §11.2.1.1 header/checksum/RTC; Fig11-12/13 FMT4/SRC1
    // DOY10012:00, Data Length10 excludes2 filler bytes. Data9 is opaque.
    let body: &[u8] = if kind == 0x11 {
        &[0x41, 0, 0, 0, 0, 0, 0, 0x12, 0, 1]
    } else {
        &[]
    };
    let mut p = vec![0; (24 + body.len()).next_multiple_of(4)];
    p[..2].copy_from_slice(&[0x25, 0xeb]);
    p[2..4].copy_from_slice(&channel.to_le_bytes());
    let len = p.len() as u32;
    p[4..8].copy_from_slice(&len.to_le_bytes());
    p[8..12].copy_from_slice(&(body.len() as u32).to_le_bytes());
    p[12] = 6;
    p[15] = kind;
    p[16..22].copy_from_slice(&ticks.to_le_bytes()[..6]);
    let sum = p[..22].chunks_exact(2).fold(0u16, |s, w| {
        s.wrapping_add(u16::from_le_bytes([w[0], w[1]]))
    });
    p[22..24].copy_from_slice(&sum.to_le_bytes());
    p[24..24 + body.len()].copy_from_slice(body);
    p
}
impl Input {
    fn new(rows: &[(u16, u8, u64)]) -> Self {
        static N: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "s17-native-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(
            &path,
            rows.iter()
                .flat_map(|&(c, k, r)| packet(c, k, r))
                .collect::<Vec<_>>(),
        )
        .unwrap();
        Self(path)
    }
    fn output(&self, command: &str, json: bool) -> (ExitClass, String) {
        let mut a = vec![command, self.0.to_str().unwrap()];
        if command == "correlate" {
            a.push("0");
        }
        if json {
            a.extend(["--format", "json"]);
        }
        a.extend(["--fail-on", "never"]);
        let mut out = Vec::new();
        let exit = run(&a, &mut out, &mut Vec::new());
        (exit, String::from_utf8(out).unwrap())
    }
}
#[test]
fn native_recording_spans_are_not_channel_reset_segments() {
    let input = Input::new(&[
        (7, 0x11, 0),
        (7, 0x11, 200_000_000),
        (7, 0x11, 180_000_000),
        (8, 9, 185_000_000),
    ]);
    let (exit, text) = input.output("summary", false);
    assert_eq!(exit, ExitClass::Success);
    assert!(text.contains("Recording-wide native sessions"));
    assert!(text.contains("Per-channel CLI reset segments"));
    let (_, out) = input.output("summary", true);
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["report"]["reset_count"], "0");
    assert_eq!(v["report"]["channels"][0]["segment_count"], "1");
    assert_eq!(
        v["report"]["native_sessions"],
        json!({"count":"2","late_bound_ns":"1100000000",
        "infer_sessions":true,"status":"complete","unavailable_reason":null,"spans":[
        {"index":"1","start":"initial","first_packet":"0","last_packet":"1","first_ticks":"0","last_ticks":"200000000","low_ticks":"0","high_ticks":"200000000","counter_span_ns":"20000000000","packets":"2"},
        {"index":"2","start":"inferred","first_packet":"2","last_packet":"3","first_ticks":"180000000","last_ticks":"185000000","low_ticks":"180000000","high_ticks":"185000000","counter_span_ns":"500000000","packets":"2"}]})
    );
    for command in ["channels", "jumps", "timeline", "correlate"] {
        let (_, out) = input.output(command, true);
        let v: Value = serde_json::from_str(&out).unwrap();
        assert!(v["report"]["native_sessions"].is_null());
    }
}
#[test]
fn forward_wrap_is_one_span_and_ambiguity_keeps_only_a_labelled_prefix() {
    let input = Input::new(&[(8, 9, (1u64 << 48) - 10), (8, 9, 20)]);
    let (_, out) = input.output("summary", true);
    let v: Value = serde_json::from_str(&out).unwrap();
    let n = &v["report"]["native_sessions"];
    assert_eq!(n["count"], "1");
    assert_eq!(
        n["spans"][0]["last_ticks"],
        ((1i128 << 48) + 20).to_string()
    );
    assert_eq!(n["spans"][0]["counter_span_ns"], "3000");
    let input = Input::new(&[(7, 0x11, 0), (8, 9, 1u64 << 47), (8, 9, (1u64 << 47) + 1)]);
    let (exit, out) = input.output("summary", true);
    assert_eq!(exit, ExitClass::Success);
    let v: Value = serde_json::from_str(&out).unwrap();
    let n = &v["report"]["native_sessions"];
    assert_eq!(n["status"], "needs-context");
    assert_eq!(n["count"], "1");
    assert_eq!(n["spans"].as_array().unwrap().len(), 1);
    assert!(n["unavailable_reason"].is_string());
    assert_eq!(n["spans"][0]["packets"], "1");
    assert_eq!(v["report"]["total_packets"], "3");
    let findings = v["findings"]["samples"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| {
            f["message"]
                .as_str()
                .unwrap()
                .contains("Native session analysis needs context")
        })
        .collect::<Vec<_>>();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0]["level"], "warning");
    assert_eq!(findings[0]["channel"], 8);
    assert_eq!(findings[0]["packet"], "1");
    assert!(findings[0]["code"].is_null());
    for (level, expected) in [
        ("error", ExitClass::Success),
        ("warning", ExitClass::Findings),
        ("never", ExitClass::Success),
    ] {
        assert_eq!(
            run(
                &["summary", input.0.to_str().unwrap(), "--fail-on", level],
                &mut Vec::new(),
                &mut Vec::new()
            ),
            expected
        );
    }
}
#[test]
fn native_text_writer_failure_is_output_four() {
    struct FailNative(bool);
    impl Write for FailNative {
        fn write(&mut self, b: &[u8]) -> io::Result<usize> {
            if String::from_utf8_lossy(b).contains("Recording-wide native sessions") {
                self.0 = true;
                return Err(io::Error::other("injected native writer"));
            }
            Ok(b.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let input = Input::new(&[(8, 9, 0)]);
    let mut out = FailNative(false);
    assert_eq!(
        run(
            &["summary", input.0.to_str().unwrap()],
            &mut out,
            &mut Vec::new()
        ),
        ExitClass::Output
    );
    assert!(out.0);
}
