//! Time Data Format 2 (0x12) — Network Time (NTP / PTP).
//!
//! Introduced in IRIG 106-17 (CR93). Provides network-protocol time from
//! NTP (RFC 5905) or PTP (IEEE 1588) sources as an alternative to the
//! BCD-encoded Format 1 time packets.
//!
//! # Requirement Traceability
//!
//! | Requirement | Description |
//! |-------------|-------------|
//! | L1-F2CSDW-001..003 | Format 2 CSDW parsing |
//! | L1-NTP-001..004 | NTP time message decoding |
//! | L1-PTP-001..004 | PTP time message decoding |
//! | L1-TAI-001..003 | Leap second offset handling |

use crate::absolute::AbsoluteTime;
use crate::error::{Result, TimeError};

// ── Constants ────────────────────────────────────────────────────────

/// Seconds between the NTP epoch (1900-01-01) and the Unix epoch (1970-01-01).
///
/// 70 years, accounting for 17 leap years in 1904..1968.
pub const NTP_UNIX_EPOCH_OFFSET: u64 = 2_208_988_800;

/// Default TAI-UTC offset in seconds (as of January 1, 2017).
///
/// This value is 37 seconds and has been stable since 2017.
/// Use `LeapSecondTable` for accurate historical conversions.
pub const DEFAULT_TAI_UTC_OFFSET: i32 = 37;

// ── Format 2 CSDW ───────────────────────────────────────────────────

/// Network time protocol discriminant from the Format 2 CSDW.
///
/// **Traces:** L1-F2CSDW-002
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkTimeProtocol {
    /// Network Time Protocol (RFC 5905). Epoch: 1900-01-01 UTC.
    Ntp,
    /// Precision Time Protocol (IEEE 1588). Epoch: 1970-01-01 TAI.
    Ptp,
    /// Reserved / unrecognized value.
    Reserved(u8),
}

/// Parsed Time Data Format 2 (0x12) Channel-Specific Data Word.
///
/// **Traces:** L1-F2CSDW-001
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeF2Csdw(u32);

impl TimeF2Csdw {
    /// Construct from a raw 32-bit value.
    #[inline]
    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Construct from 4 little-endian bytes.
    #[inline]
    pub fn from_le_bytes(buf: [u8; 4]) -> Self {
        Self(u32::from_le_bytes(buf))
    }

    /// Raw 32-bit value.
    #[inline]
    pub fn as_raw(self) -> u32 {
        self.0
    }

    /// Encode as 4 little-endian bytes.
    ///
    /// **Traces:** GAP-11
    #[inline]
    pub fn to_le_bytes(self) -> [u8; 4] {
        self.0.to_le_bytes()
    }

    /// Time protocol (bits \[3:0\]).
    ///
    /// **Traces:** L1-F2CSDW-002
    #[inline]
    pub fn time_protocol(self) -> NetworkTimeProtocol {
        match self.0 & 0x0F {
            0 => NetworkTimeProtocol::Ntp,
            1 => NetworkTimeProtocol::Ptp,
            other => NetworkTimeProtocol::Reserved(other as u8),
        }
    }

    /// Check that reserved bits \[31:4\] are zero.
    ///
    /// **Traces:** L1-F2CSDW-003
    pub fn validate_reserved(&self) -> Result<()> {
        if self.0 & 0xFFFF_FFF0 != 0 {
            Err(TimeError::ReservedBitSet {
                position: "f2_csdw bits[31:4]",
            })
        } else {
            Ok(())
        }
    }
}

// ── NTP Time ─────────────────────────────────────────────────────────

/// NTP timestamp from a Format 2 Network Time packet.
///
/// NTP time is referenced in UTC with an epoch of January 1, 1900 00:00:00.
/// The 64-bit NTP timestamp consists of 32-bit seconds and 32-bit fractional
/// seconds (each fractional unit = 2⁻³² seconds ≈ 233 picoseconds).
///
/// **Traces:** L1-NTP-001, L1-NTP-002
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NtpTime {
    /// Seconds since 1900-01-01 00:00:00 UTC.
    pub seconds: u32,
    /// Fractional seconds in units of 2⁻³² seconds.
    pub fraction: u32,
}

