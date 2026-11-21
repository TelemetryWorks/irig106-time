//! Explicit caller layouts, never recording-edition inference.
use irig106_time::csdw::{TimeF1Csdw, TimeF1Layout, TimeF1Source};
use irig106_time::{TimeFormat, TimeSource};

/// Requirements: L1-CSDW-002, L2-CSDW-102, L3-CSDW-102
#[test]
fn t6_legacy_ext_is_not_src() {
    // 106-04 Figure10-15 pp10-30/31; 106-05 Figure10-16a p10-37.
    for raw in 0..16 {
        let word = TimeF1Csdw::from_raw(raw);
        assert_eq!(
            word.time_source_for_layout(TimeF1Layout::Legacy04),
            TimeF1Source::ExternalTime {
                present: raw & 1 != 0,
                reserved_bits: (raw & 14) as u8
            }
        );
        assert_eq!(
            word.time_source_for_layout(TimeF1Layout::Source05),
            TimeF1Source::Source(TimeSource::from_raw(raw as u8))
        );
        assert_eq!(
            word.validate_reserved(TimeF1Layout::Legacy04).is_err(),
            raw & 14 != 0
        );
    }
}

/// Requirements: L1-CSDW-003, L2-CSDW-103, L3-CSDW-103
#[test]
fn t7_none_only_in_modern_layout() {
    for fmt in 0..16 {
        let word = TimeF1Csdw::from_raw(fmt << 4);
        let legacy = word.time_format_for_layout(TimeF1Layout::Legacy04);
        if fmt >= 6 {
            assert_eq!(legacy, TimeFormat::Reserved(fmt as u8));
        }
        assert_eq!(word.payload_is_valid(TimeF1Layout::Legacy04), fmt <= 5);
        assert_eq!(word.payload_is_valid(TimeF1Layout::Source05), fmt <= 5);
    }
    assert_eq!(
        TimeF1Csdw::from_raw(0xf0).time_format_for_layout(TimeF1Layout::Source05),
        TimeFormat::None
    );
}

/// Requirements: L1-CSDW-006, L2-CSDW-106, L3-CSDW-106
/// Interpretations: RDG-007
#[test]
fn t7_its_requires_explicit_applicable_layout() {
    use irig106_time::csdw::IrigTimeSource;
    let meanings = [
        IrigTimeSource::Freewheeling,
        IrigTimeSource::TimeCommand,
        IrigTimeSource::Rmm,
        IrigTimeSource::ExternalIrig,
        IrigTimeSource::ExternalGps,
        IrigTimeSource::ExternalNtp,
        IrigTimeSource::ExternalPtp,
        IrigTimeSource::Embedded,
    ];
    for raw in 0..16 {
        for fmt in 0..16 {
            let word = TimeF1Csdw::from_raw(raw << 12 | fmt << 4);
            assert_eq!(
                word.irig_time_source_for_layout(TimeF1Layout::Legacy04),
                None
            );
            assert_eq!(
                word.irig_time_source_for_layout(TimeF1Layout::Source05),
                None
            );
            let expected = if fmt <= 2 {
                Some(if raw < 8 {
                    meanings[raw as usize]
                } else {
                    IrigTimeSource::Reserved(raw as u8)
                })
            } else {
                None
            };
            assert_eq!(
                word.irig_time_source_for_layout(TimeF1Layout::SourceWithIts17),
                expected
            );
        }
    }
}

/// Requirements: L1-CSDW-007, L2-CSDW-107, L3-CSDW-107
#[test]
fn t7_reserved_masks_are_layout_specific() {
    for (layout, mask) in [
        (TimeF1Layout::Legacy04, 0xffff_fc0eu32),
        (TimeF1Layout::Source05, 0xffff_fc00),
        (TimeF1Layout::SourceWithIts17, 0xffff_0c00),
    ] {
        for bit in 0..32 {
            assert_eq!(
                TimeF1Csdw::from_raw(1 << bit)
                    .validate_reserved(layout)
                    .is_err(),
                mask & (1 << bit) != 0
            );
        }
    }
}

/// Requirements: L1-EDN-002, L2-EDN-102, L3-EDN-102
#[test]
fn t9_separate_shared_code_lists() {
    use irig106_time::version::{Irig106Version, detect_data_type_version, detect_version};
    for code in 0..=255u8 {
        assert_eq!(
            detect_version(code as u32),
            Irig106Version::from_setup_record(code)
        );
        assert_eq!(
            detect_data_type_version(code),
            Irig106Version::from_data_type_version(code)
        );
    }
    assert_eq!(detect_version(1), Irig106Version::Unknown(1));
    assert_eq!(detect_data_type_version(1), Irig106Version::V04);
    assert_eq!(detect_version(10), Irig106Version::V13);
    assert_eq!(detect_data_type_version(10), Irig106Version::V22);
    // Code-list decoding cannot automatically select TimeF1Layout.
}
