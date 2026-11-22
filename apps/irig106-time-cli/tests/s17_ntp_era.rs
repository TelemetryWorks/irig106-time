//! CLI-106 / RDG-012 adapter clauses only; no whole Requirements marker.
use irig106_time_cli::{ExitClass, run};
use serde_json::Value;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
struct Input(PathBuf);
impl Drop for Input {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
impl Input {
    fn new(seconds: u32, fraction: u32) -> Self {
        // 106-24R1 Fig11-15/16(11-27/PDF35): NTF0,TS1, seconds/fraction.
        // Header11.2.1.1: version0x0C,36 bytes,Data Length12, LE word sum.
        let mut p = vec![0; 36];
        p[..2].copy_from_slice(&[0x25, 0xeb]);
        p[2] = 9;
        p[4] = 36;
        p[8] = 12;
        p[12] = 0x0c;
        p[15] = 0x12;
        p[24] = 1;
        p[28..32].copy_from_slice(&seconds.to_le_bytes());
        p[32..36].copy_from_slice(&fraction.to_le_bytes());
        checksum(&mut p);
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "s17-era-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, p).unwrap();
        Self(path)
    }
    fn output(
        &self,
        command: &str,
        json: bool,
        era: Option<&str>,
        level: &str,
    ) -> (ExitClass, String, String) {
        let mut args = vec![command, self.0.to_str().unwrap()];
        if command == "correlate" {
            args.push("0");
        }
        if json {
            args.extend(["--format", "json"]);
        }
        if let Some(era) = era {
            args.extend(["--ntp-era", era]);
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
fn checksum(p: &mut [u8]) {
    let sum = p[..22].chunks_exact(2).fold(0u16, |s, w| {
        s.wrapping_add(u16::from_le_bytes([w[0], w[1]]))
    });
    p[22..24].copy_from_slice(&sum.to_le_bytes());
}
#[test]
fn declared_era_reaches_all_commands_and_clocks_without_37_second_shift() {
    // RFC5905 Fig4: era1 offset63104 =2036-02-08(day39), fraction1/2s.
    let input = Input::new(63104, 0x80000000);
    for cmd in [
        "summary",
        "channels",
        "jumps",
        "timeline",
        "csv",
        "correlate",
    ] {
        let (exit, text, err) = input.output(cmd, false, Some("1"), "error");
        assert_eq!(exit, ExitClass::Success, "{err}");
        assert!(!format!("{text}{err}").contains("TF-021"));
        if cmd == "csv" {
            assert!(
                text.lines()
                    .nth(1)
                    .unwrap()
                    .contains(",39,0,0,0,500000000,2036,,,0,NTP,1,Valid")
            );
            continue;
        }
        if cmd != "jumps" {
            assert!(text.contains("039:00:00:00.500000000"), "{text}");
            assert!(text.contains("caller-declared era"), "{text}");
        }
        let (exit, out, err) = input.output(cmd, true, Some("1"), "error");
        assert_eq!(exit, ExitClass::Success, "{err}");
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["options"]["ntp_era"], "1");
        let mut clocks = vec![&v["report"]["channels"][0]["first"]];
        if cmd == "timeline" {
            clocks.push(&v["report"]["rows"][0]["clock"]);
        }
        if cmd == "correlate" {
            clocks.extend([
                &v["report"]["correlation"]["any"]["clock"],
                &v["report"]["correlation"]["answers"][0]["clock"],
            ]);
        }
        for c in clocks {
            assert_eq!(c["year"], 2036);
            assert_eq!(c["second"], 0);
            assert_eq!(c["nanosecond"], 500000000);
            assert_eq!(c["scale"], "UTC");
            assert!(c["basis"].as_str().unwrap().contains("caller-declared era"));
        }
    }
}
#[test]
fn no_era_never_uses_a_dated_f1_pivot_or_assumes_era_zero() {
    let input = Input::new(3692304000, 0); //2017-01-02 per RFC epoch arithmetic.
    let (_, out, _) = input.output("summary", true, None, "never");
    let v: Value = serde_json::from_str(&out).unwrap();
    assert!(v["report"]["channels"][0]["first"].is_null());
    // Fig11-14: FMT4 DATE1,2017-01-02. Still not a caller declaration of NTP era.
    let mut f1 = vec![0; 36];
    f1[..2].copy_from_slice(&[0x25, 0xeb]);
    f1[2] = 7;
    f1[4] = 36;
    f1[8] = 12;
    f1[15] = 0x11;
    f1[24..36].copy_from_slice(&[0x41, 2, 0, 0, 0, 0, 0, 0, 2, 1, 0x17, 0x20]);
    checksum(&mut f1);
    f1.extend(std::fs::read(&input.0).unwrap());
    std::fs::write(&input.0, f1).unwrap();
    let (_, out, _) = input.output("summary", true, None, "never");
    let v: Value = serde_json::from_str(&out).unwrap();
    assert!(v["options"]["ntp_era"].is_null());
    assert!(v["report"]["channels"][1]["first"].is_null());
    assert_eq!(v["report"]["references_seen"], "1");
    assert!(
        v["findings"]["samples"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["message"]
                .as_str()
                .unwrap()
                .contains("NTP era/basis required"))
    );
}
#[test]
fn utc_epoch_dates_fraction_floor_zero_and_ambiguity_follow_rdg012() {
    // RFC5905 Fig4 + post2016 leap date. Unix = era*2^32+raw-2208988800.
    for (raw, era, year, day) in [
        (2208988800, "0", 1970, 1),
        (2272060800, "0", 1972, 1),
        (3155587200, "0", 1999, 365),
        (3692304000, "0", 2017, 2),
    ] {
        let input = Input::new(raw, 0xffffffff);
        let (exit, out, err) = input.output("timeline", true, Some(era), "error");
        assert_eq!(exit, ExitClass::Success, "{err}");
        let v: Value = serde_json::from_str(&out).unwrap();
        let c = &v["report"]["rows"][0]["clock"];
        assert_eq!(c["year"], year);
        assert_eq!(c["day"], day);
        assert_eq!(c["second"], 0);
        assert_eq!(c["nanosecond"], 999999999);
    }
    for era in [None, Some("0"), Some("1")] {
        let input = Input::new(0, 0);
        let (exit, out, _) = input.output("timeline", true, era, "warning");
        assert_eq!(exit, ExitClass::Findings);
        let v: Value = serde_json::from_str(&out).unwrap();
        assert!(v["report"]["rows"][0]["clock"].is_null());
        assert!(
            v["findings"]["samples"]
                .as_array()
                .unwrap()
                .iter()
                .any(|f| f["code"] == "TF-029" && f["level"] == "warning")
        );
    }
    let input = Input::new(3692217600, 0x80000000); //2017-01-01 insertion coordinate.
    let (exit, out, _) = input.output("timeline", true, Some("0"), "warning");
    assert_eq!(exit, ExitClass::Findings);
    let v: Value = serde_json::from_str(&out).unwrap();
    assert!(
        v["report"]["rows"][0]["clock"]["basis"]
            .as_str()
            .unwrap()
            .contains("leap-ambiguous")
    );
    assert!(
        v["findings"]["samples"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["code"] == "TF-030" && f["level"] == "warning")
    );
}
#[test]
fn era_syntax_errors_precede_input_and_extreme_values_never_panic() {
    for bad in [
        "",
        "+1",
        "1x",
        "1.0",
        "0x1",
        "9223372036854775808",
        "-9223372036854775809",
    ] {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert_eq!(
            run(
                &["summary", "missing.ch10", "--ntp-era", bad],
                &mut out,
                &mut err
            ),
            ExitClass::Usage
        );
    }
    for tail in [vec!["--ntp-era"], vec!["--ntp-era", "0", "--ntp-era", "1"]] {
        let mut args = vec!["summary", "missing.ch10"];
        args.extend(tail);
        assert_eq!(
            run(&args, &mut Vec::new(), &mut Vec::new()),
            ExitClass::Usage
        );
    }
    let input = Input::new(63104, 0);
    for era in [
        "-1",
        "0",
        "1",
        "9223372036854775807",
        "-9223372036854775808",
    ] {
        let result = std::panic::catch_unwind(|| input.output("summary", true, Some(era), "never"));
        assert!(result.is_ok());
        let (exit, out, _) = result.unwrap();
        assert_eq!(exit, ExitClass::Success);
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["options"]["ntp_era"], era);
    }
}
