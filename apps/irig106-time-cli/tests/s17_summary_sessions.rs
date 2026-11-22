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
    packet_body(channel, kind, ticks, body)
}
fn packet_body(channel: u16, kind: u8, ticks: u64, body: &[u8]) -> Vec<u8> {
    let mut p = vec![0; (24 + body.len()).next_multiple_of(4)];
    p[..2].copy_from_slice(&[0x25, 0xeb]);
    p[2..4].copy_from_slice(&channel.to_le_bytes());
    let len = p.len() as u32;
    p[4..8].copy_from_slice(&len.to_le_bytes());
    p[8..12].copy_from_slice(&(body.len() as u32).to_le_bytes());
    p[12] = if kind == 0x12 { 8 } else { 6 };
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
        Self::bytes(
            &rows
                .iter()
                .flat_map(|&(c, k, r)| packet(c, k, r))
                .collect::<Vec<_>>(),
        )
    }
    fn bytes(bytes: &[u8]) -> Self {
        static N: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "s17-native-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, bytes).unwrap();
        Self(path)
    }
    fn output(&self, command: &str, json: bool) -> (ExitClass, String) {
        self.era_output(command, json, None)
    }
    fn era_output(&self, command: &str, json: bool, era: Option<&str>) -> (ExitClass, String) {
        let mut a = vec![command, self.0.to_str().unwrap()];
        if command == "correlate" {
            a.push("0");
        }
        if json {
            a.extend(["--format", "json"]);
        }
        if let Some(era) = era {
            a.extend(["--ntp-era", era]);
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
    let mut native = v["report"]["native_sessions"].clone();
    assert_eq!(
        native["spans"][0]["reference_quality"]["channels"][0]["valid"],
        "2"
    );
    assert_eq!(
        native["spans"][1]["reference_quality"]["channels"][0]["valid"],
        "1"
    );
    for span in native["spans"].as_array_mut().unwrap() {
        assert!(span["reference_quality"]["channels"].is_array());
        span.as_object_mut().unwrap().remove("reference_quality");
    }
    assert_eq!(
        native,
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

#[test]
fn late_header_changes_low_ticks_without_moving_first_observation() {
    // §11.2.1.1 i:100ns RTC ticks. 1.0s ->0.5s is a -5_000_000-tick
    // late displacement within the native1.1s bound, not a new session.
    let input = Input::new(&[(8, 9, 10_000_000), (8, 9, 5_000_000)]);
    let (exit, out) = input.output("summary", true);
    assert_eq!(exit, ExitClass::Success);
    let v: Value = serde_json::from_str(&out).unwrap();
    let native = &v["report"]["native_sessions"];
    assert_eq!(native["count"], "1");
    let span = &native["spans"][0];
    assert_eq!(span["first_ticks"], "10000000");
    assert_eq!(span["low_ticks"], "5000000");
    assert_eq!(span["high_ticks"], "10000000");
    assert_eq!(span["first_packet"], "0");
    assert_eq!(span["last_packet"], "1");
    assert_eq!(span["last_ticks"], "5000000");
    assert_eq!(span["packets"], "2");
}

#[test]
fn native_text_caps_twenty_sessions_but_json_keeps_exact_full_inventory() {
    let rows: Vec<_> = (0..24)
        .flat_map(|_| [(8, 9, 400_000_000), (8, 9, 0)])
        .collect();
    let input = Input::new(&rows);
    let (_, text) = input.output("summary", false);
    assert_eq!(
        text.lines().filter(|l| l.starts_with("  Session ")).count(),
        20
    );
    assert!(text.contains("Native sessions: 25 (showing 20)"));
    assert!(text.contains("... and 5 more native sessions"));
    let (_, out) = input.output("summary", true);
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["report"]["native_sessions"]["count"], "25");
    assert_eq!(
        v["report"]["native_sessions"]["spans"]
            .as_array()
            .unwrap()
            .len(),
        25
    );
    assert_eq!(v["report"]["native_sessions"]["spans"][24]["index"], "25");
}

#[test]
fn native_summary_quality_orders_late_f1_and_keeps_partial_calendar() {
    // Fig11-12/13 UTC DOY10012:00, no year. RTC0/1/3s; second packet
    // at3s is followed by an in-bound late2s, so native sorting is required.
    let input = Input::new(&[
        (7, 0x11, 0),
        (7, 0x11, 10_000_000),
        (7, 0x11, 30_000_000),
        (7, 0x11, 20_000_000),
    ]);
    let (_, out) = input.output("summary", true);
    let v: Value = serde_json::from_str(&out).unwrap();
    let q = &v["report"]["native_sessions"]["spans"][0]["reference_quality"]["channels"][0];
    assert_eq!(q["channel"], 7);
    assert_eq!(q["valid"], "4");
    assert_eq!(q["interval_ns"], "3000000000");
    assert_eq!(q["min_spacing_ns"], "1000000000");
    assert_eq!(q["max_spacing_ns"], "1000000000");
    assert_eq!(q["drift_ppm"], -1_000_000.0);
    assert!(q["first"]["year"].is_null());
    assert_eq!(q["first"]["day"], 100);
    assert_eq!(q["first"]["scale"], "UTC");
    assert_eq!(q["last"]["basis"], "stated");
    assert!(q["unavailable_reason"].is_null());
    let (_, text) = input.output("summary", false);
    assert!(text.contains("Native quality ch7: valid 4; rejected 0; admitted 4"));
    assert!(text.contains("year unknown"));
    assert!(text.contains("drift -1000000 ppm"));
}

#[test]
fn quality_resets_same_channel_and_large_closed_span_roundtrips() {
    // Fig11-12/13: UTC/DOY10012:00. Four native time channels give a
    // closed span >2KiB; the SAME ch7 then starts day101 in session2.
    let mut bytes = Vec::new();
    for ch in 7..=10 {
        bytes.extend(packet(ch, 0x11, u64::from(ch - 7) * 10_000_000));
    }
    let day101 = [0x41, 0, 0, 0, 0, 0, 0, 0x12, 1, 1];
    bytes.extend(packet_body(7, 0x11, 0, &day101));
    bytes.extend(packet_body(7, 0x11, 10_000_000, &day101));
    let input = Input::bytes(&bytes);
    let (exit, out) = input.output("summary", true);
    assert_eq!(exit, ExitClass::Success);
    let v: Value = serde_json::from_str(&out).unwrap();
    let spans = &v["report"]["native_sessions"]["spans"];
    assert_eq!(spans.as_array().unwrap().len(), 2);
    assert!(serde_json::to_vec(&spans[0]).unwrap().len() > 2048);
    assert_eq!(
        spans[0]["reference_quality"]["channels"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    for row in spans[0]["reference_quality"]["channels"]
        .as_array()
        .unwrap()
    {
        assert_eq!(row["valid"], "1");
        assert_eq!(row["first"]["day"], 100);
    }
    let q = &spans[1]["reference_quality"];
    assert_eq!(q["status"], "complete");
    assert!(q["unavailable_reason"].is_null());
    assert_eq!(q["channels"].as_array().unwrap().len(), 1);
    assert_eq!(q["channels"][0]["channel"], 7);
    assert_eq!(q["channels"][0]["valid"], "2");
    assert_eq!(q["channels"][0]["first"]["day"], 101);
}
#[test]
fn live_quality_drains_more_than_1024_references_without_losing_counts() {
    let input = Input::new(
        &(0..1100)
            .map(|i| (7, 0x11, i * 10_000_000))
            .collect::<Vec<_>>(),
    );
    let (exit, out) = input.output("summary", true);
    assert_eq!(exit, ExitClass::Success);
    let v: Value = serde_json::from_str(&out).unwrap();
    let q = &v["report"]["native_sessions"]["spans"][0]["reference_quality"];
    assert_eq!(q["status"], "complete");
    assert!(q["unavailable_reason"].is_null());
    assert_eq!(q["channels"][0]["valid"], "1100");
    assert_eq!(q["channels"][0]["interval_ns"], "1099000000000");
}
#[test]
fn native_utc_quality_counts_the_2016_insertion_in_tai() {
    // Fig11-12/14 DATE1/FMT4: LY1 2016Dec31 23:59:59 -> LY0
    // 2017Jan1 00:00:00. IERS36->37 makes this2s, exactly RTC2s.
    let mut bytes = packet_body(
        7,
        0x11,
        0,
        &[0x41, 3, 0, 0, 0, 0x59, 0x59, 0x23, 0x31, 0x12, 0x16, 0x20],
    );
    bytes.extend(packet_body(
        7,
        0x11,
        20_000_000,
        &[0x41, 2, 0, 0, 0, 0, 0, 0, 1, 1, 0x17, 0x20],
    ));
    let (exit, out) = Input::bytes(&bytes).output("summary", true);
    assert_eq!(exit, ExitClass::Success);
    let v: Value = serde_json::from_str(&out).unwrap();
    let q = &v["report"]["native_sessions"]["spans"][0]["reference_quality"]["channels"][0];
    assert_eq!(q["valid"], "2");
    assert_eq!(q["interval_ns"], "2000000000");
    assert_eq!(q["drift_ppm"], 0.0);
    assert!(q["unavailable_reason"].is_null());
    assert_eq!(q["first"]["year"], 2016);
    assert_eq!(q["last"]["year"], 2017);
}
#[test]
fn native_quality_text_reports_unavailable_measures_and_cap_prefix() {
    let (_, text) = Input::new(&[(7, 0x11, 0)]).output("summary", false);
    assert!(text.contains("Unavailable: fewer than two valid references"));
    assert!(text.contains("interval unavailable; drift unavailable"));
    assert!(!text.contains("unavailable ns"));
    assert!(!text.contains("null ppm"));
    let (_, text) = Input::new(&vec![(7, 0x11, 0); 1025]).output("summary", false);
    assert!(text.contains("Quality prefix only: native quality pending cap (1024); prefix only"));
}

fn network_packet(ch: u16, ticks: u64, word: u8, seconds: u32, fraction: u32) -> Vec<u8> {
    // 106-24R1 Figs11-15..17: NTF/TS followed by seconds + fraction/ns.
    let mut body = vec![word, 0, 0, 0];
    body.extend(seconds.to_le_bytes());
    body.extend(fraction.to_le_bytes());
    packet_body(ch, 0x12, ticks, &body)
}
#[test]
fn native_network_quality_joins_ntp_ptp_and_f1_without_relabelling_tai() {
    // UTC2017Jan2=1483315200 Unix; NTP=3692304000; PTP TAI=+37s.
    // On ch7: UTC0 -> TAI38 -> UTC2 at RTC0/1/2 => zero physical drift.
    let mut bytes = network_packet(7, 0, 1, 3_692_304_000, 0);
    bytes.extend(network_packet(8, 0, 0x21, 1_483_315_237, 0));
    bytes.extend(network_packet(7, 10_000_000, 0x11, 1_483_315_238, 0));
    bytes.extend(network_packet(8, 10_000_000, 0x21, 1_483_315_238, 0));
    bytes.extend(packet_body(
        7,
        0x11,
        20_000_000,
        &[0x41, 2, 0, 0, 0, 2, 0, 0, 2, 1, 0x17, 0x20],
    ));
    let input = Input::bytes(&bytes);
    let (exit, out) = input.era_output("summary", true, Some("0"));
    assert_eq!(exit, ExitClass::Success);
    let v: Value = serde_json::from_str(&out).unwrap();
    let q = &v["report"]["native_sessions"]["spans"][0]["reference_quality"];
    assert_eq!(q["formats"], "format1+format2");
    assert_eq!(q["ntp_era"], "0");
    assert_eq!(q["ntp_era_source"], "caller-declared");
    assert_eq!(q["ntp_era_inference"], "disabled");
    let a = &q["channels"][0];
    assert_eq!(a["valid"], "3");
    assert_eq!(a["drift_ppm"], 0.0);
    assert_eq!(a["first"]["scale"], "UTC");
    assert_eq!(a["last"]["seconds"], 2);
    let b = &q["channels"][1];
    assert_eq!(b["valid"], "2");
    assert_eq!(b["drift_ppm"], 0.0);
    assert_eq!(b["first"]["scale"], "TAI");
    assert_eq!(b["first"]["seconds"], 37);
    assert_eq!(b["last"]["seconds"], 38);
    assert_eq!(b["first"]["basis"], "stated");
    let (_, text) = input.era_output("summary", false, Some("0"));
    assert!(text.contains("caller-declared era 0"));
    assert!(text.contains("drift 0 ppm"));
    assert!(text.contains("TAI (stated, year 2017)"));
}
#[test]
fn native_network_quality_rejects_unqualified_invalid_and_malformed_observations() {
    // Seconds0/fraction1 is NOT the sentinel, and must not acquire era0 implicitly.
    let mut bytes = network_packet(9, 0, 1, 0, 1);
    bytes.extend(network_packet(10, 0, 1, 0, 0));
    bytes.extend(network_packet(11, 0, 0x31, 0, 0)); // Reserved NTF.
    bytes.extend(network_packet(12, 0, 0x11, 1_483_315_237, 1_000_000_000));
    bytes.extend(network_packet(13, 0, 0x10, 1_483_315_237, 0)); // Invalid TS.
    bytes.extend(network_packet(14, 0, 0x13, 1_483_315_237, 0)); // Reserved TS.
    bytes.extend(packet_body(15, 0x12, 0, &[0x11, 0, 0, 0, 0, 0, 0, 0, 0])); //9 bytes,3 filler.
    let (_, out) = Input::bytes(&bytes).output("summary", true);
    let v: Value = serde_json::from_str(&out).unwrap();
    let q = &v["report"]["native_sessions"]["spans"][0]["reference_quality"];
    assert!(q["ntp_era"].is_null());
    assert_eq!(q["ntp_era_source"], "none");
    let rows = q["channels"].as_array().unwrap();
    assert_eq!(rows.len(), 7);
    for row in rows {
        assert_eq!(row["valid"], "0");
        assert_eq!(row["rejected"], "1");
        assert!(row["first"].is_null());
    }
    assert!(
        rows[0]["rejection_reason"]
            .as_str()
            .unwrap()
            .contains("NTP era/basis required")
    );
    assert!(rows[1]["rejection_reason"].is_null()); // Zero sentinel needs no era.
    assert!(rows[6]["rejection_reason"].is_string());
    let (_, text) = Input::bytes(&bytes).output("summary", false);
    assert!(text.contains("Rejected reference: time needs context: NTP era/basis required"));
    // Owner2026-11-17: nonzero raw fraction remains a real era0 reading,
    // even when floor rounding produces0ns. Only raw (0,0) is the sentinel.
    let (_, out) = Input::bytes(&bytes).era_output("summary", true, Some("0"));
    let v: Value = serde_json::from_str(&out).unwrap();
    let rows = &v["report"]["native_sessions"]["spans"][0]["reference_quality"]["channels"];
    assert_eq!(rows[0]["valid"], "1");
    assert_eq!(rows[0]["first"]["year"], 1900);
    assert_eq!(rows[0]["first"]["ntp_leap_status"], "coverage-unknown");
    assert_eq!(rows[1]["valid"], "0");
    assert!(rows[1]["first"].is_null());
}
#[test]
fn declared_era_is_applied_in_each_native_session() {
    // RFC5905 Fig4: era1 offset63104=2036Feb8; half fraction=.5s.
    let mut bytes = Vec::new();
    for ticks in [200_000_000, 210_000_000, 0, 10_000_000] {
        bytes.extend(network_packet(7, ticks, 1, 63_104, 0x8000_0000));
    }
    let input = Input::bytes(&bytes);
    let (_, out) = input.era_output("summary", true, Some("1"));
    let v: Value = serde_json::from_str(&out).unwrap();
    let spans = v["report"]["native_sessions"]["spans"].as_array().unwrap();
    assert_eq!(spans.len(), 2);
    for span in spans {
        let q = &span["reference_quality"];
        assert_eq!(q["ntp_era"], "1");
        assert_eq!(q["channels"][0]["valid"], "2");
        let c = &q["channels"][0]["first"];
        assert_eq!(c["year"], 2036);
        assert_eq!(c["day"], 39);
        assert_eq!(c["nanoseconds"], 500_000_000);
        assert_eq!(c["ntp_leap_status"], "coverage-unknown");
    }
}

#[test]
fn native_ntp_endpoints_keep_leap_ambiguity_without_inventing_drift() {
    // RDG-012: Unix2017Jan1=1483228800; NTP3692217600. The inserted
    // second's folded coordinate [B,B+1) is ambiguous, mapped value kept.
    let mut bytes = network_packet(7, 0, 1, 3_692_217_600, 0x8000_0000);
    bytes.extend(network_packet(7, 10_000_000, 1, 3_692_217_601, 0x8000_0000));
    let input = Input::bytes(&bytes);
    let (_, out) = input.era_output("summary", true, Some("0"));
    let v: Value = serde_json::from_str(&out).unwrap();
    let q = &v["report"]["native_sessions"]["spans"][0]["reference_quality"]["channels"][0];
    assert_eq!(q["valid"], "2");
    assert!(q["drift_ppm"].is_null());
    assert_eq!(q["first"]["ntp_leap_status"], "leap-ambiguous");
    assert_eq!(q["last"]["ntp_leap_status"], "covered");
    assert_eq!(q["first"]["nanoseconds"], 500_000_000);
    let (_, text) = input.era_output("summary", false, Some("0"));
    assert!(text.contains("[NTP leap-ambiguous]"));
}

#[test]
fn native_drift_marks_stale_offset_use_but_not_native_tai_or_absent_drift() {
    // 2017Jan2 Unix1483315200 + (18*365+4)*86400 =2035Jan2 2051308800.
    // Last-known37s offset: TAI2051308837; NTP4260297600. Dates are
    // beyond builtin table coverage. Only INTERIOR UTC points use the offset;
    // unequal cross-scale intervals prevent its cancellation in mean drift.
    let mut bytes = network_packet(7, 0, 0x21, 2_051_308_837, 0);
    bytes.extend(network_packet(8, 0, 0x21, 2_051_308_837, 0));
    bytes.extend(packet_body(
        7,
        0x11,
        10_000_000,
        &[0x41, 2, 0, 0, 0, 1, 0, 0, 2, 1, 0x35, 0x20],
    ));
    bytes.extend(network_packet(7, 20_000_000, 1, 4_260_297_602, 0));
    bytes.extend(network_packet(8, 20_000_000, 0x21, 2_051_308_839, 0));
    bytes.extend(network_packet(7, 40_000_000, 0x21, 2_051_308_841, 0));
    bytes.extend(network_packet(7, 50_000_000, 0x21, 2_051_308_842, 0));
    let input = Input::bytes(&bytes);
    let (_, out) = input.era_output("summary", true, Some("0"));
    let v: Value = serde_json::from_str(&out).unwrap();
    let rows = &v["report"]["native_sessions"]["spans"][0]["reference_quality"]["channels"];
    assert_eq!(rows[0]["drift_ppm"], 0.0);
    assert_eq!(rows[0]["first"]["scale"], "TAI");
    assert_eq!(rows[0]["last"]["scale"], "TAI");
    assert_eq!(rows[0]["drift_uses_stale_leap_table"], true);
    assert_eq!(rows[1]["drift_ppm"], 0.0);
    assert_eq!(rows[1]["drift_uses_stale_leap_table"], false);
    let (_, text) = input.era_output("summary", false, Some("0"));
    assert!(text.contains("drift 0 ppm (possibly out of date: stale leap table)"));
    // Unknown RTC scale removes the drift value, so no stale DRIFT is claimed.
    bytes.extend(packet_body(
        7,
        0x11,
        60_000_000,
        &[0x31, 2, 0, 0, 0, 6, 0, 0, 2, 1, 0x35, 0x20],
    ));
    let (_, out) = Input::bytes(&bytes).era_output("summary", true, Some("0"));
    let v: Value = serde_json::from_str(&out).unwrap();
    let row = &v["report"]["native_sessions"]["spans"][0]["reference_quality"]["channels"][0];
    assert!(row["drift_ppm"].is_null());
    assert_eq!(row["drift_uses_stale_leap_table"], false);
}

#[test]
fn native_drift_stale_first_only_and_last_only_endpoints() {
    // 2035Jan2 Unix2051308800, TAI+37; NTP+2208988800.
    for stale_first in [true, false] {
        let first = if stale_first {
            (1, 4_260_297_600)
        } else {
            (0x21, 2_051_308_837)
        };
        let last = if stale_first {
            (0x21, 2_051_308_838)
        } else {
            (1, 4_260_297_601)
        };
        let mut bytes = network_packet(7, 0, first.0, first.1, 0);
        bytes.extend(network_packet(7, 10_000_000, last.0, last.1, 0));
        let (_, out) = Input::bytes(&bytes).era_output("summary", true, Some("0"));
        let v: Value = serde_json::from_str(&out).unwrap();
        let row = &v["report"]["native_sessions"]["spans"][0]["reference_quality"]["channels"][0];
        assert_eq!(row["drift_ppm"], 0.0);
        assert_eq!(
            row["drift_uses_stale_leap_table"], true,
            "stale_first={stale_first}"
        );
    }
}

#[test]
fn native_summary_replays_future_dmy_without_crossing_sessions() {
    // Figs11-13/14: DOY36523:59:59 -> day100:00 -> DMY2024Jan100:00:01.
    // The first two references must date2023/2024 using the later DMY anchor.
    let mut bytes = packet_body(7, 0x11, 0, &[0x41, 0, 0, 0, 0, 0x59, 0x59, 0x23, 0x65, 3]);
    bytes.extend(packet_body(
        7,
        0x11,
        10_000_000,
        &[0x41, 1, 0, 0, 0, 0, 0, 0, 1, 0],
    ));
    bytes.extend(packet_body(
        7,
        0x11,
        20_000_000,
        &[0x41, 3, 0, 0, 0, 1, 0, 0, 1, 1, 0x24, 0x20],
    ));
    // A >1.1s backward header starts a separate native session, with no year.
    bytes.extend(packet_body(7, 0x11, 0, &[0x41, 0, 0, 0, 0, 0, 0, 0, 1, 0]));
    let input = Input::bytes(&bytes);
    let (_, out) = input.output("summary", true);
    let v: Value = serde_json::from_str(&out).unwrap();
    let spans = &v["report"]["native_sessions"]["spans"];
    let q = &spans[0]["reference_quality"];
    assert_eq!(q["year_inference"], "session-dmy");
    let row = &q["channels"][0];
    assert_eq!(row["valid"], "3");
    assert_eq!(row["first"]["year"], 2023);
    assert_eq!(row["first"]["year_evidence"]["source"], "session-dmy");
    assert_eq!(row["first"]["year_evidence"]["anchor_packet"], "2");
    assert_eq!(row["last"]["year"], 2024);
    assert_eq!(row["drift_ppm"], 0.0);
    assert!(spans[1]["reference_quality"]["channels"][0]["first"]["year"].is_null());
    let (_, text) = input.output("summary", false);
    assert!(text.contains("session DMY year evidence"));
    assert!(text.contains("year 2023"));
}
#[test]
fn native_year_replay_backdates_a_prefix_larger_than_the_pending_cap() {
    let mut bytes = Vec::new();
    //1320 yearless F1 points (22min), followed by dated2024Jan2.
    for i in 0..1320u64 {
        let bcd = |n: u64| (((n / 10) << 4) | (n % 10)) as u8;
        bytes.extend(packet_body(
            7,
            0x11,
            i * 10_000_000,
            &[0x41, 1, 0, 0, 0, bcd(i % 60), bcd(i / 60), 0, 2, 0],
        ));
    }
    bytes.extend(packet_body(
        7,
        0x11,
        1320 * 10_000_000,
        &[0x41, 3, 0, 0, 0, 0, 0x22, 0, 2, 1, 0x24, 0x20],
    ));
    let (_, out) = Input::bytes(&bytes).output("summary", true);
    let v: Value = serde_json::from_str(&out).unwrap();
    let q = &v["report"]["native_sessions"]["spans"][0]["reference_quality"];
    assert_eq!(q["status"], "complete");
    assert_eq!(q["channels"][0]["valid"], "1321");
    assert_eq!(q["channels"][0]["first"]["year"], 2024);
    assert_eq!(q["channels"][0]["drift_ppm"], 0.0);
}
#[test]
fn native_year_replay_reports_new_malformed_and_contradictory_years() {
    // Day366 is readable with LY=1 before a future2017 common-year anchor.
    let mut bytes = packet_body(7, 0x11, 0, &[0x41, 1, 0, 0, 0, 0, 0, 0, 0x66, 3]);
    bytes.extend(packet_body(
        7,
        0x11,
        10_000_000,
        &[0x41, 2, 0, 0, 0, 1, 0, 0, 0x31, 0x12, 0x17, 0x20],
    ));
    bytes.extend(packet_body(
        7,
        0x11,
        20_000_000,
        &[0x41, 3, 0, 0, 0, 2, 0, 0, 0x31, 0x12, 0x16, 0x20],
    ));
    let (_, out) = Input::bytes(&bytes).output("summary", true);
    let v: Value = serde_json::from_str(&out).unwrap();
    let samples = v["findings"]["samples"].as_array().unwrap();
    assert!(samples.iter().any(|f| f["code"] == "TF-004"));
    assert!(samples.iter().any(|f| f["code"] == "TF-032"));
    let evidence = &v["report"]["native_sessions"]["spans"][0]["reference_quality"]["channels"][0]
        ["last"]["year_evidence"];
    assert_eq!(evidence["source"], "packet");
    assert!(
        evidence["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["supplied"] == 2017)
    );
    assert!(
        evidence["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["supplied"] == 2016)
    );
}

#[test]
fn replay_qualifies_real_leap_and_rebuilds_forbidden_label_without_credit() {
    for (day, dmy_date, expect) in [
        (0x66, (1, 1, 0x17, 0x20), 0.0),
        (0x65, (0x31, 0x12, 0x16, 0x20), -250_000.0),
    ] {
        // RDG-010:2016 day366 allows second60, day365 forbids it.
        let mut bytes = packet_body(7, 0x11, 0, &[0x41, 1, 0, 0, 0, 0x59, 0x59, 0x23, day, 3]);
        bytes.extend(packet_body(
            7,
            0x11,
            10_000_000,
            &[0x41, 1, 0, 0, 0, 0x60, 0x59, 0x23, day, 3],
        ));
        let (d, h) = if day == 0x66 { (1, 0) } else { (0x66, 3) };
        bytes.extend(packet_body(
            7,
            0x11,
            20_000_000,
            &[0x41, 1, 0, 0, 0, 0, 0, 0, d, h],
        ));
        bytes.extend(packet_body(
            7,
            0x11,
            30_000_000,
            &[
                0x41,
                if day == 0x66 { 2 } else { 3 },
                0,
                0,
                0,
                1,
                0,
                0,
                dmy_date.0,
                dmy_date.1,
                dmy_date.2,
                dmy_date.3,
            ],
        ));
        let input = Input::bytes(&bytes);
        let (_, out) = input.output("summary", true);
        let (_, again) = input.output("summary", true);
        assert_eq!(out, again);
        let v: Value = serde_json::from_str(&out).unwrap();
        let q = &v["report"]["native_sessions"]["spans"][0]["reference_quality"]["channels"][0];
        assert_eq!(q["valid"], "3");
        assert_eq!(q["rejected"], "1"); // Native second60 remains a context-only label.
        assert_eq!(q["first"]["year"], 2016);
        assert_eq!(q["drift_ppm"], expect);
    }
}
#[test]
fn replay_does_not_extend_a_dmy_year_across_a_native_year_context_gap() {
    // Owner YEAR-PHASE F8:2025 day1 -> day365 at+181days has best
    // residual183days (>182.5day bound), so the later year stays partial.
    let mut bytes = packet_body(7, 0x11, 0, &[0x41, 0, 0, 0, 0, 0, 0, 0, 1, 0]);
    bytes.extend(packet_body(
        7,
        0x11,
        10_000_000,
        &[0x41, 2, 0, 0, 0, 1, 0, 0, 1, 1, 0x25, 0x20],
    ));
    // Intermediate opaque headers make native counter ordering unambiguous;
    // only the time references are separated by181days.
    for day in [45, 90, 135] {
        bytes.extend(packet_body(8, 9, day * 86400 * 10_000_000, &[]));
    }
    bytes.extend(packet_body(
        7,
        0x11,
        181 * 86400 * 10_000_000,
        &[0x41, 0, 0, 0, 0, 0, 0, 0, 0x65, 3],
    ));
    let (_, out) = Input::bytes(&bytes).output("summary", true);
    let v: Value = serde_json::from_str(&out).unwrap();
    let q = &v["report"]["native_sessions"]["spans"][0]["reference_quality"]["channels"][0];
    assert_eq!(q["first"]["year"], 2025);
    assert!(q["last"]["year"].is_null(), "{q}");
    assert!(q["drift_ppm"].is_null());
    assert!(
        v["findings"]["samples"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["code"] == "TF-020")
    );
}

#[test]
fn native_year_findings_keep_exact_counts_beyond_the_sample_cap() {
    let mut bytes = Vec::new();
    for i in 0..30u64 {
        let second = (((i % 60) / 10) << 4 | (i % 10)) as u8;
        let (ly, year) = if i % 2 == 0 { (2, 0x17) } else { (2, 0x18) };
        bytes.extend(packet_body(
            7,
            0x11,
            i * 10_000_000,
            &[0x41, ly, 0, 0, 0, second, 0, 0, 0x31, 0x12, year, 0x20],
        ));
    }
    let input = Input::bytes(&bytes);
    let (_, out) = input.output("summary", true);
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["findings"]["warning"], "30");
    assert_eq!(v["findings"]["samples"].as_array().unwrap().len(), 20);
    assert_eq!(v["findings"]["omitted"], "10");
    assert_eq!(
        run(
            &["summary", input.0.to_str().unwrap(), "--fail-on", "warning"],
            &mut Vec::new(),
            &mut Vec::new()
        ),
        ExitClass::Findings
    );
    // The error comes AFTER the first20 native samples. Its level/count must
    // still drive --fail-on error even though its message is omitted.
    bytes.extend(packet_body(
        7,
        0x11,
        30 * 10_000_000,
        &[0x41, 1, 0, 0, 0, 0, 0, 0, 0x66, 3],
    ));
    let input = Input::bytes(&bytes);
    let (_, out) = input.output("summary", true);
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        v["findings"]["error"], "1",
        "{}",
        v["report"]["native_sessions"]
    );
    assert_eq!(v["highest_level"], "error");
    assert!(
        v["findings"]["samples"]
            .as_array()
            .unwrap()
            .iter()
            .all(|f| f["level"] != "error")
    );
    assert_eq!(
        run(
            &["summary", input.0.to_str().unwrap()],
            &mut Vec::new(),
            &mut Vec::new()
        ),
        ExitClass::Findings
    );
}

#[test]
fn year_inference_outside_calendar_range_is_context_not_output_failure() {
    // Future DMY year0000 cannot date the preceding DOY365 as year-1.
    // It is valid Chapter10 input; retain the rest of the summary.
    let mut bytes = packet_body(7, 0x11, 0, &[0x41, 0, 0, 0, 0, 0x59, 0x59, 0x23, 0x65, 3]);
    bytes.extend(packet_body(
        7,
        0x11,
        10_000_000,
        &[0x41, 1, 0, 0, 0, 0, 0, 0, 1, 0],
    ));
    bytes.extend(packet_body(
        7,
        0x11,
        20_000_000,
        &[0x41, 3, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0],
    ));
    let input = Input::bytes(&bytes);
    let (exit, out) = input.output("summary", true);
    assert_eq!(exit, ExitClass::Success);
    let v: Value = serde_json::from_str(&out).unwrap();
    let q = &v["report"]["native_sessions"]["spans"][0]["reference_quality"]["channels"][0];
    assert_eq!(q["valid"], "2");
    assert_eq!(q["rejected"], "1");
    assert!(
        q["rejection_reason"]
            .as_str()
            .unwrap()
            .contains("year rollover outside0..9999")
    );
    assert!(
        v["findings"]["samples"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["code"] == "TF-020" && f["level"] == "info")
    );
}
