//! CLI-105/106 packet-row clauses only; JSON/correlate/qualified joining remain open.
#[path = "common/mod.rs"]
mod common;
use irig106_time_cli::{ExitClass, run};
use std::sync::atomic::{AtomicUsize, Ordering};

fn outputs(bytes: &[u8]) -> (String, String) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let path = std::env::temp_dir().join(format!(
        "s17-rows-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::write(&path, bytes).unwrap();
    let mut results = Vec::new();
    let mut diagnostics = Vec::new();
    for command in ["timeline", "csv"] {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert_eq!(
            run(
                &[command, path.to_str().unwrap(), "--fail-on", "never"],
                &mut out,
                &mut err
            ),
            ExitClass::Success
        );
        results.push(String::from_utf8(out).unwrap());
        diagnostics.push(err);
    }
    std::fs::remove_file(path).unwrap();
    assert_eq!(
        diagnostics[0], diagnostics[1],
        "same row-resolution findings"
    );
    (results.remove(0), results.remove(0))
}

#[test]
fn counter_only_csv_is_rectangular_and_matches_timeline() {
    let (text, csv) = outputs(&common::NO_TIME);
    assert!(text.lines().last().unwrap().ends_with("N/A"));
    let lines: Vec<_> = csv.lines().collect();
    assert_eq!(lines[0].split(',').count(), 15);
    assert_eq!(lines[1].split(',').count(), 15);
    assert_eq!(lines[1], "0,0x0,0,0x09,PCM-1,0,0,,,,,,,,");
}

#[test]
fn observed_clock_and_packet_identity_project_identically() {
    // 106-24R1 §11.2.3.2 Fig11-12/13: FMT4 UTC, DATE0;
    // fraction0x12 = .12s; sec04/min00/hour12/day100. Two filler bytes excluded.
    let mut p = vec![0u8; 36];
    p[..2].copy_from_slice(&[0x25, 0xeb]);
    p[2] = 7;
    p[4] = 36;
    p[8] = 10;
    p[15] = 0x11;
    // §11.2.1.1: 10,000,000 header RTC ticks × 100 ns = 1,000,000,000 ns.
    p[16..22].copy_from_slice(&10_000_000u64.to_le_bytes()[..6]);
    p[24..34].copy_from_slice(&[0x40, 0, 0, 0, 0x12, 4, 0, 0x12, 0, 1]);
    let sum = p[..22].chunks_exact(2).fold(0u16, |s, w| {
        s.wrapping_add(u16::from_le_bytes([w[0], w[1]]))
    });
    p[22..24].copy_from_slice(&sum.to_le_bytes());
    let (text, csv) = outputs(&p);
    assert!(
        text.lines()
            .last()
            .unwrap()
            .ends_with("100:12:00:04.120000000"),
        "{text}"
    );
    assert_eq!(
        csv.lines().nth(1).unwrap(),
        "0,0x0,7,0x11,Time-F1,10000000,1000000000,100,12,0,4,120000000,,,"
    );
}
