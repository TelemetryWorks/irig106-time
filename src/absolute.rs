//! Absolute time representations for IRIG 106 Chapter 10.
//!
//! This module defines two time types and three wire-format types:
//!
//! - [`AbsoluteTime`] — Day-of-year based time with optional year. The primary
//!   time type produced by BCD DOY parsing, NTP/PTP conversion, and the
//!   correlation engine.
//! - [`CalendarTime`] — Calendar-aware time with validated year, month, and
//!   day-of-month. Produced by BCD DMY parsing or by promoting an
//!   `AbsoluteTime` with a validated date via [`CalendarTime::new`].
//! - [`Ch4BinaryTime`] — Chapter 4 Binary Weighted Time (secondary headers)
//! - [`Ieee1588Time`] — IEEE-1588 Precision Time Protocol value
//! - [`Ertc`] — 64-bit Extended Relative Time Counter
//!
//! # Type Safety
//!
//! `AbsoluteTime` cannot hold partial calendar state — it has no month or
//! day-of-month fields. `CalendarTime` requires all three date components
//! (year, month, day) at construction and validates them. This is enforced
//! at the type level:
//!
//! ```
//! use irig106_time::{AbsoluteTime, CalendarTime};
//!
//! // DOY time — no calendar date, just day-of-year + time
//! let mut t = AbsoluteTime::new(100, 12, 30, 25, 340_000_000).unwrap();
//! t.set_year(Some(2025)).unwrap();
//! assert_eq!(t.day_of_year(), 100);
//! assert_eq!(t.year(), Some(2025));
//! // t.month() does not exist — compile error if you try
//!
//! // Calendar time — full date required at construction
//! let ct = CalendarTime::from_parts(2025, 4, 10, 100, 12, 30, 25, 340_000_000).unwrap();
//! assert_eq!(ct.month(), 4);
//! assert_eq!(ct.day_of_month(), 10);
//! assert_eq!(ct.hours(), 12); // Deref to AbsoluteTime
//! ```
//!
//! # Requirement Traceability
//!
//! | Requirement | Description |
//! |-------------|-------------|
//! | L3-ABS-001..005 | `AbsoluteTime` struct and operations |
//! | L3-ABS-006      | `CalendarTime` struct and construction |
//! | P4-04           | Internal u64 representation |
//! | L3-CH4-001..005 | `Ch4BinaryTime` |
//! | L3-1588-001..004 | `Ieee1588Time` |
//! | L3-ERTC-001..003 | `Ertc` |

use crate::error::{Result, TimeError};

const NANOS_PER_SECOND: u64 = 1_000_000_000;
const NANOS_PER_MINUTE: u64 = 60 * NANOS_PER_SECOND;
const NANOS_PER_HOUR: u64 = 3600 * NANOS_PER_SECOND;
const NANOS_PER_DAY: u64 = 86_400 * NANOS_PER_SECOND;

// ═══════════════════════════════════════════════════════════════════════
// AbsoluteTime — DOY-based time with optional year
// ═══════════════════════════════════════════════════════════════════════

/// Nanosecond-precision absolute time based on day-of-year.
///
/// This is the primary time type in the crate. It is produced by:
/// - BCD Day-of-Year format parsing (`DayFormatTime::to_absolute`)
/// - NTP/PTP conversion (`NtpTime::to_absolute`, `PtpTime::to_absolute`)
/// - The correlation engine (`TimeCorrelator::correlate`)
///
/// The `year` field is optional because some IRIG 106 time sources (BCD DOY,
/// IRIG-B) do not carry year information. When present, it is set by the
/// time source that produced the value (NTP, PTP, DMY BCD).
///
/// **This type cannot hold month or day-of-month.** If you need calendar
/// date fields, use [`CalendarTime`], which wraps `AbsoluteTime` with
/// validated year/month/day.
///
/// # Performance
///
/// `add_nanos` and `sub_nanos` are single `u64` arithmetic operations when
/// staying within the same year (the common case for correlation).
///
/// **Traces:** L3-ABS-001 ← L2-ABS-001, L2-ABS-002 ← L1-ABS-001, P4-04
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AbsoluteTime {
    /// Nanoseconds since start of day 1 (day 1 00:00:00.000 = 0).
    total_ns: u64,
    /// Optional year (0–9999). Present when the time source provides it
    /// (NTP, PTP, DMY BCD). Absent for DOY BCD and IRIG-B.
    year: Option<u16>,
}

