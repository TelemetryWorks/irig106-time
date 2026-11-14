//! Checked RTC quantities layered over the shared wire values.
//! §11.2.1.1 i: 48 bits, 10 MHz; no absolute epoch is inferred.
//!
//! Explicit successful shared-type conversion (no saturating arithmetic):
//! ```
//! use irig106_time::{NanosDuration, TaiSeconds, TaiUtcOffset, UnixSeconds, rtc::SignedRtcTicks};
//! let d: NanosDuration = SignedRtcTicks::new(150_000).to_nanos().unwrap().to_unsigned().unwrap();
//! assert_eq!(d.get(), 15_000_000);
//! let tai: irig106_types::TaiSeconds = TaiSeconds::new(100);
//! let utc: UnixSeconds = tai.to_unix(TaiUtcOffset::new(37));
//! assert_eq!(utc.get(), 63);
//! ```
//! Epochs cannot be interchanged implicitly:
//! ```compile_fail
//! use irig106_time::{TaiSeconds, UnixSeconds};
//! let utc: UnixSeconds = TaiSeconds::new(100);
//! ```
use crate::{NanosDuration, Result, TimeError};
pub use irig106_types::Rtc;

/// Reject out-of-width input; [`Rtc::from_raw`] remains the explicit masking API.
/// **Traces:** L3-RTC-101 ← L2-RTC-101 ← L1-RTC-001
pub fn checked_rtc(value: u64) -> Result<Rtc> {
    if value > Rtc::MAX.as_raw() {
        return Err(TimeError::CounterWidth { value });
    }
    Ok(Rtc::from_raw(value))
}
/// Signed, unwrapped ticks, not an absolute timestamp or nanoseconds.
/// **Traces:** L3-RTC-104 ← L2-RTC-104 ← L1-RTC-004
/// **Traces:** L3-API-104 ← L2-API-104 ← L1-API-004
/// ```compile_fail
/// use irig106_time::{NanosDuration, rtc::SignedRtcTicks};
/// let nanos: NanosDuration = SignedRtcTicks::new(1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignedRtcTicks(i128);
impl SignedRtcTicks {
    /// Explicit unwrapped tick count; conversion, not construction, checks ns range.
    pub const fn new(ticks: i128) -> Self {
        Self(ticks)
    }
    /// Raw signed tick count.
    pub const fn get(self) -> i128 {
        self.0
    }
    /// Multiply by exactly 100 ns per tick, preserving sign and rejecting overflow.
    pub fn to_nanos(self) -> Result<SignedNanos> {
        self.0
            .checked_mul(100)
            .map(SignedNanos)
            .ok_or(TimeError::QuantityOverflow {
                quantity: "RTC ticks to nanoseconds",
            })
    }
}
/// Signed nanoseconds; the shared [`NanosDuration`] magnitude is unsigned.
/// **Traces:** L3-RTC-104 ← L2-RTC-104 ← L1-RTC-004
/// **Traces:** L3-API-104 ← L2-API-104 ← L1-API-004
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignedNanos(i128);
impl SignedNanos {
    /// Explicit signed nanoseconds, with no epoch assumption.
    pub const fn new(nanos: i128) -> Self {
        Self(nanos)
    }
    /// Raw signed nanoseconds.
    pub const fn get(self) -> i128 {
        self.0
    }
    /// Checked shared unsigned magnitude. Inspect [`Self::get`] to retain sign.
    pub fn magnitude(self) -> Result<NanosDuration> {
        u64::try_from(self.0.unsigned_abs())
            .map(NanosDuration::new)
            .map_err(|_| TimeError::QuantityOverflow {
                quantity: "nanosecond magnitude to u64",
            })
    }
    /// Convert to shared unsigned duration only if nonnegative and within u64.
    pub fn to_unsigned(self) -> Result<NanosDuration> {
        u64::try_from(self.0)
            .map(NanosDuration::new)
            .map_err(|_| TimeError::QuantityOverflow {
                quantity: "signed to unsigned nanoseconds",
            })
    }
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
        Ok(SignedRtcTicks(ticks))
    }
}
pub(crate) fn same_session(a: usize, b: usize) -> Result<()> {
    if a == 0 || a != b {
        return Err(TimeError::CrossSessionCounterDelta);
    }
    Ok(())
}
