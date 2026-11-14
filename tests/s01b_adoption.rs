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