// ── Custom serde: serialize/deserialize as expanded fields ──────────

#[cfg(feature = "serde")]
mod serde_impl {
    use super::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    struct AbsoluteTimeFields {
        day_of_year: u16,
        hours: u8,
        minutes: u8,
        seconds: u8,
        nanoseconds: u32,
        year: Option<u16>,
    }

    impl Serialize for AbsoluteTime {
        fn serialize<S: Serializer>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error> {
            let fields = AbsoluteTimeFields {
                day_of_year: self.day_of_year(),
                hours: self.hours(),
                minutes: self.minutes(),
                seconds: self.seconds(),
                nanoseconds: self.nanoseconds(),
                year: self.year,
            };
            fields.serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for AbsoluteTime {
        fn deserialize<D: Deserializer<'de>>(
            deserializer: D,
        ) -> core::result::Result<Self, D::Error> {
            let fields = AbsoluteTimeFields::deserialize(deserializer)?;
            let mut t = AbsoluteTime::new(
                fields.day_of_year,
                fields.hours,
                fields.minutes,
                fields.seconds,
                fields.nanoseconds,
            )
            .map_err(serde::de::Error::custom)?;
            t.set_year(fields.year).map_err(serde::de::Error::custom)?;
            Ok(t)
        }
    }
}

impl AbsoluteTime {
    /// Create a new `AbsoluteTime` with day-of-year format, validating ranges.
    ///
    /// **Traces:** L3-ABS-002
    pub fn new(
        day_of_year: u16,
        hours: u8,
        minutes: u8,
        seconds: u8,
        nanoseconds: u32,
    ) -> Result<Self> {
        if day_of_year == 0 || day_of_year > 366 {
            return Err(TimeError::OutOfRange {
                field: "day_of_year",
                value: day_of_year as u32,
                max: 366,
            });
        }
        if hours > 23 {
            return Err(TimeError::OutOfRange {
                field: "hours",
                value: hours as u32,
                max: 23,
            });
        }
        if minutes > 59 {
            return Err(TimeError::OutOfRange {
                field: "minutes",
                value: minutes as u32,
                max: 59,
            });
        }
        if seconds > 59 {
            return Err(TimeError::OutOfRange {
                field: "seconds",
                value: seconds as u32,
                max: 59,
            });
        }
        if nanoseconds > 999_999_999 {
            return Err(TimeError::OutOfRange {
                field: "nanoseconds",
                value: nanoseconds,
                max: 999_999_999,
            });
        }

        let total_ns = (day_of_year as u64 - 1) * NANOS_PER_DAY
            + hours as u64 * NANOS_PER_HOUR
            + minutes as u64 * NANOS_PER_MINUTE
            + seconds as u64 * NANOS_PER_SECOND
            + nanoseconds as u64;

        Ok(Self {
            total_ns,
            year: None,
        })
    }

    // ── Field accessors ─────────────────────────────────────────────

    /// Day of year (1–366).
    #[inline]
    pub fn day_of_year(&self) -> u16 {
        (self.total_ns / NANOS_PER_DAY + 1) as u16
    }

    /// Hours (0–23).
    #[inline]
    pub fn hours(&self) -> u8 {
        ((self.total_ns % NANOS_PER_DAY) / NANOS_PER_HOUR) as u8
    }

    /// Minutes (0–59).
    #[inline]
    pub fn minutes(&self) -> u8 {
        ((self.total_ns % NANOS_PER_HOUR) / NANOS_PER_MINUTE) as u8
    }

