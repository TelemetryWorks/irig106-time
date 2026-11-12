//! Time Data Format 2 (0x12) — Network Time (NTP / PTP).
//!
//! Introduced in IRIG 106-17 (CR93). Provides network-protocol time from
//! NTP version 3 (RFC 1305) or PTP (IEEE 1588) sources as an alternative to the
//! BCD-encoded Format 1 time packets.
//!
//! # Requirement Traceability
//!
//! | Requirement | Description |
//! |-------------|-------------|
//! | L3-F2CSDW-101, L3-F2CSDW-104..106 | Format 2 CSDW parsing |
//! | L3-NTP-101..104 | NTP time message decoding |
//! | L3-PTP-102..105 | PTP time message decoding |
//! | L1-TAI-001..003 | Leap second offset handling |

use crate::absolute::AbsoluteTime;
use crate::error::{Result, TimeError};
use crate::scale::{ScaleEvidence, Scaled, TimeScale, TimeScaleTagged};
use irig106_types::{TaiSeconds, TaiUtcOffset, UnixSeconds};

// ── Constants ────────────────────────────────────────────────────────

/// Seconds between the NTP epoch (1900-01-01) and the Unix epoch (1970-01-01).
///
/// 70 years, accounting for 17 leap years in 1904..1968.
pub const NTP_UNIX_EPOCH_OFFSET: u64 = 2_208_988_800;

/// Default TAI-UTC offset in seconds (as of January 1, 2017).
///
/// This value is 37 seconds and has been stable since 2017.
/// Use `LeapSecondTable` for accurate historical conversions.
pub const DEFAULT_TAI_UTC_OFFSET: TaiUtcOffset = TaiUtcOffset::new(37);

// ── Format 2 CSDW ───────────────────────────────────────────────────

/// Network time protocol discriminant from the Format 2 CSDW.
///
/// **Traces:** L3-F2CSDW-104
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NetworkTimeProtocol {
    /// Network Time Protocol version 3 (RFC 1305). UTC, 1900 epoch; era unresolved.
    Ntp,
    /// Precision Time Protocol (IEEE 1588). Epoch: 1970-01-01 TAI.
    Ptp,
    /// Reserved / unrecognized value.
    Reserved(u8),
}

/// Exact NTF discriminant (106-24R1 Figure 11-15, printed 11-27).
/// **Traces:** L3-F2CSDW-104 ← L2-F2CSDW-104
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NetworkTimeFormat {
    /// NTP version 3.
    NtpV3,
    /// IEEE 1588-2002.
    Ptp2002,
    /// IEEE 1588-2008.
    Ptp2008,
    /// Reserved raw NTF nibble.
    Reserved(u8),
}

/// Recorded network time status, independent of format and reserved bits.
/// **Traces:** L3-F2CSDW-105 ← L2-F2CSDW-105
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeStatus {
    /// TS zero: time not valid.
    NotValid,
    /// TS one: time valid.
    Valid,
    /// Reserved raw TS nibble.
    Reserved(u8),
}

