//! Absolute time representations for IRIG 106 Chapter 10.
//!
//! This module defines the core absolute time type (`AbsoluteTime`) and the
//! three wire-format time types used in secondary headers and intra-packet
//! timestamps: Chapter 4 Binary Weighted Time, IEEE-1588, and ERTC.
//!
//! # Requirement Traceability
//!
//! | Requirement | Description |
//! |-------------|-------------|
//! | L3-ABS-001..005 | `AbsoluteTime` struct and operations |
//! | L3-CH4-001..005 | `Ch4BinaryTime` |
//! | L3-1588-001..004 | `Ieee1588Time` |
//! | L3-ERTC-001..003 | `Ertc` |

use crate::error::{Result, TimeError};

/// Nanosecond-precision absolute time.
///
/// Represents a point in time as day-of-year plus time-of-day, with optional
/// calendar date (month/day/year) when DMY data is available.
///
/// **Traces:** L3-ABS-001 ← L2-ABS-001, L2-ABS-002 ← L1-ABS-001
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbsoluteTime {
    /// Day of year (1–366).
    pub day_of_year: u16,
    /// Hours (0–23).
    pub hours: u8,
    /// Minutes (0–59).
    pub minutes: u8,
    /// Seconds (0–59).
    pub seconds: u8,
    /// Nanoseconds within the current second (0–999_999_999).
    pub nanoseconds: u32,
    /// Optional month (1–12), present when DMY format is used.
    pub month: Option<u8>,
    /// Optional day of month (1–31), present when DMY format is used.
    pub day_of_month: Option<u8>,
    /// Optional year (0–9999), present when DMY format is used.
    pub year: Option<u16>,
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
        Ok(Self {
            day_of_year,
            hours,
            minutes,
            seconds,
            nanoseconds,
            month: None,
            day_of_month: None,
            year: None,
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

    /// Add `nanos` nanoseconds, carrying into seconds, minutes, hours, and days.
    ///
    /// **Traces:** L3-ABS-004
    #[inline]
    pub fn add_nanos(&self, nanos: u64) -> Self {
        let total_ns = self.total_nanos_of_day() + nanos;

        let nanos_per_second: u64 = 1_000_000_000;
        let seconds_per_day: u64 = 86_400;

        let total_seconds = total_ns / nanos_per_second;
        let remaining_ns = (total_ns % nanos_per_second) as u32;

        let extra_days = (total_seconds / seconds_per_day) as u16;
        let seconds_of_day = total_seconds % seconds_per_day;

        let hours = (seconds_of_day / 3600) as u8;
        let minutes = ((seconds_of_day % 3600) / 60) as u8;
        let seconds = (seconds_of_day % 60) as u8;

        // Wrap day_of_year (simplistic: does not handle year rollover)
        let mut new_day = self.day_of_year + extra_days;
        if new_day > 366 {
            new_day = ((new_day - 1) % 366) + 1;
        }

        Self {
            day_of_year: new_day,
            hours,
            minutes,
            seconds,
            nanoseconds: remaining_ns,
            month: self.month,
            day_of_month: self.day_of_month,
            year: self.year,
        }
    }

    /// Subtract `nanos` nanoseconds, borrowing from seconds, minutes, hours, days.
    ///
    /// **Traces:** L3-ABS-004
    #[inline]
    pub fn sub_nanos(&self, nanos: u64) -> Self {
        let current_ns = self.total_nanos_of_day();
        if nanos <= current_ns {
            // Simple case: stays within the same day
            let remaining = current_ns - nanos;
            let nanos_per_second: u64 = 1_000_000_000;
            let total_seconds = remaining / nanos_per_second;
            let ns_part = (remaining % nanos_per_second) as u32;
            let hours = (total_seconds / 3600) as u8;
            let minutes = ((total_seconds % 3600) / 60) as u8;
            let seconds = (total_seconds % 60) as u8;
            Self {
                day_of_year: self.day_of_year,
                hours,
                minutes,
                seconds,
                nanoseconds: ns_part,
                month: self.month,
                day_of_month: self.day_of_month,
                year: self.year,
            }
        } else {
            // Borrow from previous days
            let nanos_per_day: u64 = 86_400 * 1_000_000_000;
            let deficit = nanos - current_ns;
            let days_back = deficit.div_ceil(nanos_per_day) as u16;
            let remaining = (days_back as u64) * nanos_per_day - deficit;
            let nanos_per_second: u64 = 1_000_000_000;
            let total_seconds = remaining / nanos_per_second;
            let ns_part = (remaining % nanos_per_second) as u32;
            let hours = (total_seconds / 3600) as u8;
            let minutes = ((total_seconds % 3600) / 60) as u8;
            let seconds = (total_seconds % 60) as u8;
            let new_day = if self.day_of_year > days_back {
                self.day_of_year - days_back
            } else {
                366 - (days_back - self.day_of_year)
            };
            Self {
                day_of_year: new_day,
                hours,
                minutes,
                seconds,
                nanoseconds: ns_part,
                month: self.month,
                day_of_month: self.day_of_month,
                year: self.year,
            }
        }
    }

    /// Nanoseconds elapsed since midnight of the current day.
    ///
    /// **Traces:** L3-ABS-005
    #[inline]
    pub fn total_nanos_of_day(&self) -> u64 {
        let h = self.hours as u64;
        let m = self.minutes as u64;
        let s = self.seconds as u64;
        let ns = self.nanoseconds as u64;
        ((h * 3600 + m * 60 + s) * 1_000_000_000) + ns
    }
}

// ---------------------------------------------------------------------------
// Chapter 4 Binary Weighted Time
// ---------------------------------------------------------------------------

/// IRIG 106 Chapter 4 Binary Weighted Time.
///
/// Used in secondary headers (Packet Flag bits [3:2] = 0b00) and intra-packet
/// timestamps.
///
/// **Traces:** L3-CH4-001 ← L2-ABS-003 ← L1-ABS-002
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
    /// For secondary header layout (with 2-byte unused prefix), pass the 8-byte
    /// secondary header time field starting at the unused bytes.
    ///
    /// **Traces:** L3-CH4-005
    pub fn from_secondary_bytes(buf: &[u8]) -> Result<Self> {
        if buf.len() < 8 {
            return Err(TimeError::BufferTooShort {
                expected: 8,
                actual: buf.len(),
            });
        }
        // Bytes [0..2] are unused/reserved
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
    /// Layout: `[unused(2), high(2), low(2), usec(2)]`.
    ///
    /// **Traces:** L3-CH4-005
    pub fn from_intra_packet_bytes(buf: &[u8]) -> Result<Self> {
        Self::from_secondary_bytes(buf)
    }

    /// Decode to absolute time.
    ///
    /// The combined `high_order:low_order` value encodes day-of-year in the
    /// upper bits and time-of-day in 10 ms increments in the lower bits.
    ///
    /// **Traces:** L3-CH4-002, L3-CH4-003, L3-CH4-004
    pub fn to_absolute(&self) -> Result<AbsoluteTime> {
        let combined = ((self.high_order as u32) << 16) | (self.low_order as u32);

        // Bits [16:0] = time in 10 ms increments
        let time_10ms = combined & 0x0001_FFFF;
        // Bits [25:17] = day of year (1-based)
        let day_of_year = ((combined >> 17) & 0x01FF) as u16;

        let total_ms = time_10ms * 10;
        let total_secs = total_ms / 1000;
        let remaining_ms = total_ms % 1000;

        let hours = (total_secs / 3600) as u8;
        let minutes = ((total_secs % 3600) / 60) as u8;
        let seconds = (total_secs % 60) as u8;

        // Combine remaining ms with microseconds field
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
/// Used in secondary headers (Packet Flag bits [3:2] = 0b01) and intra-packet
/// timestamps.
///
/// **Traces:** L3-1588-001 ← L2-ABS-005 ← L1-ABS-003
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
/// Same 100 ns resolution as the 48-bit RTC but with a full 64-bit range.
///
/// **Traces:** L3-ERTC-001 ← L2-ABS-007 ← L1-ABS-004
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
