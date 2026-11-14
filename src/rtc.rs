//! Checked RTC quantities layered over the shared wire values.
//! §11.2.1.1 i: 48 bits, 10 MHz; no absolute epoch is inferred.
//!
//! Explicit successful shared-type conversion (no saturating arithmetic):
//! ```
//! use irig106_time::{NanosDuration, rtc::SignedRtcTicks};
//! let d: NanosDuration = SignedRtcTicks::new(150_000).to_nanos().unwrap().to_unsigned().unwrap();
//! assert_eq!(d.get(), 15_000_000);
//! let _: irig106_types::TaiSeconds = irig106_time::TaiSeconds::new(100);
//! let _: irig106_types::UnixSeconds = irig106_time::UnixSeconds::new(63);
//! ```
//! Ticks cannot be interchanged with unsigned nanoseconds:
//! ```compile_fail
//! use irig106_time::{NanosDuration, rtc::SignedRtcTicks};
//! let nanos: NanosDuration = SignedRtcTicks::new(1);
//! ```
//! Epochs cannot be interchanged implicitly:
//! ```compile_fail
//! use irig106_time::{TaiSeconds, UnixSeconds};
//! let utc: UnixSeconds = TaiSeconds::new(100);
//! ```
use crate::{NanosDuration, Result, TimeError};
/// Shared signed quantities; Option-returning inherent conversions never saturate.
/// **Traces:** L3-API-104 ← L2-API-104 ← L1-API-004
pub use irig106_types::{Rtc, SignedNanos, SignedRtcTicks};

/// Reject out-of-width input; [`Rtc::from_raw`] remains the explicit masking API.
/// **Traces:** L3-RTC-101 ← L2-RTC-101 ← L1-RTC-001
pub fn checked_rtc(value: u64) -> Result<Rtc> {
    if value > Rtc::MAX.as_raw() {
        return Err(TimeError::CounterWidth { value });
    }
    Ok(Rtc::from_raw(value))
}
/// Checked shared tick-to-nanosecond conversion with this crate's typed error.
/// **Traces:** L3-RTC-104 ← L2-RTC-104 ← L1-RTC-004
pub fn signed_ticks_to_nanos(ticks: SignedRtcTicks) -> Result<SignedNanos> {
    ticks.to_nanos().ok_or(TimeError::QuantityOverflow {
        quantity: "RTC ticks to nanoseconds",
    })
}
/// Checked shared unsigned magnitude; callers retain sign in the original value.
/// **Traces:** L3-RTC-104 ← L2-RTC-104 ← L1-RTC-004
pub fn signed_nanos_magnitude(nanos: SignedNanos) -> Result<NanosDuration> {
    nanos.magnitude().ok_or(TimeError::QuantityOverflow {
        quantity: "nanosecond magnitude to u64",
    })
}
/// Checked shared unsigned conversion, rejecting negative or out-of-range input.
/// **Traces:** L3-RTC-104 ← L2-RTC-104 ← L1-RTC-004
pub fn signed_nanos_to_unsigned(nanos: SignedNanos) -> Result<NanosDuration> {
    nanos.to_unsigned().ok_or(TimeError::QuantityOverflow {
        quantity: "signed to unsigned nanoseconds",
    })
}
/// Caller ordering context, never an inference of multiple wraps.
/// **Traces:** L3-RTC-103 ← L2-RTC-103 ← L1-RTC-003
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CounterOrder {
    /// Signed nearest modular displacement; the exact half range is ambiguous.
    Nearest,
    /// Caller guarantees forward ordering and strictly less than one full wrap.
    ForwardWithinOneWrap,
    /// Caller guarantees backward ordering and strictly less than one full wrap.
    BackwardWithinOneWrap,
}
/// A raw shared counter in a caller-named, nonzero session.
/// **Traces:** L3-RTC-103 ← L2-RTC-103 ← L1-RTC-003
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionRtc {
    /// Nonzero session identity; raw counters cannot establish it.
    pub session: usize,
    /// Shared raw 48-bit counter, not elapsed time.
    pub rtc: Rtc,
}
impl SessionRtc {
    /// Delta from self to later under the caller's explicit ordering guarantee.
    /// Equal raw counters yield zero, never an inferred complete wrap.
    pub fn delta_to(self, later: Self, order: CounterOrder) -> Result<SignedRtcTicks> {
        same_session(self.session, later.session)?;
        let forward = i128::from(self.rtc.elapsed_ticks(later.rtc));
        let wrap = 1i128 << 48;
        let ticks = match order {
            CounterOrder::Nearest if forward == wrap / 2 => {
                return Err(TimeError::AmbiguousCounterDelta);
            }
            CounterOrder::Nearest if forward > wrap / 2 => forward - wrap,
            CounterOrder::BackwardWithinOneWrap if forward != 0 => forward - wrap,
            _ => forward,
        };
        Ok(SignedRtcTicks::new(ticks))
    }
}
pub(crate) fn same_session(a: usize, b: usize) -> Result<()> {
    if a == 0 || a != b {
        return Err(TimeError::CrossSessionCounterDelta);
    }
    Ok(())
}
