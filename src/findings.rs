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
pub struct FindingCode(u8, &'static str);
impl FindingCode {
    /// Look up an approved permanent identity, including pending-level 31/32.
    pub fn from_number(number: u8) -> Result<Self> {
        const TITLES: [&str; 32] = [
            "A session has no time packet",
            "A dynamic packet precedes the session's first time packet",
            "A time packet is marked invalid (FMT `0xF` NONE; TS \"Time Not Valid\")",
            "A time packet is malformed (a digit above 9, a field out of range, a short body)",
            "Reserved bits are set (data word, body, secondary header)",
            "A reserved value is used (SRC `0x3`–`0xE`, FMT `0x6`–`0xE`, NTF `0x3`–`0xF`, TS `0x2`–`0xF`, ITS `1000`–`1111`, flags bits 3–2 `11`)",
            "The leap-year bit disagrees with the day or the year",
            "Time packets on a channel the setup record does not declare TIMEIN",
            "A declared time channel carries no time packets",
            "TMATS and the packets differ (`TTF`, `TFMT`, `SHTF`)",
            "A channel's time source differs from `TSRC`, or changes (lock lost or regained)",
            "A secondary header fails its checksum",
            "Channels use different secondary-header time formats",
            "A secondary header's time and the counter's differ beyond the tolerance",
            "Packet flags bit 6 is set without bit 7",
            "A reference gap",
            "A time jump",
            "A counter reset: a new session",
            "A packet later than the late-packet bound",
            "A session's year is unknown",
            "A conversion falls after the leap-second table's last known date",
            "A leap second (23:59:60) was read",
            "A time scale is assumed or unknown",
            "Packet flags report an RTC sync error (bit 5)",
            "A packet's ERTC and RTC disagree (\"RTC = ERTC/100\")",
            "A Format 2 time packet in a recording declared before 106-17",
            "Secondary headers satisfy only the other checksum reading (RDG-008)",
            "Secondary-derived time has an undefined format",
            "NTP time is zero (unknown or unsynchronized)",
            "NTP time falls in a leap-second ambiguity window",
            "Packet-layout edition evidence is contradictory",
            "Year evidence is contradictory",
        ];
        TITLES
            .get(usize::from(number).wrapping_sub(1))
            .copied()
            .map(|title| Self(number, title))
            .ok_or(TimeError::InvalidTimelineInput("unknown finding ID"))
    }

    /// Numeric TF suffix, never reused.
    pub const fn number(self) -> u8 {
        self.0
    }
    /// Accepted catalog title.
    pub fn title(self) -> &'static str {
        self.1
    }

    /// TF-031 WARNING is proposed, pending owner confirmation (S03).
    /// None remains level pending owner for TF-032 (S08).
    pub fn default_severity(self) -> Option<Severity> {
        match self.0 {
            4 | 12 | 15 | 28 => Some(Severity::Error),
            9 | 11 | 18 | 20 | 22 | 23 | 27 => Some(Severity::Information),
            1..=31 => Some(Severity::Warning),
            _ => None,
        }
    }
    /// Apply a caller override to an available default, including provisional TF-031.
    pub fn resolve_level(self, level: Option<Severity>) -> Result<Severity> {
        let default = self.default_severity().ok_or(TimeError::NeedsContext {
            context: "finding level pending owner (TF-032 S08)",
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
/// ```compile_fail
/// use irig106_time::findings::{FindingEvidence, ValueValidity};
/// fn repair(e: &mut FindingEvidence) { e.validity = ValueValidity::Kept; }
/// ```
/// **Traces:** L3-FND-101 ← L2-FND-101 ← L1-FND-001
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindingEvidence {
    location: FindingLocation,
    related: Vec<FindingLocation>,
    raw_rtc: Option<Rtc>,
    raw_fields: Vec<RawField>,
    validity: ValueValidity,
    ntp: Option<NtpEvidence>,
    decode_error: Option<TimeError>,
}
impl FindingEvidence {
    /// Original observation location.
    pub fn location(&self) -> FindingLocation {
        self.location
    }
    /// Related identities, without mutable access to stored evidence.
    pub fn related(&self) -> &[FindingLocation] {
        &self.related
    }
    /// Original raw counter, when present.
    pub fn raw_rtc(&self) -> Option<Rtc> {
        self.raw_rtc
    }
    /// Exact observed/expected fields.
    pub fn raw_fields(&self) -> &[RawField] {
        &self.raw_fields
    }
    /// Factual value effect.
    pub fn validity(&self) -> ValueValidity {
        self.validity
    }
    /// A copy of the NTP evidence, if present.
    pub fn ntp(&self) -> Option<NtpEvidence> {
        self.ntp
    }
    /// Add a raw counter before finding validation.
    pub fn with_raw_rtc(mut self, rtc: Rtc) -> Self {
        self.raw_rtc = Some(rtc);
        self
    }
    /// Add a related location before finding validation.
    pub fn with_related(mut self, location: FindingLocation) -> Self {
        self.related.push(location);
        self
    }
    /// Add an exact field before finding validation.
    pub fn with_raw_field(mut self, field: RawField) -> Self {
        self.raw_fields.push(field);
        self
    }
    /// Supply NTP evidence; Finding::new validates it for the selected identity.
    pub fn with_ntp(mut self, ntp: NtpEvidence) -> Self {
        self.ntp = Some(ntp);
        self
    }

    /// Original typed decode error, retained beside packet/channel/counter context.
    /// **Traces:** L3-ERR-102 ← L2-ERR-102 ← L1-ERR-002
    pub fn decode_error(&self) -> Option<&TimeError> {
        self.decode_error.as_ref()
    }
    /// Attach a parser cause without changing factual validity or severity.
    pub fn with_decode_error(mut self, error: TimeError) -> Self {
        self.decode_error = Some(error);
        self
    }
    /// Explicit factual effect; empty optional fields stay absent.
    pub fn new(location: FindingLocation, validity: ValueValidity) -> Self {
        Self {
            location,
            related: Vec::new(),
            raw_rtc: None,
            raw_fields: Vec::new(),
            validity,
            ntp: None,
            decode_error: None,
        }
    }
}
/// Stable finding plus configured level and unchanged factual evidence.
/// ```compile_fail
/// use irig106_time::findings::*;
/// fn bypass(code: FindingCode, evidence: FindingEvidence) -> Finding {
///     Finding { code, severity: Severity::Warning, evidence }
/// }
/// ```
/// **Traces:** L3-FND-101 ← L2-FND-101 ← L1-FND-001
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    code: FindingCode,
    severity: Severity,
    evidence: FindingEvidence,
}
impl Finding {
    /// Permanent validated identity.
    pub fn code(&self) -> FindingCode {
        self.code
    }
    /// Approved default or override, with no mutation after validation.
    pub fn severity(&self) -> Severity {
        self.severity
    }
    /// Immutable factual evidence.
    pub fn evidence(&self) -> &FindingEvidence {
        &self.evidence
    }
    /// Consume the finding to obtain its evidence, requiring revalidation to rebuild.
    pub fn into_evidence(self) -> FindingEvidence {
        self.evidence
    }

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
        Self::new(FindingCode::from_number(29)?, level, evidence)
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
        Self::new(FindingCode::from_number(30)?, level, evidence)
    }
}
