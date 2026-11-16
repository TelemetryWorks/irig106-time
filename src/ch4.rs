//! Chapter 4 words with explicit caller calendar context (Q-CH4, owner2026-11-12).
//!
//! 106-24R1 §4.7/Fig4-4 (4-10/11, PDF14/15) specifies weights, not an origin.
//! Binary uses T-10's DERIVED full32-bit count. BCD's historical "characters"
//! versus "bits" remains unresolved even after a caller chooses an interpretation.
//! This API handles caller-extracted words; it does not extract PCM/1553 payloads.
//! Secondary headers/IPTS remain binary per Ch11 Fig11-4 (11-10/PDF18).
//! Calendar arithmetic here is nominal86400-second arithmetic with unknown scale,
//! not physical UTC/leap arithmetic. It neither guesses context nor searches packets.
use crate::{
    AbsoluteTime, Ch4BinaryTime, NanosDuration, Result, TimeError, edition::PacketEdition,
};

/// Three original Chapter4 words, supplied in significance order by the caller.
/// Secondary/IPTS byte decoding belongs to shared [`Ch4BinaryTime`]; retain its
/// reserved half-word separately with `secondary::validate_ch4_reserved` (TF-005).
/// **Traces:** L3-WORD-101 ← L2-WORD-101 ← L1-WORD-001
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ch4Words {
    /// High order word, without masking the BCD day fragment.
    pub high: u16,
    /// Low order word, including the BCD fixed-zero position.
    pub low: u16,
    /// Binary microseconds, valid0..9999 under either weighting.
    pub microseconds: u16,
}
impl From<Ch4BinaryTime> for Ch4Words {
    fn from(t: Ch4BinaryTime) -> Self {
        Self {
            high: t.high_order,
            low: t.low_order,
            microseconds: t.microseconds,
        }
    }
}

