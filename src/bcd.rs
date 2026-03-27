//! BCD-encoded time message decoding for IRIG 106 Time Data Format 1.
//!
//! Decodes the Day-of-Year (DOY) and Day-Month-Year (DMY) format time messages
//! found in Time Data Format 1 (0x11) packets following the CSDW.
//!
//! # Requirement Traceability
//!
//! | Requirement | Description |
//! |-------------|-------------|
//! | L3-BCD-001..007 | BCD helpers, DayFormatTime, DmyFormatTime |

use crate::absolute::AbsoluteTime;
use crate::error::{Result, TimeError};

// ---------------------------------------------------------------------------
// BCD helpers
// ---------------------------------------------------------------------------

/// Extract a multi-bit field from a u16 word and validate each 4-bit BCD nibble.
///
/// **Traces:** L3-BCD-001 ← L2-BCD-006, L2-BCD-009
#[inline]
fn extract_bcd_digit(word: u16, bit_offset: u8, position: &'static str) -> Result<u8> {
    let nibble = ((word >> bit_offset) & 0x0F) as u8;
    if nibble > 9 {
        return Err(TimeError::InvalidBcdDigit { nibble, position });
    }
    Ok(nibble)
}

/// Verify that specific bits in a word are zero.
///
/// **Traces:** L3-BCD-002 ← L2-BCD-005
#[inline]
fn check_reserved(word: u16, mask: u16, position: &'static str) -> Result<()> {
    if word & mask != 0 {
        Err(TimeError::ReservedBitSet { position })
    } else {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Day-of-Year Format
// ---------------------------------------------------------------------------

/// Decoded Day-of-Year format time message (8 bytes).
///
/// **Traces:** L3-BCD-003 ← L2-BCD-001, L2-BCD-002 ← L1-BCD-001
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DayFormatTime {
    /// Milliseconds (0–990, resolution 10 ms).
    pub milliseconds: u16,
    /// Seconds (0–59).
    pub seconds: u8,
    /// Minutes (0–59).
    pub minutes: u8,
    /// Hours (0–23).
    pub hours: u8,
    /// Day of year (1–366).
    pub day_of_year: u16,
}

impl DayFormatTime {
    /// Parse from 8 little-endian bytes (4× u16 words).
    ///
    /// # Wire Format (per RCC 123-20 Figure 5-13)
    ///
    /// Word 0: `[3:0]=Tmn [7:4]=Hmn [11:8]=Sn [14:12]=TSn [15]=rsv`
    /// Word 1: `[3:0]=Mn [6:4]=TMn [7]=rsv [11:8]=Hn [13:12]=THn [15:14]=rsv`
    /// Word 2: `[3:0]=Dn [7:4]=TDn [9:8]=HDn [15:10]=rsv`
    /// Word 3: reserved
    ///
    /// **Traces:** L3-BCD-003, L3-BCD-005, L3-BCD-006
    pub fn from_le_bytes(buf: &[u8]) -> Result<Self> {
        if buf.len() < 8 {
            return Err(TimeError::BufferTooShort {
                expected: 8,
                actual: buf.len(),
            });
        }

        let w0 = u16::from_le_bytes([buf[0], buf[1]]);
        let w1 = u16::from_le_bytes([buf[2], buf[3]]);
        let w2 = u16::from_le_bytes([buf[4], buf[5]]);
        // w3 is reserved

        // Word 0 reserved: bit 15
        check_reserved(w0, 0x8000, "day_fmt word0 bit15")?;
        // Word 1 reserved: bit 7, bits [15:14]
        check_reserved(w1, 0xC080, "day_fmt word1 bits[15:14,7]")?;
        // Word 2 reserved: bits [15:10]
        check_reserved(w2, 0xFC00, "day_fmt word2 bits[15:10]")?;

        // Milliseconds: Hmn*100 + Tmn*10 (L3-BCD-005)
        let tmn = extract_bcd_digit(w0, 0, "tens_of_ms")?;
        let hmn = extract_bcd_digit(w0, 4, "hundreds_of_ms")?;
        let milliseconds = (hmn as u16) * 100 + (tmn as u16) * 10;

        // Seconds
        let sn = extract_bcd_digit(w0, 8, "units_of_seconds")?;
        let tsn = ((w0 >> 12) & 0x07) as u8;
        if tsn > 5 {
            return Err(TimeError::InvalidBcdDigit {
                nibble: tsn,
                position: "tens_of_seconds",
            });
        }
        let seconds = tsn * 10 + sn;

        // Minutes
        let mn = extract_bcd_digit(w1, 0, "units_of_minutes")?;
        let tmn_min = ((w1 >> 4) & 0x07) as u8;
        if tmn_min > 5 {
            return Err(TimeError::InvalidBcdDigit {
                nibble: tmn_min,
                position: "tens_of_minutes",
            });
        }
        let minutes = tmn_min * 10 + mn;

        // Hours
        let hn = extract_bcd_digit(w1, 8, "units_of_hours")?;
        let thn = ((w1 >> 12) & 0x03) as u8;
        let hours = thn * 10 + hn;

        // Day of year
        let dn = extract_bcd_digit(w2, 0, "units_of_day")?;
        let tdn = extract_bcd_digit(w2, 4, "tens_of_day")?;
        let hdn = ((w2 >> 8) & 0x03) as u8;
        let day_of_year = (hdn as u16) * 100 + (tdn as u16) * 10 + (dn as u16);

        // Validate ranges (L3-BCD-006)
        if seconds > 59 {
            return Err(TimeError::OutOfRange {
                field: "seconds",
                value: seconds as u32,
                max: 59,
            });
        }
        if minutes > 59 {
            return Err(TimeError::OutOfRange {
                field: "minutes",
                value: minutes as u32,
                max: 59,
            });
        }
        if hours > 23 {
            return Err(TimeError::OutOfRange {
                field: "hours",
                value: hours as u32,
                max: 23,
            });
        }
        if day_of_year == 0 || day_of_year > 366 {
            return Err(TimeError::OutOfRange {
                field: "day_of_year",
                value: day_of_year as u32,
                max: 366,
            });
        }

        Ok(Self {
            milliseconds,
            seconds,
            minutes,
            hours,
            day_of_year,
        })
    }

    /// Convert to `AbsoluteTime`.
    ///
    /// **Traces:** L3-BCD-007 ← L2-BCD-007
    pub fn to_absolute(&self) -> AbsoluteTime {
        // Safe: values were validated in from_le_bytes
        AbsoluteTime {
            day_of_year: self.day_of_year,
            hours: self.hours,
            minutes: self.minutes,
            seconds: self.seconds,
            nanoseconds: (self.milliseconds as u32) * 1_000_000,
            month: None,
            day_of_month: None,
            year: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Day-Month-Year Format
// ---------------------------------------------------------------------------

/// Decoded Day-Month-Year format time message (10 bytes).
///
/// **Traces:** L3-BCD-004 ← L2-BCD-003, L2-BCD-004 ← L1-BCD-002
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DmyFormatTime {
    /// Milliseconds (0–990, resolution 10 ms).
    pub milliseconds: u16,
    /// Seconds (0–59).
    pub seconds: u8,
    /// Minutes (0–59).
    pub minutes: u8,
    /// Hours (0–23).
    pub hours: u8,
    /// Day of month (1–31).
    pub day: u8,
    /// Month (1–12).
    pub month: u8,
    /// Year (0–9999).
    pub year: u16,
}

impl DmyFormatTime {
    /// Parse from 10 little-endian bytes (5× u16 words).
    ///
    /// # Wire Format (per RCC 123-20 Figure 5-14)
    ///
    /// Words 0–1: identical to DayFormatTime
    /// Word 2: `[3:0]=Dn [7:4]=TDn [11:8]=On [12]=TOn [15:13]=rsv`
    /// Word 3: `[3:0]=Yn [7:4]=TYn [11:8]=HYn [13:12]=OYn [15:14]=rsv`
    /// Word 4: reserved
    ///
    /// **Traces:** L3-BCD-004, L3-BCD-005, L3-BCD-006
    pub fn from_le_bytes(buf: &[u8]) -> Result<Self> {
        if buf.len() < 10 {
            return Err(TimeError::BufferTooShort {
                expected: 10,
                actual: buf.len(),
            });
        }

        let w0 = u16::from_le_bytes([buf[0], buf[1]]);
        let w1 = u16::from_le_bytes([buf[2], buf[3]]);
        let w2 = u16::from_le_bytes([buf[4], buf[5]]);
        let w3 = u16::from_le_bytes([buf[6], buf[7]]);
        // w4 is reserved

        // Word 0 reserved: bit 15
        check_reserved(w0, 0x8000, "dmy_fmt word0 bit15")?;
        // Word 1 reserved: bit 7, bits [15:14]
        check_reserved(w1, 0xC080, "dmy_fmt word1 bits[15:14,7]")?;
        // Word 2 reserved: bits [15:13]
        check_reserved(w2, 0xE000, "dmy_fmt word2 bits[15:13]")?;
        // Word 3 reserved: bits [15:14]
        check_reserved(w3, 0xC000, "dmy_fmt word3 bits[15:14]")?;

        // Milliseconds
        let tmn = extract_bcd_digit(w0, 0, "tens_of_ms")?;
        let hmn = extract_bcd_digit(w0, 4, "hundreds_of_ms")?;
        let milliseconds = (hmn as u16) * 100 + (tmn as u16) * 10;

        // Seconds
        let sn = extract_bcd_digit(w0, 8, "units_of_seconds")?;
        let tsn = ((w0 >> 12) & 0x07) as u8;
        if tsn > 5 {
            return Err(TimeError::InvalidBcdDigit {
                nibble: tsn,
                position: "tens_of_seconds",
            });
        }
        let seconds = tsn * 10 + sn;

        // Minutes
        let mn = extract_bcd_digit(w1, 0, "units_of_minutes")?;
        let tmn_min = ((w1 >> 4) & 0x07) as u8;
        if tmn_min > 5 {
            return Err(TimeError::InvalidBcdDigit {
                nibble: tmn_min,
                position: "tens_of_minutes",
            });
        }
        let minutes = tmn_min * 10 + mn;

        // Hours
        let hn = extract_bcd_digit(w1, 8, "units_of_hours")?;
        let thn = ((w1 >> 12) & 0x03) as u8;
        let hours = thn * 10 + hn;

        // Day of month
        let dn = extract_bcd_digit(w2, 0, "units_of_day")?;
        let tdn = extract_bcd_digit(w2, 4, "tens_of_day")?;
        let day = tdn * 10 + dn;

        // Month
        let on = extract_bcd_digit(w2, 8, "units_of_month")?;
        let ton = ((w2 >> 12) & 0x01) as u8;
        let month = ton * 10 + on;

        // Year
        let yn = extract_bcd_digit(w3, 0, "units_of_year")?;
        let tyn = extract_bcd_digit(w3, 4, "tens_of_year")?;
        let hyn = extract_bcd_digit(w3, 8, "hundreds_of_year")?;
        let oyn = ((w3 >> 12) & 0x03) as u8;
        let year = (oyn as u16) * 1000 + (hyn as u16) * 100 + (tyn as u16) * 10 + (yn as u16);

        // Validate ranges (L3-BCD-006)
        if seconds > 59 {
            return Err(TimeError::OutOfRange {
                field: "seconds",
                value: seconds as u32,
                max: 59,
            });
        }
        if minutes > 59 {
            return Err(TimeError::OutOfRange {
                field: "minutes",
                value: minutes as u32,
                max: 59,
            });
        }
        if hours > 23 {
            return Err(TimeError::OutOfRange {
                field: "hours",
                value: hours as u32,
                max: 23,
            });
        }
        if day == 0 || day > 31 {
            return Err(TimeError::OutOfRange {
                field: "day",
                value: day as u32,
                max: 31,
            });
        }
        if month == 0 || month > 12 {
            return Err(TimeError::OutOfRange {
                field: "month",
                value: month as u32,
                max: 12,
            });
        }

        Ok(Self {
            milliseconds,
            seconds,
            minutes,
            hours,
            day,
            month,
            year,
        })
    }

    /// Convert to `AbsoluteTime` with full date.
    ///
    /// Note: `day_of_year` is set to `day` as a placeholder; callers needing
    /// accurate DOY should compute it from the calendar date.
    ///
    /// **Traces:** L3-BCD-007 ← L2-BCD-008
    pub fn to_absolute(&self) -> AbsoluteTime {
        // Compute day-of-year from month/day (approximate: uses 0-indexed month table)
        let doy = month_day_to_doy(self.year, self.month, self.day);
        AbsoluteTime {
            day_of_year: doy,
            hours: self.hours,
            minutes: self.minutes,
            seconds: self.seconds,
            nanoseconds: (self.milliseconds as u32) * 1_000_000,
            month: Some(self.month),
            day_of_month: Some(self.day),
            year: Some(self.year),
        }
    }
}

/// Convert month/day to day-of-year.
#[inline]
fn month_day_to_doy(year: u16, month: u8, day: u8) -> u16 {
    let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let days_before: [u16; 12] = [
        0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334,
    ];
    let m = (month as usize).saturating_sub(1).min(11);
    let mut doy = days_before[m] + day as u16;
    if is_leap && month > 2 {
        doy += 1;
    }
    doy
}

#[cfg(test)]
#[path = "bcd_tests.rs"]
mod bcd_tests;
