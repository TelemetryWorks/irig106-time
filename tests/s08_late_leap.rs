//! Partial S08 clauses: Fig11-13/14 (11-25/PDF33), RDG-010,
//! contract5.3 and owner2026-11-15 YEAR-PHASE rule. Hand-derived words.
use irig106_time::{leap::LeapSecondTable, stream::*, timeline::*, years::*, *};
const S: u64 = 10_000_000;
fn wire(dmy: bool, midnight: bool, sec: u8, caller: u16, normalize: bool) -> ReferenceInput {
    // BCD seconds58/59/60 live in word0's high byte; HHMM2359.
    // DOY366=0366; DMY MMDD=1231/0101, YYYY=2016/2017.
    let second = (sec / 10) << 4 | sec % 10;
    let mut bytes = if midnight {
        vec![
            1,
            if dmy { 2 } else { 0 },
            0,
            0,
            0,
            0,
            0,
            0,
            1,
            if dmy { 1 } else { 0 },
        ]
    } else {
        vec![
            1,
            if dmy { 3 } else { 1 },
            0,
            0,
            0,
            second,
            0x59,
            0x23,
            if dmy { 0x31 } else { 0x66 },
            if dmy { 0x12 } else { 3 },
        ]
    };
    if dmy {
        bytes.extend(if midnight { [0x17, 0x20] } else { [0x16, 0x20] });
    }
    let mut r = ReferenceInput::format1(
        &bytes,
        bytes.len(),
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap()
    .with_year_context(YearContext {
        caller: Some(caller),
        recording: None,
    })
    .unwrap();
    if normalize {
        if let ReferenceValue::Readable(t, valid) = r.value {
            // The explicit caller date needed by normalization; never normalize
            // a second60 label into AbsoluteTime or silently ignore an error.
            r.value = ReferenceValue::Readable(
                t.with_year(Some(if midnight { 2017 } else { 2016 }))
                    .unwrap(),
                valid,
            );
            if !dmy {
                r.year_origin = YearOrigin::Caller;
            }
            r = r.normalize_to_tai(LeapSecondTable::builtin()).unwrap();
        }
    }
    r
}
fn packet(id: u64, ticks: u64, r: Option<ReferenceInput>) -> PacketObservation {
    PacketObservation {
        id,
        channel: 1,
        rtc: Rtc::from_raw(ticks),
        kind: if r.is_some() {
            PacketKind::Time
        } else {
            PacketKind::Data
        },
        declared_timein: true,
        declared_external: false,
        flags: 0,
        reference: r,
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
fn late(leap: bool, mode: Option<StreamMode>) {
    let first = packet(1, S, Some(wire(false, true, 0, 2017, false)));
    let last = packet(
        2,
        0,
        Some(wire(false, false, if leap { 60 } else { 59 }, 2017, false)),
    );
    if let Some(mode) = mode {
        let mut s = StreamTimeline::new(policy(mode), None).unwrap();
        s.observe(first, 0, 100, false).unwrap();
        let batch = s.observe(last, 0, 100, false).unwrap();
        assert!(
            !batch
                .findings
                .iter()
                .any(|f| matches!(f.code, 4 | 7 | 17 | 32))
        );
        if !leap {
            let answer = s
                .observe(packet(3, 0, None), 0, 100, false)
                .unwrap()
                .answers
                .remove(0);
            assert_eq!(answer.time.unwrap().year(), Some(2016));
            assert_eq!(answer.time.unwrap().day_of_year(), 366);
        }
    } else {
        let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
        b.observe(first, false).unwrap();
        let key = b.observe(last, false).unwrap();
        let t = b.finish().unwrap();
        let r = &t.sessions()[0].references[1];
        assert_eq!(r.input.year_basis.selected().unwrap().year, 2016);
        assert_eq!(r.input.year_basis.rollovers(), -1);
        assert!(
            !t.findings()
                .iter()
                .any(|f| matches!(f.code, 4 | 7 | 17 | 32))
        );
        if leap {
            assert_eq!(r.input.value, ReferenceValue::NeedsContext);
        } else {
            let answer = t.lookup(key).unwrap().time.unwrap();
            assert_eq!((answer.year(), answer.day_of_year()), (Some(2016), 366));
            assert!(r.admitted);
        }
    }
}
fn span(dmy: bool, normalize: bool, mode: Option<StreamMode>) {
    let packets = [
        packet(1, 0, Some(wire(dmy, false, 59, 2016, normalize))),
        packet(2, S, Some(wire(dmy, false, 60, 2016, normalize))),
        packet(3, 2 * S, Some(wire(dmy, true, 0, 2016, normalize))),
    ];
    if let Some(mode) = mode {
        let mut s = StreamTimeline::new(policy(mode), None).unwrap();
        let mut findings = Vec::new();
        for p in packets {
            findings.extend(s.observe(p, 0, 100, false).unwrap().findings);
        }
        let jumps: Vec<_> = findings
            .iter()
            .filter(|f| f.code == 17)
            .map(|f| (f.packet, f.delta_ns))
            .collect();
        assert!(jumps.is_empty(), "{jumps:?}");
        assert_eq!(
            s.observe(packet(4, 2 * S, None), 0, 100, false)
                .unwrap()
                .answers[0]
                .time
                .unwrap()
                .year(),
            Some(2017)
        );
    } else {
        let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
        for p in packets {
            b.observe(p, false).unwrap();
        }
        let t = b.finish().unwrap();
        let jumps: Vec<_> = t
            .findings()
            .iter()
            .filter(|f| f.code == 17)
            .map(|f| (f.packet, f.delta_ns))
            .collect();
        assert!(jumps.is_empty(), "{jumps:?}");
        assert!(
            t.quality(t.sessions()[0].id, 1)
                .unwrap()
                .drift_ppm
                .is_some_and(|p| p.abs() < 0.0001)
        );
        assert!(
            matches!(t.sessions()[0].references[2].input.value,ReferenceValue::Readable(t,_) if t.year()==Some(2017))
        );
    }
}
macro_rules! scenarios {
    ($name:ident,$body:ident,$a:expr,$b:expr) => {
        mod $name {
            use super::*;
            #[test]
            fn offline() {
                $body($a, $b, None);
            }
            #[test]
            fn standard() {
                $body($a, $b, Some(StreamMode::Standard));
            }
            #[test]
            fn interpolate() {
                $body($a, $b, Some(StreamMode::Interpolate));
            }
        }
    };
}
// Late helper has one parameter fewer; its three paths are explicit.
#[test]
fn q14_offline() {
    late(false, None);
}
#[test]
fn q14_standard() {
    late(false, Some(StreamMode::Standard));
}
#[test]
fn q14_interpolate() {
    late(false, Some(StreamMode::Interpolate));
}
#[test]
fn q13_offline() {
    late(true, None);
}
#[test]
fn q13_standard() {
    late(true, Some(StreamMode::Standard));
}
#[test]
fn q13_interpolate() {
    late(true, Some(StreamMode::Interpolate));
}
scenarios!(q5_doy, span, false, false);
scenarios!(q5n_doy_normalized, span, false, true);
scenarios!(q15n_dmy_normalized, span, true, true);