    /// Seconds (0–59).
    #[inline]
    pub fn seconds(&self) -> u8 {
        ((self.total_ns % NANOS_PER_MINUTE) / NANOS_PER_SECOND) as u8
    }

    /// Nanoseconds within the current second (0–999_999_999).
    #[inline]
    pub fn nanoseconds(&self) -> u32 {
        (self.total_ns % NANOS_PER_SECOND) as u32
    }

    /// Optional year (0–9999). Present when the time source provides it.
    #[inline]
    pub fn year(&self) -> Option<u16> {
        self.year
    }

    // ── Year mutator ────────────────────────────────────────────────

    /// Set the year field, validating the range (0–9999).
    ///
    /// This is the only setter on `AbsoluteTime`. Year arrives independently
    /// from NTP/PTP time sources and the correlation engine. Month and
    /// day-of-month require [`CalendarTime`].
    ///
    /// Passing `None` clears the year. Passing `Some(year)` validates that
    /// `year <= 9999`.
    ///
    /// **Traces:** L3-ABS-002
    #[inline]
    pub fn set_year(&mut self, year: Option<u16>) -> Result<()> {
        if let Some(y) = year {
            if y > 9999 {
                return Err(TimeError::OutOfRange {
                    field: "year",
                    value: y as u32,
                    max: 9999,
                });
            }
        }
        self.year = year;
        Ok(())
    }

    // ── Arithmetic ──────────────────────────────────────────────────

    /// Add `nanos` nanoseconds. Single `u64` add on the common path.
    ///
    /// **Traces:** L3-ABS-004, P4-04
    #[inline]
    pub fn add_nanos(&self, nanos: u64) -> Self {
        let new_total = self.total_ns + nanos;
        let max_year_ns = 366 * NANOS_PER_DAY;

        Self {
            total_ns: if new_total >= max_year_ns {
                new_total % max_year_ns
            } else {
                new_total
            },
            year: self.year,
        }
    }

    /// Subtract `nanos` nanoseconds. Single `u64` sub on the common path.
    ///
    /// Handles year rollover when subtracting past day 1 (GAP-05).
    ///
    /// **Traces:** L3-ABS-004, P4-04
    #[inline]
    pub fn sub_nanos(&self, nanos: u64) -> Self {
        if nanos <= self.total_ns {
            Self {
                total_ns: self.total_ns - nanos,
                year: self.year,
            }
        } else {
            self.sub_nanos_year_rollover(nanos)
        }
    }

    /// Slow path for sub_nanos when crossing year boundary.
    fn sub_nanos_year_rollover(&self, nanos: u64) -> Self {
        let deficit = nanos - self.total_ns;
        let mut new_year = self.year;

        let mut remaining = deficit;
        loop {
            let prev_year = new_year.map(|y| y.saturating_sub(1));
            let days_in_prev_year = match prev_year {
                // Uses crate::util::is_leap_year — see MSRV note in util.rs
                Some(y) => {
                    if crate::util::is_leap_year(y) {
                        366u64
                    } else {
                        365
                    }
                }
                None => 365,
            };
            let prev_year_ns = days_in_prev_year * NANOS_PER_DAY;
            new_year = prev_year;
            if remaining <= prev_year_ns {
                return Self {
                    total_ns: prev_year_ns - remaining,
                    year: new_year,
                };
            }
            remaining -= prev_year_ns;
        }
    }

    /// Nanoseconds elapsed since midnight of the current day.
    ///
    /// **Traces:** L3-ABS-005
    #[inline]
    pub fn total_nanos_of_day(&self) -> u64 {
        self.total_ns % NANOS_PER_DAY
    }

