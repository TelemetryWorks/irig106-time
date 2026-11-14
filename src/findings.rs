//! Stable identities and reusable factual evidence (architecture §8).
//! No era, leap-window, edition or year inference occurs in this module.
//! **Traces:** L3-FND-101 ← L2-FND-101 ← L1-FND-001
extern crate alloc;
use crate::{NtpTime, Result, Rtc, TimeError, UnixSeconds};
use alloc::vec::Vec;
/// Accepted finding level, independent of factual validity/admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Context or diagnostic evidence.
    Information,
    /// Value may remain usable; individual reading controls its use.
    Warning,
    /// Affected value cannot be used.
    Error,
}
/// Permanent catalog identity. Unknown IDs cannot be constructed.
/// **Traces:** L3-FND-101 ← L2-FND-101 ← L1-FND-001
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FindingCode(u8);
impl FindingCode {
    /// Look up an approved permanent identity, including pending-level 31/32.
    pub fn from_number(number: u8) -> Result<Self> {
        if !(1..=32).contains(&number) {
            return Err(TimeError::InvalidTimelineInput("unknown finding ID"));
        }
        Ok(Self(number))
    }
    /// Numeric TF suffix, never reused.
    pub const fn number(self) -> u8 {
        self.0
    }
    /// Accepted catalog title.
    pub fn title(self) -> &'static str {
        match self.0 {
            1 => "A session has no time packet",
            2 => "A dynamic packet precedes the session's first time packet",
            3 => "A time packet is marked invalid (FMT `0xF` NONE; TS \"Time Not Valid\")",
            4 => "A time packet is malformed (a digit above 9, a field out of range, a short body)",
            5 => "Reserved bits are set (data word, body, secondary header)",
            6 => {
                "A reserved value is used (SRC `0x3`–`0xE`, FMT `0x6`–`0xE`, NTF `0x3`–`0xF`, TS `0x2`–`0xF`, ITS `1000`–`1111`, flags bits 3–2 `11`)"
            }
            7 => "The leap-year bit disagrees with the day or the year",
            8 => "Time packets on a channel the setup record does not declare TIMEIN",
            9 => "A declared time channel carries no time packets",
            10 => "TMATS and the packets differ (`TTF`, `TFMT`, `SHTF`)",
            11 => "A channel's time source differs from `TSRC`, or changes (lock lost or regained)",
            12 => "A secondary header fails its checksum",
            13 => "Channels use different secondary-header time formats",
            14 => "A secondary header's time and the counter's differ beyond the tolerance",
            15 => "Packet flags bit 6 is set without bit 7",
            16 => "A reference gap",
            17 => "A time jump",
            18 => "A counter reset: a new session",
            19 => "A packet later than the late-packet bound",
            20 => "A session's year is unknown",
            21 => "A conversion falls after the leap-second table's last known date",
            22 => "A leap second (23:59:60) was read",
            23 => "A time scale is assumed or unknown",
            24 => "Packet flags report an RTC sync error (bit 5)",
            25 => "A packet's ERTC and RTC disagree (\"RTC = ERTC/100\")",
            26 => "A Format 2 time packet in a recording declared before 106-17",
            27 => "Secondary headers satisfy only the other checksum reading (RDG-008)",
            28 => "Secondary-derived time has an undefined format",
            29 => "NTP time is zero (unknown or unsynchronized)",
            30 => "NTP time falls in a leap-second ambiguity window",
            31 => "Packet-layout edition evidence is contradictory",
            32 => "Year evidence is contradictory",
            _ => unreachable!("validated finding identity"),
        }
    }
    /// None means level pending owner in S03 (31) / S08 (32), not WARNING.
    pub fn default_severity(self) -> Option<Severity> {
        match self.0 {
            4 | 12 | 15 | 28 => Some(Severity::Error),
            9 | 11 | 18 | 20 | 22 | 23 | 27 => Some(Severity::Information),
            1..=30 => Some(Severity::Warning),
            _ => None,
        }
    }
    /// Apply a caller override only after a default level has owner approval.
    pub fn resolve_level(self, level: Option<Severity>) -> Result<Severity> {
        let default = self.default_severity().ok_or(TimeError::NeedsContext {
            context: "finding level pending owner (TF-031 S03 / TF-032 S08)",
        })?;
        Ok(level.unwrap_or(default))
    }
}
/// Factual effect is never changed by a caller severity override.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueValidity {
    /// Affected mapped value is retained, possibly labelled ambiguous.
    Kept,
    /// No usable absolute value from this affected input.
    Unusable,
    /// Finding alone does not establish validity or admission.
    Undetermined,
}
/// Optional recording identity; a packet ID is never silently a byte offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FindingLocation {
    /// Caller session, when known.
    pub session: Option<usize>,
    /// Caller packet identity, when known.
    pub packet: Option<u64>,
    /// Channel, when applicable.
    pub channel: Option<u16>,
    /// Actual supplied byte offset, never inferred from packet identity.
    pub byte_offset: Option<u64>,
}
/// Exact raw wire/derived integer quantity, independent of JSON representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawValue {
    /// Unsigned bits or count.
    Unsigned(u128),
    /// Signed difference/residual.
    Signed(i128),
}
/// Named observed/expected pair; the name specifies the field and units.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawField {
    /// Wire field or explicitly unit-labelled derived quantity.
    pub name: &'static str,
    /// Exact observed value.
    pub observed: RawValue,
    /// Expected value if a comparison exists.
    pub expected: Option<RawValue>,
}
/// Raw NTP evidence; fraction stays in 2^-32 s units, never rounded away.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NtpEvidence {
    /// Wire seconds/fraction, no inferred era.
    pub raw: NtpTime,
    /// Mapped Unix whole seconds only when supplied by a reviewed converter.
    pub mapped_unix: Option<UnixSeconds>,
}
/// Reusable evidence container; no serialization schema is frozen here.
/// **Traces:** L3-FND-101 ← L2-FND-101 ← L1-FND-001
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindingEvidence {
    /// Location where observed.
    pub location: FindingLocation,
    /// Other packet/reference identities, in caller order.
    pub related: Vec<FindingLocation>,
    /// Original raw counter, including when no timestamp is readable.
    pub raw_rtc: Option<Rtc>,
    /// Exact raw fields and comparisons.
    pub raw_fields: Vec<RawField>,
    /// Factual value effect, independent of configured severity.
    pub validity: ValueValidity,
    /// Original NTP words and optional mapped value for TF-029/030.
    pub ntp: Option<NtpEvidence>,
}
impl FindingEvidence {
    /// Explicit factual effect; empty optional fields stay absent.
    pub fn new(location: FindingLocation, validity: ValueValidity) -> Self {
        Self {
            location,
            related: Vec::new(),
            raw_rtc: None,
            raw_fields: Vec::new(),
            validity,
            ntp: None,
        }
    }
}
/// Stable finding plus configured level and unchanged factual evidence.
/// **Traces:** L3-FND-101 ← L2-FND-101 ← L1-FND-001
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// Permanent identity.
    pub code: FindingCode,
    /// Approved default or explicit caller override.
    pub severity: Severity,
    /// Factual evidence, not modified by severity selection.
    pub evidence: FindingEvidence,
}
impl Finding {
    /// Build evidence without accepting an unknown/pending-level emission.
    pub fn new(
        code: FindingCode,
        level: Option<Severity>,
        evidence: FindingEvidence,
    ) -> Result<Self> {
        let ntp_consistent = match code.number() {
            29 => {
                evidence.validity == ValueValidity::Unusable
                    && evidence.ntp.is_some_and(|n| {
                        n.raw.seconds == 0 && n.raw.fraction == 0 && n.mapped_unix.is_none()
                    })
            }
            30 => {
                evidence.validity == ValueValidity::Kept
                    && evidence.ntp.is_some_and(|n| {
                        (n.raw.seconds != 0 || n.raw.fraction != 0) && n.mapped_unix.is_some()
                    })
            }
            _ => true,
        };
        if !ntp_consistent {
            return Err(TimeError::InvalidTimelineInput(
                "inconsistent TF-029/030 evidence",
            ));
        }
        Ok(Self {
            code,
            severity: code.resolve_level(level)?,
            evidence,
        })
    }
    /// Attach TF-029 to a known all-zero sentinel. No absolute value is produced.
    /// RFC5905 §6 p14; rejects nonzero words rather than falsely labelling them.
    pub fn ntp_zero(
        location: FindingLocation,
        raw: NtpTime,
        level: Option<Severity>,
    ) -> Result<Self> {
        if raw.seconds != 0 || raw.fraction != 0 {
            return Err(TimeError::InvalidTimelineInput(
                "TF-029 requires both NTP words zero",
            ));
        }
        let mut evidence = FindingEvidence::new(location, ValueValidity::Unusable);
        evidence.ntp = Some(NtpEvidence {
            raw,
            mapped_unix: None,
        });
        Self::new(FindingCode(29), level, evidence)
    }
    /// Attach TF-030 after the caller/converter establishes a covered ambiguity.
    /// RFC1305 E.8 pp78–79; preserves supplied mapped seconds plus raw fraction.
    /// This does not detect a leap window, resolve an era or consult a leap table.
    pub fn ntp_ambiguity(
        location: FindingLocation,
        raw: NtpTime,
        mapped_unix: UnixSeconds,
        level: Option<Severity>,
    ) -> Result<Self> {
        let mut evidence = FindingEvidence::new(location, ValueValidity::Kept);
        evidence.ntp = Some(NtpEvidence {
            raw,
            mapped_unix: Some(mapped_unix),
        });
        Self::new(FindingCode(30), level, evidence)
    }
}
