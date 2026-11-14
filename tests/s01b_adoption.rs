//! S01b adoption boundary, not a new calendar or leap-table implementation.
use irig106_time::{PtpTime, TaiUtcOffset};
/// Requirements: L1-API-004, L2-API-104, L3-API-104
#[test]
fn legacy_calendar_rejects_pre1970_instead_of_clamping() {
    // Ch11 §11.2.3.3: PTP-epoch TAI seconds minus a caller-supplied offset.
    // 0 - 10 = -10 Unix seconds, not 0 (1970-01-01). Calendar support is later.
    let raw = PtpTime {
        seconds: 0,
        nanoseconds: 0,
    };
    assert!(raw.to_absolute(TaiUtcOffset::new(10)).is_err());
}

/// Requirements: L1-API-004, L2-API-104, L3-API-104, L1-RTC-004, L2-RTC-104, L3-RTC-104
#[test]
fn signed_quantities_are_shared_and_checked_errors_remain_typed() {
    use irig106_time::{SignedUnixSeconds, TimeError, rtc::*};
    let ticks: irig106_types::SignedRtcTicks = SignedRtcTicks::new(-150_000);
    let nanos: irig106_types::SignedNanos = signed_ticks_to_nanos(ticks).unwrap();
    assert_eq!(nanos.get(), -15_000_000); // Ch11 §11.2.1.1 i: 100 ns/tick.
    assert_eq!(signed_nanos_magnitude(nanos).unwrap().get(), 15_000_000);
    assert!(matches!(
        signed_nanos_to_unsigned(nanos),
        Err(TimeError::QuantityOverflow { .. })
    ));
    assert!(matches!(
        signed_ticks_to_nanos(SignedRtcTicks::new(i128::MAX)),
        Err(TimeError::QuantityOverflow { .. })
    ));
    let unix: irig106_types::SignedUnixSeconds = SignedUnixSeconds::new(-10);
    assert!(unix.to_unsigned().is_none());
}

/// Requirements: L1-API-004, L2-API-104, L3-API-104
#[test]
fn ptp_retains_signed_unix_and_unsigned_failure_is_explicit() {
    use irig106_time::{
        LeapSecondTable, NanosDuration, NetworkTime, Rtc, SignedUnixSeconds,
        StreamingTimeCorrelator, TimeCorrelator, TimeError,
    };
    let raw = PtpTime {
        seconds: 0,
        nanoseconds: 0,
    };
    let signed: SignedUnixSeconds = raw.to_signed_utc_seconds(TaiUtcOffset::new(10));
    assert_eq!(signed.get(), -10);
    assert!(matches!(
        raw.to_utc_seconds(TaiUtcOffset::new(10)),
        Err(TimeError::QuantityOverflow { .. })
    ));
    let table = LeapSecondTable::builtin();
    let network = NetworkTime::Ptp(raw);
    let mut offline = TimeCorrelator::new();
    assert!(
        offline
            .add_reference_f2(1, Rtc::ZERO, &network, &table)
            .is_err()
    );
    assert_eq!(offline.len(), 0);
    let mut stream = StreamingTimeCorrelator::new(NanosDuration::new(1_000_000_000));
    assert!(
        stream
            .add_reference_f2(1, Rtc::ZERO, &network, &table)
            .is_err()
    );
    assert_eq!(stream.len(), 0);
    // A negative supplied offset adds seconds, without unsigned sign casts.
    assert_eq!(raw.to_signed_utc_seconds(TaiUtcOffset::new(-1)).get(), 1);
    assert_eq!(raw.to_utc_seconds(TaiUtcOffset::new(-1)).unwrap(), 1);
}

/// Requirements: L1-API-004, L2-API-104, L3-API-104
#[test]
fn upstream_unit_and_epoch_conversions_do_not_saturate() {
    use irig106_time::{NanosDuration, TaiSeconds};
    for (scale, ctor) in [
        (
            1_000_000_000,
            NanosDuration::from_secs as fn(u64) -> Option<NanosDuration>,
        ),
        (1_000_000, NanosDuration::from_millis),
        (1_000, NanosDuration::from_micros),
    ] {
        let bound = u64::MAX / scale;
        assert_eq!(ctor(bound).unwrap().get(), bound * scale);
        assert_eq!(ctor(bound + 1), None);
    }
    let time = TaiSeconds::new(u64::MAX);
    assert_eq!(time.to_unix(TaiUtcOffset::new(-1)), None);
    assert_eq!(
        time.to_signed_unix(TaiUtcOffset::new(-1)).get(),
        i128::from(u64::MAX) + 1
    );
}