    /// Total internal nanosecond count (nanoseconds since start of day 1).
    ///
    /// This is the raw internal representation. Useful for efficient
    /// comparison and serialization.
    #[inline]
    pub fn as_total_ns(&self) -> u64 {
        self.total_ns
    }
}

impl core::fmt::Display for AbsoluteTime {
    /// Formats as `YYYY Day DDD HH:MM:SS.mmm.uuu` when year is present,
    /// or `Day DDD HH:MM:SS.mmm.uuu` otherwise.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let ns = self.nanoseconds();
        let ms = ns / 1_000_000;
        let us = (ns / 1_000) % 1_000;
        match self.year {
            Some(y) => {
                write!(
                    f,
                    "{:04} Day {:03} {:02}:{:02}:{:02}.{:03}.{:03}",
                    y,
                    self.day_of_year(),
                    self.hours(),
                    self.minutes(),
                    self.seconds(),
                    ms,
                    us
                )
            }
            None => {
                write!(
                    f,
                    "Day {:03} {:02}:{:02}:{:02}.{:03}.{:03}",
                    self.day_of_year(),
                    self.hours(),
                    self.minutes(),
                    self.seconds(),
                    ms,
                    us
                )
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// CalendarTime — AbsoluteTime enriched with validated calendar date
// ═══════════════════════════════════════════════════════════════════════

/// Calendar-aware absolute time with validated year, month, and day-of-month.
///
/// This type wraps [`AbsoluteTime`] and adds calendar metadata that is
/// guaranteed to be present and validated. It is produced by:
/// - BCD Day-Month-Year format parsing (`DmyFormatTime::to_calendar_time`)
/// - Promotion from `AbsoluteTime` via [`CalendarTime::new`]
/// - Conversion from `chrono::NaiveDateTime` (with the `chrono` feature)
///
/// # Type Safety
///
/// Unlike `AbsoluteTime`, which can represent partial states (year without
/// month, or no date at all), `CalendarTime` enforces that year, month, and
/// day-of-month are all present and valid. This is checked at construction
/// time — there are no piecemeal setters.
///
/// # Accessing Time Fields
///
/// `CalendarTime` implements `Deref<Target = AbsoluteTime>`, so all
/// `AbsoluteTime` methods (`hours()`, `minutes()`, `seconds()`,
/// `nanoseconds()`, `day_of_year()`, `year()`, `add_nanos()`, etc.)
/// are directly callable:
///
/// ```
/// use irig106_time::{AbsoluteTime, CalendarTime};
///
/// let ct = CalendarTime::from_parts(2025, 4, 10, 100, 12, 30, 25, 340_000_000).unwrap();
/// assert_eq!(ct.hours(), 12);        // via Deref
/// assert_eq!(ct.month(), 4);          // CalendarTime's own method
/// assert_eq!(ct.day_of_month(), 10);  // CalendarTime's own method
/// ```
///
/// **Traces:** L3-ABS-006 ← L2-ABS-001 ← L1-ABS-001
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CalendarTime {
    /// The underlying DOY-based time. Year is always `Some`.
    time: AbsoluteTime,
    /// Month (1–12), validated at construction.
    month: u8,
    /// Day of month (1–31), validated at construction.
    day_of_month: u8,
}

impl CalendarTime {
    /// Create a `CalendarTime` from an `AbsoluteTime` and validated date fields.
    ///
    /// Validates:
    /// - `time.year()` is `Some` (year must be set)
    /// - `month` is 1–12
    /// - `day_of_month` is 1–N where N = days in that month/year (leap-aware)
    /// - `time.day_of_year()` matches the DOY computed from year/month/day
    ///
    /// **Traces:** L3-ABS-006
    pub fn new(time: AbsoluteTime, month: u8, day_of_month: u8) -> Result<Self> {
        let year = time.year().ok_or(TimeError::OutOfRange {
            field: "year",
            value: 0,
            max: 9999,
        })?;
        if month == 0 || month > 12 {
            return Err(TimeError::OutOfRange {
                field: "month",
                value: month as u32,
                max: 12,
            });
        }
        let max_day = crate::util::days_in_month(year, month);
        if day_of_month == 0 || day_of_month > max_day {
            return Err(TimeError::OutOfRange {
                field: "day_of_month",
                value: day_of_month as u32,
                max: max_day as u32,
            });
        }
        let expected_doy = crate::util::month_day_to_doy(year, month, day_of_month);
        if time.day_of_year() != expected_doy {
            return Err(TimeError::OutOfRange {
                field: "day_of_year",
                value: time.day_of_year() as u32,
                max: expected_doy as u32,
            });
        }

        Ok(Self {
            time,
            month,
            day_of_month,
        })
    }

    /// Create a `CalendarTime` from all component parts.
    ///
    /// Convenience constructor that combines `AbsoluteTime::new` + `set_year`
    /// + `CalendarTime::new` into a single validated call.
    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        year: u16,
        month: u8,
        day_of_month: u8,
        day_of_year: u16,
        hours: u8,
        minutes: u8,
        seconds: u8,
        nanoseconds: u32,
    ) -> Result<Self> {
        let mut t = AbsoluteTime::new(day_of_year, hours, minutes, seconds, nanoseconds)?;
        t.set_year(Some(year))?;
        Self::new(t, month, day_of_month)
    }

    // ── Calendar accessors ──────────────────────────────────────────

    /// Month (1–12). Always present and valid.
    #[inline]
    pub fn month(&self) -> u8 {
        self.month
    }

    /// Day of month (1–31). Always present and valid.
    #[inline]
    pub fn day_of_month(&self) -> u8 {
        self.day_of_month
    }

    /// Get the inner `AbsoluteTime` (year is always `Some`).
    #[inline]
    pub fn as_absolute_time(&self) -> &AbsoluteTime {
        &self.time
    }

    /// Consume self and return the inner `AbsoluteTime`.
    #[inline]
    pub fn into_absolute_time(self) -> AbsoluteTime {
        self.time
    }
}

impl core::ops::Deref for CalendarTime {
    type Target = AbsoluteTime;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.time
    }
}

impl From<CalendarTime> for AbsoluteTime {
    /// Convert to `AbsoluteTime`, preserving the year but dropping
    /// month and day-of-month (which `AbsoluteTime` cannot hold).
    fn from(ct: CalendarTime) -> Self {
        ct.time
    }
}

impl core::fmt::Display for CalendarTime {
    /// Formats as `YYYY-MM-DD HH:MM:SS.mmm.uuu`.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let ns = self.nanoseconds();
        let ms = ns / 1_000_000;
        let us = (ns / 1_000) % 1_000;
        let y = self.time.year().unwrap_or(0);
        write!(
            f,
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}.{:03}",
            y,
            self.month,
            self.day_of_month,
            self.hours(),
            self.minutes(),
            self.seconds(),
            ms,
            us
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Chapter 4 Binary Weighted Time
// ═══════════════════════════════════════════════════════════════════════

/// IRIG 106 Chapter 4 Binary Weighted Time.
///
/// Used in secondary headers (Packet Flag bits \[3:2\] = 0b00) and intra-packet
/// timestamps.
///
/// **Traces:** L3-CH4-001 ← L2-ABS-003 ← L1-ABS-002
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ch4BinaryTime {
    /// High-order 16 bits of the binary time word.
    pub high_order: u16,
    /// Low-order 16 bits of the binary time word.
    pub low_order: u16,
    /// Microsecond component.
    pub microseconds: u16,
}

impl Ch4BinaryTime {
    /// Parse from a 6-byte little-endian buffer: `[unused(2), high(2), low(2), usec(2)]`.
    ///
    /// **Traces:** L3-CH4-005
    pub fn from_secondary_bytes(buf: &[u8]) -> Result<Self> {
        if buf.len() < 8 {
            return Err(TimeError::BufferTooShort {
                expected: 8,
                actual: buf.len(),
            });
        }
        let high_order = u16::from_le_bytes([buf[2], buf[3]]);
        let low_order = u16::from_le_bytes([buf[4], buf[5]]);
        let microseconds = u16::from_le_bytes([buf[6], buf[7]]);
        Ok(Self {
            high_order,
            low_order,
            microseconds,
        })
    }