impl NtpTime {
    /// Parse from an 8-byte little-endian buffer.
    ///
    /// Layout: `[seconds(4 LE), fraction(4 LE)]`
    ///
    /// **Traces:** L1-NTP-001
    #[inline]
    pub fn from_le_bytes(buf: &[u8]) -> Result<Self> {
        if buf.len() < 8 {
            return Err(TimeError::BufferTooShort {
                expected: 8,
                actual: buf.len(),
            });
        }
        Ok(Self {
            seconds: u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]),
            fraction: u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]),
        })
    }

    /// Convert fractional seconds to nanoseconds.
    ///
    /// Formula: `(fraction as u64 * 1_000_000_000) >> 32`
    ///
    /// **Traces:** L1-NTP-003
    #[inline]
    pub fn fraction_as_nanos(&self) -> u32 {
        // (fraction * 10^9) / 2^32, using u64 to avoid overflow
        ((self.fraction as u64 * 1_000_000_000) >> 32) as u32
    }

    /// Total nanoseconds since the NTP epoch (1900-01-01 UTC).
    pub fn to_nanos_since_ntp_epoch(&self) -> u64 {
        (self.seconds as u64) * 1_000_000_000 + (self.fraction_as_nanos() as u64)
    }

    /// Convert to Unix seconds (epoch 1970-01-01 UTC).
    ///
    /// Returns `None` if the NTP time is before the Unix epoch.
    ///
    /// **Traces:** L1-NTP-004
    pub fn to_unix_seconds(&self) -> Option<u64> {
        (self.seconds as u64).checked_sub(NTP_UNIX_EPOCH_OFFSET)
    }

    /// Convert to `AbsoluteTime` (day-of-year format).
    ///
    /// Uses the Unix epoch conversion internally. Returns the time as
    /// day-of-year within the current year.
    pub fn to_absolute(&self) -> Result<AbsoluteTime> {
        let unix_secs = self.to_unix_seconds().ok_or(TimeError::OutOfRange {
            field: "ntp_seconds",
            value: self.seconds,
            max: u32::MAX,
        })?;

        let nanos = self.fraction_as_nanos();

        // Convert Unix seconds to year/day-of-year/time-of-day
        let (year, doy, hour, minute, second) = unix_seconds_to_ymd(unix_secs);

        let mut abs = AbsoluteTime::new(doy, hour, minute, second, nanos)?;
        abs.year = Some(year);
        Ok(abs)
    }

    /// Encode as 8 little-endian bytes: 4B seconds + 4B fraction.
    ///
    /// This is the inverse of `from_le_bytes`.
    ///
    /// **Traces:** GAP-11
    #[inline]
    pub fn to_le_bytes(&self) -> [u8; 8] {
        let mut buf = [0u8; 8];
        buf[0..4].copy_from_slice(&self.seconds.to_le_bytes());
        buf[4..8].copy_from_slice(&self.fraction.to_le_bytes());
        buf
    }
}

// ── PTP Time ─────────────────────────────────────────────────────────

/// PTP (IEEE 1588) timestamp from a Format 2 Network Time packet.
///
/// PTP time is referenced in TAI (International Atomic Time) with an epoch
/// of January 1, 1970 00:00:00 TAI. TAI does not include leap seconds,
/// so PTP time differs from UTC by the accumulated leap-second offset.
///
/// **Traces:** L1-PTP-001, L1-PTP-002
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PtpTime {
    /// Seconds since 1970-01-01 00:00:00 TAI (48 bits).
    pub seconds: u64,
    /// Nanoseconds within the current second (0–999,999,999).
    pub nanoseconds: u32,
}

impl PtpTime {
    /// Parse from a 10-byte little-endian buffer.
    ///
    /// Layout: `[seconds(6 LE), nanoseconds(4 LE)]`
    ///
    /// **Traces:** L1-PTP-001
    #[inline]
    pub fn from_le_bytes(buf: &[u8]) -> Result<Self> {
        if buf.len() < 10 {
            return Err(TimeError::BufferTooShort {
                expected: 10,
                actual: buf.len(),
            });
        }
        let seconds = u64::from_le_bytes([buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], 0, 0]);
        let nanoseconds = u32::from_le_bytes([buf[6], buf[7], buf[8], buf[9]]);

