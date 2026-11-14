//! S04 probes executable on S03 main. IERS Leap_Second.dat / Bulletin C72:
//! 1972-07-01 UTC Unix78796800 takes offset11; TAI transition is78796811.
use irig106_time::{LeapSecondEntry, LeapSecondTable, TaiSeconds, TaiUtcOffset, UnixSeconds};
/// Requirements: L1-TAI-001, L2-TAI-101, L3-TAI-101, L1-PTP-004, L2-PTP-104, L3-PTP-104
#[test]
fn historical_tai_transition_does_not_use_the_modern_offset_guess() {
    assert_eq!(
        LeapSecondTable::builtin()
            .offset_at_tai(TaiSeconds::new(78_796_811))
            .unwrap()
            .get(),
        11
    );
}
/// Requirements: L1-TAI-003, L2-TAI-103, L3-TAI-103
#[test]
fn conflicting_add_does_not_install_or_grow_the_table() {
    let mut table = LeapSecondTable::builtin();
    let before = table.len();
    let _ = table.add(LeapSecondEntry {
        effective_unix: UnixSeconds::new(63_072_000),
        tai_utc_offset: TaiUtcOffset::new(12),
    });
    assert_eq!(table.len(), before);
}
