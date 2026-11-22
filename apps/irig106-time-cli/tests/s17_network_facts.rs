//! CLI-105 partial packet/query protocol clauses; no whole Requirements marker.
use irig106_time_cli::{ExitClass, run};
use serde_json::{Value, json};
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
    fn new(bytes: &[u8]) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "s17-network-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, bytes).unwrap();
        Self(path)
    }
    fn output(&self, command: &str, json: bool) -> String {
        let mut args = vec![command, self.0.to_str().unwrap()];
        if command == "correlate" {
            args.push("2FAF080");
        } // RTC5s
        if json {
            args.extend(["--format", "json"]);
        }
        args.extend(["--fail-on", "never"]);
        let mut out = Vec::new();
        assert_eq!(run(&args, &mut out, &mut Vec::new()), ExitClass::Success);
        String::from_utf8(out).unwrap()
    }
}
fn packet(ch: u16, rtc: u64, kind: u8, body: &[u8]) -> Vec<u8> {
    // 106-24R1 §11.2.1.1: valid LE word-sum,100ns RTC, Data Length excludes filler.
    let mut p = vec![0; (24 + body.len()).next_multiple_of(4)];
    p[..2].copy_from_slice(&[0x25, 0xeb]);
    p[2..4].copy_from_slice(&ch.to_le_bytes());
    let len = p.len() as u32;
    p[4..8].copy_from_slice(&len.to_le_bytes());
    p[8..12].copy_from_slice(&(body.len() as u32).to_le_bytes());
    p[15] = kind;
    p[16..22].copy_from_slice(&(rtc * 10_000_000).to_le_bytes()[..6]);
    p[24..24 + body.len()].copy_from_slice(body);
    let sum = p[..22].chunks_exact(2).fold(0u16, |s, w| {
        s.wrapping_add(u16::from_le_bytes([w[0], w[1]]))
    });
    p[22..24].copy_from_slice(&sum.to_le_bytes());
    p
}
fn f2(ntf: u8, ts: u8, rtc: u64, offset: u32) -> Vec<u8> {
    // Fig11-15 (11-27/PDF35): NTF7..4,TS3..0; Fig11-17 PTP body.
    // Unix2017-01-02=1483315200 +37s=1483315237 TAI; fraction0ns.
    let mut body = vec![(ntf << 4) | ts, 0, 0, 0];
    body.extend((1483315237 + offset).to_le_bytes());
    body.extend([0; 4]);
    packet(9, rtc, 0x12, &body)
}
fn facts(ntf: u8, ts: u8) -> Value {
    json!({"ntf":ntf,"protocol":match ntf {0=>"NTP",1=>"IEEE1588-2002",2=>"IEEE1588-2008",_=>"reserved"},
        "ts":ts,"status":match ts {0=>"not-valid",1=>"valid",_=>"reserved"}})
}
#[test]
fn timeline_preserves_observed_protocol_and_status_even_without_a_clock() {
    for ntf in 0..=15 {
        for ts in 0..=15 {
            let input = Input::new(&f2(ntf, ts, 0, 0));
            let v: Value = serde_json::from_str(&input.output("timeline", true)).unwrap();
            let row = &v["report"]["rows"][0];
            assert_eq!(row["network_time"], facts(ntf, ts));
            assert_eq!(row["clock"].is_null(), !(matches!(ntf, 1 | 2) && ts == 1));
            let text = input.output("timeline", false);
            let protocol = if ntf >= 3 {
                format!("Reserved NTF {ntf}")
            } else {
                facts(ntf, ts)["protocol"].as_str().unwrap().to_owned()
            };
            assert!(
                text.contains(&format!(
                    "[packet: {protocol} TS={ts} {}]",
                    facts(ntf, ts)["status"].as_str().unwrap()
                )),
                "{text}"
            );
        }
    }
    for body in [&[0x21, 0][..], &[0x21, 0, 0, 0][..]] {
        let input = Input::new(&packet(9, 0, 0x12, body));
        let v: Value = serde_json::from_str(&input.output("timeline", true)).unwrap();
        let row = &v["report"]["rows"][0];
        assert!(row["clock"].is_null());
        assert_eq!(
            row["network_time"],
            if body.len() == 4 {
                facts(2, 1)
            } else {
                Value::Null
            }
        );
    }
}
#[test]
fn correlate_retains_selected_protocol_across_later_updates_and_reset_spool() {
    let input = Input::new(&[f2(1, 1, 0, 0), f2(2, 1, 40, 40)].concat());
    let v: Value = serde_json::from_str(&input.output("correlate", true)).unwrap();
    assert_eq!(
        v["report"]["correlation"]["any"]["network_time"],
        facts(1, 1)
    );
    assert_eq!(
        v["report"]["correlation"]["answers"][0]["network_time"],
        facts(1, 1)
    );
    assert!(
        input
            .output("correlate", false)
            .contains("[reference: IEEE1588-2002 TS=1 valid]")
    );
    let reset = Input::new(
        &[
            f2(1, 1, 0, 0),
            f2(2, 1, 40, 40),
            f2(2, 1, 1, 61),
            f2(2, 1, 10, 70),
        ]
        .concat(),
    );
    let v: Value = serde_json::from_str(&reset.output("correlate", true)).unwrap();
    let answers = v["report"]["correlation"]["answers"].as_array().unwrap();
    assert_eq!(answers.len(), 2);
    assert_eq!(answers[0]["network_time"], facts(1, 1));
    assert_eq!(answers[1]["network_time"], facts(2, 1));
    assert!(v["report"]["correlation"]["any"]["network_time"].is_null());
    let text = reset.output("correlate", false);
    assert!(
        text.lines()
            .any(|l| l.contains("Segment   1")
                && l.contains("[reference: IEEE1588-2002 TS=1 valid]"))
    );
    assert!(
        text.lines()
            .any(|l| l.contains("Segment   2")
                && l.contains("[reference: IEEE1588-2008 TS=1 valid]"))
    );
}
#[test]
fn packet_observation_is_distinct_from_the_query_reference() {
    // Fig11-12/13: ch7 IRIG-B,DOY10012:00; any-channel equal-RTC tie selects7.
    let irig = packet(7, 0, 0x11, &[1, 0, 0, 0, 0, 0, 0, 0x12, 0, 1]);
    let input = Input::new(&[irig, f2(2, 1, 0, 0), packet(8, 0, 9, &[])].concat());
    let v: Value = serde_json::from_str(&input.output("timeline", true)).unwrap();
    let rows = &v["report"]["rows"];
    assert!(rows[0]["network_time"].is_null());
    assert_eq!(rows[1]["network_time"], facts(2, 1));
    assert_eq!(rows[1]["clock"]["day"], 100);
    assert!(rows[2]["network_time"].is_null());
    let v: Value = serde_json::from_str(&input.output("correlate", true)).unwrap();
    assert!(v["report"]["correlation"]["any"]["network_time"].is_null());
}

