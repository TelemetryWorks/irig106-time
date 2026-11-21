//! Partial CLI-105/106 text clauses only; no whole-requirement markers.
use irig106_time_cli::{ExitClass, run};
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
            "s17-report-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, bytes).unwrap();
        Self(path)
    }
    fn text(&self, command: &str) -> String {
        let mut out = Vec::new();
        assert_eq!(
            run(
                &[command, self.0.to_str().unwrap(), "--fail-on", "never"],
                &mut out,
                &mut Vec::new()
            ),
            ExitClass::Success
        );
        String::from_utf8(out).unwrap()
    }
}
impl Drop for Input {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
fn packet(kind: u8, rtc: u64, payload: &[u8]) -> Vec<u8> {
    // §11.2.1.1: length includes filler, Data Length excludes it; LE word sum.
    let mut p = vec![0; (24 + payload.len()).next_multiple_of(4)];
    p[..2].copy_from_slice(&[0x25, 0xeb]);
    p[2] = 7;
    let len = p.len() as u32;
    p[4..8].copy_from_slice(&len.to_le_bytes());
    p[8..12].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    p[12] = 6;
    p[15] = kind;
    p[16..22].copy_from_slice(&rtc.to_le_bytes()[..6]);
    p[24..24 + payload.len()].copy_from_slice(payload);
    let sum = p[..22].chunks_exact(2).fold(0u16, |sum, w| {
        sum.wrapping_add(u16::from_le_bytes([w[0], w[1]]))
    });
    p[22..24].copy_from_slice(&sum.to_le_bytes());
    p
}
#[test]
fn jumps_render_doy_clock_not_counted_nanoseconds() {
    // Fig11-13: day100 12:00:00 -> 12:00:06 at RTC+1s: expected01, actual06.
    let input = Input::new(
        &[
            packet(0x11, 0, &[0, 0, 0, 0, 0, 0, 0, 0x12, 0, 1]),
            packet(0x11, 10_000_000, &[0, 0, 0, 0, 0, 6, 0, 0x12, 0, 1]),
        ]
        .concat(),
    );
    let golden = "  Channel   7 Ref #1    Expected: 100:12:00:01.000000000 Actual: 100:12:00:06.000000000 Delta: +5000.000 ms";
    for command in ["summary", "jumps"] {
        let text = input.text(command);
        assert!(text.lines().any(|line| line == golden), "{text}");
        assert!(!text.contains(" ns Actual:"));
    }
}
#[test]
fn every_network_revision_and_status_is_named_even_without_utc() {
    // Fig11-15 NTF bits7..4, TS bits3..0. Fig11-16 NTP and Fig11-17 PTP:
    // seconds1483315237 TAI -37 = 2017-01-02 UTC, fraction/nanoseconds0.
    for (ntf, label) in [(0, "NTP"), (1, "IEEE 1588-2002"), (2, "IEEE 1588-2008")] {
        for (ts, status) in [(0, "NotValid"), (1, "Valid")] {
            let mut payload = vec![(ntf << 4) | ts, 0, 0, 0];
            payload.extend_from_slice(&1483315237u32.to_le_bytes());
            payload.extend_from_slice(&0u32.to_le_bytes());
            let input = Input::new(&packet(0x12, 0, &payload));
            for command in ["summary", "channels"] {
                let text = input.text(command);
                assert!(text.contains(label), "{text}");
                assert!(text.contains(status), "{text}");
                assert!(!text.contains("GPS"), "{text}");
                assert!(
                    text.contains(if ntf == 0 {
                        "UTC (standard)"
                    } else {
                        "TAI (standard)"
                    }),
                    "{text}"
                );
            }
        }
    }
}
#[test]
fn native_gps_is_not_a_network_protocol() {
    // Fig11-12 FMT5/SRC1; Fig11-13 day100 12:00:00.
    let input = Input::new(&packet(0x11, 0, &[0x51, 0, 0, 0, 0, 0, 0, 0x12, 0, 1]));
    for command in ["summary", "channels"] {
        let text = input.text(command);
        assert!(text.contains("GPS"));
        assert!(!text.contains("IEEE 1588"));
    }
}
#[test]
fn report_writer_failure_remains_output_exit() {
    struct Broken;
    impl io::Write for Broken {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("report writer"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let input = Input::new(&packet(0x11, 0, &[0, 0, 0, 0, 0, 0, 0, 0x12, 0, 1]));
    for command in ["summary", "channels", "jumps"] {
        assert_eq!(
            run(
                &[command, input.0.to_str().unwrap()],
                &mut Broken,
                &mut Vec::new()
            ),
            ExitClass::Output
        );
    }
}

fn channel(mut p: Vec<u8>, id: u16) -> Vec<u8> {
    p[2..4].copy_from_slice(&id.to_le_bytes());
    let sum = p[..22].chunks_exact(2).fold(0u16, |s, w| {
        s.wrapping_add(u16::from_le_bytes([w[0], w[1]]))
    });
    p[22..24].copy_from_slice(&sum.to_le_bytes());
    p
}
fn network(word: u8, ns: u32, rtc: u64) -> Vec<u8> {
    // Fig11-15/17: NTF2/TS1, 1483315237 TAI ->2017-002 UTC with offset37.
    let mut body = vec![word, 0, 0, 0];
    body.extend_from_slice(&1483315237u32.to_le_bytes());
    body.extend_from_slice(&ns.to_le_bytes());
    packet(0x12, rtc, &body)
}
#[test]
fn mixed_inventory_columns_match_golden_and_label_converted_clocks() {
    let input = Input::new(
        &[
            channel(packet(0x11, 0, &[0, 0, 0, 0, 0, 0, 0, 0x12, 0, 1]), 1),
            channel(network(1, 0, 0), 2),
            channel(network(0x21, 123456, 0), 3),
            channel(network(0x31, 0, 0), 4),
        ]
        .concat(),
    );
    assert_eq!(
        input.text("channels"),
        include_str!("golden/s17_channels.txt")
    );
    let summary = input.text("summary");
    assert!(summary.contains("UTC (from TAI via leap table)"));
    assert!(summary.contains("Reserved NTF"));
    assert!(summary.contains("unknown scale"));
}
#[test]
fn report_rollover_is_not_a_jump_but_boundary_discontinuity_is() {
    // Fig11-12 LY; Fig11-13: day365/366 23:59:59 ->day1 midnight, RTC+1s.
    for (day, ly) in [(0x65, 0), (0x66, 1)] {
        let first = packet(0x11, 0, &[0, ly, 0, 0, 0, 0x59, 0x59, 0x23, day, 3]);
        for (second, jumps) in [(0, 0), (5, 1)] {
            let input = Input::new(
                &[
                    first.clone(),
                    packet(0x11, 10_000_000, &[0, ly, 0, 0, 0, second, 0, 0, 1, 0]),
                ]
                .concat(),
            );
            for command in ["summary", "jumps"] {
                let text = input.text(command);
                if jumps == 0 {
                    assert!(!text.contains("Expected:"), "{text}");
                } else {
                    assert!(
                        text.contains("Expected: 001:00:00:00.000000000 (+1 year)"),
                        "{text}"
                    );
                    assert!(
                        text.contains("Actual: 001:00:00:05.000000000 (+1 year)"),
                        "{text}"
                    );
                    assert!(text.contains("Delta: +5000.000 ms"), "{text}");
                }
                assert!(!text.contains("367:"), "{text}");
            }
            assert!(input.text("summary").contains("001:00:00:"));
            assert!(input.text("channels").contains("(+1 year)"));
        }
    }
}
#[test]
fn inventory_without_references_does_not_add_query_findings_or_jump_channels() {
    let good = packet(0x11, 0, &[0, 0, 0, 0, 0, 0, 0, 0x12, 0, 1]);
    let input = Input::new(&[good, channel(network(1, 0, 0), 2)].concat());
    let jumps = input.text("jumps");
    assert!(
        jumps.contains("No time jumps detected across 1 channel(s)."),
        "{jumps}"
    );
    let (mut out, mut err) = (Vec::new(), Vec::new());
    assert_eq!(
        run(
            &[
                "correlate",
                input.0.to_str().unwrap(),
                "0",
                "--fail-on",
                "never"
            ],
            &mut out,
            &mut err
        ),
        ExitClass::Success
    );
    let err = String::from_utf8(err).unwrap();
    assert!(err.contains("warning=1"), "{err}");
    assert!(!err.contains("Correlation query:"), "{err}");
}
#[test]
fn sub_millisecond_clock_and_f2_to_f1_labels_are_preserved() {
    let input = Input::new(
        &[
            network(0x21, 123456, 0),
            packet(0x11, 10_000_000, &[0, 0, 0, 0, 0, 0, 0, 0x12, 0, 1]),
        ]
        .concat(),
    );
    let text = input.text("channels");
    assert!(
        text.contains(".000123456 UTC (from TAI via leap table)"),
        "{text}"
    );
    // Current inventory switches to F1; the first PTP clock keeps its own basis.
    assert!(!text.contains("IEEE1588"), "{text}");
    assert!(!text.contains("IEEE 1588"), "{text}");
    assert!(text.contains("IRIG-B"), "{text}");
}

#[test]
fn segment_finishing_skips_inventory_without_references() {
    let input = Input::new(
        &[
            packet(0x11, 400_000_000, &[0, 0, 0, 0, 0, 0x40, 0, 0x12, 0, 1]),
            packet(0x11, 10_000_000, &[0, 0, 0, 0, 0, 1, 0, 0x12, 0, 1]),
            channel(network(1, 0, 0), 2),
        ]
        .concat(),
    );
    let (mut out, mut err) = (Vec::new(), Vec::new());
    assert_eq!(
        run(
            &[
                "correlate",
                input.0.to_str().unwrap(),
                "989680",
                "--fail-on",
                "never"
            ],
            &mut out,
            &mut err
        ),
        ExitClass::Success
    );
    let err = String::from_utf8(err).unwrap();
    assert!(err.contains("warning=2"), "{err}");
    assert!(!err.contains("Segment query:"), "{err}");
    assert!(!err.contains("No reset segment covers"), "{err}");
}
