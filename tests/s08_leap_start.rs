//! Partial COR-110/RDG-010 clause evidence (no whole-requirement marker).
//! 106-24R1 Fig11-13, printed11-25/PDF33: DOY, no year; second60 retained.
//! Owner2026-11-15 YEAR-PHASE policy / contract5.3: explicit leap-labelled
//! endpoints take part in phase selection independently of absolute admission.
use irig106_time::{format1::*, stream::*, timeline::*, version::*, years::*, *};
fn input(first: bool, qualified: bool, none_source: bool) -> ReferenceInput {
    // First: CSDW0x00000101 (LY=1,DATE=0,SRC=1), words6000/2359/0366:
    // day366 23:59:60, caller2016. Next: CSDW1, words0000/0000/0001:
    // day1 midnight at+1s RTC, therefore2017. No DMY or host date is supplied.
    let mut bytes = if first {
        [1, 1, 0, 0, 0, 0x60, 0x59, 0x23, 0x66, 3]
    } else {
        [1, 0, 0, 0, 0, 0, 0, 0, 1, 0]
    };
    if none_source {
        bytes[0] = 15;
    }
    let r = if qualified {
        ReferenceInput::qualified_format1(
            decode_qualified_time_f1(
                &bytes,
                10,
                EditionEvidence {
                    override_edition: Some(PacketEdition::V24R1),
                    ..Default::default()
                },
                ScalePolicy::default(),
            )
            .unwrap(),
        )
        .unwrap()
    } else {
        ReferenceInput::format1(
            &bytes,
            10,
            TimeF1Layout::SourceWithIts17,
            ScalePolicy::default(),
        )
        .unwrap()
    };
    r.with_year_context(YearContext {
        caller: Some(2016),
        recording: None,
    })
    .unwrap()
}
fn packet(id: u64, ticks: u64, reference: Option<ReferenceInput>) -> PacketObservation {
    PacketObservation {
        id,
        channel: 1,
        rtc: Rtc::from_raw(ticks),
        kind: if reference.is_some() {
            PacketKind::Time
        } else {
            PacketKind::Data
        },
        declared_timein: true,
        declared_external: false,
        flags: 0,
        reference,
    }
}
fn policy(mode: StreamMode) -> StreamPolicy {
    let mut p = StreamPolicy::new(
        1,
        BufferCaps {
            packets: 4,
            bytes: 400,
            references: 3,
        },
    );
    p.mode = mode;
    p.offline.channel = Some(1);
    p
}
fn offline(qualified: bool) {
    let first = input(true, qualified, false);
    assert_eq!(first.value, ReferenceValue::NeedsContext);
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    b.observe(packet(1, 0, Some(first)), false).unwrap();
    let key = b
        .observe(
            packet(2, 10_000_000, Some(input(false, qualified, false))),
            false,
        )
        .unwrap();
    let t = b.finish().unwrap();
    let refs = &t.sessions()[0].references;
    let answer = t.lookup(key).unwrap();
    let time = answer.time.unwrap();
    assert_eq!((time.year(), time.day_of_year()), (Some(2017), 1));
    assert_eq!(refs[1].input.year_basis.rollovers(), 1);
    assert_eq!(refs[0].input.year_basis.selected().unwrap().year, 2016);
    assert_eq!(refs[1].input.year_basis.selected().unwrap().year, 2017);
    assert!(
        !refs[0].admitted,
        "year facts do not admit a NeedsContext leap as an absolute anchor"
    );
    assert!(t.findings().iter().any(|f| f.code == 22 && f.packet == 1));
    assert!(
        !t.findings()
            .iter()
            .any(|f| matches!(f.code, 7 | 17 | 20 | 32))
    );
}
fn stream(qualified: bool, mode: StreamMode) {
    let mut s = StreamTimeline::new(policy(mode), None).unwrap();
    let first = s
        .observe(
            packet(1, 0, Some(input(true, qualified, false))),
            0,
            100,
            false,
        )
        .unwrap();
    assert!(first.findings.iter().any(|f| f.code == 22));
    // Still no usable absolute reference: phase evidence must not bypass admission.
    let before = s.observe(packet(2, 0, None), 0, 100, false).unwrap();
    assert!(before.answers[0].time.is_none());
    let next = s
        .observe(
            packet(3, 10_000_000, Some(input(false, qualified, false))),
            0,
            100,
            false,
        )
        .unwrap();
    let answer = s
        .observe(packet(4, 10_000_000, None), 0, 100, false)
        .unwrap()
        .answers
        .remove(0);
    let time = answer.time.unwrap();
    assert_eq!((time.year(), time.day_of_year()), (Some(2017), 1));
    let r = answer.basis.preceding.unwrap();
    assert_eq!(r.input.year_basis.rollovers(), 1);
    assert_eq!(r.key.packet, 3);
    assert!(
        !next
            .findings
            .iter()
            .any(|f| matches!(f.code, 7 | 17 | 20 | 32))
    );
}
#[test]
fn raw_offline() {
    offline(false);
}
#[test]
fn qualified_offline() {
    offline(true);
}
#[test]
fn raw_standard() {
    stream(false, StreamMode::Standard);
}
#[test]
fn raw_interpolate() {
    stream(false, StreamMode::Interpolate);
}
#[test]
fn qualified_standard() {
    stream(true, StreamMode::Standard);
}
#[test]
fn qualified_interpolate() {
    stream(true, StreamMode::Interpolate);
}
#[test]
fn unusable_leap_observations_do_not_supply_phase() {
    for qualified in [false, true] {
        for undeclared in [false, true] {
            let mut first = packet(1, 0, Some(input(true, qualified, !undeclared)));
            first.declared_timein = !undeclared;
            let next = packet(2, 10_000_000, Some(input(false, qualified, false)));
            let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
            b.observe(first.clone(), false).unwrap();
            let key = b.observe(next.clone(), false).unwrap();
            let t = b.finish().unwrap();
            assert_eq!(t.lookup(key).unwrap().time.unwrap().year(), Some(2016));
            assert_eq!(
                t.sessions()[0].references[1].input.year_basis.rollovers(),
                0
            );
            for mode in [StreamMode::Standard, StreamMode::Interpolate] {
                let mut s = StreamTimeline::new(policy(mode), None).unwrap();
                s.observe(first.clone(), 0, 100, false).unwrap();
                s.observe(next.clone(), 0, 100, false).unwrap();
                assert_eq!(
                    s.observe(packet(3, 10_000_000, None), 0, 100, false)
                        .unwrap()
                        .answers[0]
                        .time
                        .unwrap()
                        .year(),
                    Some(2016)
                );
            }
        }
    }
}