#[test]
fn invalid_packet_label_is_explicit_when_clock_comes_from_other_protocol() {
    // Fig11-15/17: ch9 PTP2008 TS1 atRTC0; ch10 PTP2002 TS1 atRTC1;
    // ch9 PTP2008 TS0 atRTC2. Its body cannot supply a usable clock.
    let mut second = f2(1, 1, 1, 100);
    second[2] = 10;
    let sum = second[..22].chunks_exact(2).fold(0u16, |s, w| {
        s.wrapping_add(u16::from_le_bytes([w[0], w[1]]))
    });
    second[22..24].copy_from_slice(&sum.to_le_bytes());
    let input = Input::new(&[f2(2, 1, 0, 0), second, f2(2, 0, 2, 500)].concat());
    let v: Value = serde_json::from_str(&input.output("timeline", true)).unwrap();
    let row = &v["report"]["rows"][2];
    assert_eq!(row["network_time"], facts(2, 0));
    // Nearest usable reference is ch10 atRTC1, UTC00:01:40; +1s=00:01:41.
    // ch9's preceding reference would give00:00:02; its invalid body gives00:08:20.
    assert_eq!(row["clock"]["minute"], 1);
    assert_eq!(row["clock"]["second"], 41);
    let text = input.output("timeline", false);
    let line = text
        .lines()
        .find(|l| l.split_whitespace().next() == Some("2"))
        .unwrap();
    assert!(line.contains("002:00:01:41.000000000"), "{line}");
    assert!(
        line.ends_with("[packet: IEEE1588-2008 TS=0 not-valid]"),
        "{line}"
    );
    assert!(!line.contains("[reference:"), "{line}");
    let query = input.output("correlate", false);
    let any = query.lines().find(|l| l.contains("Any channel")).unwrap();
    assert!(any.contains("002:00:01:44.000000000"), "{any}");
    assert!(
        any.ends_with("[reference: IEEE1588-2002 TS=1 valid]"),
        "{any}"
    );
}