/// Caller-selected interpretation, NOT a resolution of §4.7's wording conflict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum BcdDayInterpretation {
    /// Compare the three wire bits to the low3 bits of the packed BCD ordinal day.
    /// Only its units digit contributes: day131 -> BCD0x131 -> fragment1;
    /// day139 -> BCD0x139 -> fragment1 too. Full day must still be supplied.
    LowThreePackedBcdBits,
}
/// Independently optional context; incomplete context produces labelled partials.
/// No default year, day, edition or interpretation is selected.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BcdContext {
    /// Explicit Gregorian year0..9999, never inferred from the fragment.
    pub year: Option<u16>,
    /// Full ordinal day1..365/366, never recovered from three fragment bits.
    pub full_day: Option<u16>,
    /// Explicit interpretation chosen by caller, not an implicit standard claim.
    pub interpretation: Option<BcdDayInterpretation>,
    /// Caller-declared edition solely to label the unresolved wording.
    pub edition: Option<PacketEdition>,
}
/// Exact caller context retained with the result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Ch4Context {
    /// Binary elapsed count relative to this supplied nominal calendar origin.
    Binary {
        /// None means no origin; an undated origin also remains partial.
        origin: Option<AbsoluteTime>,
    },
    /// BCD full calendar and explicitly selected day interpretation.
    Bcd(
        #[cfg_attr(
            feature = "serde",
            serde(serialize_with = "crate::snapshot::wide::serialize")
        )]
        BcdContext,
    ),
}
/// Missing inputs, without fabricated date fields or nearest-reference fallback.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MissingCh4Context {
    /// Binary calendar origin was not supplied.
    pub origin: bool,
    /// A year was not supplied (including an undated binary origin).
    pub year: bool,
    /// BCD full day was not supplied.
    pub full_day: bool,
    /// BCD day interpretation was not supplied.
    pub interpretation: bool,
}
/// Original edition-dependent phrase, never a settled interpretation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum BcdDayWording {
    /// 106-04..106-15: "three least significant characters of the BCD Julian date".
    Characters,
    /// 106-17+: "three least significant bits of the BCD Julian date".
    Bits,
    /// No edition declared; both historical phrases remain applicable evidence.
    EditionNotDeclared,
}
/// Labelled decoded value before any calendar reconstruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Ch4Value {
    /// Full binary elapsed count, never split into an invented day field.
    BinaryElapsed(
        #[cfg_attr(
            feature = "serde",
            serde(serialize_with = "crate::snapshot::wide::serialize")
        )]
        NanosDuration,
    ),
    /// Partial BCD clock plus raw day fragment, not a complete day of year.
    BcdTimeOfDay {
        /// Raw3-bit day fragment.
        day_fragment: u8,
        /// Decoded hour0..23.
        hours: u8,
        /// Decoded minute0..59.
        minutes: u8,
        /// Decoded second0..59 (no explicit second60 representation here).
        seconds: u8,
        /// Hundredths plus binary microseconds,0..999999000.
        nanoseconds: u32,
    },
}
/// Validated value with raw words, context, partial labels and ambiguity evidence.
/// Fields are immutable; no Deserialize bypass of context validation is provided.
/// **Traces:** L3-WORD-101 ← L2-WORD-101 ← L1-WORD-001
/// **Traces:** L3-ABS-102 ← L2-ABS-102 ← L1-ABS-002
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ch4Reading {
    raw: Ch4Words,
    context: Ch4Context,
    value: Ch4Value,
    calendar: Option<AbsoluteTime>,
    missing: MissingCh4Context,
    wording: Option<BcdDayWording>,
    reserved: u16,
}
impl Ch4Reading {
    /// Original words, including warning-level reserved bits.
    pub fn raw(&self) -> Ch4Words {
        self.raw
    }
    /// Exact context provided by caller.
    pub fn context(&self) -> Ch4Context {
        self.context
    }
    /// Elapsed count or partial BCD time of day; always available on success.
    pub fn value(&self) -> Ch4Value {
        self.value
    }
    /// Nominal calendar only when complete, validated context was supplied.
    /// This unscaled calendar is not labelled UTC and performs no leap conversion.
    pub fn calendar(&self) -> Option<AbsoluteTime> {
        self.calendar
    }
    /// Chapter4 weights establish no physical scale; even an explicit nominal
    /// origin does not silently label the result UTC/TAI/GPS.
    pub fn scale(&self) -> crate::scale::ScaleEvidence {
        crate::scale::ScaleEvidence::unknown()
    }
    /// Missing context labels; unresolved BCD wording is independent of these.
    pub fn missing(&self) -> MissingCh4Context {
        self.missing
    }
    /// BCD historical wording, or None for binary weighting.
    pub fn bcd_wording(&self) -> Option<BcdDayWording> {
        self.wording
    }
    /// True for every BCD result, even with explicit complete caller context.
    pub fn bcd_wording_unresolved(&self) -> bool {
        self.wording.is_some()
    }
    /// Fixed-zero bits set in BCD low word: TF-005 warning, value kept.
    /// Binary low bit15 is count data (T-10), not reserved.
    pub fn reserved_bits(&self) -> u16 {
        self.reserved
    }
}
impl Ch4Words {
    fn microseconds_ns(self) -> Result<u64> {
        if self.microseconds > 9999 {
            return Err(TimeError::OutOfRange {
                field: "ch4_microseconds",
                value: self.microseconds.into(),
                max: 9999,
            });
        }
        Ok(u64::from(self.microseconds) * 1000)
    }
    /// Decode all32 binary count bits; reconstruct only from explicit dated origin.
    /// Count unit10ms, binary microsecond unit1us. Max fits u64 exactly.
    /// **Traces:** L3-ABS-102 ← L2-ABS-102 ← L1-ABS-002
    /// **Traces:** L3-WORD-101 ← L2-WORD-101 ← L1-WORD-001
    pub fn binary(self, origin: Option<AbsoluteTime>) -> Result<Ch4Reading> {
        let ns = ((u64::from(self.high) << 16) | u64::from(self.low)) * 10_000_000
            + self.microseconds_ns()?;
        let calendar = match origin {
            Some(o) if o.year().is_some() => Some(o.add_nanos(ns)?),
            _ => None,
        };
        Ok(Ch4Reading {
            raw: self,
            context: Ch4Context::Binary { origin },
            value: Ch4Value::BinaryElapsed(NanosDuration::new(ns)),
            calendar,
            missing: MissingCh4Context {
                origin: origin.is_none(),
                year: origin.is_some_and(|o| o.year().is_none()),
                ..MissingCh4Context::default()
            },
            wording: None,
            reserved: 0,
        })
    }
    /// Decode Fig4-4 BCD digits; calendar requires year, full day and interpretation.
    /// The selected packed-BCD-bits interpretation is caller policy, not a silent
    /// resolution of the older "characters" versus current "bits" discrepancy.
    /// Range/digit errors reject the value; reserved low bit15 is separate TF-005.
    /// **Traces:** L3-WORD-101 ← L2-WORD-101 ← L1-WORD-001
    pub fn bcd(self, context: BcdContext) -> Result<Ch4Reading> {
        let us = self.microseconds_ns()? as u32;
        let digit = |word: u8, raw: u16, bit: u8, index: u8| -> Result<u8> {
            let nibble = ((raw >> bit) & 15) as u8;
            if nibble > 9 {
                return Err(TimeError::InvalidBcdDigit {
                    nibble,
                    position: "Chapter4 word",
                    word,
                    digit: index,
                    bit_offset: bit,
                    raw_word: raw,
                });
            }
            Ok(nibble)
        };
        let hours = ((self.high >> 11) & 3) as u8 * 10 + digit(0, self.high, 7, 0)?;
        let minutes = ((self.high >> 4) & 7) as u8 * 10 + digit(0, self.high, 0, 0)?;
        let seconds = ((self.low >> 12) & 7) as u8 * 10 + digit(1, self.low, 8, 0)?;
        let nanos = u32::from(digit(1, self.low, 4, 1)?) * 100_000_000
            + u32::from(digit(1, self.low, 0, 0)?) * 10_000_000
            + us;
        // Validate provided fields independently, including partial inputs.
        // No placeholder day/year is constructed for missing context.
        for (field, value, max) in [
            ("hours", u32::from(hours), 23),
            ("minutes", u32::from(minutes), 59),
            ("seconds", u32::from(seconds), 59),
        ] {
            if value > max {
                return Err(TimeError::OutOfRange { field, value, max });
            }
        }
        if let Some(year) = context.year {
            if year > 9999 {
                return Err(TimeError::OutOfRange {
                    field: "year",
                    value: year.into(),
                    max: 9999,
                });
            }
        }
        if let Some(day) = context.full_day {
            let max = if context.year.is_some_and(|y| !crate::util::is_leap_year(y)) {
                365
            } else {
                366
            };
            if day == 0 || day > max {
                return Err(TimeError::OutOfRange {
                    field: "day_of_year",
                    value: day.into(),
                    max: max.into(),
                });
            }
        }
        let fragment = (self.high >> 13) as u8;
        if let (Some(day), Some(BcdDayInterpretation::LowThreePackedBcdBits)) =
            (context.full_day, context.interpretation)
        {
            if fragment != ((day % 10) as u8 & 7) {
                return Err(TimeError::InvalidTimelineInput(
                    "Chapter4 day fragment contradicts caller context",
                ));
            }
        }
        let calendar = match (context.year, context.full_day, context.interpretation) {
            (Some(year), Some(day), Some(_)) => Some(
                AbsoluteTime::new(day, hours, minutes, seconds, nanos)?.with_year(Some(year))?,
            ),
            _ => None,
        };
        use PacketEdition::*;
        let wording = match context.edition {
            Some(V04 | V05 | V07 | V09 | V11 | V13 | V15) => BcdDayWording::Characters,
            Some(_) => BcdDayWording::Bits,
            None => BcdDayWording::EditionNotDeclared,
        };
        Ok(Ch4Reading {
            raw: self,
            context: Ch4Context::Bcd(context),
            value: Ch4Value::BcdTimeOfDay {
                day_fragment: fragment,
                hours,
                minutes,
                seconds,
                nanoseconds: nanos,
            },
            calendar,
            missing: MissingCh4Context {
                origin: false,
                year: context.year.is_none(),
                full_day: context.full_day.is_none(),
                interpretation: context.interpretation.is_none(),
            },
            wording: Some(wording),
            reserved: self.low & 0x8000,
        })
    }
}

#[cfg(feature = "serde")]
crate::snapshot::wide::serialize_struct!(Ch4Words {
    high,
    low,
    microseconds
});
#[cfg(feature = "serde")]
crate::snapshot::wide::serialize_struct!(BcdContext {
    year,
    full_day,
    interpretation,
    edition
});
#[cfg(feature = "serde")]
crate::snapshot::wide::serialize_struct!(MissingCh4Context {
    origin,
    year,
    full_day,
    interpretation
});
#[cfg(feature = "serde")]
crate::snapshot::wide::serialize_struct!(Ch4Reading {
    raw,
    context,
    value,
    calendar,
    missing,
    wording,
    reserved
});
