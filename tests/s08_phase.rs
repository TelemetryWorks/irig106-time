//! Partial L3-COR-110 clause checks, not whole-requirement coverage.
//! Fig11-13/14 (11-25/PDF33): decimal words0 seconds,1 HHMM,2 day;
//! owner2026-11-15 YEAR-PHASE reader policy; contract5.3.
use irig106_time::{stream::*, timeline::*, years::*, *};
const S: u64 = 10_000_000;
const D: u64 = 86400 * S;
fn bcd(v: u16) -> u16 {
    (v / 1000) << 12 | (v / 100 % 10) << 8 | (v / 10 % 10) << 4 | v % 10
}
#[derive(Clone, Copy)]
struct Step {
    day: u16,
    h: u8,
    m: u8,
    s: u8,
    ticks: u64,
    year: Option<u16>,
    jump: Option<i128>,
}
fn at(day: u16, h: u8, m: u8, s: u8, ticks: u64, year: Option<u16>, jump: Option<i128>) -> Step {
    Step {
        day,
        h,
        m,
        s,
        ticks,
        year,
        jump,
    }
}
fn wire(step: Step, ctx: Option<u16>) -> ReferenceInput {
    // Hand-place Fig11-13 words; no encoder-under-test or host date.
    let mut bytes = vec![1, u8::from(step.year.is_some_and(|y| y % 4 == 0)), 0, 0];
    for w in [
        bcd(step.s.into()) << 8,
        bcd(step.h.into()) << 8 | bcd(step.m.into()),
        bcd(step.day),
    ] {
        bytes.extend(w.to_le_bytes());
    }
    ReferenceInput::format1(
        &bytes,
        10,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap()
    .with_year_context(YearContext {
        caller: ctx,
        recording: None,
    })
    .unwrap()
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
fn run(label: &str, ctx: Option<u16>, steps: &[Step], stream: bool) {
    let mut packets = Vec::new();
    let mut id = 1;
    let mut last = 0;
    for &step in steps {
        while step.ticks > last + 100 * D {
            last += 100 * D;
            packets.push(packet(id, last, None));
            id += 1;
        }
        packets.push(packet(id, step.ticks, Some(wire(step, ctx))));
        id += 2;
        last = step.ticks;
    }
    if !stream {
        let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
        for p in &packets {
            b.observe(p.clone(), false).unwrap();
        }
        let t = b.finish().unwrap();
        for (r, step) in t.sessions()[0].references.iter().zip(steps) {
            let ReferenceValue::Readable(value, _) = r.input.value else {
                panic!("{label}: unreadable")
            };
            assert_eq!(
                (value.year(), value.day_of_year()),
                (step.year, step.day),
                "{label} offline"
            );
            let jumps: Vec<_> = t
                .findings()
                .iter()
                .filter(|f| f.code == 17 && f.packet == r.key.packet)
                .map(|f| f.delta_ns)
                .collect();
            assert_eq!(
                jumps,
                step.jump.map(|j| vec![Some(j)]).unwrap_or_default(),
                "{label} offline jumps"
            );
        }
        return;
    }
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        let mut s = StreamTimeline::new(policy(mode), None).unwrap();
        let mut index = 0;
        for p in &packets {
            let batch = s.observe(p.clone(), 0, 100, false).unwrap();
            if p.reference.is_none() {
                continue;
            }
            let step = steps[index];
            index += 1;
            let jumps: Vec<_> = batch
                .findings
                .iter()
                .filter(|f| f.code == 17)
                .map(|f| f.delta_ns)
                .collect();
            assert_eq!(
                jumps,
                step.jump.map(|j| vec![Some(j)]).unwrap_or_default(),
                "{label} {mode:?} jumps"
            );
            let batch = s
                .observe(packet(p.id + 1, step.ticks, None), 0, 100, false)
                .unwrap();
            let value = batch.answers[0].time.unwrap();
            assert_eq!(
                (value.year(), value.day_of_year()),
                (step.year, step.day),
                "{label} {mode:?}"
            );
        }
    }
}
macro_rules! case {
    ($name:ident,$ctx:expr,[$($s:expr),+ $(,)?]) => {
        mod $name {
            use super::*;
            #[test] fn offline() {run(stringify!($name),$ctx,&[$($s),+],false);}
            #[test] fn stream() {run(stringify!($name),$ctx,&[$($s),+],true);}
        }
    };
}
case!(
    a_sparse,
    Some(2023),
    [
        at(365, 23, 59, 59, 0, Some(2023), None),
        at(2, 0, 0, 0, 86401 * S, Some(2024), None)
    ]
);
case!(
    b1_leap,
    Some(2024),
    [
        at(366, 23, 59, 59, 0, Some(2024), None),
        at(1, 0, 0, 0, S, Some(2025), None)
    ]
);
case!(
    b3_leap_sparse,
    Some(2024),
    [
        at(366, 23, 59, 59, 0, Some(2024), None),
        at(3, 0, 0, 0, 172801 * S, Some(2025), None)
    ]
);
case!(
    c_backward,
    Some(2023),
    [
        at(200, 0, 0, 5, 0, Some(2023), None),
        at(199, 23, 59, 59, S, Some(2023), Some(-7_000_000_000))
    ]
);
case!(
    d_same_day,
    Some(2023),
    [
        at(200, 0, 0, 5, 0, Some(2023), None),
        at(200, 0, 0, 6, S, Some(2023), None)
    ]
);
case!(
    e_equal_counter,
    Some(2023),
    [
        at(200, 0, 0, 5, 0, Some(2023), None),
        at(200, 0, 0, 6, 0, Some(2023), Some(1_000_000_000))
    ]
);
case!(
    f_plus_five,
    Some(2023),
    [
        at(365, 23, 59, 59, 0, Some(2023), None),
        at(1, 0, 0, 5, S, Some(2024), Some(5_000_000_000)),
        at(1, 0, 0, 6, 2 * S, Some(2024), None)
    ]
);
case!(
    f_unknown,
    None,
    [
        at(365, 23, 59, 59, 0, None, None),
        at(1, 0, 0, 5, S, None, Some(5_000_000_000)),
        at(1, 0, 0, 6, 2 * S, None, None)
    ]
);
case!(
    g_minus_five,
    Some(2023),
    [
        at(365, 23, 59, 59, 0, Some(2023), None),
        at(365, 23, 59, 55, S, Some(2023), Some(-5_000_000_000)),
        at(1, 0, 0, 0, 6 * S, Some(2024), None)
    ]
);
case!(
    m_backward_across_new_year,
    Some(2024),
    [
        at(1, 0, 0, 2, 0, Some(2024), None),
        at(365, 23, 59, 59, S, Some(2023), Some(-4_000_000_000)),
        at(1, 0, 0, 0, 2 * S, Some(2024), None)
    ]
);
case!(
    m3_corrections,
    Some(2023),
    [
        at(365, 23, 59, 58, 0, Some(2023), None),
        at(1, 0, 0, 2, S, Some(2024), Some(3_000_000_000)),
        at(365, 23, 59, 59, 2 * S, Some(2023), Some(-4_000_000_000)),
        at(1, 0, 0, 0, 3 * S, Some(2024), None)
    ]
);
case!(
    n_forward_same_day_order,
    Some(2023),
    [
        at(10, 0, 0, 0, 0, Some(2023), None),
        at(
            20,
            0,
            0,
            0,
            360 * D,
            Some(2024),
            Some(15 * 86_400_000_000_000)
        )
    ]
);
case!(
    p_previous_year,
    Some(2023),
    [
        at(10, 0, 0, 0, 0, Some(2023), None),
        at(
            300,
            0,
            0,
            0,
            S,
            Some(2022),
            Some(-75 * 86_400_000_000_000 - 1_000_000_000)
        )
    ]
);
case!(
    tie_keeps_phase,
    Some(2023),
    [
        at(10, 0, 0, 0, 0, Some(2023), None),
        at(
            10,
            0,
            0,
            0,
            365 * D / 2,
            Some(2023),
            Some(-365 * 86_400_000_000_000 / 2)
        )
    ]
);
case!(
    k_inclusive_half_year_bound,
    Some(2023),
    [
        at(365, 23, 59, 59, 0, Some(2023), None),
        at(
            1,
            0,
            0,
            0,
            365 * D / 2 + S,
            Some(2024),
            Some(-365 * 86_400_000_000_000 / 2)
        )
    ]
);
case!(
    unknown_leap_year_uses_366,
    None,
    [
        at(366, 23, 59, 59, 0, None, None),
        at(1, 0, 0, 0, S, None, None)
    ]
);
#[test]
fn l_outside_bound_keeps_gap_and_context_evidence() {
    let ctx = Some(2023);
    let first = at(365, 23, 59, 59, 0, Some(2023), None);
    let next = at(1, 0, 0, 0, 365 * D / 2 + S + 1, None, None);
    let packets = [
        packet(1, 0, Some(wire(first, ctx))),
        packet(2, 100 * D, None),
        packet(3, next.ticks, Some(wire(next, ctx))),
    ];
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    for p in &packets {
        b.observe(p.clone(), false).unwrap();
    }
    let t = b.finish().unwrap();
    let r = &t.sessions()[0].references[1];
    assert!(matches!(r.input.value,ReferenceValue::Readable(t,_) if t.year().is_none()));
    assert_eq!(
        r.input.year_basis.gap_ticks().unwrap().get(),
        i128::from(next.ticks)
    );
    assert_eq!(r.input.year_basis.context().caller, ctx);
    assert!(r.input.year_basis.year_issue().is_some());
    let f = t
        .findings()
        .iter()
        .find(|f| f.code == 20 && f.packet == 3)
        .unwrap();
    assert!(
        f.evidence
            .raw_fields()
            .iter()
            .any(|f| f.name == "year.gap_ticks")
    );
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        let mut s = StreamTimeline::new(policy(mode), None).unwrap();
        for p in &packets[..2] {
            s.observe(p.clone(), 0, 100, false).unwrap();
        }
        assert!(matches!(
            s.observe(packets[2].clone(), 0, 100, false),
            Err(TimeError::NeedsContext { .. })
        ));
        s.observe(packets[2].clone(), 0, 100, true).unwrap();
    }
}
fn dmy(year: u16, dec: bool, seconds: u8) -> ReferenceInput {
    // Fig11-14: word2=MMDD, word3=YYYY, HHMM=2359 for Dec31.
    let words = [
        bcd(seconds.into()) << 8,
        if dec { 0x2359 } else { 0 },
        if dec { 0x1231 } else { 0x0101 },
        bcd(year),
    ];
    let mut bytes = vec![1, 2 | u8::from(year % 4 == 0), 0, 0];
    for w in words {
        bytes.extend(w.to_le_bytes());
    }
    ReferenceInput::format1(
        &bytes,
        12,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap()
}
#[test]
fn stable_first_dmy_advances_and_only_contrary_year_is_a_conflict() {
    for variant in 0..3 {
        let next = match variant {
            0 => wire(at(1, 0, 0, 0, S, Some(2024), None), None),
            1 => dmy(2024, false, 0),
            _ => dmy(2023, false, 0),
        };
        let packets = [
            packet(1, 0, Some(dmy(2023, true, 59))),
            packet(2, S, Some(next)),
        ];
        let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
        for p in &packets {
            b.observe(p.clone(), false).unwrap();
        }
        let t = b.finish().unwrap();
        let r = &t.sessions()[0].references[1];
        assert_eq!(r.input.year_basis.rollovers(), 1);
        assert_eq!(t.findings().iter().any(|f| f.code == 32), variant == 2);
        if variant != 2 {
            assert!(matches!(r.input.value,ReferenceValue::Readable(t,_) if t.year()==Some(2024)));
        }
        for mode in [StreamMode::Standard, StreamMode::Interpolate] {
            let mut s = StreamTimeline::new(policy(mode), None).unwrap();
            s.observe(packets[0].clone(), 0, 100, false).unwrap();
            let batch = s.observe(packets[1].clone(), 0, 100, false).unwrap();
            assert_eq!(batch.findings.iter().any(|f| f.code == 32), variant == 2);
            if variant != 2 {
                assert_eq!(
                    s.observe(packet(3, S, None), 0, 100, false)
                        .unwrap()
                        .answers[0]
                        .time
                        .unwrap()
                        .year(),
                    Some(2024)
                );
            }
        }
    }
}
#[test]
fn late_crossing_records_phase_and_signed_correction() {
    for ctx in [Some(2024), None] {
        let jan = at(1, 0, 0, 5, S, ctx, None);
        let dec = at(365, 23, 59, 59, 0, ctx.map(|_| 2023), None);
        let packets = [
            packet(1, S, Some(wire(jan, ctx))),
            packet(2, 0, Some(wire(dec, ctx))),
        ];
        let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
        for p in &packets {
            b.observe(p.clone(), false).unwrap();
        }
        let t = b.finish().unwrap();
        assert_eq!(
            t.sessions()[0].references[1].input.year_basis.rollovers(),
            -1
        );
        for mode in [StreamMode::Standard, StreamMode::Interpolate] {
            let mut s = StreamTimeline::new(policy(mode), None).unwrap();
            s.observe(packets[0].clone(), 0, 100, false).unwrap();
            let batch = s.observe(packets[1].clone(), 0, 100, false).unwrap();
            assert!(
                batch
                    .findings
                    .iter()
                    .any(|f| f.code == 17 && f.delta_ns == Some(5_000_000_000))
            );
        }
    }
}
#[test]
fn crossing_threshold_is_strict_and_override_is_respected() {
    for threshold in [5_000_000_000, 6_000_000_000, 4_999_999_999] {
        let mut op = OfflinePolicy::default();
        op.jump_threshold_ns = Some(threshold);
        let packets = [
            packet(1, 0, Some(wire(at(365, 23, 59, 59, 0, None, None), None))),
            packet(2, S, Some(wire(at(1, 0, 0, 5, S, None, None), None))),
        ];
        let mut b = TimelineBuilder::new(op.clone()).unwrap();
        for p in &packets {
            b.observe(p.clone(), false).unwrap();
        }
        assert_eq!(
            b.finish().unwrap().findings().iter().any(|f| f.code == 17),
            threshold < 5_000_000_000
        );
        for mode in [StreamMode::Standard, StreamMode::Interpolate] {
            let mut p = policy(mode);
            p.offline = op.clone();
            let mut s = StreamTimeline::new(p, None).unwrap();
            s.observe(packets[0].clone(), 0, 100, false).unwrap();
            assert_eq!(
                s.observe(packets[1].clone(), 0, 100, false)
                    .unwrap()
                    .findings
                    .iter()
                    .any(|f| f.code == 17),
                threshold < 5_000_000_000
            );
        }
    }
}
#[test]
fn linear_span_keeps_an_explicit_leap_label() {
    // Fig11-14 literal2024Jun30 23:59:60; next day183/Jul1 midnight.
    let bytes = [1, 3, 0, 0, 0, 0x60, 0x59, 0x23, 0x30, 6, 0x24, 0x20];
    let leap = ReferenceInput::format1(
        &bytes,
        12,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap();
    let next = wire(at(183, 0, 0, 0, S, Some(2024), None), None);
    let packets = [packet(1, 0, Some(leap)), packet(2, S, Some(next))];
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    for p in &packets {
        b.observe(p.clone(), false).unwrap();
    }
    assert!(!b.finish().unwrap().findings().iter().any(|f| f.code == 17));
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        let mut s = StreamTimeline::new(policy(mode), None).unwrap();
        s.observe(packets[0].clone(), 0, 100, false).unwrap();
        assert!(
            !s.observe(packets[1].clone(), 0, 100, false)
                .unwrap()
                .findings
                .iter()
                .any(|f| f.code == 17)
        );
    }
}
#[test]
fn day366_does_not_repair_a_known_common_year() {
    let packets = [
        packet(
            1,
            0,
            Some(wire(at(365, 23, 59, 59, 0, Some(2023), None), Some(2023))),
        ),
        packet(
            2,
            S,
            Some(wire(at(366, 0, 0, 0, S, Some(2023), None), Some(2023))),
        ),
    ];
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    for p in &packets {
        b.observe(p.clone(), false).unwrap();
    }
    let t = b.finish().unwrap();
    assert!(!t.sessions()[0].references[1].admitted);
    assert!(t.findings().iter().any(|f| f.code == 4 && f.packet == 2));
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        let mut s = StreamTimeline::new(policy(mode), None).unwrap();
        s.observe(packets[0].clone(), 0, 100, false).unwrap();
        assert!(
            s.observe(packets[1].clone(), 0, 100, false)
                .unwrap()
                .findings
                .iter()
                .any(|f| f.code == 4)
        );
    }
}
