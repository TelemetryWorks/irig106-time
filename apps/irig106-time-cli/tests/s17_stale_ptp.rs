//! Partial CLI-104/105/106 adapter clauses; no whole-requirement markers.
use irig106_time_cli::{ExitClass, run};
use serde_json::Value;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
struct Input(PathBuf);
impl Input {
    fn new(ntf: u8, seconds: u32) -> Self {
        // 106-24R1 Figures11-15/17: NTF1/2, TS1,32-bit TAI seconds,123ns.
        // §11.2.1.1: 36-byte packet,12-byte Data Length, LE header word sum.
        let mut p = vec![0; 36];
        p[..2].copy_from_slice(&[0x25, 0xeb]);
        p[2] = 7;
        p[4] = 36;
        p[8] = 12;
        p[15] = 0x12;
        p[24] = (ntf << 4) | 1;
        p[28..32].copy_from_slice(&seconds.to_le_bytes());
        p[32..36].copy_from_slice(&123u32.to_le_bytes());
        let checksum = p[..22].chunks_exact(2).fold(0u16, |s, w| {
            s.wrapping_add(u16::from_le_bytes([w[0], w[1]]))
        });
        p[22..24].copy_from_slice(&checksum.to_le_bytes());
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "s17-stale-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, p).unwrap();
        Self(path)
    }
    fn output(&self, command: &str, json: bool, level: &str) -> (ExitClass, String, String) {
        self.output_at(command, json, level, 0)
    }
    fn output_at(
        &self,
        command: &str,
        json: bool,
        level: &str,
        rtc_seconds: u64,
    ) -> (ExitClass, String, String) {
        let target = format!("{:X}", rtc_seconds * 10_000_000);
        let mut args = vec![command, self.0.to_str().unwrap()];
        if command == "correlate" {
            args.push(&target);
        }
        if json {
            args.extend(["--format", "json"]);
        }
        args.extend(["--fail-on", level]);
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let exit = run(&args, &mut out, &mut err);
        (
            exit,
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
const STALE_MESSAGE: &str = "TF-021: UTC 2030 002:00:00:00.000000123 possibly out of date (value kept); raw TAI=1893542437s+123ns; TAI-UTC=37s; table=IERS-C72-2026-07-06/6cb6f5d4b819f2e568e25db4b0b26d89dedf031fdffb18bc94d40f4e94e268d7; known through Unix=1814140799s";
#[test]
fn stale_ptp_keeps_utc_and_warns_at_exact_threshold_in_every_command() {
    // Unix2030-01-02=1893542400; last offset37s ->1893542437 TAI.
    // This fixed date is beyond the S04 IERS-C72 table's declared coverage.
    for ntf in [1, 2] {
        let input = Input::new(ntf, 1_893_542_437);
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
                ("never", ExitClass::Success),
            ] {
                let (exit, out, err) = input.output(command, false, level);
                assert_eq!(exit, expected, "{command}: {err}");
                // Summary renders diagnostics on stdout; other commands use stderr.
                let diagnostics = format!("{out}{err}");
                assert!(
                    diagnostics.contains(STALE_MESSAGE),
                    "{command}: {diagnostics}"
                );
                assert!(
                    diagnostics.contains("error=0, warning=1, info=0 (total=1)"),
                    "{diagnostics}"
                );
                if command == "csv" {
                    assert_eq!(
                        out.lines().nth(1).unwrap(),
                        format!(
                            "0,0x0,7,0x12,Time-F2,0,0,2,0,0,0,123,2030,,,{ntf},IEEE1588-{},1,Valid",
                            if ntf == 1 { 2002 } else { 2008 }
                        )
                    );
                }
                if ["summary", "channels", "timeline", "correlate"].contains(&command) {
                    assert!(out.contains("002:00:00:00.000000123"), "{out}");
                }
            }
        }
    }
}
#[test]
fn stale_ptp_json_keeps_packet_location_raw_operands_and_clock() {
    for ntf in [1, 2] {
        let input = Input::new(ntf, 1_893_542_437);
        for command in ["summary", "channels", "jumps", "timeline", "correlate"] {
            let (exit, out, err) = input.output(command, true, "warning");
            assert_eq!(exit, ExitClass::Findings, "{err}");
            let v: Value = serde_json::from_str(&out).unwrap();
            assert_eq!(v["findings"]["warning"], "1");
            assert_eq!(
                v["findings"]["samples"],
                serde_json::json!([{"level":"warning","code":"TF-021","channel":7,"packet":"0","message":STALE_MESSAGE}])
            );
            let t = &v["report"]["channels"][0]["last"];
            assert_eq!(t["year"], 2030);
            assert_eq!(t["day"], 2);
            assert_eq!(t["nanosecond"], 123);
            assert_eq!(t["clock_count_ns"], "86400000000123");
            assert_eq!(t["scale"], "UTC");
            assert_eq!(t["basis"], "derived, stale leap table");
            assert_eq!(v["report"]["reference_count"], "1");
        }
    }
}
#[test]
fn covered_and_missing_offset_ptp_do_not_get_stale_findings() {
    for seconds in [1_483_315_237, 0] {
        let input = Input::new(2, seconds);
        let (_, out, _) = input.output("summary", true, "never");
        let v: Value = serde_json::from_str(&out).unwrap();
        assert!(
            !v["findings"]["samples"]
                .as_array()
                .unwrap()
                .iter()
                .any(|f| f["code"] == "TF-021")
        );
        assert_eq!(v["report"]["channels"][0]["last"].is_null(), seconds == 0);
    }
}
#[test]
fn csv_is_its_own_machine_format_and_has_no_json_mode() {
    let input = Input::new(2, 1_483_315_237);
    let (exit, out, _) = input.output("csv", true, "never");
    assert_eq!(exit, ExitClass::Usage);
    assert!(out.is_empty());
}

fn coverage_pair(reset: bool) -> Input {
    // C72's last covered UTC second1814140799, +37 TAI; next second stale.
    let covered = Input::new(2, 1_814_140_836);
    let stale = Input::new(2, 1_814_140_837);
    let mut bytes = std::fs::read(&covered.0).unwrap();
    let mut next = std::fs::read(&stale.0).unwrap();
    next[16..22].copy_from_slice(&10_000_000u64.to_le_bytes()[..6]);
    if reset {
        bytes[16..22].copy_from_slice(&400_000_000u64.to_le_bytes()[..6]);
    }
    for p in [&mut bytes, &mut next] {
        let sum = p[..22].chunks_exact(2).fold(0u16, |s, w| {
            s.wrapping_add(u16::from_le_bytes([w[0], w[1]]))
        });
        p[22..24].copy_from_slice(&sum.to_le_bytes());
    }
    bytes.extend(next);
    std::fs::write(&covered.0, bytes).unwrap();
    covered
}
const COVERED_LABEL: &str = "UTC (from TAI via leap table)";
const STALE_LABEL: &str = "UTC (from TAI via stale leap table, possibly out of date)";
#[test]
fn coverage_edge_basis_is_retained_in_all_clocks_and_text() {
    let input = coverage_pair(false);
    for command in ["summary", "channels", "timeline"] {
        let (_, out, _) = input.output(command, false, "never");
        assert!(
            out.lines()
                .any(|s| s.contains("178:23:59:59.000000123") && s.contains(COVERED_LABEL)),
            "{command}: {out}"
        );
        assert!(
            out.lines()
                .any(|s| s.contains("179:00:00:00.000000123") && s.contains(STALE_LABEL)),
            "{command}: {out}"
        );
    }
    for command in ["summary", "channels", "jumps", "timeline", "correlate"] {
        let (_, out, _) = input.output(command, true, "never");
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["schema_revision"], 12);
        let ch = &v["report"]["channels"][0];
        assert_eq!(ch["first"]["basis"], "derived, leap table");
        assert_eq!(ch["last"]["basis"], "derived, stale leap table");
        assert_eq!(ch["segments"][0]["first"]["basis"], "derived, leap table");
        assert_eq!(
            ch["segments"][0]["last"]["basis"],
            "derived, stale leap table"
        );
        if command == "timeline" {
            assert_eq!(
                v["report"]["rows"][0]["clock"]["basis"],
                "derived, leap table"
            );
            assert_eq!(
                v["report"]["rows"][1]["clock"]["basis"],
                "derived, stale leap table"
            );
        }
    }
    for (rtc, label, basis) in [
        (0, COVERED_LABEL, "derived, leap table"),
        (1, STALE_LABEL, "derived, stale leap table"),
    ] {
        let (_, out, _) = input.output_at("correlate", false, "never", rtc);
        assert!(
            out.lines()
                .filter(|s| s.contains('→'))
                .all(|s| s.contains(label)),
            "{out}"
        );
        let (_, out, _) = input.output_at("correlate", true, "never", rtc);
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["report"]["correlation"]["any"]["clock"]["basis"], basis);
        assert_eq!(
            v["report"]["correlation"]["answers"][0]["clock"]["basis"],
            basis
        );
    }
}
#[test]
fn closed_stale_segment_answer_keeps_basis_and_v4_consumer_declines_new_enum() {
    let input = coverage_pair(true);
    let (_, out, _) = input.output_at("correlate", true, "never", 1);
    let v: Value = serde_json::from_str(&out).unwrap();
    let ch = &v["report"]["channels"][0];
    assert_eq!(ch["segments"][0]["last"]["basis"], "derived, leap table");
    assert_eq!(
        ch["segments"][1]["last"]["basis"],
        "derived, stale leap table"
    );
    let answer = &v["report"]["correlation"]["answers"][0]["clock"];
    assert_eq!(answer["basis"], "derived, stale leap table");
    // Minimal revision4 consumer: accept supported major version, ignore added
    // fields, but decline an unsupported basis enum without rejecting document.
    fn v4_view(basis: &str) -> &str {
        if [
            "standard",
            "stated FMT",
            "assumed, RDG-009",
            "derived, leap table",
            "unresolved",
        ]
        .contains(&basis)
        {
            "supported"
        } else {
            "unsupported-view"
        }
    }
    assert_eq!(v["schema_version"], 1);
    assert_eq!(
        v4_view(ch["segments"][0]["last"]["basis"].as_str().unwrap()),
        "supported"
    );
    assert_eq!(
        v4_view(answer["basis"].as_str().unwrap()),
        "unsupported-view"
    );
    let (_, out, _) = input.output_at("correlate", false, "never", 1);
    assert!(
        out.lines()
            .any(|s| s.contains("Segment   2") && s.contains(STALE_LABEL)),
        "{out}"
    );
}

