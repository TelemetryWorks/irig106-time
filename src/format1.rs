//! Scale-qualified, leap-preserving Format 1 wire readings. Legacy calendar
//! arithmetic is deliberately not used to encode second 60 as the next day.
use crate::absolute::AbsoluteTime;
use crate::csdw::{DateFormat, TimeF1Csdw, TimeF1Layout, TimeFormat};
use crate::error::{Result, TimeError};
use crate::scale::{ScaleEvidence, ScalePolicy, Scaled, TimeScale, TimeScaleTagged};

/// Calendar fields retained without normalization, including second 60.
/// Parser results are validated. Publicly constructed/deserialized fields must
/// pass [`Self::validate`] before use; [`Self::to_absolute`] always checks them.
/// **Traces:** L3-TAI-107 ← L2-TAI-107
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CalendarReading {
    /// Explicit year if DATE=1 or caller supplied; never invented.
    pub year: Option<u16>,
    /// Day of year, one based.
    pub day_of_year: u16,
    /// Hour 0..23.
    pub hours: u8,
    /// Minute 0..59.
    pub minutes: u8,
    /// Second 0..59, or the accepted UTC leap label 60.
    pub seconds: u8,
    /// Nanoseconds within the second.
    pub nanoseconds: u32,
    /// Scale and assignment provenance.
    pub scale: ScaleEvidence,
}
impl CalendarReading {
    /// A retained leap-second label (TF-022); validation is independent.
    pub fn is_leap_second(&self) -> bool {
        self.seconds == 60
    }

    /// Validate fields and RDG-010's scale/date/time constraints. An eligible
    /// date is not proof that a leap was historically scheduled on that date.
    /// **Traces:** L3-TAI-107 ← L2-TAI-107
    pub fn validate(&self) -> Result<()> {
        AbsoluteTime::new(
            self.day_of_year,
            self.hours,
            self.minutes,
            0,
            self.nanoseconds,
        )?
        .with_year(self.year)?;
        let eligible_day = match self.year {
            None => matches!(self.day_of_year, 181 | 182 | 365 | 366),
            Some(y) => {
                let leap = crate::util::is_leap_year(y);
                self.day_of_year == if leap { 182 } else { 181 }
                    || self.day_of_year == if leap { 366 } else { 365 }
            }
        };
        let leap_ok = self.scale.effective() == TimeScale::Utc
            && eligible_day
            && self.hours == 23
            && self.minutes == 59;
        range(
            "seconds",
            self.seconds as u32,
            0,
            if leap_ok { 60 } else { 59 },
        )
    }

    /// Shift a labelled reading without silently normalizing a leap label.
    /// UTC day crossings and nonzero shifts of second60 require leap context;
    /// unknown scale also needs context. Zero preserves any validated label.
    /// **Traces:** L3-ABS-101 ← L2-ABS-101 ← L1-ABS-001
    pub fn checked_shift(&self, nanos: crate::SignedNanos) -> Result<Self> {
        self.validate()?;
        if nanos.get() == 0 {
            return Ok(*self);
        }
        if self.is_leap_second() || self.scale.effective() == TimeScale::Unknown {
            return Err(TimeError::NeedsContext {
                context: "leap/scale context for calendar arithmetic",
            });
        }
        let t = AbsoluteTime::new(
            self.day_of_year,
            self.hours,
            self.minutes,
            self.seconds,
            self.nanoseconds,
        )?
        .with_year(self.year)?
        .checked_shift(nanos)?;
        if self.scale.effective() == TimeScale::Utc
            && (t.day_of_year() != self.day_of_year || t.year() != self.year)
        {
            return Err(TimeError::NeedsContext {
                context: "UTC day crossing requires leap context",
            });
        }
        Ok(Self {
            year: t.year(),
            day_of_year: t.day_of_year(),
            hours: t.hours(),
            minutes: t.minutes(),
            seconds: t.seconds(),
            nanoseconds: t.nanoseconds(),
            scale: self.scale,
        })
    }

    /// Attach an explicit year and recheck the leap-day interpretation.
    pub fn with_year(mut self, year: u16) -> Result<Self> {
        self.year = Some(year);
        self.validate()?;
        Ok(self)
    }

    /// Project ordinary UTC readings to the prototype calendar type, retaining
    /// the scale wrapper. A leap label cannot fit that type; other scales need
    /// an explicit conversion. Neither is silently relabelled/normalized.
    pub fn to_absolute(&self) -> Result<Scaled<AbsoluteTime>> {
        self.validate()?;
        if self.scale.effective() != TimeScale::Utc || self.is_leap_second() {
            return Err(TimeError::NeedsContext {
                context: "scale conversion or leap-preserving calendar representation",
            });
        }
        let value = AbsoluteTime::new(
            self.day_of_year,
            self.hours,
            self.minutes,
            self.seconds,
            self.nanoseconds,
        )?
        .with_year(self.year)?;
        Ok(Scaled {
            value,
            scale: self.scale,
        })
    }
}
impl TimeScaleTagged for CalendarReading {
    fn scale(&self) -> ScaleEvidence {
        self.scale
    }
}

