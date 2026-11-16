//! Caller-supplied governing declarations; no TMATS parser or inserted defaults.
//! Contract2.3C/4.5; Ch9 Table9-4; RDG-001..006 and owner Q-TFMT-DEFAULT.
use crate::{
    Result, SignedRtcTicks, TimeError,
    findings::*,
    timeline::{PacketKey, PacketObservation},
};
use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::String,
    vec::Vec,
};

#[path = "declaration_checks.rs"]
mod checks;
pub use checks::{
    DeclarationAudit, DeclarationFinding, DetailedComparison, secondary_mapping, source_mapping,
};
/// Half-open unwrapped counter range, scoped to one caller session.
/// **Traces:** L3-DECL-101 ← L2-DECL-101 ← L1-DECL-001
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoverningRange {
    session: usize,
    start: SignedRtcTicks,
    end: Option<SignedRtcTicks>,
}
impl GoverningRange {
    /// Validate session and [start,end); counters are never unwrapped here.
    pub fn new(session: usize, start: SignedRtcTicks, end: Option<SignedRtcTicks>) -> Result<Self> {
        if session == 0 || end.is_some_and(|end| end.get() <= start.get()) {
            return Err(TimeError::InvalidTimelineInput("invalid declaration range"));
        }
        Ok(Self {
            session,
            start,
            end,
        })
    }
    /// Caller session identity.
    pub fn session(&self) -> usize {
        self.session
    }
    /// Inclusive unwrapped RTC start.
    pub fn start(&self) -> SignedRtcTicks {
        self.start
    }
    /// Exclusive end; None means the remaining session.
    pub fn end(&self) -> Option<SignedRtcTicks> {
        self.end
    }
    fn contains(&self, key: PacketKey) -> bool {
        key.session == self.session
            && key.ticks >= self.start.get()
            && self.end.is_none_or(|end| key.ticks < end.get())
    }
}
/// Raw structured attribute values; original spelling/absence are preserved.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct ChannelDeclaration {
    /// Packet channel identifier, supplied by the caller's configuration join.
    pub id: u16,
    /// R-x\CDT-n; only exact TIMEIN declares a time channel.
    pub cdt: Option<String>,
    /// R-x\TTF-n.
    pub ttf: Option<String>,
    /// R-x\TFMT-n, never filled with a default.
    pub tfmt: Option<String>,
    /// R-x\TSRC-n; compared with qualified SRC, also static selection evidence.
    pub tsrc: Option<String>,
    /// R-x\SHTF-n; compared only when the secondary header is present.
    pub shtf: Option<String>,
}
/// One caller-supplied configuration snapshot; no dependency on irig106-tmats.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupDeclarations {
    /// Counter interval governed by this snapshot.
    pub range: GoverningRange,
    /// Actual setup-record packet identity, not an invented byte offset.
    pub setup_packet: u64,
    /// Caller source/provenance, unchanged.
    pub provenance: String,
    /// Declared recording-format version, including unknown spelling.
    pub declared_version: Option<String>,
    /// Original R-x\RI4 value; never G\OD and not parsed here.
    pub ri4: Option<String>,
    /// Optional year extracted by the caller from governing RI4.
    pub recording_year: Option<u16>,
    /// Channel declarations, including non-TIMEIN and missing attributes.
    pub channels: Vec<ChannelDeclaration>,
}
/// Declaration availability is independent of whether a channel is declared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum DeclarationPresence {
    /// No setup declarations were supplied.
    Absent,
    /// A setup snapshot governs, possibly with no channels.
    Present,
    /// Setup data was supplied but no range covers this session/counter.
    NoGoverningRange,
}
/// Governing setup identity, copied into retained answer/finding context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoverningSetup {
    /// Exact counter interval.
    pub range: GoverningRange,
    /// Setup-record identity.
    pub setup_packet: u64,
    /// Original source/provenance.
    pub provenance: String,
    /// Original declared version, not an inferred packet layout.
    pub declared_version: Option<String>,
    /// Original RI4.
    pub ri4: Option<String>,
    /// Caller-extracted governing recording year.
    pub recording_year: Option<u16>,
}
/// Explicit absence/presence and immutable raw declaration snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclarationBasis {
    /// Whether any setup/range was given.
    pub presence: DeclarationPresence,
    /// Selected range/source, if any.
    pub governing: Option<GoverningSetup>,
    /// Raw selected channel; None is not an empty channel declaration.
    pub channel: Option<ChannelDeclaration>,
}
impl DeclarationBasis {
    fn timein(&self) -> bool {
        self.channel
            .as_ref()
            .is_some_and(|c| c.cdt.as_deref() == Some("TIMEIN"))
    }
}
/// Prepared observation plus provenance, to be retained with consumer results.
#[derive(Debug, Clone)]
pub struct PreparedObservation {
    /// Original counter/payload; governing admission flags and RI4 are refreshed.
    pub observation: PacketObservation,
    /// Declaration availability and exact governing setup/channel.
    pub declarations: DeclarationBasis,
}
/// Validated non-overlapping configuration ranges; caller owns their extent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declarations {
    snapshots: Option<Vec<SetupDeclarations>>,
}
impl Declarations {
    /// Explicit declaration-free mode, distinct from present-empty setup data.
    pub fn absent() -> Self {
        Self { snapshots: None }
    }
    /// Validate ranges and channel uniqueness, preserving raw attribute values.
    pub fn present(mut snapshots: Vec<SetupDeclarations>) -> Result<Self> {
        snapshots.sort_by_key(|s| (s.range.session, s.range.start.get()));
        for s in &snapshots {
            let mut ids = BTreeSet::new();
            if s.recording_year.is_some_and(|y| y > 9999)
                || s.channels.iter().any(|c| !ids.insert(c.id))
            {
                return Err(TimeError::InvalidTimelineInput(
                    "duplicate channel or invalid recording year",
                ));
            }
        }
        for p in snapshots.windows(2) {
            if p[0].range.session == p[1].range.session
                && p[0]
                    .range
                    .end
                    .is_none_or(|end| end.get() > p[1].range.start.get())
            {
                return Err(TimeError::InvalidTimelineInput(
                    "overlapping declaration ranges",
                ));
            }
        }
        Ok(Self {
            snapshots: Some(snapshots),
        })
    }
    /// Read-only raw snapshots; no effective/default value is written here.
    pub fn snapshots(&self) -> Option<&[SetupDeclarations]> {
        self.snapshots.as_deref()
    }
    /// Select by session and unwrapped counter, never by channel-number guesses.
    /// **Traces:** L3-DECL-101 ← L2-DECL-101 ← L1-DECL-001
    pub fn basis(&self, key: PacketKey, channel: u16) -> Result<DeclarationBasis> {
        if key.session == 0 {
            return Err(TimeError::InvalidTimelineInput(
                "declaration session is zero",
            ));
        }
        let snapshot = self
            .snapshots
            .as_ref()
            .and_then(|all| all.iter().find(|s| s.range.contains(key)));
        Ok(DeclarationBasis {
            presence: if snapshot.is_some() {
                DeclarationPresence::Present
            } else if self.snapshots.is_none() {
                DeclarationPresence::Absent
            } else {
                DeclarationPresence::NoGoverningRange
            },
            governing: snapshot.map(|s| GoverningSetup {
                range: s.range.clone(),
                setup_packet: s.setup_packet,
                provenance: s.provenance.clone(),
                declared_version: s.declared_version.clone(),
                ri4: s.ri4.clone(),
                recording_year: s.recording_year,
            }),
            channel: snapshot
                .and_then(|s| s.channels.iter().find(|c| c.id == channel))
                .cloned(),
        })
    }
    /// Apply only governing admission/RI4 facts; packet decoding stays authoritative.
    /// Keep the returned context with downstream answers; legacy cores do not store it.
    /// **Traces:** L3-DECL-102 ← L2-DECL-102 ← L1-DECL-002
    pub fn prepare(
        &self,
        mut observation: PacketObservation,
        key: PacketKey,
    ) -> Result<PreparedObservation> {
        if key.packet != observation.id {
            return Err(TimeError::InvalidTimelineInput(
                "declaration packet identity differs",
            ));
        }
        let declarations = self.basis(key, observation.channel)?;
        observation.declared_timein = declarations.timein();
        observation.declared_external = declarations.timein()
            && declarations
                .channel
                .as_ref()
                .is_some_and(|c| c.tsrc.as_deref() == Some("E"));
        if let Some(input) = observation.reference.take() {
            let mut context = input.year_basis.context();
            context.recording = declarations
                .governing
                .as_ref()
                .and_then(|s| s.recording_year);
            observation.reference = Some(input.with_year_context(context)?);
        }
        Ok(PreparedObservation {
            observation,
            declarations,
        })
    }
    /// Context-free RDG-001..004 comparisons; never change packet facts.
    /// Use [`Self::compare_qualified`] for SRC/SHTF with edition/flag evidence.
    /// **Traces:** L3-RDG-101 ← L2-RDG-101 ← L1-RDG-001
    pub fn compare(
        &self,
        packet: PacketFields,
        policy: &DeclarationPolicy,
    ) -> Result<ComparisonReport> {
        if packet.format.is_some_and(|f| f > 15) || packet.source.is_some_and(|s| s > 15) {
            return Err(TimeError::InvalidTimelineInput(
                "packet field exceeds nibble",
            ));
        }
        let basis = self.basis(packet.key, packet.channel)?;
        let channel = basis.channel.as_ref();
        let applicable = basis.timein();
        let raw_ttf = channel.and_then(|c| c.ttf.clone());
        let expected = if applicable {
            match raw_ttf.as_deref() {
                Some("1") => Some(0x11),
                Some("2") => Some(0x12),
                _ => None,
            }
        } else {
            None
        }; // RDG-001
        let ttf = Comparison {
            raw: raw_ttf.clone(),
            effective: raw_ttf.clone(),
            published_default: None,
            default_derived: false,
            inferred: false,
            reading: Reading::Rdg001,
            expected,
            observed: Some(packet.data_type),
            status: if !applicable {
                ComparisonStatus::NotApplicable
            } else {
                match raw_ttf.as_deref() {
                    None => ComparisonStatus::MissingDeclaration,
                    Some("0") => ComparisonStatus::ReservedDeclaration,
                    Some("1" | "2") => {
                        if expected == Some(packet.data_type) {
                            ComparisonStatus::Match
                        } else {
                            ComparisonStatus::Mismatch
                        }
                    }
                    _ => ComparisonStatus::UnknownDeclaration,
                }
            },
        };
        let raw = channel.and_then(|c| c.tfmt.clone());
        let default_derived = applicable && raw.is_none() && policy.use_tfmt_default;
        let effective = raw
            .clone()
            .or_else(|| default_derived.then(|| String::from("A"))); // RDG-002 opt-in only
        let reading = if packet.data_type == 0x12 {
            Reading::Rdg004
        } else if effective.as_deref() == Some("I") {
            Reading::Rdg003
        } else {
            Reading::Rdg002
        };
        let context = applicable
            && match packet.data_type {
                0x11 => raw_ttf.as_deref() == Some("1"),
                0x12 => raw_ttf.as_deref() == Some("2"),
                _ => false,
            };
        let mapping = if context {
            effective.as_deref().and_then(|s| match packet.data_type {
                0x11 => format1_mapping(s),
                0x12 => format2_mapping(s),
                _ => None,
            })
        } else {
            None
        };
        let tfmt = Comparison {
            raw: raw.clone(),
            effective: effective.clone(),
            published_default: applicable.then_some("A"),
            default_derived,
            inferred: mapping.is_some_and(|m| m.inferred),
            reading,
            expected: mapping.map(|m| m.wire),
            observed: packet.format,
            status: if !applicable {
                ComparisonStatus::NotApplicable
            } else if effective.is_none() {
                ComparisonStatus::MissingDeclaration
            } else if !context {
                ComparisonStatus::ContextMismatch
            } else if mapping.is_none() {
                ComparisonStatus::UnknownDeclaration
            } else if packet.format.is_none() {
                ComparisonStatus::UnavailablePacketField
            } else if mapping.map(|m| m.wire) == packet.format {
                ComparisonStatus::Match
            } else {
                ComparisonStatus::Mismatch
            },
        };
        let mut findings = Vec::new();
        for (field, c) in [
            (DeclarationField::Ttf, &ttf),
            (DeclarationField::Tfmt, &tfmt),
        ] {
            if c.status == ComparisonStatus::NotApplicable {
                continue;
            }
            if field == DeclarationField::Tfmt && raw.is_none() {
                findings.push(diagnostic(
                    &packet,
                    &basis,
                    c,
                    field,
                    DeclarationIssue::MissingRequired,
                    10,
                    policy,
                )?);
            }
            let reason = match c.status {
                ComparisonStatus::Mismatch => Some(DeclarationIssue::Mismatch),
                ComparisonStatus::UnknownDeclaration => Some(DeclarationIssue::UnknownDeclaration),
                ComparisonStatus::ReservedDeclaration => {
                    Some(DeclarationIssue::ReservedDeclaration)
                }
                ComparisonStatus::MissingDeclaration if field == DeclarationField::Ttf => {
                    Some(DeclarationIssue::MissingDeclaration)
                }
                _ => None,
            };
            if let Some(reason) = reason {
                findings.push(diagnostic(&packet, &basis, c, field, reason, 10, policy)?);
            }
        }
        if applicable
            && packet.format.is_some_and(|f| match packet.data_type {
                0x11 => (6..15).contains(&f),
                0x12 => f >= 3,
                _ => false,
            })
        {
            findings.push(diagnostic(
                &packet,
                &basis,
                &tfmt,
                DeclarationField::Tfmt,
                DeclarationIssue::ReservedObserved,
                6,
                policy,
            )?);
        }
        Ok(ComparisonReport {
            packet,
            basis,
            ttf,
            tfmt,
            findings,
        })
    }
}
/// The only accepted declaration-to-packet mapping vocabularies (D8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reading {
    /// TTF to packet type.
    Rdg001,
    /// Format1 named letters.
    Rdg002,
    /// Inferred Internal to RTC.
    Rdg003,
    /// Numeric Format2 NTF.
    Rdg004,
    /// TSRC letters to qualified Format1 SRC.
    Rdg005,
    /// SHTF to enabled secondary-header flags.
    Rdg006,
}
impl Reading {
    /// Permanent interpretation identity.
    pub fn id(self) -> &'static str {
        match self {
            Self::Rdg001 => "RDG-001",
            Self::Rdg002 => "RDG-002",
            Self::Rdg003 => "RDG-003",
            Self::Rdg004 => "RDG-004",
            Self::Rdg005 => "RDG-005",
            Self::Rdg006 => "RDG-006",
        }
    }
}
/// Mapping provenance; inference is never silently promoted to a stated fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mapping {
    /// Exact declared designation, including I whose pairing is inferred.
    pub declared: &'static str,
    /// Packet field value.
    pub wire: u8,
    /// Permanent reading identity.
    pub reading: Reading,
    /// True only for RDG-003.
    pub inferred: bool,
}
// RDG-002/003: Table9-4 versus §11.2.3.2a; owner2026-11-11 accepts I inference.
const FORMAT1: [(&str, u8); 7] = [
    ("B", 0),
    ("A", 1),
    ("G", 2),
    ("I", 3),
    ("U", 4),
    ("N", 5),
    ("X", 15),
];
/// Exact declared Format1 letter to FMT, without case/whitespace guesses.
/// **Traces:** L3-RDG-203 ← L2-RDG-203 ← L1-RDG-002
pub fn format1_mapping(letter: &str) -> Option<Mapping> {
    FORMAT1
        .iter()
        .find(|(s, _)| *s == letter)
        .map(|(s, v)| Mapping {
            declared: s,
            wire: *v,
            reading: if *s == "I" {
                Reading::Rdg003
            } else {
                Reading::Rdg002
            },
            inferred: *s == "I",
        })
}
/// Reverse mapping for every defined FMT, including inferred I and X/NONE.
pub fn format1_declaration(fmt: u8) -> Option<Mapping> {
    FORMAT1
        .iter()
        .find(|(_, v)| *v == fmt)
        .and_then(|(s, _)| format1_mapping(s))
}
/// RDG-004 numeric Format2 vocabulary; never reuse Format1 letter mappings.
pub fn format2_mapping(value: &str) -> Option<Mapping> {
    let wire = match value {
        "0" => 0,
        "1" => 1,
        "2" => 2,
        _ => return None,
    };
    Some(Mapping {
        declared: match wire {
            0 => "0",
            1 => "1",
            _ => "2",
        },
        wire,
        reading: Reading::Rdg004,
        inferred: false,
    })
}
/// Independent comparison policy; missing-required validation is unconditional.
#[derive(Debug, Default, Clone)]
pub struct DeclarationPolicy {
    /// Owner2026-11-13 opt-in to published A, always labelled default-derived.
    pub use_tfmt_default: bool,
    /// Finding-level overrides never change raw facts or reference admission.
    pub levels: BTreeMap<u8, Severity>,
}
/// Raw packet facts supplied by a header/CSDW reader, not inferred from TMATS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketFields {
    /// Session, unwrapped RTC and actual packet identity.
    pub key: PacketKey,
    /// Observed channel.
    pub channel: u16,
    /// Header data type byte.
    pub data_type: u8,
    /// Caller-qualified FMT/NTF nibble; None if unreadable or meaning unresolved.
    /// Edition qualification is S03's responsibility, never inferred here.
    pub format: Option<u8>,
    /// Original source nibble; SRC comparison additionally requires QualifiedCsdw.
    pub source: Option<u8>,
}
/// Why a declaration/packet comparison cannot silently imply equality.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonStatus {
    /// No governing TIMEIN channel.
    NotApplicable,
    /// Raw declaration missing; no default comparison requested.
    MissingDeclaration,
    /// Unknown exact declaration spelling/value.
    UnknownDeclaration,
    /// TTF0 is explicitly reserved.
    ReservedDeclaration,
    /// TTF does not establish the observed format's vocabulary.
    ContextMismatch,
    /// No readable CSDW field to compare.
    UnavailablePacketField,
    /// Defined mapping agrees.
    Match,
    /// Defined mapping differs; packet remains authoritative.
    Mismatch,
}
/// One RDG-qualified comparison, retaining raw/effective/default separately.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comparison {
    /// Original declaration, never default-filled.
    pub raw: Option<String>,
    /// Explicit value or caller-opted-in effective default.
    pub effective: Option<String>,
    /// Table9-4 published default, distinct from application.
    pub published_default: Option<&'static str>,
    /// Default was applied only to effective processing.
    pub default_derived: bool,
    /// RDG-003 inference label, even for a match.
    pub inferred: bool,
    /// Permanent reading identity.
    pub reading: Reading,
    /// Expected packet value if a mapping exists.
    pub expected: Option<u8>,
    /// Actual packet value, unchanged.
    pub observed: Option<u8>,
    /// Agreement or explicit reason comparison is unavailable.
    pub status: ComparisonStatus,
}
/// Declaration attribute involved in a finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarationField {
    /// R-x\TTF-n.
    Ttf,
    /// R-x\TFMT-n.
    Tfmt,
    /// R-x\TSRC-n.
    Tsrc,
    /// R-x\SHTF-n.
    Shtf,
}
/// Diagnostic reason, independent of accepted severity defaults.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarationIssue {
    /// No declared TTF to compare; no default is invented.
    MissingDeclaration,
    /// Required declaration absent, including when default processing is enabled.
    MissingRequired,
    /// Exact spelling has no accepted mapping.
    UnknownDeclaration,
    /// Explicit reserved declaration value.
    ReservedDeclaration,
    /// Defined declaration/packet difference.
    Mismatch,
    /// Packet field itself is reserved (TF-006).
    ReservedObserved,
    /// Qualified SRC changed from the previous observed defined source.
    SourceChanged,
}
/// Finding with standalone string/range/RDG provenance beside numeric evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComparisonFinding {
    /// Approved stable finding and configured severity.
    pub finding: Finding,
    /// Attribute name.
    pub field: DeclarationField,
    /// Factual reason, never replaced by severity overrides.
    pub reason: DeclarationIssue,
    /// Raw/effective/observed/RDG/default/inference evidence.
    pub comparison: Comparison,
    /// Exact governing range/channel and declaration presence.
    pub basis: DeclarationBasis,
}
/// Packet-authoritative comparison results; no source or payload is rewritten.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComparisonReport {
    /// Original packet bits.
    pub packet: PacketFields,
    /// Governing declarations or explicit absence.
    pub basis: DeclarationBasis,
    /// RDG-001 result.
    pub ttf: Comparison,
    /// RDG-002/003/004 result.
    pub tfmt: Comparison,
    /// Missing, unknown, reserved and mismatch findings with raw provenance.
    pub findings: Vec<ComparisonFinding>,
}
fn diagnostic(
    packet: &PacketFields,
    basis: &DeclarationBasis,
    comparison: &Comparison,
    field: DeclarationField,
    reason: DeclarationIssue,
    code: u8,
    policy: &DeclarationPolicy,
) -> Result<ComparisonFinding> {
    let location = FindingLocation {
        session: Some(packet.key.session),
        packet: Some(packet.key.packet),
        channel: Some(packet.channel),
        byte_offset: None,
    };
    let mut evidence = FindingEvidence::new(location, ValueValidity::Undetermined);
    if let Some(s) = &basis.governing {
        evidence = evidence.with_related(FindingLocation {
            packet: Some(s.setup_packet),
            channel: None,
            ..location
        });
    }
    if let Some(observed) = comparison.observed {
        evidence = evidence.with_raw_field(RawField {
            name: match field {
                DeclarationField::Ttf => "packet.data_type",
                DeclarationField::Tfmt => "packet.time_format",
                DeclarationField::Tsrc => "packet.time_source",
                DeclarationField::Shtf => "packet.secondary_time_format",
            },
            observed: RawValue::Unsigned(observed.into()),
            expected: comparison.expected.map(|e| RawValue::Unsigned(e.into())),
        });
    }
    evidence = evidence.with_raw_field(RawField {
        name: "packet.rtc_ticks_unwrapped",
        observed: RawValue::Signed(packet.key.ticks),
        expected: None,
    });
    let finding = Finding::new(
        FindingCode::from_number(code)?,
        policy.levels.get(&code).copied(),
        evidence,
    )?;
    Ok(ComparisonFinding {
        finding,
        field,
        reason,
        comparison: comparison.clone(),
        basis: basis.clone(),
    })
}

#[cfg(feature = "serde")]
crate::snapshot::wide::serialize_struct!(GoverningRange {
    session,
    start,
    end
});
#[cfg(feature = "serde")]
crate::snapshot::wide::serialize_struct!(SetupDeclarations {
    range,
    setup_packet,
    provenance,
    declared_version,
    ri4,
    recording_year,
    channels
});
#[cfg(feature = "serde")]
crate::snapshot::wide::serialize_struct!(GoverningSetup {
    range,
    setup_packet,
    provenance,
    declared_version,
    ri4,
    recording_year
});
#[cfg(feature = "serde")]
crate::snapshot::wide::serialize_struct!(DeclarationBasis {
    presence,
    governing,
    channel
});
#[cfg(feature = "serde")]
crate::snapshot::wide::serialize_struct!(Declarations { snapshots });
