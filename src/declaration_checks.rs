//! RDG005/006: Table9-4 9-72; §11.2.3.2a 11-24; §11.2.1.1g 11-8.
//! Analysis and clause boundaries: docs/verification-s09b.md.
use super::*;
use crate::csdw::{QualifiedCsdw, TimeF1Source};

/// TSRC I/E/R/X to SRC0/1/2/15; never infer GPS from reserved3.
/// **Traces:** L3-RDG-205 ← L2-RDG-205 ← L1-RDG-002
pub fn source_mapping(value: &str) -> Option<Mapping> {
    let (declared, wire) = match value {
        "I" => ("I", 0),
        "E" => ("E", 1),
        "R" => ("R", 2),
        "X" => ("X", 15),
        _ => return None,
    };
    Some(Mapping {
        declared,
        wire,
        reading: Reading::Rdg005,
        inferred: false,
    })
}
/// SHTF0/1/2 to flags00/01/10. Chapter11 binary decoding governs0;
/// Chapter9's "Chapter 4 BCD" wording remains in DetailedComparison.
/// **Traces:** L3-RDG-206 ← L2-RDG-206 ← L1-RDG-002
pub fn secondary_mapping(value: &str) -> Option<Mapping> {
    let (declared, wire) = match value {
        "0" => ("0", 0),
        "1" => ("1", 1),
        "2" => ("2", 2),
        _ => return None,
    };
    Some(Mapping {
        declared,
        wire,
        reading: Reading::Rdg006,
        inferred: false,
    })
}
/// Complete registered comparisons, with retained layout and raw flag evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetailedComparison {
    /// Unchanged S09a TTF/TFMT result and packet/declaration basis.
    pub basic: ComparisonReport,
    /// RDG005. EXT/unresolved source is unavailable, never labelled SRC.
    pub tsrc: Comparison,
    /// RDG006, applicable only with an enabled secondary header.
    pub shtf: Comparison,
    /// Original S03 qualification, including all unresolved/contrary evidence.
    pub csdw: Option<QualifiedCsdw>,
    /// Original primary flags, not reconstructed from SHTF.
    pub flags: Option<u8>,
    /// Chapter9 versus Chapter11 wording when packet format00 is observed.
    pub secondary_wording: Option<&'static str>,
    /// All comparison findings, including basic.findings (not just new fields).
    pub findings: Vec<ComparisonFinding>,
    /// Non-mapping activity findings, populated by DeclarationAudit (TF008).
    pub activity: Vec<DeclarationFinding>,
}
fn plain_comparison(
    raw: Option<String>,
    mapping: Option<Mapping>,
    observed: Option<u8>,
    applicable: bool,
    context: bool,
    reading: Reading,
) -> Comparison {
    let status = if !applicable {
        ComparisonStatus::NotApplicable
    } else if !context {
        ComparisonStatus::UnavailablePacketField
    } else if raw.is_none() {
        ComparisonStatus::MissingDeclaration
    } else if mapping.is_none() {
        ComparisonStatus::UnknownDeclaration
    } else if observed.is_none() {
        ComparisonStatus::UnavailablePacketField
    } else if mapping.map(|m| m.wire) == observed {
        ComparisonStatus::Match
    } else {
        ComparisonStatus::Mismatch
    };
    Comparison {
        effective: raw.clone(),
        raw,
        published_default: None,
        default_derived: false,
        inferred: false,
        reading,
        expected: mapping.map(|m| m.wire),
        observed,
        status,
    }
}
impl Declarations {
    /// Compare only RDG001..006; keep values and static channel choice unchanged.
    /// QualifiedCsdw must agree with supplied raw fields and Format1 type.
    /// Missing SRC qualification never falls back to PacketFields.source.
    /// Missing/unknown TSRC is visible but does not invent lost lock/requiredness.
    /// SHTF applies to any declared channel with bit7 set; no defaults inserted.
    /// **Traces:** L3-FND-104 ← L2-FND-104 ← L1-FND-004
    /// **Traces:** L3-RDG-205 ← L2-RDG-205 ← L1-RDG-002
    /// **Traces:** L3-RDG-206 ← L2-RDG-206 ← L1-RDG-002
    pub fn compare_qualified(
        &self,
        packet: PacketFields,
        csdw: Option<&QualifiedCsdw>,
        flags: Option<u8>,
        policy: &DeclarationPolicy,
    ) -> Result<DetailedComparison> {
        if let Some(q) = csdw {
            let raw = q.raw().as_raw();
            if packet.data_type != 0x11
                || packet.source.is_some_and(|v| u32::from(v) != raw & 15)
                || packet
                    .format
                    .is_some_and(|v| u32::from(v) != (raw >> 4) & 15)
            {
                return Err(TimeError::InvalidTimelineInput(
                    "qualified CSDW and packet facts differ",
                ));
            }
        }
        let basic = self.compare(packet, policy)?;
        let basis = &basic.basis;
        let ch = basis.channel.as_ref();
        let source_context = basis.timein() && packet.data_type == 0x11;
        let observed = csdw.and_then(|q| match q.source() {
            Some(TimeF1Source::Source(_)) => Some((q.raw().as_raw() & 15) as u8),
            _ => None,
        });
        let raw = ch.and_then(|c| c.tsrc.clone());
        let mapping = raw.as_deref().and_then(source_mapping);
        let tsrc = plain_comparison(
            raw,
            mapping,
            observed,
            source_context,
            observed.is_some(),
            Reading::Rdg005,
        );
        let enabled = flags.is_some_and(|f| f & 0x80 != 0);
        let observed_secondary = flags.filter(|_| enabled).map(|f| (f >> 2) & 3);
        let raw = ch.and_then(|c| c.shtf.clone());
        let mapping = raw.as_deref().and_then(secondary_mapping);
        let shtf = plain_comparison(
            raw,
            mapping,
            observed_secondary,
            ch.is_some() && flags.is_none_or(|_| enabled),
            flags.is_some(),
            Reading::Rdg006,
        );
        let mut findings = basic.findings.clone();
        for (field, c, code) in [
            (DeclarationField::Tsrc, &tsrc, 11),
            (DeclarationField::Shtf, &shtf, 10),
        ] {
            let reason = match c.status {
                ComparisonStatus::Mismatch => Some(DeclarationIssue::Mismatch),
                ComparisonStatus::MissingDeclaration if field == DeclarationField::Shtf => {
                    Some(DeclarationIssue::MissingDeclaration)
                }
                ComparisonStatus::UnknownDeclaration if field == DeclarationField::Shtf => {
                    Some(DeclarationIssue::UnknownDeclaration)
                }
                _ => None,
            };
            if let Some(reason) = reason {
                findings.push(diagnostic(&packet, basis, c, field, reason, code, policy)?);
            }
            let reserved = c.status != ComparisonStatus::NotApplicable
                && match field {
                    DeclarationField::Tsrc => c.observed.is_some_and(|v| (3..=14).contains(&v)),
                    DeclarationField::Shtf => c.observed == Some(3),
                    _ => false,
                };
            if reserved {
                findings.push(diagnostic(
                    &packet,
                    basis,
                    c,
                    field,
                    DeclarationIssue::ReservedObserved,
                    6,
                    policy,
                )?);
            }
        }
        let secondary_wording=observed_secondary.filter(|v|*v==0).map(|_|
            "RDG-006: Table 9-4 says 0 Chapter 4 BCD; packet flags say 00 = Chapter 4 binary weighted 48-bit time format; Chapter 11 governs packet decoding");
        Ok(DetailedComparison {
            basic,
            tsrc,
            shtf,
            csdw: csdw.cloned(),
            flags,
            secondary_wording,
            findings,
            activity: Vec::new(),
        })
    }
}
/// Membership/absence finding (TF008/009), not an extra mapping vocabulary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclarationFinding {
    /// Stable approved identity, level and location.
    pub finding: Finding,
    /// Governing setup/range/channel, or explicit absence for TF008.
    pub basis: DeclarationBasis,
}
fn activity_finding(
    code: u8,
    key: PacketKey,
    channel: u16,
    basis: DeclarationBasis,
    data_type: Option<u8>,
    policy: &DeclarationPolicy,
) -> Result<DeclarationFinding> {
    let location = FindingLocation {
        session: Some(key.session),
        packet: data_type.map(|_| key.packet),
        channel: Some(channel),
        byte_offset: None,
    };
    let mut evidence = FindingEvidence::new(location, ValueValidity::Undetermined);
    if let Some(g) = &basis.governing {
        evidence = evidence.with_related(FindingLocation {
            packet: Some(g.setup_packet),
            channel: None,
            ..location
        });
    }
    if let Some(t) = data_type {
        evidence = evidence.with_raw_field(RawField {
            name: "packet.data_type",
            observed: RawValue::Unsigned(t.into()),
            expected: None,
        });
    }
    Ok(DeclarationFinding {
        finding: Finding::new(
            FindingCode::from_number(code)?,
            policy.levels.get(&code).copied(),
            evidence,
        )?,
        basis,
    })
}
/// Caller-fed scope audit. State is bounded by supplied TIMEIN declaration
/// inventory, never packet count. Observe in capture order; session IDs isolate
/// source history. finish consumes the audit and closes all supplied ranges.
/// Does not select/switch channels, parse bodies, or buffer/revise answers.
#[derive(Debug)]
pub struct DeclarationAudit<'a> {
    declarations: &'a Declarations,
    seen: BTreeSet<(usize, i128, u16)>,
    last: BTreeMap<(usize, u16), (u8, PacketKey)>,
}
impl<'a> DeclarationAudit<'a> {
    /// Borrow immutable governing declarations without copying packet history.
    pub fn new(declarations: &'a Declarations) -> Self {
        Self {
            declarations,
            seen: BTreeSet::new(),
            last: BTreeMap::new(),
        }
    }
    /// Report comparisons and source transitions without discarding raw values.
    /// Only qualified defined SRC0/1/2/15 can enter source history.
    /// **Traces:** L3-FND-104 ← L2-FND-104 ← L1-FND-004
    pub fn observe(
        &mut self,
        packet: PacketFields,
        csdw: Option<&QualifiedCsdw>,
        flags: Option<u8>,
        policy: &DeclarationPolicy,
    ) -> Result<DetailedComparison> {
        let mut r = self
            .declarations
            .compare_qualified(packet, csdw, flags, policy)?;
        if matches!(packet.data_type, 0x11 | 0x12) {
            if !r.basic.basis.timein() {
                r.activity.push(activity_finding(
                    8,
                    packet.key,
                    packet.channel,
                    r.basic.basis.clone(),
                    Some(packet.data_type),
                    policy,
                )?);
            } else if let Some(g) = &r.basic.basis.governing {
                self.seen
                    .insert((g.range.session, g.range.start.get(), packet.channel));
            }
        }
        if r.basic.basis.timein() && packet.data_type == 0x11 {
            if let Some(source) = r.tsrc.observed.filter(|v| matches!(v, 0 | 1 | 2 | 15)) {
                let identity = (packet.key.session, packet.channel);
                if let Some((previous, related)) =
                    self.last.get(&identity).filter(|(s, _)| *s != source)
                {
                    let mut f = diagnostic(
                        &packet,
                        &r.basic.basis,
                        &r.tsrc,
                        DeclarationField::Tsrc,
                        DeclarationIssue::SourceChanged,
                        11,
                        policy,
                    )?;
                    let evidence = f
                        .finding
                        .evidence()
                        .clone()
                        .with_related(FindingLocation {
                            session: Some(related.session),
                            packet: Some(related.packet),
                            channel: Some(packet.channel),
                            byte_offset: None,
                        })
                        .with_raw_field(RawField {
                            name: "packet.previous_source",
                            observed: RawValue::Unsigned((*previous).into()),
                            expected: None,
                        });
                    f.finding =
                        Finding::new(f.finding.code(), Some(f.finding.severity()), evidence)?;
                    r.findings.push(f);
                }
                self.last.insert(identity, (source, packet.key));
            }
        }
        Ok(r)
    }
    /// Close the supplied scope; count time packet types, not admitted references.
    /// Empty scopes report every declared TIMEIN range; no fake observed packet.
    pub fn finish(self, policy: &DeclarationPolicy) -> Result<Vec<DeclarationFinding>> {
        let mut findings = Vec::new();
        for setup in self.declarations.snapshots.as_deref().unwrap_or(&[]) {
            for ch in setup
                .channels
                .iter()
                .filter(|c| c.cdt.as_deref() == Some("TIMEIN"))
            {
                let identity = (setup.range.session, setup.range.start.get(), ch.id);
                if !self.seen.contains(&identity) {
                    let key = PacketKey {
                        session: setup.range.session,
                        ticks: setup.range.start.get(),
                        packet: setup.setup_packet,
                    };
                    let basis = self.declarations.basis(key, ch.id)?;
                    findings.push(activity_finding(9, key, ch.id, basis, None, policy)?);
                }
            }
        }
        Ok(findings)
    }
}
