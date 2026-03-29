//! Absolute time representations for IRIG 106 Chapter 10.
//!
//! This module defines the core absolute time type (`AbsoluteTime`) and the
//! three wire-format time types used in secondary headers and intra-packet
//! timestamps: Chapter 4 Binary Weighted Time, IEEE-1588, and ERTC.
//!
//! # v0.7.0 Breaking Change
//!
//! `AbsoluteTime` fields are now accessed via methods (`hours()`, `minutes()`,
//! etc.) instead of direct field access. The internal representation is a
//! single `u64` (nanoseconds since start of day 1), making `add_nanos` and
//! `sub_nanos` single arithmetic operations on the common path.
//!
//! # Requirement Traceability
//!
//! | Requirement | Description |
//! |-------------|-------------|
//! | L3-ABS-001..005 | `AbsoluteTime` struct and operations |
//! | P4-04           | Internal u64 representation |
//! | L3-CH4-001..005 | `Ch4BinaryTime` |
//! | L3-1588-001..004 | `Ieee1588Time` |
//! | L3-ERTC-001..003 | `Ertc` |

use crate::error::{Result, TimeError};

const NANOS_PER_SECOND: u64 = 1_000_000_000;
const NANOS_PER_MINUTE: u64 = 60 * NANOS_PER_SECOND;
const NANOS_PER_HOUR: u64 = 3600 * NANOS_PER_SECOND;
const NANOS_PER_DAY: u64 = 86_400 * NANOS_PER_SECOND;

/// Nanosecond-precision absolute time.
///
/// Internally stored as a single `u64` (nanoseconds since start of day 1),
/// with optional calendar metadata (year, month, day-of-month). Field access
/// is via methods: `day_of_year()`, `hours()`, `minutes()`, `seconds()`,
/// `nanoseconds()`.
///
/// # Construction
///
/// ```
/// use irig106_time::AbsoluteTime;
///
/// let t = AbsoluteTime::new(100, 12, 30, 25, 340_000_000).unwrap();
/// assert_eq!(t.day_of_year(), 100);
/// assert_eq!(t.hours(), 12);
/// ```
///
/// # Performance
///
/// `add_nanos` and `sub_nanos` are single `u64` arithmetic operations when
/// staying within the same year (the common case for correlation).
///
/// **Traces:** L3-ABS-001 ← L2-ABS-001, L2-ABS-002 ← L1-ABS-001, P4-04
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbsoluteTime {
    /// Nanoseconds since start of day 1 (day 1 00:00:00.000 = 0).
    total_ns: u64,
    /// Optional year (0–9999), present when DMY format is used.
    year: Option<u16>,
    /// Optional month (1–12), present when DMY format is used.
    month: Option<u8>,
    /// Optional day of month (1–31), present when DMY format is used.
    day_of_month: Option<u8>,
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
        month: Option<u8>,
        day_of_month: Option<u8>,
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
                month: self.month,
                day_of_month: self.day_of_month,
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
            t.year = fields.year;
            t.month = fields.month;
            t.day_of_month = fields.day_of_month;
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
            month: None,
            day_of_month: None,
        })
    }

    /// Attach optional DMY calendar date fields.
    ///
    /// **Traces:** L3-ABS-003
    pub fn with_date(mut self, year: u16, month: u8, day: u8) -> Result<Self> {
        if month == 0 || month > 12 {
            return Err(TimeError::OutOfRange {
                field: "month",
                value: month as u32,
                max: 12,
            });
        }
        if day == 0 || day > 31 {
            return Err(TimeError::OutOfRange {
                field: "day_of_month",
                value: day as u32,
                max: 31,
            });
        }
        if year > 9999 {
            return Err(TimeError::OutOfRange {
                field: "year",
                value: year as u32,
                max: 9999,
            });
        }
        self.year = Some(year);
        self.month = Some(month);
        self.day_of_month = Some(day);
        Ok(self)
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

    /// Optional year (0–9999), present when DMY format is used.
    #[inline]
    pub fn year(&self) -> Option<u16> {
        self.year
    }

    /// Optional month (1–12), present when DMY format is used.
    #[inline]
    pub fn month(&self) -> Option<u8> {
        self.month
    }

    /// Optional day of month (1–31), present when DMY format is used.
    #[inline]
    pub fn day_of_month(&self) -> Option<u8> {
        self.day_of_month
    }

    // ── Mutators for calendar metadata ──────────────────────────────

    /// Set the year field.
    #[inline]
    pub fn set_year(&mut self, year: Option<u16>) {
        self.year = year;
    }

    /// Set the month field.
    #[inline]
    pub fn set_month(&mut self, month: Option<u8>) {
        self.month = month;
    }

    /// Set the day-of-month field.
    #[inline]
    pub fn set_day_of_month(&mut self, day: Option<u8>) {
        self.day_of_month = day;
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
            month: self.month,
            day_of_month: self.day_of_month,
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
            // Common fast path: stays within the same year
            Self {
                total_ns: self.total_ns - nanos,
                year: self.year,
                month: self.month,
                day_of_month: self.day_of_month,
            }
        } else {
            // Year rollover path
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
                    month: self.month,
                    day_of_month: self.day_of_month,
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
    /// Formats as `YYYY-MM-DD HH:MM:SS.mmm.uuu` when year/month/day are
    /// available, or `Day DDD HH:MM:SS.mmm.uuu` otherwise.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let ns = self.nanoseconds();
        let ms = ns / 1_000_000;
        let us = (ns / 1_000) % 1_000;
        match (self.year, self.month, self.day_of_month) {
            (Some(y), Some(m), Some(d)) => {
                write!(
                    f,
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}.{:03}",
                    y,
                    m,
                    d,
                    self.hours(),
                    self.minutes(),
                    self.seconds(),
                    ms,
                    us
                )
            }
            _ => {
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

// ---------------------------------------------------------------------------
// Chapter 4 Binary Weighted Time
// ---------------------------------------------------------------------------

/// IRIG 106 Chapter 4 Binary Weighted Time.
///
/// Used in secondary headers (Packet Flag bits \[3:2\] = 0b00) and intra-packet
/// timestamps.
///
/// **Traces:** L3-CH4-001 ← L2-ABS-003 ← L1-ABS-002
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

// ---------------------------------------------------------------------------
// IEEE-1588 Time
// ---------------------------------------------------------------------------

/// IEEE-1588 Precision Time Protocol time value.
///
/// **Traces:** L3-1588-001 ← L2-ABS-005 ← L1-ABS-003
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

// ---------------------------------------------------------------------------
// Extended RTC (ERTC)
// ---------------------------------------------------------------------------

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