/// Format 1 raw data word and decoded calendar reading, without correlation.
/// **Traces:** L3-TAI-105 ← L2-TAI-105
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Format1Reading {
    /// Raw CSDW; its reserved findings remain separately inspectable.
    pub csdw: TimeF1Csdw,
    /// No time for reserved/NONE FMT; otherwise fields with scale evidence.
    pub time: Option<CalendarReading>,
    /// Raw per-word reserved intersections, for TF-005 warnings. No fifth word.
    pub reserved_body_bits: [u16; 4],
    /// Bytes consumed, excluding filler/checksum.
    pub consumed: usize,
}

/// Caller-qualified bounded Format 1 input; payload need not exclude filler.
/// **Traces:** L3-CSDW-105 ← L2-CSDW-105 ← L1-CSDW-005
#[derive(Debug, Clone, Copy)]
pub struct Format1Input<'a> {
    /// Original bytes, including the data word.
    pub payload: &'a [u8],
    /// Header Data Length in bytes, excluding trailer/filler.
    pub data_length: usize,
    /// Explicit caller packet layout; never inferred from a version hint.
    pub layout: TimeF1Layout,
    /// Explicit scale assumptions for formats without a stated scale.
    pub scales: ScalePolicy,
}
impl Format1Input<'_> {
    /// Inspect raw always-zero intersections without requiring valid BCD digits.
    pub fn reserved_body_bits(self) -> Result<[u16; 4]> {
        bounded_words(self.payload, self.data_length).map(|(_, _, bits, _)| bits)
    }
    /// Decode with exact Data Length bounds and figure-defined diagnostics.
    pub fn decode(self) -> Result<Format1Reading> {
        decode_time_f1(self.payload, self.data_length, self.layout, self.scales)
    }
}

/// Decode only within Data Length. Layout and scale assumptions are caller
/// inputs, never inferred from a header, host clock, or TMATS default.
/// Figures 11-13/14 (106-24R1 printed 11-25/PDF33); RDG-009/010.
/// **Traces:** L3-ERR-105 ← L2-ERR-105
/// **Traces:** L3-BCD-103 ← L2-BCD-103 ← L1-BCD-003
/// **Traces:** L3-BCD-104 ← L2-BCD-104 ← L1-BCD-004
/// **Traces:** L3-BCD-105 ← L2-BCD-105
/// **Traces:** L3-BCD-106 ← L2-BCD-106
pub fn decode_time_f1(
    payload: &[u8],
    data_length: usize,
    layout: TimeF1Layout,
    policy: ScalePolicy,
) -> Result<Format1Reading> {
    let (csdw, words, reserved_body_bits, consumed) = bounded_words(payload, data_length)?;
    let time = if csdw.payload_is_valid(layout) {
        Some(decode_calendar(
            words,
            csdw.date_format() == DateFormat::DayMonthYear,
            csdw.time_format_for_layout(layout),
            policy,
        )?)
    } else {
        None
    };
    Ok(Format1Reading {
        csdw,
        time,
        reserved_body_bits,
        consumed,
    })
}
fn decode_calendar(
    words: [u16; 4],
    calendar: bool,
    format: TimeFormat,
    policy: ScalePolicy,
) -> Result<CalendarReading> {
    let nanoseconds = (digit(words[0], 0, 0, 4, "milliseconds", 1)? * 10
        + digit(words[0], 0, 4, 4, "milliseconds", 2)? * 100)
        * 1_000_000;
    let seconds = (digit(words[0], 0, 8, 4, "seconds", 0)?
        + digit(words[0], 0, 12, 3, "seconds", 1)? * 10) as u8;
    let minutes = (digit(words[1], 1, 0, 4, "minutes", 0)?
        + digit(words[1], 1, 4, 3, "minutes", 1)? * 10) as u8;
    let hours =
        (digit(words[1], 1, 8, 4, "hours", 0)? + digit(words[1], 1, 12, 2, "hours", 1)? * 10) as u8;
    let (year, day_of_year) = if calendar {
        let day = digit(words[2], 2, 0, 4, "day_of_month", 0)?
            + digit(words[2], 2, 4, 4, "day_of_month", 1)? * 10;
        let month =
            digit(words[2], 2, 8, 4, "month", 0)? + digit(words[2], 2, 12, 1, "month", 1)? * 10;
        let year = digit(words[3], 3, 0, 4, "year", 0)?
            + digit(words[3], 3, 4, 4, "year", 1)? * 10
            + digit(words[3], 3, 8, 4, "year", 2)? * 100
            + digit(words[3], 3, 12, 2, "year", 3)? * 1000;
        range("month", month, 1, 12)?;
        let months = month_lengths(year as u16);
        range("day_of_month", day, 1, months[month as usize - 1] as u32)?;
        (
            Some(year as u16),
            months[..month as usize - 1]
                .iter()
                .map(|d| *d as u16)
                .sum::<u16>()
                + day as u16,
        )
    } else {
        (
            None,
            (digit(words[2], 2, 0, 4, "day_of_year", 0)?
                + digit(words[2], 2, 4, 4, "day_of_year", 1)? * 10
                + digit(words[2], 2, 8, 2, "day_of_year", 2)? * 100) as u16,
        )
    };
    let time = CalendarReading {
        year,
        day_of_year,
        hours,
        minutes,
        seconds,
        nanoseconds,
        scale: policy.for_format1(format),
    };
    if time.seconds == 60 && format == TimeFormat::Rtc {
        return Err(TimeError::OutOfRange {
            field: "seconds",
            value: 60,
            max: 59,
        });
    }
    time.validate()?;
    Ok(time)
}
// Inspect figure words independently of decimal/calendar validity, so one
// malformed digit cannot suppress an independently observed zero-bit warning.
fn bounded_words(
    payload: &[u8],
    data_length: usize,
) -> Result<(TimeF1Csdw, [u16; 4], [u16; 4], usize)> {
    let buf = payload
        .get(..data_length)
        .ok_or(TimeError::BufferTooShort {
            expected: data_length,
            actual: payload.len(),
        })?;
    if buf.len() < 4 {
        return Err(TimeError::BufferTooShort {
            expected: 4,
            actual: buf.len(),
        });
    }
    let csdw = TimeF1Csdw::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
    let calendar = csdw.date_format() == DateFormat::DayMonthYear;
    let consumed = if calendar { 12 } else { 10 };
    if buf.len() < consumed {
        return Err(TimeError::BufferTooShort {
            expected: consumed,
            actual: buf.len(),
        });
    }
    let mut words = [0u16; 4];
    for (word, bytes) in words.iter_mut().zip(buf[4..consumed].chunks_exact(2)) {
        *word = u16::from_le_bytes([bytes[0], bytes[1]]);
    }
    let masks = if calendar {
        [0x8000, 0xc080, 0xe000, 0xc000]
    } else {
        [0x8000, 0xc080, 0xfc00, 0]
    };
    let mut reserved_body_bits = [0; 4];
    for i in 0..4 {
        reserved_body_bits[i] = words[i] & masks[i];
    }
    Ok((csdw, words, reserved_body_bits, consumed))
}

