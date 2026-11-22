//! Partial CLI-106 correlate report parity; no whole-requirement marker.
use irig106_time_cli::{ExitClass, run};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
struct Input(PathBuf);
impl Input {
    fn new(packets: &[(u16, u64, u32)]) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "s17-query-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(
            &path,
            packets
                .iter()
                .flat_map(|&(ch, rtc, sec)| packet(ch, rtc, sec))
                .collect::<Vec<_>>(),
        )
        .unwrap();
        Self(path)
    }
    fn query(&self) -> (String, String) {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert_eq!(
            run(
                &[
                    "correlate",
                    self.0.to_str().unwrap(),
                    "2FAF080",
                    "--fail-on",
                    "never"
                ],
                &mut out,
                &mut err
            ),
            ExitClass::Success
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
fn packet(ch: u16, rtc: u64, sec: u32) -> Vec<u8> {
    // 106-24R1 §11.2.1.1, Figures11-12/13: FMT4/DATE0, DOY100,
    // specified seconds after 12:00, RTC100ns ticks; Data Length10 plus filler2.
    let bcd = |v: u32| ((v / 10) * 16 + v % 10) as u8;
    let mut p = vec![0; 36];
    p[..2].copy_from_slice(&[0x25, 0xeb]);
    p[2..4].copy_from_slice(&ch.to_le_bytes());
    p[4] = 36;
    p[8] = 10;
    p[15] = 0x11;
    p[16..22].copy_from_slice(&(rtc * 10_000_000).to_le_bytes()[..6]);
    p[24..34].copy_from_slice(&[
        0x40,
        0,
        0,
        0,
        0,
        bcd(sec % 60),
        bcd(sec / 60 % 60),
        bcd(12 + sec / 3600),
        0,
        1,
    ]);
    let sum = p[..22].chunks_exact(2).fold(0u16, |s, w| {
        s.wrapping_add(u16::from_le_bytes([w[0], w[1]]))
    });
    p[22..24].copy_from_slice(&sum.to_le_bytes());
    p
}
#[test]
fn in_order_query_keeps_any_and_each_channel_text_exact() {
    let (out, err) = Input::new(&[(7, 0, 0), (8, 0, 3600), (7, 40, 40), (8, 41, 3641)]).query();
    assert_eq!(
        out,
        "Resolving RTC 0x000002FAF080 (50000000 ticks, 5000000000 ns)\n\n  Any channel  → 100:12:00:05.000000000\n  Channel   7  → 100:12:00:05.000000000\n  Channel   8  → 100:13:00:05.000000000\n"
    );
    assert_eq!(err, "");
}
#[test]
fn reset_query_keeps_both_segment_answers_and_other_channel_exact() {
    // >30s backward step40->1 starts segment2; RTC5 lies in both7 ranges.
    let (out, err) = Input::new(&[
        (7, 0, 0),
        (8, 0, 3600),
        (7, 40, 40),
        (7, 1, 61),
        (7, 10, 70),
        (8, 41, 3641),
    ])
    .query();
    assert_eq!(
        out,
        "Resolving RTC 0x000002FAF080 (50000000 ticks, 5000000000 ns)\n\n  Channel   7 Segment   1  → 100:12:00:05.000000000\n  Channel   7 Segment   2  → 100:12:01:05.000000000\n  Channel   8  → 100:13:00:05.000000000\n"
    );
    assert!(err.contains("warning=2"), "{err}");
    assert!(err.contains("RTC is ambiguous across reset segments on channel 7"));
    assert_eq!(
        out,
        Input::new(&[
            (7, 0, 0),
            (8, 0, 3600),
            (7, 40, 40),
            (7, 1, 61),
            (7, 10, 70),
            (8, 41, 3641)
        ])
        .query()
        .0
    );
}

#[test]
fn correlate_report_writer_failure_still_overrides_findings() {
    struct Broken;
    impl std::io::Write for Broken {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("query writer probe"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let input = Input::new(&[(7, 0, 0), (7, 40, 40), (7, 1, 61), (7, 10, 70)]);
    assert_eq!(
        run(
            &[
                "correlate",
                input.0.to_str().unwrap(),
                "2FAF080",
                "--fail-on",
                "warning"
            ],
            &mut Broken,
            &mut Vec::new()
        ),
        ExitClass::Output
    );
}
