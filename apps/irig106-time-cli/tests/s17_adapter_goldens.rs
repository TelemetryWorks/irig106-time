//! CLI-104/105/106 clause evidence only; no whole-requirement markers.
use irig106_time_cli::{ExitClass, run};
use serde_json::{Value, json};
use std::{
    io,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Input(PathBuf);
impl Input {
    fn new(bytes: &[u8]) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "s17-adapters-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, bytes).unwrap();
        Self(path)
    }
    fn args<'a>(&'a self, command: &'a str, json: bool) -> Vec<&'a str> {
        let mut args = vec![command, self.0.to_str().unwrap()];
        if command == "correlate" {
            args.push("0");
        }
        if json {
            args.extend(["--format", "json"]);
        }
        args.extend(["--fail-on", "never"]);
        args
    }
    fn output(&self, command: &str, json: bool) -> (String, String) {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert_eq!(
            run(&self.args(command, json), &mut out, &mut err),
            ExitClass::Success,
            "{}",
            String::from_utf8_lossy(&err)
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

fn packet(kind: u8, rtc: u64, body: &[u8]) -> Vec<u8> {
    // 106-24R1 §11.2.1.1: header LE word sum, 100ns RTC, Data Length
    // excludes DD filler. Fig11-12/13/15..17 supply the literal bodies below.
    let mut p = vec![0xdd; (24 + body.len()).next_multiple_of(4)];
    p[..24].fill(0);
    p[..2].copy_from_slice(&[0x25, 0xeb]);
    p[2] = 7;
    let len = p.len() as u32;
    p[4..8].copy_from_slice(&len.to_le_bytes());
    p[8..12].copy_from_slice(&(body.len() as u32).to_le_bytes());
    p[12] = 6;
    p[15] = kind;
    p[16..22].copy_from_slice(&(rtc * 10_000_000).to_le_bytes()[..6]);
    p[24..24 + body.len()].copy_from_slice(body);
    let sum = p[..22].chunks_exact(2).fold(0u16, |s, w| {
        s.wrapping_add(u16::from_le_bytes([w[0], w[1]]))
    });
    p[22..24].copy_from_slice(&sum.to_le_bytes());
    p
}
fn body(case: &Value) -> Vec<u8> {
    let mut body = (case["word"].as_u64().unwrap() as u32)
        .to_le_bytes()
        .to_vec();
    if case["kind"] == 17 {
        // Fig11-13: DOY100 12:00:00, unknown year, all reserved bits clear.
        body.extend([0, 0, 0, 0x12, 0, 1]);
    } else {
        // RFC5905 Fig4:1970 NTP2208988800, fraction0x80000000=0.5s.
        // PTP: Unix2017-01-02=1483315200; +37=1483315237 TAI,123ns.
        // 2030-01-02 Unix1893542400;+37=1893542437, beyond C72 currency.
        body.extend((case["seconds"].as_u64().unwrap() as u32).to_le_bytes());
        let fraction = if case["word"].as_u64().unwrap() >> 4 == 0 {
            0x80000000u32
        } else {
            123
        };
        body.extend(fraction.to_le_bytes());
    }
    body
}
fn clock(time: &Value, scale: &Value, basis: &Value) -> Value {
    if time.is_null() {
        return Value::Null;
    }
    // Independent calendar-count arithmetic, not a production decoder/encoder.
    let count = (((time[1].as_u64().unwrap() - 1) * 86400
        + time[2].as_u64().unwrap() * 3600
        + time[3].as_u64().unwrap() * 60
        + time[4].as_u64().unwrap())
        * 1_000_000_000
        + time[5].as_u64().unwrap())
    .to_string();
    json!({"year":time[0],"day":time[1],"hour":time[2],"minute":time[3],"second":time[4],"nanosecond":time[5],
        "clock_count_ns":count,"scale":scale,"basis":basis,"year_offset":"0"})
}
fn text_clock(time: &Value) -> String {
    format!(
        "{:03}:{:02}:{:02}:{:02}.{:09}",
        time[1].as_u64().unwrap(),
        time[2].as_u64().unwrap(),
        time[3].as_u64().unwrap(),
        time[4].as_u64().unwrap(),
        time[5].as_u64().unwrap()
    )
}
const COMMANDS: [&str; 6] = [
    "summary",
    "channels",
    "jumps",
    "timeline",
    "csv",
    "correlate",
];
#[test]
fn full_adapter_projection_goldens_cover_protocols_status_scales_and_absence() {
    let cases: Value = serde_json::from_str(include_str!("golden/s17_adapter_cases.json")).unwrap();
    for case in cases.as_array().unwrap() {
        let kind = case["kind"].as_u64().unwrap() as u8;
        let input = Input::new(&packet(kind, 0, &body(case)));
        let time = &case["clock"];
        let ptp = kind == 18 && !time.is_null();
        let value_scale = if ptp {
            json!("UTC")
        } else {
            case["scale"].clone()
        };
        let expected_inventory = clock(time, &value_scale, &case["clock_basis"]);
        let expected_row = expected_inventory.clone();
        for command in COMMANDS {
            let (text, stderr) = input.output(command, false);
            if command == "csv" {
                let calendar = if time.is_null() {
                    vec![String::new(); 8]
                } else {
                    let mut fields = (1..6).map(|i| time[i].to_string()).collect::<Vec<_>>();
                    fields.extend([
                        time[0].as_u64().map(|n| n.to_string()).unwrap_or_default(),
                        String::new(),
                        String::new(),
                    ]);
                    fields
                };
                let mut expected = format!(
                    "0,0x0,7,0x{kind:02X},Time-F{},0,0,{}",
                    if kind == 17 { 1 } else { 2 },
                    calendar.join(",")
                );
                if kind == 17 {
                    expected.push_str(",,,,");
                } else {
                    let word = case["word"].as_u64().unwrap();
                    let ts = word & 15;
                    let status = match ts {
                        0 => "NotValid".into(),
                        1 => "Valid".into(),
                        _ => format!("RSVD-{ts}"),
                    };
                    expected.push_str(&format!(
                        ",{},{},{ts},{status}",
                        word >> 4,
                        case["format"].as_str().unwrap()
                    ));
                }
                assert_eq!(text.lines().nth(1), Some(expected.as_str()), "{case}");
                assert_eq!(text.lines().nth(1).unwrap().split(',').count(), 19);
                continue;
            }
            if matches!(command, "summary" | "channels") {
                let label = if case["format"] == "reserved" {
                    if command == "channels" {
                        "RSVD-3"
                    } else {
                        "Reserved NTF"
                    }
                } else {
                    case["format"].as_str().unwrap()
                };
                assert!(
                    text.replace(' ', "").contains(&label.replace(' ', "")),
                    "{case}: {text}"
                );
            }
            let (out, json_err) = input.output(command, true);
            let v: Value = serde_json::from_str(&out).unwrap();
            let channel = &v["report"]["channels"][0];
            for (key, golden) in [
                ("format", "format"),
                ("native_scale", "scale"),
                ("scale_basis", "basis"),
                ("status", "status"),
            ] {
                assert_eq!(channel[key], case[golden], "{command} {case}");
            }
            assert_eq!(channel["first"], expected_inventory, "{command} {case}");
            assert_eq!(channel["last"], expected_inventory, "{command} {case}");
            if let Some(code) = case["finding"].as_str() {
                assert!(
                    v["findings"]["samples"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|f| f["code"] == code && f["level"] == "warning"),
                    "{case}: {v}"
                );
            }
            if command == "timeline" {
                assert_eq!(v["report"]["rows"][0]["clock"], expected_row, "{case}");
                if time.is_null() {
                    assert!(text.contains("N/A"));
                }
            } else if command == "correlate" {
                assert_eq!(
                    v["report"]["correlation"]["any"]["clock"], expected_row,
                    "{case}"
                );
                if !time.is_null() {
                    assert_eq!(
                        v["report"]["correlation"]["answers"][0]["clock"],
                        expected_row
                    );
                }
            }
            if matches!(command, "timeline" | "correlate") && !time.is_null() {
                let mut expected_text = text_clock(time);
                if ptp {
                    expected_text.push_str(if case["name"] == "ptp-stale" {
                        " UTC (from TAI via stale leap table, possibly out of date)"
                    } else {
                        " UTC (from TAI via leap table)"
                    });
                }
                assert!(text.contains(&expected_text), "{case}: {text}");
            }
            // Summary has its own stdout diagnostics; other commands preserve
            // identical stderr findings across the two renderers.
            if command != "summary" {
                assert_eq!(stderr, json_err, "{command} {case}");
            }
        }
    }
}
#[test]
fn every_adapter_writer_failure_remains_exit_four() {
    struct Broken;
    impl io::Write for Broken {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("adapter writer"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let input = Input::new(&packet(0x11, 0, &[0x51, 0, 0, 0, 0, 0, 0, 0x12, 0, 1]));
    for command in COMMANDS {
        for json in [false, true] {
            if command == "csv" && json {
                continue;
            }
            assert_eq!(
                run(&input.args(command, json), &mut Broken, &mut Vec::new()),
                ExitClass::Output,
                "{command} json={json}"
            );
        }
    }
}

#[test]
fn reset_segment_goldens_agree_across_inventory_rows_csv_and_query() {
    // Fig11-12/13: FMT4/SRC1, DOY100 at 12:00:00/40 then12:01:01/10.
    // Coordinator reset policy:40->1 is >30s; query5 overlaps both ranges.
    let mut bytes = Vec::new();
    for (rtc, minute, second) in [(0, 0, 0), (40, 0, 0x40), (1, 1, 1), (10, 1, 0x10)] {
        bytes.extend(packet(
            0x11,
            rtc,
            &[0x41, 0, 0, 0, 0, second, minute, 0x12, 0, 1],
        ));
    }
    bytes.extend(packet(9, 5, &[])); // Global RTC alone proves no source association.
    let input = Input::new(&bytes);
    let expected_segments = json!([
        {"index":"1","first_rtc":"0","last_rtc":"400000000",
         "first":clock(&json!([null,100,12,0,0,0]),&json!("UTC"),&json!("stated FMT")),
         "last":clock(&json!([null,100,12,0,40,0]),&json!("UTC"),&json!("stated FMT"))},
        {"index":"2","first_rtc":"10000000","last_rtc":"100000000",
         "first":clock(&json!([null,100,12,1,1,0]),&json!("UTC"),&json!("stated FMT")),
         "last":clock(&json!([null,100,12,1,10,0]),&json!("UTC"),&json!("stated FMT"))}
    ]);
    for command in ["summary", "channels", "jumps", "timeline"] {
        let (out, _) = input.output(command, true);
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["report"]["reset_count"], "1");
        assert!(v["report"]["rtc_duration_ns"].is_null());
        assert_eq!(v["report"]["channels"][0]["segments"], expected_segments);
        assert!(
            v["findings"]["samples"]
                .as_array()
                .unwrap()
                .iter()
                .any(|f| f["message"]
                    .as_str()
                    .unwrap()
                    .contains("RTC reset on channel 7")
                    && f["code"].is_null()
                    && f["level"] == "warning")
        );
        if command == "timeline" {
            let rows = v["report"]["rows"].as_array().unwrap();
            let expected = [(0, 0), (0, 40), (1, 1), (1, 10)];
            for (row, (minute, second)) in rows.iter().zip(expected) {
                assert_eq!(
                    row["clock"],
                    clock(
                        &json!([null, 100, 12, minute, second, 0]),
                        &json!("UTC"),
                        &json!("stated FMT")
                    )
                );
                assert_eq!(row["association"], "association_unestablished");
            }
            assert_eq!(rows.len(), 5);
            assert_eq!(rows[4]["kind"], "counter-only");
            assert!(rows[4]["clock"].is_null());
        }
    }
    let (csv, _) = input.output("csv", false);
    assert_eq!(
        csv.lines().skip(1).collect::<Vec<_>>(),
        [
            "0,0x0,7,0x11,Time-F1,0,0,100,12,0,0,0,,,,,,,",
            "1,0x24,7,0x11,Time-F1,400000000,40000000000,100,12,0,40,0,,,,,,,",
            "2,0x48,7,0x11,Time-F1,10000000,1000000000,100,12,1,1,0,,,,,,,",
            "3,0x6C,7,0x11,Time-F1,100000000,10000000000,100,12,1,10,0,,,,,,,",
            "4,0x90,7,0x09,PCM-1,50000000,5000000000,,,,,,,,,,,,"
        ]
    );
    for json in [false, true] {
        let mut args = input.args("correlate", json);
        args[2] = "2FAF080";
        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert_eq!(run(&args, &mut out, &mut err), ExitClass::Success);
        assert!(
            String::from_utf8(err)
                .unwrap()
                .contains("RTC is ambiguous across reset segments on channel 7")
        );
        if json {
            let v: Value = serde_json::from_slice(&out).unwrap();
            let answers = v["report"]["correlation"]["answers"].as_array().unwrap();
            assert_eq!(answers.len(), 2);
            for (answer, segment, minute) in [(&answers[0], "1", 0), (&answers[1], "2", 1)] {
                assert_eq!(answer["segment"], segment);
                assert_eq!(answer["channel"], 7);
                assert_eq!(answer["labelled"], true);
                assert_eq!(
                    answer["clock"],
                    clock(
                        &json!([null, 100, 12, minute, 5, 0]),
                        &json!("UTC"),
                        &json!("stated FMT")
                    )
                );
            }
            assert!(v["report"]["correlation"]["any"]["clock"].is_null());
        } else {
            assert_eq!(
                String::from_utf8(out).unwrap(),
                "Resolving RTC 0x000002FAF080 (50000000 ticks, 5000000000 ns)\n\n  Channel   7 Segment   1  → 100:12:00:05.000000000 UTC (stated FMT)\n  Channel   7 Segment   2  → 100:12:01:05.000000000 UTC (stated FMT)\n"
            );
        }
    }
}