#[test]
fn stale_answer_basis_is_not_replaced_by_later_covered_channel_state() {
    fn wire(seconds: u32, rtc: u64) -> Vec<u8> {
        let p = Input::new(2, seconds);
        let mut bytes = std::fs::read(&p.0).unwrap();
        bytes[16..22].copy_from_slice(&(rtc * 10_000_000).to_le_bytes()[..6]);
        let sum = bytes[..22].chunks_exact(2).fold(0u16, |s, w| {
            s.wrapping_add(u16::from_le_bytes([w[0], w[1]]))
        });
        bytes[22..24].copy_from_slice(&sum.to_le_bytes());
        bytes
    }
    // Three overlapping reset windows; the final channel state is covered.
    let input = Input::new(2, 1_814_140_796);
    let bytes = [
        wire(1_814_140_796, 0),
        wire(1_814_140_836, 40),
        wire(1_814_140_837, 0),
        wire(1_814_140_877, 40),
        wire(1_483_315_237, 0),
        wire(1_483_315_238, 1),
    ]
    .concat();
    std::fs::write(&input.0, bytes).unwrap();
    let (_, out, _) = input.output("correlate", true, "never");
    let v: Value = serde_json::from_str(&out).unwrap();
    let answers = v["report"]["correlation"]["answers"].as_array().unwrap();
    assert_eq!(answers.len(), 3);
    assert_eq!(
        answers
            .iter()
            .map(|a| a["clock"]["basis"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "derived, leap table",
            "derived, stale leap table",
            "derived, leap table"
        ]
    );
    let (_, out, _) = input.output("correlate", false, "never");
    assert!(
        out.lines()
            .any(|s| s.contains("Segment   2") && s.contains(STALE_LABEL)),
        "{out}"
    );
    // Reset-own-time path must retain its basis; unassociated data stays empty.
    let input = coverage_pair(true);
    let (_, out, _) = input.output("timeline", true, "never");
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        v["report"]["rows"][1]["clock"]["basis"],
        "derived, stale leap table"
    );
    let (_, out, _) = input.output("timeline", false, "never");
    assert!(
        out.lines()
            .any(|s| s.contains("179:00:00:00.000000123") && s.contains(STALE_LABEL))
    );
}