fn range(field: &'static str, value: u32, min: u32, max: u32) -> Result<()> {
    if value < min || value > max {
        Err(TimeError::OutOfRange { field, value, max })
    } else {
        Ok(())
    }
}
fn digit(
    word: u16,
    word_index: u8,
    shift: u8,
    bits: u8,
    position: &'static str,
    digit: u8,
) -> Result<u32> {
    let nibble = ((word >> shift) & ((1 << bits) - 1)) as u8;
    if nibble > 9 {
        Err(TimeError::InvalidBcdDigit {
            nibble,
            position,
            word: word_index,
            digit,
            bit_offset: shift,
            raw_word: word,
        })
    } else {
        Ok(nibble as u32)
    }
}
fn month_lengths(year: u16) -> [u8; 12] {
    [
        31,
        if crate::util::is_leap_year(year) {
            29
        } else {
            28
        },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ]
}

/// Bounded Format1 reading with immutable edition, source and ITS qualification.
/// **Traces:** L3-CSDW-101 ← L2-CSDW-101 ← L1-CSDW-001
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualifiedFormat1Reading {
    /// Word and competing meanings; no guessed source on unresolved layout.
    pub csdw: crate::csdw::QualifiedCsdw,
    /// Common readable calendar value, even when source qualification is withheld.
    pub time: Option<CalendarReading>,
    /// Body zero-mask operands, independent of edition ambiguity.
    pub reserved_body_bits: [u16; 4],
    /// Bytes consumed, excluding adjacent filler.
    pub consumed: usize,
}
/// Resolve evidence before field interpretation, then decode only readable FMT.
/// Unknown layout can still read common FMT0..5; FMT15 stays unresolved, not None.
/// **Traces:** L3-CSDW-101 ← L2-CSDW-101 ← L1-CSDW-001
/// **Traces:** L3-EDN-201 ← L2-EDN-201 ← L1-EDN-002
pub fn decode_qualified_time_f1(
    payload: &[u8],
    data_length: usize,
    evidence: crate::edition::EditionEvidence,
    policy: ScalePolicy,
) -> Result<QualifiedFormat1Reading> {
    let (word, words, reserved_body_bits, consumed) = bounded_words(payload, data_length)?;
    let csdw = word.qualify(crate::edition::resolve_edition(evidence));
    let time = match csdw.format() {
        Some(format) if !matches!(format, TimeFormat::None | TimeFormat::Reserved(_)) => {
            Some(decode_calendar(
                words,
                word.date_format() == DateFormat::DayMonthYear,
                format,
                policy,
            )?)
        }
        _ => None,
    };
    Ok(QualifiedFormat1Reading {
        csdw,
        time,
        reserved_body_bits,
        consumed,
    })
}