    /// Parse from an 8-byte intra-packet time stamp buffer.
    ///
    /// **Traces:** L3-CH4-005
    pub fn from_intra_packet_bytes(buf: &[u8]) -> Result<Self> {
        Self::from_secondary_bytes(buf)
    }

    /// Decode to absolute time.
    ///
    /// **Traces:** L3-CH4-002, L3-CH4-003, L3-CH4-004
    pub fn to_absolute(&self) -> Result<AbsoluteTime> {
        let combined = ((self.high_order as u32) << 16) | (self.low_order as u32);
        let time_10ms = combined & 0x0001_FFFF;
        let day_of_year = ((combined >> 17) & 0x01FF) as u16;

        let total_ms = time_10ms * 10;
        let total_secs = total_ms / 1000;
        let remaining_ms = total_ms % 1000;

        let hours = (total_secs / 3600) as u8;
        let minutes = ((total_secs % 3600) / 60) as u8;
        let seconds = (total_secs % 60) as u8;
        let nanoseconds = (remaining_ms * 1_000_000) + (self.microseconds as u32 * 1_000);

        AbsoluteTime::new(
            if day_of_year == 0 { 1 } else { day_of_year },
            hours,
            minutes,
            seconds,
            nanoseconds,
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════
// IEEE-1588 Time
// ═══════════════════════════════════════════════════════════════════════

/// IEEE-1588 Precision Time Protocol time value.
///
/// **Traces:** L3-1588-001 ← L2-ABS-005 ← L1-ABS-003
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ieee1588Time {
    /// Nanoseconds within the current second (0–999_999_999).
    pub nanoseconds: u32,
    /// Seconds since the IEEE-1588 epoch.
    pub seconds: u32,
}

impl Ieee1588Time {
    /// Parse from an 8-byte little-endian buffer: `[nanoseconds(4), seconds(4)]`.
    ///
    /// **Traces:** L3-1588-002
    #[inline]
    pub fn from_le_bytes(buf: &[u8]) -> Result<Self> {
        if buf.len() < 8 {
            return Err(TimeError::BufferTooShort {
                expected: 8,
                actual: buf.len(),
            });
        }
        let nanoseconds = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let seconds = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
        if nanoseconds >= 1_000_000_000 {
            return Err(TimeError::OutOfRange {
                field: "ieee1588_nanoseconds",
                value: nanoseconds,
                max: 999_999_999,
            });
        }
        Ok(Self {
            nanoseconds,
            seconds,
        })
    }

    /// Total nanoseconds since the IEEE-1588 epoch.
    ///
    /// **Traces:** L3-1588-003 ← L2-ABS-006
    #[inline]
    pub fn to_nanos_since_epoch(&self) -> u64 {
        (self.seconds as u64) * 1_000_000_000 + (self.nanoseconds as u64)
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Extended RTC (ERTC)
// ═══════════════════════════════════════════════════════════════════════

/// 64-bit Extended Relative Time Counter.
///
/// **Traces:** L3-ERTC-001 ← L2-ABS-007 ← L1-ABS-004
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ertc(u64);

impl Ertc {
    /// Parse from an 8-byte little-endian buffer.
    ///
    /// **Traces:** L3-ERTC-002
    #[inline]
    pub fn from_le_bytes(buf: &[u8]) -> Result<Self> {
        if buf.len() < 8 {
            return Err(TimeError::BufferTooShort {
                expected: 8,
                actual: buf.len(),
            });
        }
        Ok(Ertc(u64::from_le_bytes([
            buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
        ])))
    }

    /// Raw 64-bit tick count.
    pub fn as_raw(self) -> u64 {
        self.0
    }

    /// Convert to nanoseconds. Returns `u128` to avoid overflow.
    ///
    /// **Traces:** L3-ERTC-003 ← L2-ABS-008
    pub fn to_nanos(self) -> u128 {
        (self.0 as u128) * 100
    }
}

#[cfg(test)]
#[path = "absolute_tests.rs"]
mod absolute_tests;
