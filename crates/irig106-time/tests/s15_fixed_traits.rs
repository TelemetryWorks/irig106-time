//! Fixed-reader trait clauses; owning recording values are deliberately separate.
//! Partial API-101/102/106, no whole marker.
use core::{fmt::Debug, hash::Hash};
use irig106_time::{ch4::*, format1::*, network_time::*, rtc::*, *};
fn fixed<T: Debug + Clone + Copy + PartialEq + Eq + Hash>() {}
fn owned<T: Debug + Clone>() {}
#[test]
fn fixed_wire_calendar_and_context_traits() {
    fixed::<Rtc>();
    fixed::<Ertc>();
    fixed::<Ch4BinaryTime>();
    fixed::<Ieee1588Time>();
    fixed::<SignedNanos>();
    fixed::<SignedRtcTicks>();
    fixed::<SignedUnixSeconds>();
    fixed::<TaiSeconds>();
    fixed::<TaiUtcOffset>();
    fixed::<UnixSeconds>();
    fixed::<AbsoluteTime>();
    fixed::<CalendarTime>();
    fixed::<CalendarReading>();
    fixed::<DayFormatTime>();
    fixed::<DmyFormatTime>();
    fixed::<Format1Reading>();
    fixed::<Format1Input<'static>>();
    fixed::<EncodedFormat1>();
    fixed::<TimeF1Csdw>();
    fixed::<TimeF1Layout>();
    fixed::<TimeF1Source>();
    fixed::<csdw::ItsQualification>();
    fixed::<TimeF2Csdw>();
    fixed::<NtpTime>();
    fixed::<PtpTime>();
    fixed::<Format2Reading>();
    fixed::<IntraPacketTime>();
    fixed::<SecondaryHeaderTime>();
    fixed::<secondary::SecondaryChecksum>();
    fixed::<CounterOrder>();
    fixed::<SessionRtc>();
    fixed::<Ch4Words>();
    fixed::<BcdContext>();
    fixed::<BcdDayInterpretation>();
    fixed::<Ch4Context>();
    fixed::<Ch4Reading>();
    fixed::<ScaleEvidence>();
    // Public errors/recording snapshots retain owned/reference-rich shapes.
    owned::<TimeError>();
    owned::<LeapSecondTable>();
    owned::<answer::AnswerView>();
}
#[cfg(feature = "jiff")]
#[test]
fn feature_gated_civil_wrapper_is_a_fixed_value() {
    fixed::<Scaled<jiff::civil::DateTime>>();
}

#[test]
fn fixed_readings_and_owning_recording_results_have_distinct_drop_boundaries() {
    assert!(!core::mem::needs_drop::<Format1Reading>());
    assert!(!core::mem::needs_drop::<Format2Reading>());
    assert!(!core::mem::needs_drop::<Ch4Reading>());
    assert!(core::mem::needs_drop::<LeapSecondTable>());
    assert!(core::mem::needs_drop::<answer::AnswerView>());
}
