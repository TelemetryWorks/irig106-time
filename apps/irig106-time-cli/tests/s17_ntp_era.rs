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
    // Full32-bit era, not the RFC half-era pivot interval: final era1 second.
    let edge = Input::new(u32::MAX, u32::MAX);
    let (_, out, _) = edge.output("timeline", true, Some("1"), "error");
    let v: Value = serde_json::from_str(&out).unwrap();
    let c = &v["report"]["rows"][0]["clock"];
    assert_eq!(c["year"], 2172);
    assert_eq!(c["day"], 75);
    assert_eq!(c["second"], 31);
    assert_eq!(c["nanosecond"], 999999999);
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
    // Fraction-only raw values are not the all-zero sentinel, even when floor=0ns.
    let input = Input::new(0, 1);
    let (exit, out, _) = input.output("timeline", true, Some("1"), "error");
    assert_eq!(exit, ExitClass::Success);
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["report"]["rows"][0]["clock"]["second"], 16);
    assert_eq!(v["report"]["rows"][0]["clock"]["nanosecond"], 0);
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
        "-",
        "--1",
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
    for era in ["-1", "0", "1", "-14", "59"] {
        let (exit, out, _) = input.output("summary", true, Some(era), "never");
        assert_eq!(exit, ExitClass::Success);
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["options"]["ntp_era"], era);
    }
    // Exact calendar endpoints within the two partially supported eras.
    for (era, seconds, year, day) in [("-14", 171311744, 0, 1), ("59", 2208219135, 9999, 365)] {
        let edge = Input::new(seconds, 0);
        let (exit, out, _) = edge.output("timeline", true, Some(era), "never");
        assert_eq!(exit, ExitClass::Success);
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["report"]["rows"][0]["clock"]["year"], year);
        assert_eq!(v["report"]["rows"][0]["clock"]["day"], day);
    }
    // Proleptic Gregorian year0 inclusive to year10000 exclusive, relative1900:
    // [-59958230400,255611289600) seconds. Floor-divide endpoints by2^32.
    for (era, expected) in [
        ("-15", ExitClass::Usage),
        ("-14", ExitClass::Input),
        ("59", ExitClass::Input),
        ("60", ExitClass::Usage),
        ("9223372036854775807", ExitClass::Usage),
        ("-9223372036854775808", ExitClass::Usage),
    ] {
        assert_eq!(
            run(
                &["summary", "missing.ch10", "--ntp-era", era],
                &mut Vec::new(),
                &mut Vec::new()
            ),
            expected,
            "{era}"
        );
    }
}

#[test]
fn declared_era_does_not_repair_status_and_survives_reset_spooling() {
    for word in [0, 3, 0x31] {
        let input = Input::new(63104, 0);
        let mut bytes = std::fs::read(&input.0).unwrap();
        bytes[24] = word;
        std::fs::write(&input.0, bytes).unwrap();
        let (_, out, _) = input.output("timeline", true, Some("1"), "never");
        let v: Value = serde_json::from_str(&out).unwrap();
        assert!(v["report"]["rows"][0]["clock"].is_null());
        assert!(
            v["findings"]["samples"]
                .as_array()
                .unwrap()
                .iter()
                .any(|f| f["code"] == if word == 0 { "TF-003" } else { "TF-006" })
        );
    }
    let input = Input::new(63104, 0x80000000);
    let mut bytes = Vec::new();
    for (rtc, seconds) in [(0u64, 63104), (40, 63144), (0, 63164), (10, 63174)] {
        let wire = Input::new(seconds, 0x80000000);
        let mut p = std::fs::read(&wire.0).unwrap();
        p[16..22].copy_from_slice(&(rtc * 10_000_000).to_le_bytes()[..6]);
        checksum(&mut p);
        bytes.extend(p);
    }
    std::fs::write(&input.0, bytes).unwrap();
    let (_, out, _) = input.output("correlate", true, Some("1"), "never");
    let v: Value = serde_json::from_str(&out).unwrap();
    let answers = v["report"]["correlation"]["answers"].as_array().unwrap();
    assert_eq!(answers.len(), 2);
    for (a, minute) in answers.iter().zip([0, 1]) {
        assert_eq!(a["clock"]["minute"], minute);
        assert_eq!(a["clock"]["year"], 2036);
        assert_eq!(
            a["clock"]["basis"],
            "caller-declared era, leap coverage uncertain"
        );
    }
}

#[test]
fn fraction_only_requires_declaration_and_plain_basis_survives_every_path() {
    let input = Input::new(0, 0x80000000);
    let (_, out, _) = input.output("timeline", true, None, "never");
    let v: Value = serde_json::from_str(&out).unwrap();
    assert!(v["report"]["rows"][0]["clock"].is_null());
    assert_eq!(v["report"]["policy"]["ntp_era_source"], "none");
    assert_eq!(v["report"]["policy"]["ntp_era_inference"], "disabled");
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
    // RFC5905 §6: seconds+fraction BOTH zero is the sentinel. With era0,
    // a fraction-only wire value legitimately dates1900-01-01.
    let (_, out, _) = input.output("timeline", true, Some("0"), "never");
    let v: Value = serde_json::from_str(&out).unwrap();
    let c = &v["report"]["rows"][0]["clock"];
    assert_eq!(c["year"], 1900);
    assert_eq!(c["day"], 1);
    assert_eq!(c["nanosecond"], 500000000);
    assert!(
        !v["findings"]["samples"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["code"] == "TF-029")
    );

    // Covered2017-01-02, not ambiguity/uncertain coverage: exact plain basis.
    // Force a reset so correlate reloads the first answer from its segment spool.
    let mut bytes = Vec::new();
    for (rtc, seconds) in [
        (0u64, 3692304000),
        (40, 3692304040),
        (0, 3692304060),
        (10, 3692304070),
    ] {
        let wire = Input::new(seconds, 0x80000000);
        let mut p = std::fs::read(&wire.0).unwrap();
        p[16..22].copy_from_slice(&(rtc * 10_000_000).to_le_bytes()[..6]);
        checksum(&mut p);
        bytes.extend(p);
    }
    std::fs::write(&input.0, bytes).unwrap();
    for cmd in ["summary", "timeline", "correlate"] {
        let (_, out, _) = input.output(cmd, true, Some("0"), "never");
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["report"]["policy"]["ntp_era_source"], "caller-declared");
        assert_eq!(v["report"]["policy"]["ntp_era_inference"], "disabled");
        assert_eq!(
            v["report"]["channels"][0]["first"]["basis"],
            "caller-declared era"
        );
        if cmd == "timeline" {
            for row in v["report"]["rows"].as_array().unwrap() {
                assert_eq!(row["clock"]["basis"], "caller-declared era");
            }
        }
        if cmd == "correlate" {
            let answers = v["report"]["correlation"]["answers"].as_array().unwrap();
            assert_eq!(answers.len(), 2);
            for a in answers {
                assert_eq!(a["clock"]["basis"], "caller-declared era");
            }
        }
    }
}