#[test]
fn decoded_packet_row_basis_follows_selected_reference_not_latest_channel() {
    // 106-24R1 Fig11-13: IRIG-B DOY100 12:00:00/01, CSDW0, BCD body.
    // Fig11-15/17: ch9 PTP2008 TS1, stale TAI1893542437/38 (+123ns).
    // Equal RTCs select lower ch7 in the existing any-channel correlator.
    for ptp_first in [false, true] {
        let input = Input::new(2, 1_893_542_437);
        let template = std::fs::read(&input.0).unwrap();
        let wire = |ptp: bool, second: u8| {
            let mut p = template.clone();
            p[2] = if ptp { 9 } else { 7 };
            p[16..22].copy_from_slice(&(u64::from(second) * 10_000_000).to_le_bytes()[..6]);
            if ptp {
                p[28..32].copy_from_slice(&(1_893_542_437u32 + u32::from(second)).to_le_bytes());
            } else {
                p[8] = 10;
                p[15] = 0x11;
                p[24..].fill(0);
                p[28..34].copy_from_slice(&[0, second, 0, 0x12, 0, 1]);
            }
            let sum = p[..22].chunks_exact(2).fold(0u16, |s, w| {
                s.wrapping_add(u16::from_le_bytes([w[0], w[1]]))
            });
            p[22..24].copy_from_slice(&sum.to_le_bytes());
            p
        };
        let mut bytes = Vec::new();
        for second in [0, 1] {
            bytes.extend(wire(ptp_first, second));
            bytes.extend(wire(!ptp_first, second));
        }
        // Once both exact anchors exist, this decoded PTP row must use IRIG,
        // even though its own channel's latest reference has a stale basis.
        bytes.extend(wire(true, 1));
        std::fs::write(&input.0, bytes).unwrap();
        let (exit, out, err) = input.output("timeline", true, "never");
        assert_eq!(exit, ExitClass::Success, "{err}");
        let v: Value = serde_json::from_str(&out).unwrap();
        let rows = v["report"]["rows"].as_array().unwrap();
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[4]["channel"], 9);
        assert_eq!(rows[4]["clock"]["day"], 100);
        assert_eq!(rows[4]["clock"]["second"], 1);
        let (_, text, _) = input.output("timeline", false, "never");
        for row in rows.iter().filter(|row| row["clock"]["day"] == 100) {
            assert_eq!(row["clock"]["basis"], "assumed, RDG-009", "{row}");
            assert_eq!(row["clock"]["scale"], "UTC", "{row}");
            let line = text
                .lines()
                .find(|line| line.split_whitespace().next() == row["packet"].as_str())
                .unwrap();
            assert!(!line.contains("leap table"), "{line}");
            assert!(line.contains("100:12:00:"), "{line}");
        }
    }
}