        if nanoseconds >= 1_000_000_000 {
            return Err(TimeError::OutOfRange {
                field: "ptp_nanoseconds",
                value: nanoseconds,
                max: 999_999_999,
            });
        }

        Ok(Self {
            seconds,
            nanoseconds,
        })
    }

    /// Total nanoseconds since the PTP/TAI epoch.
    pub fn to_nanos_since_tai_epoch(&self) -> u128 {
        (self.seconds as u128) * 1_000_000_000 + (self.nanoseconds as u128)
    }

    /// Convert TAI seconds to UTC seconds by subtracting the leap-second offset.
    ///
    /// **Traces:** L1-PTP-004
    #[inline]
    pub fn to_utc_seconds(&self, tai_utc_offset: i32) -> u64 {
        if tai_utc_offset >= 0 {
            self.seconds.saturating_sub(tai_utc_offset as u64)
        } else {
            self.seconds.saturating_add((-tai_utc_offset) as u64)
        }
    }

    /// Convert to `AbsoluteTime` using the given TAI-UTC offset.
    ///
    /// The offset accounts for accumulated leap seconds. As of 2017, the
    /// offset is 37 seconds (TAI = UTC + 37).
    pub fn to_absolute(&self, tai_utc_offset: i32) -> Result<AbsoluteTime> {
        let utc_secs = self.to_utc_seconds(tai_utc_offset);
        let (year, doy, hour, minute, second) = unix_seconds_to_ymd(utc_secs);

        let mut abs = AbsoluteTime::new(doy, hour, minute, second, self.nanoseconds)?;
        abs.year = Some(year);
        Ok(abs)
    }

    /// Encode as 10 little-endian bytes: 6B seconds (48-bit) + 4B nanoseconds.
    ///
    /// This is the inverse of `from_le_bytes`.
    ///
    /// **Traces:** GAP-11
    #[inline]
    pub fn to_le_bytes(&self) -> [u8; 10] {
        let mut buf = [0u8; 10];
        let sec_bytes = self.seconds.to_le_bytes();
        buf[0..6].copy_from_slice(&sec_bytes[0..6]);
        buf[6..10].copy_from_slice(&self.nanoseconds.to_le_bytes());
        buf
    }
}

// ── Parsed Network Time ──────────────────────────────────────────────

/// Parsed payload from a Format 2 Network Time packet.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkTime {
    /// NTP time (UTC epoch 1900-01-01).
    Ntp(NtpTime),
    /// PTP time (TAI epoch 1970-01-01).
    Ptp(PtpTime),
}

/// Parse a Format 2 time packet payload.
///
/// `payload` starts after the packet header (and secondary header if present).
/// The first 4 bytes are the CSDW; the remainder is the time data.
///
/// **Traces:** L1-F2CSDW-001, L1-NTP-001, L1-PTP-001
pub fn parse_time_f2_payload(payload: &[u8]) -> Result<(TimeF2Csdw, NetworkTime)> {
    if payload.len() < 4 {
        return Err(TimeError::BufferTooShort {
            expected: 4,
            actual: payload.len(),
        });
    }

    let csdw = TimeF2Csdw::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
    let time_data = &payload[4..];

    let network_time = match csdw.time_protocol() {
        NetworkTimeProtocol::Ntp => {
            let ntp = NtpTime::from_le_bytes(time_data)?;
            NetworkTime::Ntp(ntp)
        }
        NetworkTimeProtocol::Ptp => {
            let ptp = PtpTime::from_le_bytes(time_data)?;
            NetworkTime::Ptp(ptp)
        }
        NetworkTimeProtocol::Reserved(v) => {
            return Err(TimeError::OutOfRange {
                field: "network_time_protocol",
                value: v as u32,
                max: 1,
            });
        }
    };

    Ok((csdw, network_time))
}

