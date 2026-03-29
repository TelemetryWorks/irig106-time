use super::*;
use crate::absolute::AbsoluteTime;
use crate::rtc::Rtc;

#[test]
fn event_type_started() {
    assert_eq!(RecordingEventType::from_index(0x01), RecordingEventType::Started);
}

#[test]
fn event_type_stopped() {
    assert_eq!(RecordingEventType::from_index(0x02), RecordingEventType::Stopped);
}

#[test]
fn event_type_overrun() {
    assert_eq!(RecordingEventType::from_index(0x03), RecordingEventType::Overrun);
}

#[test]
fn event_type_index_point() {
    assert_eq!(RecordingEventType::from_index(0x04), RecordingEventType::IndexPoint(0x04));
    assert_eq!(RecordingEventType::from_index(0x0F), RecordingEventType::IndexPoint(0x0F));
}

#[test]
fn event_type_reserved() {
    assert_eq!(RecordingEventType::from_index(0x10), RecordingEventType::Reserved(0x10));
    assert_eq!(RecordingEventType::from_index(0xFF), RecordingEventType::Reserved(0xFF));
}

#[test]
fn may_cause_time_gap() {
    assert!(!RecordingEventType::Started.may_cause_time_gap());
    assert!(RecordingEventType::Stopped.may_cause_time_gap());
    assert!(RecordingEventType::Overrun.may_cause_time_gap());
    assert!(!RecordingEventType::IndexPoint(4).may_cause_time_gap());
}

#[test]
fn recording_event_with_abs_time() {
    let abs = AbsoluteTime::new(100, 12, 0, 0, 0).unwrap();
    let event = RecordingEvent::new(0x01, 1, Rtc::from_raw(10_000_000), Some(abs));
    assert_eq!(event.event_type, RecordingEventType::Started);
    assert!(event.has_reference_time());
}

#[test]
fn recording_event_without_abs_time() {
    let event = RecordingEvent::new(0x04, 2, Rtc::from_raw(20_000_000), None);
    assert_eq!(event.event_type, RecordingEventType::IndexPoint(4));
    assert!(!event.has_reference_time());
}

#[test]
fn event_type_display() {
    assert_eq!(format!("{}", RecordingEventType::Started), "Recording Started");
    assert_eq!(format!("{}", RecordingEventType::Overrun), "Recording Overrun");
    assert_eq!(format!("{}", RecordingEventType::IndexPoint(5)), "Index Point 5");
    assert_eq!(format!("{}", RecordingEventType::Reserved(0x20)), "Reserved(0x20)");
}