#[test]
fn csv_packet_fact_columns_append_to_every_legacy_row_golden() {
    // Fig11-12/13 FMT4/SRC1 DOY10012:00; Fig11-15/17 packet facts.
    // PTP UTC2017-01-02 atRTC1; later invalid/malformed packets use that
    // reference's clock, but their CSV protocol/status remain packet-observed.
    let input = Input::new(
        &[
            packet(7, 0, 0x11, &[0x41, 0, 0, 0, 0, 0, 0, 0x12, 0, 1]),
            f2(2, 1, 1, 0),
            f2(1, 0, 2, 999),
            f2(15, 15, 3, 999),
            packet(9, 4, 0x12, &[0x21, 0]),
            packet(9, 5, 0x12, &[0x21, 0, 0, 0]),
            packet(8, 6, 9, &[]),
        ]
        .concat(),
    );
    let out = input.output("csv", false);
    assert_eq!(out, include_str!("golden/s17_packet_facts.csv"));
    let rows = out
        .lines()
        .map(|l| l.split(',').collect::<Vec<_>>())
        .collect::<Vec<_>>();
    assert!(rows.iter().all(|r| r.len() == 19));
    // Append-only contract: the first15 names stay in their original order.
    assert_eq!(
        rows[0][..15].join(","),
        "packet_num,offset_hex,channel_id,data_type_hex,data_type_name,rtc_raw,rtc_nanos,day_of_year,hours,minutes,seconds,nanoseconds,year,month,day"
    );
    assert_eq!(&rows[3][15..], ["1", "IEEE1588-2002", "0", "NotValid"]);
    assert_eq!(rows[3][10], "1"); // Not the invalid packet's raw body (999s).
    for index in [1, 5, 7] {
        assert_eq!(&rows[index][15..], ["", "", "", ""]);
    }
}

#[test]
fn csv_raw_and_named_status_are_exact_for_all_nibbles() {
    for ntf in 0..=15 {
        for ts in 0..=15 {
            let input = Input::new(&f2(ntf, ts, 0, 0));
            let out = input.output("csv", false);
            let row = out.lines().nth(1).unwrap().split(',').collect::<Vec<_>>();
            assert_eq!(row.len(), 19);
            let protocol = match ntf {
                0 => "NTP",
                1 => "IEEE1588-2002",
                2 => "IEEE1588-2008",
                _ => "reserved",
            };
            let status = match ts {
                0 => "NotValid".into(),
                1 => "Valid".into(),
                _ => format!("RSVD-{ts}"),
            };
            assert_eq!(row[15], ntf.to_string());
            assert_eq!(row[16], protocol);
            assert_eq!(row[17], ts.to_string());
            assert_eq!(row[18], status);
            // Independent original15-field prefix, including eight empty calendar
            // fields whenever the single packet cannot supply a usable PTP time.
            let expected = if matches!(ntf, 1 | 2) && ts == 1 {
                "0,0x0,9,0x12,Time-F2,0,0,2,0,0,0,0,2017,,"
            } else {
                "0,0x0,9,0x12,Time-F2,0,0,,,,,,,,"
            };
            assert_eq!(row[..15].join(","), expected);
        }
    }
}