// ── Leap Second Table ────────────────────────────────────────────────

/// An entry in the leap-second table.
///
/// **Traces:** L1-TAI-001
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeapSecondEntry {
    /// Unix timestamp (UTC) at which this offset became effective.
    pub effective_unix: u64,
    /// TAI-UTC offset in seconds (TAI = UTC + offset).
    pub tai_utc_offset: i32,
}

/// Table of historical TAI-UTC leap-second offsets.
///
/// **Traces:** L1-TAI-001, L1-TAI-002, L1-TAI-003
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct LeapSecondTable {
    /// Entries sorted by `effective_unix` ascending.
    entries: alloc::vec::Vec<LeapSecondEntry>,
}

extern crate alloc;
use alloc::vec::Vec;

impl LeapSecondTable {
    /// Create the built-in table current as of 2026.
    ///
    /// **Traces:** L1-TAI-002
    pub fn builtin() -> Self {
        // Selected entries since 1972 (when the modern leap-second system began).
        // Prior to 1972, the TAI-UTC relationship was more complex.
        let entries = alloc::vec![
            LeapSecondEntry {
                effective_unix: 63_072_000,
                tai_utc_offset: 10
            }, // 1972-01-01
            LeapSecondEntry {
                effective_unix: 78_796_800,
                tai_utc_offset: 11
            }, // 1972-07-01
            LeapSecondEntry {
                effective_unix: 94_694_400,
                tai_utc_offset: 12
            }, // 1973-01-01
            LeapSecondEntry {
                effective_unix: 126_230_400,
                tai_utc_offset: 13
            }, // 1974-01-01
            LeapSecondEntry {
                effective_unix: 157_766_400,
                tai_utc_offset: 14
            }, // 1975-01-01
            LeapSecondEntry {
                effective_unix: 189_302_400,
                tai_utc_offset: 15
            }, // 1976-01-01
            LeapSecondEntry {
                effective_unix: 220_924_800,
                tai_utc_offset: 16
            }, // 1977-01-01
            LeapSecondEntry {
                effective_unix: 252_460_800,
                tai_utc_offset: 17
            }, // 1978-01-01
            LeapSecondEntry {
                effective_unix: 283_996_800,
                tai_utc_offset: 18
            }, // 1979-01-01
            LeapSecondEntry {
                effective_unix: 315_532_800,
                tai_utc_offset: 19
            }, // 1980-01-01
            LeapSecondEntry {
                effective_unix: 362_793_600,
                tai_utc_offset: 20
            }, // 1981-07-01
            LeapSecondEntry {
                effective_unix: 394_329_600,
                tai_utc_offset: 21
            }, // 1982-07-01
            LeapSecondEntry {
                effective_unix: 425_865_600,
                tai_utc_offset: 22
            }, // 1983-07-01
            LeapSecondEntry {
                effective_unix: 489_024_000,
                tai_utc_offset: 23
            }, // 1985-07-01
            LeapSecondEntry {
                effective_unix: 567_993_600,
                tai_utc_offset: 24
            }, // 1988-01-01
            LeapSecondEntry {
                effective_unix: 631_152_000,
                tai_utc_offset: 25
            }, // 1990-01-01
            LeapSecondEntry {
                effective_unix: 662_688_000,
                tai_utc_offset: 26
            }, // 1991-01-01
            LeapSecondEntry {
                effective_unix: 709_948_800,
                tai_utc_offset: 27
            }, // 1992-07-01
            LeapSecondEntry {
                effective_unix: 741_484_800,
                tai_utc_offset: 28
            }, // 1993-07-01
            LeapSecondEntry {
                effective_unix: 773_020_800,
                tai_utc_offset: 29
            }, // 1994-07-01
            LeapSecondEntry {
                effective_unix: 820_454_400,
                tai_utc_offset: 30
            }, // 1996-01-01
            LeapSecondEntry {
                effective_unix: 867_715_200,
                tai_utc_offset: 31
            }, // 1997-07-01
            LeapSecondEntry {
                effective_unix: 915_148_800,
                tai_utc_offset: 32
            }, // 1999-01-01
            LeapSecondEntry {
                effective_unix: 1_136_073_600,
                tai_utc_offset: 33
            }, // 2006-01-01
            LeapSecondEntry {
                effective_unix: 1_230_768_000,
                tai_utc_offset: 34
            }, // 2009-01-01
            LeapSecondEntry {
                effective_unix: 1_341_100_800,
                tai_utc_offset: 35
            }, // 2012-07-01
            LeapSecondEntry {
                effective_unix: 1_435_708_800,
                tai_utc_offset: 36
            }, // 2015-07-01
            LeapSecondEntry {
                effective_unix: 1_483_228_800,
                tai_utc_offset: 37
            }, // 2017-01-01
        ];
        Self { entries }
    }

