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
        let mut args = vec![command, self.0.to_str().unwrap()];
        if command == "correlate" {
            args.push("0");
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
                assert!(err.contains(STALE_MESSAGE), "{command}: {err}");
                assert!(
                    err.contains("error=0, warning=1, info=0 (total=1)"),
                    "{err}"
                );
                if command == "csv" {
                    assert_eq!(
                        out.lines().nth(1).unwrap(),
                        "0,0x0,7,0x12,Time-F2,0,0,2,0,0,0,123,2030,,"
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
            assert_eq!(t["basis"], "derived, leap table");
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
