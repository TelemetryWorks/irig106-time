//! S10a1: §11.2.1.1g (11-9/PDF17), Fig11-4/5/6 (11-10/11/PDF18/19),
//! §11.2.1.2, RDG008/RCC123-20 B-31. Literal wire/checksum arithmetic.
use irig106_time::{findings::*, scale::*, secondary::*, secondary_packet::*, timeline::*, *};
const IEEE: [u8; 12] = [0x15, 0xcd, 0x5b, 7, 4, 3, 2, 1, 0, 0, 0x4e, 1];
fn packet(flags: u8, bytes: &[u8]) -> SecondaryPacket<'_> {
    SecondaryPacket {
        key: PacketKey {
            session: 2,
            packet: 9,
            ticks: 0,
        },
        channel: 7,
        data_type: 0x19,
        rtc: Rtc::from_raw(0),
        flags,
        bytes,
        byte_offset: Some(24),
        applicability: TimeApplicability::FirstDataBit,
    }
}
fn ids(r: &SecondaryInspection) -> Vec<u8> {
    r.findings().iter().map(|f| f.code().number()).collect()
}
fn inspect(flags: u8, b: &[u8]) -> SecondaryInspection {
    inspect_secondary(packet(flags, b), &Default::default()).unwrap()
}
// Partial ABS103: conversion/epoch context follows in S10a2.
#[test]
fn ieee_wire_order_range_and_applicability_are_preserved() {
    let r = inspect(0x84, &IEEE);
    assert!(r.errors().is_empty());
    let Some(SecondaryHeaderTime::Ieee1588(t)) = r.time() else {
        panic!("IEEE required")
    };
    assert_eq!((t.seconds, t.nanoseconds), (0x01020304, 123456789));
    assert_eq!(r.raw_bytes()[..8], IEEE[..8]);
    assert_eq!(r.scale(), ScaleEvidence::unknown());
    assert_eq!(r.applicability(), &TimeApplicability::FirstDataBit);
    let mut p = packet(0x84, &IEEE);
    p.applicability = TimeApplicability::CitedException {
        data_type: 0x19,
        citation: "synthetic caller instrument contract fixture (not an RCC exception claim)"
            .into(),
    };
    let expected = p.applicability.clone();
    assert_eq!(
        inspect_secondary(p, &Default::default())
            .unwrap()
            .applicability(),
        &expected
    );
    let ns_max = [0xff, 0xc9, 0x9a, 0x3b, 0, 0, 0, 0, 0, 0, 0x9d, 2]; // sum0x29d
    assert!(inspect(0x84, &ns_max).time().is_some());
    let ns_bad = [0, 0xca, 0x9a, 0x3b, 0, 0, 0, 0, 0, 0, 0x9f, 1];
    let r = inspect(0x84, &ns_bad);
    assert_eq!(r.time(), None);
    assert!(ids(&r).contains(&4));
    assert!(
        r.findings()
            .iter()
            .any(|f| f.code().number() == 4 && f.evidence().validity() == ValueValidity::Unusable)
    );
}
// Partial SEC104/RDG208: synthetic trust rules, not local recordings/human settlement.
/// Interpretations: RDG-008
#[test]
fn both_checksum_readings_trust_and_aggregate_counts_are_independent() {
    let both = [1, 0, 2, 0, 3, 0, 4, 0, 0, 0, 10, 0];
    let mut word = IEEE;
    word[10..].copy_from_slice(&[0x76, 0xd8]);
    let mut neither = IEEE;
    neither[10..].copy_from_slice(&[0, 0]);
    let mut a = SecondaryAudit::new();
    for (b, byte, words, trusted, expected) in [
        (&IEEE, true, false, true, vec![]),
        (&word, false, true, false, vec![12, 27]),
        (&both, true, true, true, vec![]),
        (&neither, false, false, false, vec![12]),
    ] {
        let r = a.observe(packet(0x84, b), &Default::default()).unwrap();
        let c = r.checksum().unwrap();
        assert_eq!((c.byte_valid(), c.word_valid()), (byte, words));
        assert_eq!(r.time().is_some(), trusted);
        assert_eq!(ids(&r), expected);
        assert_eq!(r.raw_bytes(), b);
        assert_eq!(r.key().session, 2);
        assert_eq!(r.flags(), 0x84);
        if b == &IEEE {
            assert_eq!((c.stored, c.byte_sum, c.word_sum), (0x14e, 0x14e, 0xd876));
        }
        for f in r.findings() {
            assert_eq!(f.evidence().raw_rtc(), Some(Rtc::from_raw(0)));
            for (name, value) in [
                ("secondary.checksum.stored", c.stored),
                ("secondary.checksum.byte_sum", c.byte_sum),
                ("secondary.checksum.word_sum", c.word_sum),
            ] {
                assert!(
                    f.evidence()
                        .raw_fields()
                        .iter()
                        .any(|v| v.name == name && v.observed == RawValue::Unsigned(value.into()))
                );
            }
        }
    }
    a.observe(packet(0x84, &IEEE[..11]), &Default::default())
        .unwrap();
    let c = a.counts();
    assert_eq!(
        (c.byte_only, c.word_only, c.both, c.neither, c.short),
        (1, 1, 1, 1, 1)
    );
    for index in [10, 11] {
        let mut b = IEEE;
        b[index] ^= 1;
        let r = inspect(0x84, &b);
        assert!(r.time().is_none());
        assert!(ids(&r).contains(&12));
    }
}
/// Requirements: L1-SEC-001, L2-SEC-101, L3-SEC-101
#[test]
fn bounded_ch4_header_and_independent_reserved_fields_keep_raw_values() {
    // low0x8000=32768 centiseconds, usec9999=0x270f; byte sum0xb6.
    let b = [0, 0, 0x0f, 0x27, 0, 0x80, 0, 0, 0, 0, 0xb6, 0];
    let r = inspect(0x80, &b);
    let Some(SecondaryHeaderTime::Ch4(t)) = r.time() else {
        panic!("Ch4 expected")
    };
    assert_eq!(
        (t.high_order, t.low_order, t.microseconds),
        (0, 0x8000, 9999)
    );
    for len in 0..12 {
        let r = inspect(0x80, &b[..len]);
        assert!(r.time().is_none());
        assert!(r.checksum().is_none());
        assert!(ids(&r).contains(&4));
        assert!(matches!(
            r.errors()[0],
            TimeError::BufferTooShort { expected: 12, .. }
        ));
    }
    let mut longer = b.to_vec();
    longer.extend([0xff; 64]);
    let r = inspect(0x80, &longer);
    assert_eq!(r.raw_bytes(), &b);
    assert_eq!(r.checksum().unwrap().byte_sum, 0xb6);
    let both_reserved = [1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 3, 0];
    let r = inspect(0x80, &both_reserved);
    assert!(r.time().is_some());
    assert_eq!(ids(&r), vec![5, 5]);
    for offset in [24, 32] {
        assert!(
            r.findings()
                .iter()
                .any(|f| f.evidence().location().byte_offset == Some(offset))
        );
    }
    let invalid = [0, 0, 0x10, 0x27, 0, 0, 0, 0, 0, 0, 0x37, 0];
    let r = inspect(0x80, &invalid);
    assert_eq!(r.time(), None);
    assert!(ids(&r).contains(&4));
}
/// Requirements: L1-SEC-005, L2-SEC-105, L3-SEC-105
#[test]
fn every_reserved_bit_has_its_own_position_and_kept_value() {
    for offset in [0, 8] {
        for bit in 0..16 {
            let mut b = [0u8; 12];
            let mask = 1u16 << bit;
            b[offset..offset + 2].copy_from_slice(&mask.to_le_bytes());
            // One set bit: contribution is its byte weight, independent of word sum.
            b[10] = (1u16 << (bit % 8)) as u8;
            let r = inspect(0x80, &b);
            assert!(r.time().is_some());
            assert_eq!(ids(&r), vec![5]);
            let f = &r.findings()[0];
            assert_eq!(
                f.evidence().location().byte_offset,
                Some(24 + offset as u64)
            );
            assert_eq!(f.evidence().validity(), ValueValidity::Kept);
            assert!(
                f.evidence()
                    .raw_fields()
                    .iter()
                    .any(|v| v.observed == RawValue::Unsigned(mask.into())
                        && v.expected == Some(RawValue::Unsigned(0)))
            );
            assert_eq!(r.raw_bytes(), &b);
        }
    }
    // Header-reserved word exists for IEEE and ERTC; their first word is time.
    for flags in [0x84, 0x88] {
        let b = [1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 3, 0];
        let r = inspect(flags, &b);
        assert_eq!(ids(&r).iter().filter(|i| **i == 5).count(), 1);
    }
    let b = [1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0];
    let r = inspect(0x80, &b);
    assert!(r.time().is_none());
    assert_eq!(ids(&r).iter().filter(|i| **i == 5).count(), 2);
    assert!(ids(&r).contains(&12));
}
// Partial ABS104/SEC103/FND105: no absolute ERTC session correlation in S10a1.
#[test]
fn ertc_units_floor_modular_tick_tolerance_and_half_range_are_explicit() {
    for n in [0u64, 1, 99, 100, 199, u64::MAX] {
        let mut b = [0u8; 12];
        b[..8].copy_from_slice(&n.to_le_bytes());
        // Independent fixture calculation: exactly these eight byte values.
        let sum: u16 = n.to_le_bytes().iter().map(|v| u16::from(*v)).sum();
        b[10..].copy_from_slice(&sum.to_le_bytes());
        let mut p = packet(0x88, &b);
        p.rtc = Rtc::from_raw(n / 100);
        let r = inspect_secondary(p, &Default::default()).unwrap();
        let Some(SecondaryHeaderTime::Ertc(e)) = r.time() else {
            panic!("relative ERTC expected")
        };
        assert_eq!(e.to_nanos(), u128::from(n));
        let c = r.rtc_ertc().unwrap();
        assert_eq!(c.expected_rtc(), Rtc::from_raw(n / 100));
        assert_eq!(c.remainder_ns(), (n % 100) as u8);
        assert_eq!(c.delta_ticks().unwrap().get(), 0);
        assert!(!ids(&r).contains(&25));
        assert_eq!(r.scale(), ScaleEvidence::unknown());
    }
    let b = [199, 0, 0, 0, 0, 0, 0, 0, 0, 0, 199, 0];
    let mask = (1u64 << 48) - 1;
    for (rtc, delta) in [(mask, -2i128), (0, -1), (1, 0), (2, 1), (3, 2)] {
        let mut p = packet(0x88, &b);
        p.rtc = Rtc::from_raw(rtc);
        let r = inspect_secondary(p, &Default::default()).unwrap();
        assert_eq!(r.rtc_ertc().unwrap().delta_ticks().unwrap().get(), delta);
        assert_eq!(ids(&r).contains(&25), delta.abs() > 1);
        if delta.abs() > 1 {
            let f = r
                .findings()
                .iter()
                .find(|f| f.code().number() == 25)
                .unwrap();
            assert_eq!(f.evidence().raw_rtc(), Some(Rtc::from_raw(rtc)));
            assert!(
                f.evidence()
                    .raw_fields()
                    .iter()
                    .any(|v| v.name == "rtc_ertc.delta_ticks"
                        && v.observed == RawValue::Signed(delta))
            );
        }
    }
    for (n, rtc, delta) in [(mask * 100 + 99, 0, 1), ((1u64 << 48) * 100 + 99, mask, -1)] {
        let mut b = [0u8; 12];
        b[..8].copy_from_slice(&n.to_le_bytes());
        let sum: u16 = n.to_le_bytes().iter().map(|v| u16::from(*v)).sum();
        b[10..].copy_from_slice(&sum.to_le_bytes());
        let mut p = packet(0x88, &b);
        p.rtc = Rtc::from_raw(rtc);
        let r = inspect_secondary(p, &Default::default()).unwrap();
        assert_eq!(r.rtc_ertc().unwrap().delta_ticks().unwrap().get(), delta);
        assert!(!ids(&r).contains(&25));
    }
    let zero = [0; 12];
    let mut p = packet(0x88, &zero);
    p.rtc = Rtc::from_raw(1 << 47);
    let r = inspect_secondary(p, &Default::default()).unwrap();
    assert_eq!(
        r.rtc_ertc().unwrap().delta_ticks(),
        Err(TimeError::AmbiguousCounterDelta)
    );
    assert!(ids(&r).contains(&25));
    assert!(inspect(0x84, &IEEE).rtc_ertc().is_none());
}
/// Requirements: L1-SEC-006, L2-SEC-106, L3-SEC-106
#[test]
fn formats_are_recording_wide_but_each_header_uses_its_own_flags() {
    let zero = [0; 12];
    for left in 0..3 {
        for right in 0..3 {
            let mut a = SecondaryAudit::new();
            a.observe(packet(0x80 | (left << 2), &zero), &Default::default())
                .unwrap();
            let mut p = packet(0x80 | (right << 2), &zero);
            p.channel = 8;
            p.key.packet = 10;
            let r = a.observe(p, &Default::default()).unwrap();
            assert_eq!(ids(&r).contains(&13), left != right);
            assert_eq!(
                r.format(),
                Some(SecHdrTimeFormat::from_packet_flags(0x80 | (right << 2)))
            );
            match (right, r.time().unwrap()) {
                (0, SecondaryHeaderTime::Ch4(_))
                | (1, SecondaryHeaderTime::Ieee1588(_))
                | (2, SecondaryHeaderTime::Ertc(_)) => {}
                _ => panic!("wrong per-packet dispatch"),
            }
            let mut p = packet(0x80 | (right << 2), &zero);
            p.key.packet = 11;
            let r = a.observe(p, &Default::default()).unwrap();
            assert_eq!(ids(&r).contains(&13), left != right);
        }
    }
    let mut a = SecondaryAudit::new();
    let r = a.observe(packet(0, &zero), &Default::default()).unwrap();
    assert_eq!(r.format(), None);
    assert!(r.checksum().is_none());
    assert!(r.time().is_none());
    assert!(r.raw_bytes().is_empty());
    assert!(!ids(&a.observe(packet(0x84, &zero), &Default::default()).unwrap()).contains(&13));
    let r = a.observe(packet(0x8c, &zero), &Default::default()).unwrap();
    assert!(ids(&r).contains(&6));
    assert!(ids(&r).contains(&28));
    assert!(r.time().is_none());
}
fn offline_counter() -> OfflineAnswer {
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    let bytes = [1, 0, 0, 0, 0, 0, 0, 0, 1, 0];
    let reference = ReferenceInput::format1(
        &bytes,
        10,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap();
    b.observe(
        PacketObservation {
            id: 1,
            channel: 7,
            rtc: Rtc::from_raw(0),
            kind: PacketKind::Time,
            declared_timein: true,
            declared_external: false,
            flags: 0,
            reference: Some(reference),
        },
        false,
    )
    .unwrap();
    let k = b
        .observe(
            PacketObservation {
                id: 2,
                channel: 7,
                rtc: Rtc::from_raw(1),
                kind: PacketKind::Data,
                declared_timein: false,
                declared_external: false,
                flags: 0,
                reference: None,
            },
            false,
        )
        .unwrap();
    b.finish().unwrap().lookup(k).unwrap()
}
// Partial ERR104/ANS102: exact borrowed counter preservation; no own calendar conversion.
#[test]
fn failed_own_time_keeps_independent_counter_and_policy_cannot_repair_validity() {
    let counter = offline_counter();
    assert!(counter.time.is_some());
    let mut bad = IEEE;
    bad[10] ^= 1;
    let mut p = SecondaryPolicy::default();
    for id in [4, 5, 6, 12, 13, 25, 27, 28] {
        p.levels.insert(id, Severity::Information);
    }
    let r = with_counter(&counter, packet(0x84, &bad), &p).unwrap();
    assert!(core::ptr::eq(r.counter, &counter));
    assert_eq!(r.counter.time, counter.time);
    assert!(r.header.time().is_none());
    assert!(
        r.header
            .findings()
            .iter()
            .all(|f| f.severity() == Severity::Information
                && f.evidence().validity() == ValueValidity::Unusable)
    );
    let r = with_counter(&counter, packet(0x88, &[0; 11]), &p).unwrap();
    assert!(core::ptr::eq(r.counter, &counter));
    assert!(r.header.time().is_none());
    let r = inspect_secondary(packet(0x8c, &[0; 12]), &p).unwrap();
    assert!(
        r.findings()
            .iter()
            .all(|f| f.severity() == Severity::Information)
    );
    let mut invalid = packet(0x84, &IEEE);
    invalid.key.session = 0;
    assert!(inspect_secondary(invalid, &p).is_err());
}