    /// Create an empty table (for testing or custom construction).
    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add an entry. Maintains sort order.
    pub fn add(&mut self, entry: LeapSecondEntry) {
        let pos = self
            .entries
            .binary_search_by_key(&entry.effective_unix, |e| e.effective_unix)
            .unwrap_or_else(|e| e);
        self.entries.insert(pos, entry);
    }

    /// Look up the TAI-UTC offset for a given Unix timestamp (UTC).
    ///
    /// Returns the most recent offset that is effective at or before the given time.
    /// Returns `DEFAULT_TAI_UTC_OFFSET` if the table is empty.
    ///
    /// **Traces:** L1-TAI-003
    pub fn offset_at_unix(&self, unix_seconds: u64) -> i32 {
        if self.entries.is_empty() {
            return DEFAULT_TAI_UTC_OFFSET;
        }

        // Find the last entry with effective_unix <= unix_seconds
        match self
            .entries
            .binary_search_by_key(&unix_seconds, |e| e.effective_unix)
        {
            Ok(i) => self.entries[i].tai_utc_offset,
            Err(0) => {
                // Before the first entry — use first offset as best guess
                self.entries[0].tai_utc_offset
            }
            Err(i) => self.entries[i - 1].tai_utc_offset,
        }
    }

    /// Look up the TAI-UTC offset for a given TAI timestamp.
    ///
    /// Approximates by converting TAI to UTC using the current best guess,
    /// then looking up the offset for that UTC time.
    pub fn offset_at_tai(&self, tai_seconds: u64) -> i32 {
        // First approximation: assume current default offset
        let approx_utc = tai_seconds.saturating_sub(DEFAULT_TAI_UTC_OFFSET as u64);
        self.offset_at_unix(approx_utc)
    }

    /// Number of entries in the table.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the table is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for LeapSecondTable {
    fn default() -> Self {
        Self::builtin()
    }
}

// ── Helper: Unix seconds to year/doy/time ────────────────────────────

/// Convert Unix seconds to (year, day_of_year, hour, minute, second).
///
/// Crate-internal entry point for use by the correlation module.
pub(crate) fn unix_seconds_to_ymd_pub(unix_secs: u64) -> (u16, u16, u8, u8, u8) {
    unix_seconds_to_ymd(unix_secs)
}

/// Convert Unix seconds to (year, day_of_year, hour, minute, second).
///
/// Simplified civil time computation. Handles leap years but not leap seconds.
fn unix_seconds_to_ymd(unix_secs: u64) -> (u16, u16, u8, u8, u8) {
    let secs_per_day: u64 = 86_400;
    let mut days = unix_secs / secs_per_day;
    let time_of_day = unix_secs % secs_per_day;

    let hour = (time_of_day / 3600) as u8;
    let minute = ((time_of_day % 3600) / 60) as u8;
    let second = (time_of_day % 60) as u8;

    // Walk years from 1970
    let mut year: u16 = 1970;
    loop {
        // Guard against malformed timestamps that would overflow u16 year
        if year == u16::MAX {
            break;
        }
        let days_in_year: u64 = if is_leap_year(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year = year.saturating_add(1);
    }

    let doy = (days as u16) + 1; // 1-based day of year

    (year, doy, hour, minute, second)
}

#[inline]
fn is_leap_year(year: u16) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

#[cfg(test)]
#[path = "network_time_tests.rs"]
mod network_time_tests;
