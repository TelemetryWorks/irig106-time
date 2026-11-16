//! S10a1 validation/evidence. Ch11 §11.2.1.1g (11-9/PDF17),
//! §11.2.1.2/Fig11-4/5/6 (11-10, PDF18); RDG008 (suspect).
//! Epoch/calendar conversion and own-minus-counter/TF014 remain S10a2.
use crate::{
    Result, Rtc, SignedRtcTicks, TimeError,
    findings::*,
    rtc::{CounterOrder, SessionRtc},
    scale::ScaleEvidence,
    secondary::*,
    timeline::PacketKey,
};
use alloc::{collections::BTreeMap, string::String, vec::Vec};
/// What data instant the own time describes; exceptions are caller-cited facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeApplicability {
    /// §11.2.1.2a default: first data bit of the packet body.
    FirstDataBit,
    /// Explicit caller-supplied data-type exception; the citation is not verified here.
    CitedException {
        /// Data type to which this exception applies.
        data_type: u8,
        /// Nonempty source, retained verbatim; never an invented standard claim.
        citation: String,
    },
}
/// Primary facts and bounded secondary bytes, supplied by the packet reader.
#[derive(Debug, Clone)]
pub struct SecondaryPacket<'a> {
    /// Caller-resolved session, unwrapped counter and packet identity.
    pub key: PacketKey,
    /// Header channel ID.
    pub channel: u16,
    /// Header data type, for applicability validation only.
    pub data_type: u8,
    /// Independent primary counter, never reconstructed from own time.
    pub rtc: Rtc,
    /// Original primary flags; only presence/format are handled by this layer.
    pub flags: u8,
    /// Secondary header prefix (may include trailing body bytes, ignored).
    pub bytes: &'a [u8],
    /// Absolute byte offset of the secondary header if known.
    pub byte_offset: Option<u64>,
    /// First data bit or an explicit cited data-type exception.
    pub applicability: TimeApplicability,
}
impl SecondaryPacket<'_> {
    fn location(&self, relative: Option<u64>) -> FindingLocation {
        FindingLocation {
            session: Some(self.key.session),
            packet: Some(self.key.packet),
            channel: Some(self.channel),
            byte_offset: relative.and_then(|n| self.byte_offset.and_then(|b| b.checked_add(n))),
        }
    }
    fn evidence(&self, validity: ValueValidity, offset: Option<u64>) -> FindingEvidence {
        FindingEvidence::new(self.location(offset), validity)
            .with_raw_rtc(self.rtc)
            .with_raw_field(RawField {
                name: "packet.rtc_ticks_unwrapped",
                observed: RawValue::Signed(self.key.ticks),
                expected: None,
            })
    }
    fn validate(&self) -> Result<()> {
        if self.key.session == 0 {
            return Err(TimeError::InvalidTimelineInput("secondary session is zero"));
        }
        if self
            .byte_offset
            .is_some_and(|o| o.checked_add(11).is_none())
        {
            return Err(TimeError::QuantityOverflow {
                quantity: "secondary byte offset",
            });
        }
        if let TimeApplicability::CitedException {
            data_type,
            citation,
        } = &self.applicability
        {
            if *data_type != self.data_type || citation.trim().is_empty() {
                return Err(TimeError::InvalidTimelineInput(
                    "secondary applicability requires matching type and citation",
                ));
            }
        }
        Ok(())
    }
}
/// Default finding levels are the owner-approved catalog; overrides change no facts.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SecondaryPolicy {
    /// Stable TF-number severity overrides.
    pub levels: BTreeMap<u8, Severity>,
}
fn finding(
    out: &mut SecondaryInspection,
    policy: &SecondaryPolicy,
    code: u8,
    evidence: FindingEvidence,
) -> Result<()> {
    out.findings.push(Finding::new(
        FindingCode::from_number(code)?,
        policy.levels.get(&code).copied(),
        evidence,
    )?);
    Ok(())
}
/// Same-packet phase check, not an absolute ERTC epoch/correlation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtcErtcCheck {
    expected: Rtc,
    remainder: u8,
    delta: Result<SignedRtcTicks>,
    distance: u64,
}
impl RtcErtcCheck {
    /// floor(ERTC/100) modulo2^48.
    pub fn expected_rtc(&self) -> Rtc {
        self.expected
    }
    /// Sub-tick remainder,0..99 nanoseconds.
    pub fn remainder_ns(&self) -> crate::NanosDuration {
        crate::NanosDuration::new(u64::from(self.remainder))
    }
    /// Observed-minus-expected nearest signed phase; half-range is ambiguous.
    pub fn delta_ticks(&self) -> Result<SignedRtcTicks> {
        self.delta.clone()
    }
    /// Minimal unsigned phase distance; known even when its sign is ambiguous.
    pub fn phase_distance_ticks(&self) -> SignedRtcTicks {
        SignedRtcTicks::new(i128::from(self.distance))
    }
}
/// Immutable raw/trust/evidence result. No absolute calendar or scale is guessed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecondaryInspection {
    data_type: u8,
    byte_offset: Option<u64>,
    key: PacketKey,
    channel: u16,
    flags: u8,
    applicability: TimeApplicability,
    raw: Vec<u8>,
    format: Option<SecHdrTimeFormat>,
    checksum: Option<SecondaryChecksum>,
    time: Option<SecondaryHeaderTime>,
    consistency: Option<RtcErtcCheck>,
    errors: Vec<TimeError>,
    findings: Vec<Finding>,
}
impl SecondaryInspection {
    /// Original packet data type, including for explicit applicability citations.
    pub fn data_type(&self) -> u8 {
        self.data_type
    }
    /// Original secondary-header byte offset, even when no finding was raised.
    pub fn byte_offset(&self) -> Option<u64> {
        self.byte_offset
    }
    /// Original packet identity/session/counter coordinate.
    pub fn key(&self) -> PacketKey {
        self.key
    }
    /// Original packet channel.
    pub fn channel(&self) -> u16 {
        self.channel
    }
    /// Original packet flags.
    pub fn flags(&self) -> u8 {
        self.flags
    }
    /// Original applicability metadata.
    pub fn applicability(&self) -> &TimeApplicability {
        &self.applicability
    }
    /// At most12 original bytes; absence consumes none, trailing body is excluded.
    pub fn raw_bytes(&self) -> &[u8] {
        &self.raw
    }
    /// Actual enabled format, never inherited from another channel or absence.
    pub fn format(&self) -> Option<SecHdrTimeFormat> {
        self.format
    }
    /// Stored/byte/word readings; None if fewer than12 bytes.
    pub fn checksum(&self) -> Option<SecondaryChecksum> {
        self.checksum
    }
    /// Raw own time only after byte-checksum and field validation.
    pub fn time(&self) -> Option<SecondaryHeaderTime> {
        self.time
    }
    /// Named-session same-packet ERTC comparison, separate from calendar context.
    pub fn rtc_ertc(&self) -> Option<&RtcErtcCheck> {
        self.consistency.as_ref()
    }
    /// Raw own formats establish no epoch/scale by themselves.
    pub fn scale(&self) -> ScaleEvidence {
        ScaleEvidence::unknown()
    }
    /// Own-time errors, independent of the caller's counter answer.
    pub fn errors(&self) -> &[TimeError] {
        &self.errors
    }
    /// Structured findings with raw operands, exact locations and configured levels.
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }
}
/// Validate own time without using it to repair/replace the primary RTC.
/// Reserved warnings are independent of checksum/field failures; byte trust
/// never falls back to word trust. Primary flag/stamp diagnostics remain separate.
/// **Traces:** L3-SEC-101 ← L2-SEC-101 ← L1-SEC-001
/// **Traces:** L3-SEC-105 ← L2-SEC-105 ← L1-SEC-005
/// **Traces:** L3-SEC-104 ← L2-SEC-104 ← L1-SEC-004
/// **Traces:** L3-ERR-104 ← L2-ERR-104 ← L1-ERR-004
/// **Traces:** L3-RDG-208 ← L2-RDG-208 ← L1-RDG-002
/// **Traces:** L3-FND-105 ← L2-FND-105 ← L1-FND-005
pub fn inspect_secondary(
    packet: SecondaryPacket<'_>,
    policy: &SecondaryPolicy,
) -> Result<SecondaryInspection> {
    packet.validate()?;
    let mut out = SecondaryInspection {
        data_type: packet.data_type,
        byte_offset: packet.byte_offset,
        key: packet.key,
        channel: packet.channel,
        flags: packet.flags,
        applicability: packet.applicability.clone(),
        raw: Vec::new(),
        format: None,
        checksum: None,
        time: None,
        consistency: None,
        errors: Vec::new(),
        findings: Vec::new(),
    };
    if packet.flags & 0x80 == 0 {
        return Ok(out);
    }
    let format = SecHdrTimeFormat::from_packet_flags(packet.flags);
    out.format = Some(format);
    out.raw.extend(packet.bytes.iter().take(12).copied());
    if matches!(format, SecHdrTimeFormat::Reserved(_)) {
        let e = TimeError::UndefinedSecondaryTimeFormat {
            flags: packet.flags,
        };
        for (code, validity) in [(6, ValueValidity::Kept), (28, ValueValidity::Unusable)] {
            let error = if code == 6 {
                TimeError::ReservedPacketTimeFormat {
                    flags: packet.flags,
                }
            } else {
                e.clone()
            };
            finding(
                &mut out,
                policy,
                code,
                packet
                    .evidence(validity, None)
                    .with_raw_field(RawField {
                        name: "packet.flags",
                        observed: RawValue::Unsigned(packet.flags.into()),
                        expected: None,
                    })
                    .with_decode_error(error),
            )?;
        }
        out.errors.push(e);
    }
    for (offset, applicable, name) in [
        (0, format == SecHdrTimeFormat::Ch4, "secondary.ch4_reserved"),
        (8, true, "secondary.header_reserved"),
    ] {
        if applicable {
            if let Some(bytes) = packet.bytes.get(offset..offset + 2) {
                let bits = u16::from_le_bytes([bytes[0], bytes[1]]);
                if bits != 0 {
                    finding(
                        &mut out,
                        policy,
                        5,
                        packet
                            .evidence(ValueValidity::Kept, Some(offset as u64))
                            .with_raw_field(RawField {
                                name,
                                observed: RawValue::Unsigned(bits.into()),
                                expected: Some(RawValue::Unsigned(0)),
                            }),
                    )?;
                }
            }
        }
    }
    if packet.bytes.len() < 12 {
        let e = TimeError::BufferTooShort {
            expected: 12,
            actual: packet.bytes.len(),
        };
        finding(
            &mut out,
            policy,
            4,
            packet
                .evidence(ValueValidity::Unusable, None)
                .with_decode_error(e.clone()),
        )?;
        out.errors.push(e);
        return Ok(out);
    }
    let c = inspect_secondary_checksum(packet.bytes)?;
    out.checksum = Some(c);
    if !c.byte_valid() {
        let e = TimeError::ChecksumMismatch {
            stored: c.stored,
            computed: c.byte_sum,
        };
        let evidence = packet
            .evidence(ValueValidity::Unusable, Some(10))
            .with_decode_error(e.clone())
            .with_raw_field(RawField {
                name: "secondary.checksum.stored",
                observed: RawValue::Unsigned(c.stored.into()),
                expected: Some(RawValue::Unsigned(c.byte_sum.into())),
            })
            .with_raw_field(RawField {
                name: "secondary.checksum.byte_sum",
                observed: RawValue::Unsigned(c.byte_sum.into()),
                expected: None,
            })
            .with_raw_field(RawField {
                name: "secondary.checksum.word_sum",
                observed: RawValue::Unsigned(c.word_sum.into()),
                expected: None,
            });
        finding(&mut out, policy, 12, evidence.clone())?;
        if c.word_only() {
            finding(&mut out, policy, 27, evidence)?;
        }
        out.errors.push(e);
        return Ok(out);
    }
    if !matches!(format, SecHdrTimeFormat::Reserved(_)) {
        match parse_secondary_header(packet.bytes, format) {
            Ok(time) => out.time = Some(time),
            Err(e) => {
                finding(
                    &mut out,
                    policy,
                    4,
                    packet
                        .evidence(ValueValidity::Unusable, None)
                        .with_decode_error(e.clone()),
                )?;
                out.errors.push(e);
            }
        }
    }
    if let Some(SecondaryHeaderTime::Ertc(e)) = out.time {
        let ns = e.to_nanos(); // Shared ERTC:1ns/tick, full64-bit raw range.
        // The mask proves this conversion fits u64; remainder is bounded by99.
        let expected = Rtc::from_raw(((ns / 100) & ((1u128 << 48) - 1)) as u64);
        let forward = expected.elapsed_ticks(packet.rtc);
        let distance = forward.min((1u64 << 48) - forward);
        let delta = SessionRtc {
            session: packet.key.session,
            rtc: expected,
        }
        .delta_to(
            SessionRtc {
                session: packet.key.session,
                rtc: packet.rtc,
            },
            CounterOrder::Nearest,
        );
        if distance > 1 {
            let mut evidence = packet
                .evidence(ValueValidity::Kept, None)
                .with_raw_field(RawField {
                    name: "ertc.nanoseconds",
                    observed: RawValue::Unsigned(ns),
                    expected: None,
                })
                .with_raw_field(RawField {
                    name: "rtc_ertc.rtc_ticks",
                    observed: RawValue::Unsigned(packet.rtc.as_raw().into()),
                    expected: Some(RawValue::Unsigned(expected.as_raw().into())),
                })
                .with_raw_field(RawField {
                    name: "rtc_ertc.phase_distance_ticks",
                    observed: RawValue::Unsigned(distance.into()),
                    expected: None,
                });
            match &delta {
                Ok(d) => {
                    evidence = evidence.with_raw_field(RawField {
                        name: "rtc_ertc.delta_ticks",
                        observed: RawValue::Signed(d.get()),
                        expected: None,
                    });
                }
                Err(e) => {
                    evidence = evidence.with_decode_error(e.clone());
                }
            }
            finding(&mut out, policy, 25, evidence)?;
        }
        out.consistency = Some(RtcErtcCheck {
            expected,
            remainder: (ns % 100) as u8,
            delta,
            distance,
        });
    }
    Ok(out)
}
/// Borrowed independent answer plus own-time evidence; no compatibility assumed.
/// The caller supplies the same packet's counter answer. Borrowing guarantees
/// header errors cannot consume it, including caller-metadata validation errors.
#[derive(Debug)]
pub struct SecondaryWithCounter<'a, C> {
    /// Exact caller-owned counter answer (offline, stream, or another adapter).
    pub counter: &'a C,
    /// Independent own-time validation/partial raw result.
    pub header: SecondaryInspection,
}
/// Keep the original counter answer available without attempting an own conversion.
/// **Traces:** L3-ERR-104 ← L2-ERR-104 ← L1-ERR-004
pub fn with_counter<'a, C>(
    counter: &'a C,
    packet: SecondaryPacket<'_>,
    policy: &SecondaryPolicy,
) -> Result<SecondaryWithCounter<'a, C>> {
    Ok(SecondaryWithCounter {
        counter,
        header: inspect_secondary(packet, policy)?,
    })
}
/// Mutually exclusive checksum classifications, never counted as interpretation settlement.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ChecksumCounts {
    /// Only the standard-wording byte checksum matches.
    pub byte_only: u64,
    /// Only the alternative word checksum matches.
    pub word_only: u64,
    /// Both match.
    pub both: u64,
    /// Neither matches.
    pub neither: u64,
    /// Enabled header shorter than12 bytes.
    pub short: u64,
}
/// One-recording format/checksum audit, bounded by u16 channel IDs, no packet queue.
#[derive(Debug, Default)]
pub struct SecondaryAudit {
    counts: ChecksumCounts,
    first: Option<(SecHdrTimeFormat, FindingLocation)>,
    channels: BTreeMap<u16, (SecHdrTimeFormat, FindingLocation)>,
}
impl SecondaryAudit {
    /// Start a new recording scope; session resets do not change the format rule.
    pub fn new() -> Self {
        Self::default()
    }
    /// Snapshot checksum classifications; absence is not format00 or a header.
    pub fn counts(&self) -> ChecksumCounts {
        self.counts
    }
    /// Decode each packet by its own flags, then report recording/channel differences.
    /// **Traces:** L3-SEC-106 ← L2-SEC-106 ← L1-SEC-006
    pub fn observe(
        &mut self,
        packet: SecondaryPacket<'_>,
        policy: &SecondaryPolicy,
    ) -> Result<SecondaryInspection> {
        let mut out = inspect_secondary(packet.clone(), policy)?;
        if let Some(format) = out.format {
            let counter = match out.checksum {
                None => &mut self.counts.short,
                Some(c) => match (c.byte_valid(), c.word_valid()) {
                    (true, false) => &mut self.counts.byte_only,
                    (false, true) => &mut self.counts.word_only,
                    (true, true) => &mut self.counts.both,
                    (false, false) => &mut self.counts.neither,
                },
            };
            *counter = counter.checked_add(1).ok_or(TimeError::QuantityOverflow {
                quantity: "secondary checksum count",
            })?;
            if !matches!(format, SecHdrTimeFormat::Reserved(_)) {
                let previous = self.channels.get(&packet.channel).copied();
                if let Some((canonical, related)) = self.first {
                    if canonical != format || previous.is_some_and(|(f, _)| f != format) {
                        let mut evidence = packet
                            .evidence(ValueValidity::Kept, None)
                            .with_related(related)
                            .with_raw_field(RawField {
                                name: "secondary.format",
                                observed: RawValue::Unsigned(u128::from((packet.flags >> 2) & 3)),
                                expected: Some(RawValue::Unsigned(format_number(canonical))),
                            });
                        if let Some((old, location)) = previous {
                            evidence = evidence.with_raw_field(RawField {
                                name: "secondary.previous_channel_format",
                                observed: RawValue::Unsigned(format_number(old)),
                                expected: None,
                            });
                            if location != related {
                                evidence = evidence.with_related(location);
                            }
                        }
                        finding(&mut out, policy, 13, evidence)?;
                    }
                } else {
                    self.first = Some((format, packet.location(None)));
                }
                self.channels
                    .insert(packet.channel, (format, packet.location(None)));
            }
        }
        Ok(out)
    }
}
fn format_number(format: SecHdrTimeFormat) -> u128 {
    match format {
        SecHdrTimeFormat::Ch4 => 0,
        SecHdrTimeFormat::Ieee1588 => 1,
        SecHdrTimeFormat::Ertc => 2,
        SecHdrTimeFormat::Reserved(v) => v.into(),
    }
}
