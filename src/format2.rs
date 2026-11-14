//! Format-neutral capture metadata and the Format 2 conversion/admission boundary.
//! No timeline insertion, channel selection or rate evaluation occurs here.
//! 106-24R1 §11.2.3.3 / Figs11-15..17, printed11-26/27 (PDF34/35).
extern crate alloc;
use crate::edition::PacketEdition;
use crate::findings::*;
use crate::leap::{LeapSecondTable, PtpConversion};
use crate::network_time::*;
use crate::ntp::{NtpContext, NtpConversion, NtpConversionStatus, convert_ntp};
use crate::scale::{ScaleEvidence, TimeScaleTagged};
use crate::{Result, Rtc, TimeError};
use alloc::{collections::BTreeMap, vec::Vec};

/// Recording identity and header capture counter, independent of time format.
/// **Traces:** L3-F2CSDW-101 ← L2-F2CSDW-101 ← L1-F2CSDW-001
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReferenceCapture {
    /// Nonzero caller session identity; never inferred here.
    pub session: usize,
    /// Caller packet identity, not a byte offset.
    pub packet: u64,
    /// Actual packet channel.
    pub channel: u16,
    /// Header RTC at capture; not a converted timestamp.
    pub rtc: Rtc,
    /// Optional supplied file position.
    pub byte_offset: Option<u64>,
}
impl ReferenceCapture {
    fn location(self) -> FindingLocation {
        FindingLocation {
            session: Some(self.session),
            packet: Some(self.packet),
            channel: Some(self.channel),
            byte_offset: self.byte_offset,
        }
    }
}
/// Caller inputs for one Data-Length-bounded observation.
#[derive(Debug, Clone, Copy)]
pub struct Format2Input<'a> {
    /// CSDW and body; may include adjacent filler outside Data Length.
    pub payload: &'a [u8],
    /// Header Data Length, excluding trailer/filler.
    pub data_length: usize,
    /// Header capture identity, shared by raw reading and findings.
    pub capture: ReferenceCapture,
    /// Explicit declaration only; a header version never selects an edition.
    pub declared_edition: Option<PacketEdition>,
    /// Raw header evidence, not a declaration of the recording's edition.
    pub data_type_version: u8,
    /// Caller-supplied governing TIMEIN declaration, not inferred from packets.
    pub declared_timein: bool,
}
/// Immutable raw observation. Malformed bodies retain independent CSDW evidence.
/// No Format 1 SRC/FMT/ITS fields are invented for a network reading.
/// **Traces:** L3-F2CSDW-101 ← L2-F2CSDW-101 ← L1-F2CSDW-001
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Format2Observation {
    capture: ReferenceCapture,
    declared_edition: Option<PacketEdition>,
    data_type_version: u8,
    declared_timein: bool,
    csdw: TimeF2Csdw,
    raw_body: [u8; 8],
    reading: Option<Format2Reading>,
    decode_error: Option<TimeError>,
}
impl Format2Input<'_> {
    /// Read at most twelve bytes within Data Length. Underlength is an error;
    /// a full malformed PTP body is retained for TF-004 beside TS/NTF findings.
    /// **Traces:** L3-ERR-105 ← L2-ERR-105 ← L1-ERR-005
    /// **Traces:** L3-F2CSDW-101 ← L2-F2CSDW-101 ← L1-F2CSDW-001
    pub fn decode(self) -> Result<Format2Observation> {
        if self.capture.session == 0 {
            return Err(TimeError::InvalidTimelineInput("session zero"));
        }
        let bounded = self
            .payload
            .get(..self.data_length)
            .ok_or(TimeError::BufferTooShort {
                expected: self.data_length,
                actual: self.payload.len(),
            })?;
        if bounded.len() < 12 {
            return Err(TimeError::BufferTooShort {
                expected: 12,
                actual: bounded.len(),
            });
        }
        let csdw = TimeF2Csdw::from_le_bytes([bounded[0], bounded[1], bounded[2], bounded[3]]);
        let mut raw_body = [0; 8];
        raw_body.copy_from_slice(&bounded[4..12]);
        let (reading, decode_error) = match decode_time_f2(bounded, 12) {
            Ok(r) => (Some(r), None),
            Err(e) => (None, Some(e)),
        };
        Ok(Format2Observation {
            capture: self.capture,
            declared_edition: self.declared_edition,
            data_type_version: self.data_type_version,
            declared_timein: self.declared_timein,
            csdw,
            raw_body,
            reading,
            decode_error,
        })
    }
}
/// Explicit admission switches, independent of status and finding severities.
/// Defaults exclude invalid/undeclared references, as contract5.2 requires.
/// **Traces:** L3-F2CSDW-105 ← L2-F2CSDW-105 ← L1-F2CSDW-005
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AdmissionPolicy {
    /// Allow readable invalid/reserved TS; preserve the status and label the use.
    pub include_invalid: bool,
    /// Allow a readable channel not declared TIMEIN; retain TF-008.
    pub include_undeclared: bool,
    /// Overrides by permanent TF number; pending/unknown levels are rejected.
    pub finding_levels: BTreeMap<u8, Severity>,
}
/// Admission result, separate from the recorded status and conversion basis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetworkAdmission {
    /// Eligible as native time under this policy, not yet inserted in an engine.
    pub eligible: bool,
    /// An eligible answer would use an invalid status by explicit opt-in.
    pub invalid: bool,
    /// An eligible answer would use an undeclared channel by explicit opt-in.
    pub undeclared: bool,
}
/// Native-format conversion, preserving full S04/S05 basis and raw values.
/// Missing UTC for PTP does not remove its readable TAI value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkConversion {
    /// Stated UTC with era, floor rounding and leap-window/table evidence.
    Ntp(NtpConversion),
    /// Stated native TAI and a separate table-qualified UTC projection.
    Ptp(PtpConversion),
    /// Reserved NTF: raw body kept, no decoded time, never policy-admitted.
    UnsupportedFormat,
    /// Known NTF with a malformed body; never policy-admitted.
    Malformed,
}
/// Qualified reference boundary; neither public construction nor mutation can
/// detach findings/admission from the raw word, captured RTC or policy snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Format2Reference {
    observation: Format2Observation,
    conversion: NetworkConversion,
    admission: NetworkAdmission,
    policy: AdmissionPolicy,
    findings: Vec<Finding>,
}
impl Format2Observation {
    /// Header capture point, independent of protocol epoch/scale.
    pub fn capture(&self) -> ReferenceCapture {
        self.capture
    }
    /// Exact original data word, including TS, NTF and zero-mask intersections.
    pub fn csdw(&self) -> TimeF2Csdw {
        self.csdw
    }
    /// Eight body bytes only; never filler or a synthetic Format 1 body.
    pub fn raw_body(&self) -> &[u8; 8] {
        &self.raw_body
    }
    /// Bytes consumed, excluding adjacent filler/checksum, including malformed bodies.
    pub fn consumed(&self) -> usize {
        12
    }
    /// Supported parsed body even if status is invalid; None for malformed/NTF.
    pub fn reading(&self) -> Option<Format2Reading> {
        self.reading
    }
    /// Typed body parser failure, independent of CSDW status/reserved findings.
    pub fn decode_error(&self) -> Option<&TimeError> {
        self.decode_error.as_ref()
    }
    /// Actual explicit edition; None stays unknown regardless of header version.
    pub fn declared_edition(&self) -> Option<PacketEdition> {
        self.declared_edition
    }
    /// Raw data type version evidence, not a layout inference.
    pub fn data_type_version(&self) -> u8 {
        self.data_type_version
    }
    /// Governing channel declaration supplied by the caller.
    pub fn declared_timein(&self) -> bool {
        self.declared_timein
    }
    /// Native scale stated by the standard; reserved NTF remains unknown.
    /// No caller scale override relabels a stated NTP/PTP scale.
    /// **Traces:** L3-TAI-105 ← L2-TAI-105 ← L1-TAI-005
    /// **Traces:** L3-TAI-106 ← L2-TAI-106 ← L1-TAI-006
    /// **Traces:** L3-PTP-102 ← L2-PTP-102 ← L1-PTP-002
    pub fn scale(&self) -> ScaleEvidence {
        match self.csdw.time_protocol() {
            NetworkTimeProtocol::Ntp => ScaleEvidence::standard(crate::scale::TimeScale::Utc),
            NetworkTimeProtocol::Ptp => ScaleEvidence::standard(crate::scale::TimeScale::Tai),
            NetworkTimeProtocol::Reserved(_) => ScaleEvidence::unknown(),
        }
    }
    /// Convert independently of admission; readable invalid data stays visible.
    /// PTP keeps native TAI when there is no applicable UTC offset. NTP sentinel
    /// or unresolved era cannot be admitted. No offset is added to NTP UTC.
    /// Raw-network spacing/TF-016 evaluation belongs to S06b, not this boundary.
    /// **Traces:** L3-COR-106 ← L2-COR-106 ← L1-COR-006
    /// **Traces:** L3-FND-101 ← L2-FND-101 ← L1-FND-001
    /// **Traces:** L3-F2CSDW-104 ← L2-F2CSDW-104 ← L1-F2CSDW-004
    /// **Traces:** L3-F2CSDW-105 ← L2-F2CSDW-105 ← L1-F2CSDW-005
    /// **Traces:** L3-F2CSDW-106 ← L2-F2CSDW-106 ← L1-F2CSDW-006
    /// **Traces:** L3-EDN-101 ← L2-EDN-101 ← L1-EDN-001
    pub fn convert(
        &self,
        context: &NtpContext,
        table: &LeapSecondTable,
        policy: AdmissionPolicy,
    ) -> Result<Format2Reference> {
        for (id, level) in &policy.finding_levels {
            FindingCode::from_number(*id)?.resolve_level(Some(*level))?;
        }
        let conversion = match self.reading.and_then(|r| r.time) {
            Some(t) => match t.value {
                NetworkTime::Ntp(raw) => {
                    if context.session != self.capture.session {
                        return Err(TimeError::InvalidTimelineInput(
                            "NTP context session differs from capture",
                        ));
                    }
                    NetworkConversion::Ntp(convert_ntp(raw, context, Some(table)))
                }
                NetworkTime::Ptp(raw) => NetworkConversion::Ptp(table.convert_ptp(raw)?),
            },
            None if self.decode_error.is_some() => NetworkConversion::Malformed,
            None => NetworkConversion::UnsupportedFormat,
        };
        let native = match &conversion {
            NetworkConversion::Ntp(c) => c.status() == NtpConversionStatus::Mapped,
            NetworkConversion::Ptp(_) => true,
            _ => false,
        };
        let invalid = self.csdw.time_status() != TimeStatus::Valid;
        let undeclared = !self.declared_timein;
        let eligible = native
            && (!invalid || policy.include_invalid)
            && (!undeclared || policy.include_undeclared);
        let admission = NetworkAdmission {
            eligible,
            invalid: eligible && invalid,
            undeclared: eligible && undeclared,
        };
        let mut findings = Vec::new();
        let validity = if native {
            ValueValidity::Kept
        } else {
            ValueValidity::Unusable
        };
        let mut report = |id, name, observed, effect, cause: Option<TimeError>| -> Result<()> {
            let mut evidence = FindingEvidence::new(self.capture.location(), effect)
                .with_raw_rtc(self.capture.rtc)
                .with_raw_field(RawField {
                    name,
                    observed: RawValue::Unsigned(observed),
                    expected: None,
                })
                .with_raw_field(RawField {
                    name: "csdw",
                    observed: RawValue::Unsigned(self.csdw.as_raw().into()),
                    expected: None,
                });
            if let Some(error) = cause {
                evidence = evidence.with_decode_error(error);
            }
            findings.push(Finding::new(
                FindingCode::from_number(id)?,
                policy.finding_levels.get(&id).copied(),
                evidence,
            )?);
            Ok(())
        };
        match self.csdw.time_status() {
            TimeStatus::NotValid => report(3, "ts", 0, validity, None)?,
            TimeStatus::Reserved(n) => report(6, "ts", n.into(), validity, None)?,
            _ => (),
        }
        if let NetworkTimeFormat::Reserved(n) = self.csdw.time_format() {
            report(6, "ntf", n.into(), ValueValidity::Unusable, None)?;
        }
        if self.csdw.reserved_bits() != 0 {
            report(
                5,
                "reserved_csdw_bits",
                self.csdw.reserved_bits().into(),
                validity,
                None,
            )?;
        }
        if let Some(error) = &self.decode_error {
            report(
                4,
                "body_bytes",
                8,
                ValueValidity::Unusable,
                Some(error.clone()),
            )?;
        }
        if self.declared_edition.is_some_and(|e| !e.rules().format2) {
            report(26, "data_type", 0x12, validity, None)?;
        }
        if undeclared {
            report(8, "channel", self.capture.channel.into(), validity, None)?;
        }
        for f in &mut findings {
            if f.code().number() == 26 {
                let evidence = f.evidence().clone().with_raw_field(RawField {
                    name: "data_type_version",
                    observed: RawValue::Unsigned(self.data_type_version.into()),
                    expected: None,
                });
                *f = Finding::new(f.code(), Some(f.severity()), evidence)?;
            }
        }
        let converted = match &conversion {
            NetworkConversion::Ntp(c) => c.findings(self.capture.location(), None)?,
            NetworkConversion::Ptp(c) => c.findings(self.capture.location(), None)?,
            _ => Vec::new(),
        };
        for f in converted {
            let id = f.code();
            let evidence = f.into_evidence().with_raw_rtc(self.capture.rtc);
            findings.push(Finding::new(
                id,
                policy.finding_levels.get(&id.number()).copied(),
                evidence,
            )?);
        }
        Ok(Format2Reference {
            observation: self.clone(),
            conversion,
            admission,
            policy,
            findings,
        })
    }
}
impl TimeScaleTagged for Format2Observation {
    fn scale(&self) -> ScaleEvidence {
        self.scale()
    }
}
impl Format2Reference {
    /// Original observation; no source/format labels invented by conversion.
    pub fn observation(&self) -> &Format2Observation {
        &self.observation
    }
    /// Full protocol-specific conversion result including raw/basis/currency.
    pub fn conversion(&self) -> &NetworkConversion {
        &self.conversion
    }
    /// Policy-qualified native reference eligibility, independent of severity.
    pub fn is_eligible(&self) -> bool {
        self.admission.eligible
    }
    /// Explicit opt-in reasons; false labels on every rejected reference.
    pub fn admission(&self) -> NetworkAdmission {
        self.admission
    }
    /// Actual policy snapshot, not a reference to mutable caller settings.
    pub fn policy(&self) -> &AdmissionPolicy {
        &self.policy
    }
    /// All independent status/reserved/edition/body/conversion observations.
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }
}
