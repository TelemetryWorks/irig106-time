//! CLI-106 partial scale/basis clauses; no whole Requirements marker.
use irig106_time_cli::{ExitClass, run};
use serde_json::Value;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
struct Input(PathBuf);
impl Input {
    fn new(bytes: &[u8]) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let p = std::env::temp_dir().join(format!(
            "s17-f1-basis-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&p, bytes).unwrap();
        Self(p)
    }
    fn output(&self, command: &str, json: bool, target: u64) -> String {
        let target = format!("{:X}", target * 10_000_000);
        let mut args = vec![command, self.0.to_str().unwrap()];
        if command == "correlate" {
            args.push(&target);
        }
        if json {
            args.extend(["--format", "json"]);
        }
        args.extend(["--fail-on", "never"]);
        let mut out = Vec::new();
        assert_eq!(run(&args, &mut out, &mut Vec::new()), ExitClass::Success);
        String::from_utf8(out).unwrap()
    }
}
impl Drop for Input {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
fn packet(ch: u16, rtc: u64, body: &[u8]) -> Vec<u8> {
    // 106-24R1 §11.2.1.1: checksum,100ns RTC,Data Length excludes filler.
    let mut p = vec![0; (24 + body.len()).next_multiple_of(4)];
    p[..2].copy_from_slice(&[0x25, 0xeb]);
    p[2..4].copy_from_slice(&ch.to_le_bytes());
    let len = p.len() as u32;
    p[4..8].copy_from_slice(&len.to_le_bytes());
    p[8] = body.len() as u8;
    p[15] = if body.is_empty() { 9 } else { 0x11 };
    p[16..22].copy_from_slice(&(rtc * 10_000_000).to_le_bytes()[..6]);
    p[24..24 + body.len()].copy_from_slice(body);
    let sum = p[..22].chunks_exact(2).fold(0u16, |s, w| {
        s.wrapping_add(u16::from_le_bytes([w[0], w[1]]))
    });
    p[22..24].copy_from_slice(&sum.to_le_bytes());
    p
}
fn f1(ch: u16, fmt: u8, rtc: u64, seconds: u8) -> Vec<u8> {
    // Fig11-12/13 (printed11-23/25,PDF31/33): SRC1,FMT0..5,DATE0;
    // BCD day10012:mm:ss, .12s. No leap offset applied to native GPS.
    let bcd = |v: u8| v / 10 * 16 + v % 10;
    packet(
        ch,
        rtc,
        &[
            (fmt << 4) | 1,
            0,
            0,
            0,
            0x12,
            bcd(seconds % 60),
            bcd(seconds / 60),
            0x12,
            0,
            1,
        ],
    )
}
fn assert_basis(clock: &Value, scale: &str, basis: &str) {
    assert_eq!(clock["scale"], scale);
    assert_eq!(clock["basis"], basis);
    assert!(clock["year"].is_null()); // No year or origin is invented by labels.
}
const CASES: [(u8, &str, &str, &str); 6] = [
    (0, "UTC", "assumed, RDG-009", "UTC (assumed, RDG-009)"),
    (1, "UTC", "assumed, RDG-009", "UTC (assumed, RDG-009)"),
    (2, "UTC", "assumed, RDG-009", "UTC (assumed, RDG-009)"),
    (3, "unknown", "unresolved", "scale unknown"),
    (4, "UTC", "stated FMT", "UTC (stated FMT)"),
    (5, "GPS", "stated FMT", "GPS (stated FMT)"),
];
#[test]
fn every_f1_format_labels_time_and_counter_rows_and_query_answers() {
    for (fmt, scale, basis, label) in CASES {
        let input = Input::new(&[f1(7, fmt, 0, 0), packet(8, 1, &[])].concat());
        let v: Value = serde_json::from_str(&input.output("timeline", true, 1)).unwrap();
        for (row, second) in v["report"]["rows"].as_array().unwrap().iter().zip([0, 1]) {
            assert_basis(&row["clock"], scale, basis);
            assert_eq!(row["clock"]["second"], second);
            assert_eq!(row["clock"]["nanosecond"], 120000000);
            assert_eq!(row["association"], "association_unestablished");
        }
        let text = input.output("timeline", false, 1);
        for second in [0, 1] {
            assert!(
                text.contains(&format!("100:12:00:{second:02}.120000000 {label}")),
                "{text}"
            );
        }
        let v: Value = serde_json::from_str(&input.output("correlate", true, 1)).unwrap();
        for clock in [
            &v["report"]["correlation"]["any"]["clock"],
            &v["report"]["correlation"]["answers"][0]["clock"],
        ] {
            assert_basis(clock, scale, basis);
            assert_eq!(clock["second"], 1);
        }
        let text = input.output("correlate", false, 1);
        assert_eq!(text.matches(label).count(), 2, "{text}");
    }
}
#[test]
fn selected_f1_basis_is_not_the_packets_or_latest_channels_basis() {
    for gps_first in [false, true] {
        let pair = if gps_first {
            [f1(7, 5, 0, 0), f1(9, 4, 0, 20)]
        } else {
            [f1(9, 4, 0, 20), f1(7, 5, 0, 0)]
        };
        let input = Input::new(&[pair.concat(), f1(9, 4, 0, 20)].concat());
        let v: Value = serde_json::from_str(&input.output("timeline", true, 0)).unwrap();
        let row = &v["report"]["rows"][2];
        assert_eq!(row["channel"], 9);
        assert_eq!(row["clock"]["second"], 0);
        assert_basis(&row["clock"], "GPS", "stated FMT");
    }
    let input = Input::new(&[f1(7, 5, 0, 0), f1(7, 0, 40, 40)].concat());
    let v: Value = serde_json::from_str(&input.output("correlate", true, 5)).unwrap();
    assert_basis(
        &v["report"]["correlation"]["answers"][0]["clock"],
        "GPS",
        "stated FMT",
    );
    assert_eq!(v["report"]["channels"][0]["native_scale"], "UTC");
}
#[test]
fn reset_own_rows_and_spooled_answers_keep_each_f1_basis() {
    let input = Input::new(
        &[
            f1(7, 5, 0, 0),
            f1(7, 4, 40, 40),
            f1(7, 0, 1, 61),
            f1(7, 4, 10, 70),
        ]
        .concat(),
    );
    let v: Value = serde_json::from_str(&input.output("timeline", true, 5)).unwrap();
    assert_basis(&v["report"]["rows"][2]["clock"], "UTC", "assumed, RDG-009");
    assert_basis(&v["report"]["rows"][3]["clock"], "UTC", "stated FMT");
    let v: Value = serde_json::from_str(&input.output("correlate", true, 5)).unwrap();
    let answers = &v["report"]["correlation"]["answers"];
    assert_eq!(answers.as_array().unwrap().len(), 2);
    assert_basis(&answers[0]["clock"], "GPS", "stated FMT");
    assert_basis(&answers[1]["clock"], "UTC", "assumed, RDG-009");
    let text = input.output("correlate", false, 5);
    assert!(
        text.lines()
            .any(|l| l.contains("Segment   1") && l.contains("GPS (stated FMT)"))
    );
    assert!(
        text.lines()
            .any(|l| l.contains("Segment   2") && l.contains("UTC (assumed, RDG-009)"))
    );
    assert!(v["report"]["correlation"]["any"]["clock"].is_null());
}
