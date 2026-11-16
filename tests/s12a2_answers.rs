//! Partial ANS101/105 and POL101/102 replay clauses, ADR0010/0011.
//! No whole markers: serialized/schema verification remains S12b.
use irig106_time::{answer::*, policy::*, scale::*, stream::*, timeline::*, *};
fn packet(id: u64, ticks: u64, reference: Option<ReferenceInput>) -> PacketObservation {
    PacketObservation {
        id,
        channel: 7,
        rtc: Rtc::from_raw(ticks),
        kind: if reference.is_some() {
            PacketKind::Time
        } else {
            PacketKind::Data
        },
        declared_timein: true,
        declared_external: true,
        flags: 0,
        reference,
    }
}
fn reference() -> ReferenceInput {
    // Fig11-14 (11-25/PDF33): DMY2017-01-01, DATE1, SRC1/FMT4.
    ReferenceInput::format1(
        &[0x41, 2, 0, 0, 0, 0, 0, 0, 1, 1, 0x17, 0x20],
        12,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap()
}
fn offline() -> OfflineAnswer {
    let p = SharedPolicy::new(7, SharedSettings::default()).unwrap();
    let mut b = TimelineBuilder::new(p.offline(OfflinePolicy::default()).unwrap()).unwrap();
    b.observe(packet(1, 0, Some(reference())), false).unwrap();
    let key = b.observe(packet(2, 10, None), false).unwrap();
    b.finish().unwrap().lookup(key).unwrap()
}
#[test]
fn offline_replay_owns_basis_and_typed_duration_without_timeline() {
    let a = offline();
    let v = AnswerView::offline(&a, &[]).unwrap();
    assert_eq!(a.counter_duration(), SignedNanos::new(1000));
    assert_eq!(v.value().counter_duration, Some(SignedNanos::new(1000)));
    assert_eq!(v.value().calendar.unwrap().nanoseconds, 1000);
    let AnswerBasis::Counter(c) = v.basis() else {
        panic!()
    };
    let ClockBasis::Offline(b) = c.clock() else {
        panic!()
    };
    assert_eq!(b.session_origin.ticks, 0);
    assert_eq!(b.policy.shared.as_ref().unwrap().id(), 7);
    drop(a);
    assert_eq!(v.basis().replay().unwrap().value(), v.value());
}
#[test]
fn public_native_result_tampering_is_rejected_not_copied_into_replay() {
    let mut a = offline();
    a.counter_ns += 1;
    assert!(AnswerView::offline(&a, &[]).is_err());
    let mut a = offline();
    a.reading.as_mut().unwrap().nanoseconds = 999;
    assert!(AnswerView::offline(&a, &[]).is_err());
    let mut a = offline();
    a.basis.reference.as_mut().unwrap().key.session = 2;
    assert!(AnswerView::offline(&a, &[]).is_err());
}
#[test]
fn negative_counter_only_duration_is_checked_signed_and_replayable() {
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    let k = b.observe(packet(1, 5, None), false).unwrap();
    let a = b
        .finish()
        .unwrap()
        .lookup(PacketKey {
            ticks: 4,
            packet: 2,
            ..k
        })
        .unwrap();
    let v = AnswerView::offline(&a, &[]).unwrap();
    assert_eq!(v.value().counter_duration, Some(SignedNanos::new(-100)));
    assert!(v.value().calendar.is_none());
    assert_eq!(v.basis().replay().unwrap().value(), v.value());
}
fn network(id: u64, ticks: u64, seconds: u32, nanos: u32) -> ReferenceInput {
    use irig106_time::{edition::PacketEdition, format2::*, leap::*, ntp::*};
    let mut bytes = [0; 12];
    bytes[0] = 0x11;
    bytes[4..8].copy_from_slice(&seconds.to_le_bytes());
    bytes[8..].copy_from_slice(&nanos.to_le_bytes());
    let o = Format2Input {
        payload: &bytes,
        data_length: 12,
        capture: ReferenceCapture {
            session: 1,
            packet: id,
            channel: 7,
            rtc: Rtc::from_raw(ticks),
            byte_offset: Some(id * 100),
        },
        declared_edition: Some(PacketEdition::V24R1),
        data_type_version: 8,
        declared_timein: true,
    }
    .decode()
    .unwrap();
    ReferenceInput::format2(o, NtpContext::new(1), LeapSecondTable::builtin(), false).unwrap()
}
#[test]
fn interpolated_replay_uses_both_normalized_endpoints_and_floor_after_engine_drop() {
    let mut p = StreamPolicy::new(
        1,
        BufferCaps {
            packets: 2,
            bytes: 100,
            references: 3,
        },
    );
    p.mode = StreamMode::Interpolate;
    p.offline.channel = Some(7);
    let mut s = StreamTimeline::new(p, None).unwrap();
    // Ch11 Fig11-17: native TAI. Absolute span1.000010s / RTC span1s;
    // midpoint displacement0.500005s, correction5000ns. S04 offset37s.
    s.observe(
        packet(1, 0, Some(network(1, 0, 1_483_228_837, 0))),
        0,
        12,
        false,
    )
    .unwrap();
    s.observe(packet(2, 5_000_000, None), 1, 12, false).unwrap();
    let a = s
        .observe(
            packet(
                3,
                10_000_000,
                Some(network(3, 10_000_000, 1_483_228_838, 10000)),
            ),
            2,
            12,
            false,
        )
        .unwrap()
        .answers
        .remove(0);
    assert_eq!(a.counter_duration(), SignedNanos::new(500_000_000));
    let v = AnswerView::stream(&a, &[]).unwrap();
    assert_eq!(v.value().calendar.unwrap().nanoseconds, 500_005_000);
    let AnswerBasis::Counter(c) = v.basis() else {
        panic!()
    };
    let ClockBasis::Stream(b) = c.clock() else {
        panic!()
    };
    assert_eq!(b.drift_correction_ns, Some(5000));
    assert_eq!(b.release, ReleaseReason::Interpolated);
    assert!(b.following.as_ref().unwrap().input.network.is_some());
    drop((a, s));
    assert_eq!(v.basis().replay().unwrap().value(), v.value());
}
