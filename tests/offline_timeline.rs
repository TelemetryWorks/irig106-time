//! E1 source fixtures. Numeric baselines precede the new timeline API.
use irig106_time::format1::decode_time_f1;
use irig106_time::scale::ScalePolicy;
use irig106_time::{AbsoluteTime, Rtc, TimeCorrelator, TimeF1Layout};

// RCC 123-20 §5.6 printed 5-44/PDF78 supplies the numeric example, not
// complete packet bytes. These literal wire bytes are hand-derived from
// 106-24R1 §11.2.1.1 and Figures 11-12/13; no crate encoder is an oracle.
const HANDBOOK: [u8; 36] = [
    0x25, 0xeb, 1, 0, 0x24, 0, 0, 0, 10, 0, 0, 0, 6, 0, 0, 0x11, 0x40, 0x42, 0x0f, 0, 0, 0, 0xa9,
    0x3e, 1, 0x30, 0, 0, 0, 0x25, 0x30, 0x12, 0, 1, 0, 0,
];
// Contract §7.2, byte for byte, including Data Length=10 and trailer filler.
const CONTRACT: [u8; 36] = [
    0x25, 0xeb, 1, 0, 0x24, 0, 0, 0, 10, 0, 0, 0, 6, 0, 0, 0x11, 0, 0xe1, 0xf5, 5, 0, 0, 0x4f,
    0xe3, 1, 0x30, 0, 0, 0x35, 0x27, 0x45, 0x13, 0x87, 1, 0, 0,
];
fn reading(bytes: &[u8; 36]) -> (Rtc, AbsoluteTime) {
    let sum = bytes[..22].chunks_exact(2).fold(0u16, |s, w| {
        s.wrapping_add(u16::from_le_bytes([w[0], w[1]]))
    });
    assert_eq!(sum, u16::from_le_bytes([bytes[22], bytes[23]]));
    let r = decode_time_f1(
        &bytes[24..],
        10,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap();
    (
        Rtc::from_le_bytes(bytes[16..22].try_into().unwrap()),
        r.time.unwrap().to_absolute().unwrap().value,
    )
}

/// Requirements: L1-COR-002, L2-COR-102, L3-COR-102
#[test]
fn handbook_example_literal_wire_baseline() {
    let (rtc, time) = reading(&HANDBOOK);
    assert_eq!(rtc.as_raw(), 1_000_000);
    let mut c = TimeCorrelator::new();
    c.add_reference(1, rtc, time);
    assert_eq!(
        c.correlate(Rtc::from_raw(1_150_000), Some(1)).unwrap(),
        AbsoluteTime::new(100, 12, 30, 25, 15_000_000).unwrap()
    );
}

/// Requirements: L1-COR-002, L2-COR-102, L3-COR-102, L1-ANS-001, L2-ANS-101, L3-ANS-101
#[test]
fn contract_example_literal_wire_baseline() {
    let (rtc, time) = reading(&CONTRACT);
    let mut c = TimeCorrelator::new();
    c.add_reference(1, rtc, time);
    assert_eq!(
        c.correlate(Rtc::from_raw(104_200_000), Some(1)).unwrap(),
        AbsoluteTime::new(187, 13, 45, 27, 770_000_000).unwrap()
    );
}

/// Requirements: L1-COR-009, L2-COR-109, L3-COR-109
#[test]
fn accepted_late_default_is_1100_ms() {
    // Ch10 §10.6.1 b/c: 100 ms generation + 1000 ms commit, ADR-0010.
    assert_eq!(TimeCorrelator::DEFAULT_OOO_WINDOW_NS, 1_100_000_000);
}

/// Requirements: L1-COR-013, L2-COR-113, L3-COR-113
#[test]
fn quality_drift_uses_absolute_over_rtc_sign() {
    // Independent arithmetic: RTC=1 s, absolute=1.0001 s -> +100 ppm.
    let mut c = TimeCorrelator::new();
    let t = AbsoluteTime::new(100, 0, 0, 0, 0).unwrap();
    c.add_reference(1, Rtc::from_raw(0), t);
    c.add_reference(1, Rtc::from_raw(10_000_000), t.add_nanos(1_000_100_000));
    let q = irig106_time::quality::compute_quality(c.references());
    assert!((q.drift_ppm_per_channel[0].1 - 100.0).abs() < 0.001);
}