/// Parsed Time Data Format 2 (0x12) Channel-Specific Data Word.
///
/// **Traces:** L3-F2CSDW-101
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    /// Broad protocol from NTF bits 7–4. Use [`Self::time_format`] for PTP revision.
    /// **Traces:** L3-F2CSDW-104 ← L2-F2CSDW-104
    pub fn time_protocol(self) -> NetworkTimeProtocol {
        match self.time_format() {
            NetworkTimeFormat::NtpV3 => NetworkTimeProtocol::Ntp,
            NetworkTimeFormat::Ptp2002 | NetworkTimeFormat::Ptp2008 => NetworkTimeProtocol::Ptp,
            NetworkTimeFormat::Reserved(v) => NetworkTimeProtocol::Reserved(v),
        }
    }

    /// NTF bits 7–4 (Figure 11-15).
    /// **Traces:** L3-F2CSDW-104 ← L2-F2CSDW-104
    pub fn time_format(self) -> NetworkTimeFormat {
        match (self.0 >> 4) & 15 {
            0 => NetworkTimeFormat::NtpV3,
            1 => NetworkTimeFormat::Ptp2002,
            2 => NetworkTimeFormat::Ptp2008,
            v => NetworkTimeFormat::Reserved(v as u8),
        }
    }

    /// TS bits 3–0, preserved even when invalid or reserved.
    /// **Traces:** L3-F2CSDW-105 ← L2-F2CSDW-105
    pub fn time_status(self) -> TimeStatus {
        match self.0 & 15 {
            0 => TimeStatus::NotValid,
            1 => TimeStatus::Valid,
            v => TimeStatus::Reserved(v as u8),
        }
    }

    /// Raw reserved bits 31–8 for TF-005 reporting.
    /// **Traces:** L3-F2CSDW-106 ← L2-F2CSDW-106
    pub fn reserved_bits(self) -> u32 {
        self.0 & 0xffff_ff00
    }

    /// Separate TF-005 warning; it does not discard the raw reading.
    /// **Traces:** L3-F2CSDW-106 ← L2-F2CSDW-106
    pub fn validate_reserved(&self) -> Result<()> {
        if self.reserved_bits() != 0 {
            Err(TimeError::ReservedBitSet {
                position: "f2_csdw bits[31:8]",
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
/// **Traces:** L3-NTP-101, L3-NTP-102
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NtpTime {
    /// Raw 32-bit UTC-labelled seconds relative to 1900; era is unresolved.
    pub seconds: u32,
    /// Fractional seconds in units of 2⁻³² seconds.
    pub fraction: u32,
}

impl NtpTime {
    /// Parse from an 8-byte little-endian buffer.
    ///
    /// Layout: `[seconds(4 LE), fraction(4 LE)]`
    ///
    /// **Traces:** L3-NTP-101
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
    /// **Traces:** L3-NTP-103
    #[inline]
    pub fn fraction_as_nanos(&self) -> u32 {
        // (fraction * 10^9) / 2^32, using u64 to avoid overflow
        ((self.fraction as u64 * 1_000_000_000) >> 32) as u32
    }

    /// Nanoseconds represented by the raw modular seconds/fraction words.
    /// No NTP era or UTC/Unix leap mapping is inferred.
    pub fn raw_fixed_point_nanos(&self) -> u64 {
        self.seconds as u64 * 1_000_000_000 + self.fraction_as_nanos() as u64
    }

    /// Absolute NTP-epoch conversion is unresolved pending Q-NTP.
    /// Use [`Self::raw_fixed_point_nanos`] for an explicitly modular view.
    /// **Traces:** L3-NTP-102 ← L2-NTP-102
    pub fn to_nanos_since_ntp_epoch(&self) -> Result<u64> {
        Err(TimeError::NeedsContext {
            context: "Q-NTP: era and UTC/leap mapping",
        })
    }

    /// No era or leap mapping is assumed; returns NeedsContext pending Q-NTP.
    /// **Traces:** L3-NTP-104 ← L2-NTP-104
    pub fn to_unix_seconds(&self) -> Result<u64> {
        Err(TimeError::NeedsContext {
            context: "Q-NTP: era and UTC/leap mapping",
        })
    }

    /// No calendar conversion is implemented pending Q-NTP.
    /// **Traces:** L3-NTP-104 ← L2-NTP-104
    pub fn to_absolute(&self) -> Result<AbsoluteTime> {
        Err(TimeError::NeedsContext {
            context: "Q-NTP: era and UTC/leap mapping",
        })
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
/// **Traces:** L3-PTP-105, L3-PTP-102
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PtpTime {
    /// Seconds since 1970-01-01 00:00:00 TAI (32 bits).
    pub seconds: u32,
    /// Nanoseconds within the current second (0–999,999,999).
    pub nanoseconds: u32,
}

impl PtpTime {
    /// Parse from an 8-byte little-endian buffer.
    ///
    /// Layout: `[seconds(4 LE), nanoseconds(4 LE)]` (Figure 11-17).
    ///
    /// **Traces:** L3-PTP-105
    #[inline]
    pub fn from_le_bytes(buf: &[u8]) -> Result<Self> {
        if buf.len() < 8 {
            return Err(TimeError::BufferTooShort {
                expected: 8,
                actual: buf.len(),
            });
        }
        let seconds = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let nanoseconds = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);

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
    pub fn to_utc_seconds(&self, offset: TaiUtcOffset) -> u64 {
        self.tai_seconds().to_unix(offset).get()
    }

    /// The seconds field as a [`TaiSeconds`] value.
    #[inline]
    pub fn tai_seconds(&self) -> TaiSeconds {
        TaiSeconds::new(self.seconds as u64)
    }

    /// Convert to `AbsoluteTime` using the given TAI-UTC offset.
    ///
    /// The offset accounts for accumulated leap seconds. As of 2017, the
    /// offset is 37 seconds (TAI = UTC + 37).
    pub fn to_absolute(&self, offset: TaiUtcOffset) -> Result<AbsoluteTime> {
        let utc_secs = self.to_utc_seconds(offset);
        let (year, doy, hour, minute, second) = unix_seconds_to_ymd(utc_secs);

        let abs = AbsoluteTime::new(doy, hour, minute, second, self.nanoseconds)?
            .with_year(Some(year))?;
        Ok(abs)
    }

    /// Encode as 8 little-endian bytes: 4B seconds + 4B nanoseconds.
    ///
    /// This is the inverse of `from_le_bytes`.
    ///
    /// **Traces:** GAP-11
    #[inline]
    pub fn to_le_bytes(&self) -> [u8; 8] {
        let mut buf = [0u8; 8];
        buf[0..4].copy_from_slice(&self.seconds.to_le_bytes());
        buf[4..8].copy_from_slice(&self.nanoseconds.to_le_bytes());
        buf
    }
}

// ── Parsed Network Time ──────────────────────────────────────────────

/// Parsed payload from a Format 2 Network Time packet.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NetworkTime {
    /// NTP time (UTC epoch 1900-01-01).
    Ntp(NtpTime),
    /// PTP time (TAI epoch 1970-01-01).
    Ptp(PtpTime),
}

/// Raw Format 2 reading; invalid status remains recorded, never promoted.
/// **Traces:** L3-F2CSDW-105 ← L2-F2CSDW-105
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Format2Reading {
    /// Raw data word and independent NTF/TS/reserved evidence.
    pub csdw: TimeF2Csdw,
    /// Decoded body and stated scale; None for reserved NTF.
    pub time: Option<Scaled<NetworkTime>>,
    /// Number of bytes consumed, excluding filler/checksum.
    pub consumed: usize,
}
impl Format2Reading {
    /// Recorded TS=1 and a supported body; no reference-admission policy implied.
    pub fn is_usable(&self) -> bool {
        self.time.is_some() && self.csdw.time_status() == TimeStatus::Valid
    }
}

/// Decode within caller-supplied Data Length, excluding trailer filler/checksum.
/// Exactly twelve bytes are consumed; all reserved/status evidence remains visible.
/// **Traces:** L3-F2CSDW-101 ← L2-F2CSDW-101
/// **Traces:** L3-ERR-105 ← L2-ERR-105
pub fn decode_time_f2(payload: &[u8], data_length: usize) -> Result<Format2Reading> {
    let bounded = payload
        .get(..data_length)
        .ok_or(TimeError::BufferTooShort {
            expected: data_length,
            actual: payload.len(),
        })?;
    if bounded.len() < 12 {
        return Err(TimeError::BufferTooShort {
            expected: 12,
            actual: bounded.len(),
        });
    }
    let csdw = TimeF2Csdw::from_le_bytes([bounded[0], bounded[1], bounded[2], bounded[3]]);
    let body = &bounded[4..12];
    let time = match csdw.time_protocol() {
        NetworkTimeProtocol::Ntp => Some(NetworkTime::Ntp(NtpTime::from_le_bytes(body)?)),
        NetworkTimeProtocol::Ptp => Some(NetworkTime::Ptp(PtpTime::from_le_bytes(body)?)),
        NetworkTimeProtocol::Reserved(_) => None,
    }
    .map(|value| Scaled {
        scale: value.scale(),
        value,
    });
    Ok(Format2Reading {
        csdw,
        time,
        consumed: 12,
    })
}

/// Legacy tuple parser for a slice already bounded to Data Length.
/// Rejects invalid/reserved TS and reserved NTF. Use [`decode_time_f2`] to
/// preserve invalid raw readings separately from admission.
/// **Traces:** L3-F2CSDW-105 ← L2-F2CSDW-105
pub fn parse_time_f2_payload(payload: &[u8]) -> Result<(TimeF2Csdw, NetworkTime)> {
    let reading = decode_time_f2(payload, payload.len())?;
    if reading.csdw.time_status() != TimeStatus::Valid {
        return Err(TimeError::InvalidTimeStatus {
            status: (reading.csdw.as_raw() & 15) as u8,
        });
    }
    let time = reading.time.ok_or(TimeError::OutOfRange {
        field: "network_time_format",
        value: (reading.csdw.as_raw() >> 4) & 15,
        max: 2,
    })?;
    Ok((reading.csdw, time.value))
}

impl TimeScaleTagged for NtpTime {
    fn scale(&self) -> ScaleEvidence {
        ScaleEvidence::standard(TimeScale::Utc)
    }
}
impl TimeScaleTagged for PtpTime {
    fn scale(&self) -> ScaleEvidence {
        ScaleEvidence::standard(TimeScale::Tai)
    }
}
impl TimeScaleTagged for NetworkTime {
    fn scale(&self) -> ScaleEvidence {
        match self {
            Self::Ntp(t) => t.scale(),
            Self::Ptp(t) => t.scale(),
        }
    }
}

// ── Leap Second Table ────────────────────────────────────────────────

/// An entry in the leap-second table.
///
/// **Traces:** L1-TAI-001
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    pub fn offset_at_unix(&self, time: UnixSeconds) -> TaiUtcOffset {
        if self.entries.is_empty() {
            return DEFAULT_TAI_UTC_OFFSET;
        }

        // Find the last entry with effective_unix <= time
        let raw = match self
            .entries
            .binary_search_by_key(&time.get(), |e| e.effective_unix)
        {
            Ok(i) => self.entries[i].tai_utc_offset,
            Err(0) => {
                // Before the first entry — use first offset as best guess
                self.entries[0].tai_utc_offset
            }
            Err(i) => self.entries[i - 1].tai_utc_offset,
        };
        TaiUtcOffset::new(raw)
    }

    /// Look up the TAI-UTC offset for a given TAI timestamp.
    ///
    /// Approximates by converting TAI to UTC using the current best guess,
    /// then looking up the offset for that UTC time.
    pub fn offset_at_tai(&self, time: TaiSeconds) -> TaiUtcOffset {
        // First approximation: assume current default offset
        self.offset_at_unix(time.to_unix(DEFAULT_TAI_UTC_OFFSET))
    }

    /// Number of entries in the table.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the table is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Look up the TAI-UTC offset for a Format 1 (BCD) timestamp.
    ///
    /// Converts the year/day-of-year to an approximate Unix timestamp
    /// and delegates to `offset_at_unix`. This allows F1 time sources
    /// (IRIG-B, GPS, internal clock) to use the same leap second table
    /// as PTP sources.
    ///
    /// **Traces:** GAP-04
    pub fn offset_for_f1(&self, year: u16, day_of_year: u16) -> TaiUtcOffset {
        // Approximate Unix seconds for this year/doy
        let mut unix_secs: u64 = 0;
        for y in 1970..year {
            let days: u64 = if is_leap_year(y) { 366 } else { 365 };
            unix_secs += days * 86_400;
        }
        unix_secs += (day_of_year as u64).saturating_sub(1) * 86_400;
        self.offset_at_unix(UnixSeconds::new(unix_secs))
    }

    /// Check if a Unix timestamp falls within a window of a leap second boundary.
    ///
    /// Returns `true` if `time` is within `window_secs` of any
    /// leap second insertion point. Useful for flagging F1 time packets
    /// that may be affected by a leap second transition.
    ///
    /// **Traces:** GAP-04
    pub fn is_near_leap_second(&self, time: UnixSeconds, window_secs: u64) -> bool {
        // Uses crate::util::abs_diff_u64 — see MSRV note in util.rs
        self.entries
            .iter()
            .any(|e| crate::util::abs_diff_u64(time.get(), e.effective_unix) <= window_secs)
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

/// Leap year check — delegates to [`crate::util::is_leap_year`].
///
/// See `util.rs` for MSRV rationale.
#[inline]
fn is_leap_year(year: u16) -> bool {
    crate::util::is_leap_year(year)
}

#[cfg(test)]
#[path = "network_time_tests.rs"]
mod network_time_tests;
