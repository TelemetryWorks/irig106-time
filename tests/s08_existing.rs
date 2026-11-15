//! S08 main-first probes; Fig11-13/14 printed11-25/PDF33; contract5.3.
use irig106_time::{findings::*, timeline::*, *};
fn packet(id: u64, ticks: u64, reference: ReferenceInput) -> PacketObservation {
    PacketObservation {
        id,
        channel: 1,
        rtc: Rtc::from_raw(ticks),
        kind: PacketKind::Time,
        declared_timein: true,
        declared_external: false,
        flags: 0,
        reference: Some(reference),
    }
}
fn wire(bytes: &[u8]) -> ReferenceInput {
    ReferenceInput::format1(
        bytes,
        bytes.len(),
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap()
}
#[test]
fn tf032_has_owner_accepted_warning_level() {
    let code = FindingCode::from_number(32).unwrap();
    assert_eq!(code.title(), "Year evidence is contradictory");
    assert_eq!(code.default_severity(), Some(Severity::Warning));
    assert_eq!(code.resolve_level(None).unwrap(), Severity::Warning);
}
// Partial COR-110 evidence, not a whole-requirement marker.
#[test]
fn offline_later_same_session_dmy_dates_the_earlier_doy_reference() {
    // DOY1 at RTC0; later DMY2024-01-01 at RTC1s, body seconds1.
    let doy = [1, 0, 0, 0, 0, 0, 0, 0, 1, 0];
    let dmy = [1, 3, 0, 0, 0, 1, 0, 0, 1, 1, 0x24, 0x20];
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    let k = b.observe(packet(1, 0, wire(&doy)), false).unwrap();
    b.observe(packet(2, 10_000_000, wire(&dmy)), false).unwrap();
    let t = b.finish().unwrap();
    assert_eq!(t.lookup(k).unwrap().time.unwrap().year(), Some(2024));
}
// Partial CSDW-104 evidence, not a whole-requirement marker.
#[test]
fn day366_with_ly_clear_is_reported_even_when_year_is_unknown() {
    // Fig11-13 word2=BCD366, LY(bit8)=0; ordinary time remains partial.
    let bytes = [1, 0, 0, 0, 0, 0, 0, 0, 0x66, 3];
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    b.observe(packet(1, 0, wire(&bytes)), false).unwrap();
    assert!(b.finish().unwrap().findings().iter().any(|f| f.code == 7));
}
