//! Shared reference evidence and explicit TAI arithmetic normalization.
//! Native scale is retained. A Unix coordinate in an NTP insertion window is
//! not a uniquely known physical instant and never supplies a bracket.
extern crate alloc;
use crate::{format1::CalendarReading, format2::*, leap::*, ntp::*, scale::*, timeline::*, *};
use alloc::{boxed::Box, vec::Vec};

/// Native reading normalized explicitly to continuous TAI for arithmetic.
/// Caller-supplied table is an owned snapshot, including coverage/source.
/// **Traces:** L3-F2COR-102 ← L2-F2COR-102 ← L1-F2COR-002
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Normalization {
    tai_ns: i128,
    native_ns: i128,
    scale: ScaleEvidence,
    applied_seconds: i32,
    possibly_out_of_date: bool,
    table: LeapSecondTable,
}
impl Normalization {
    /// Continuous physical coordinate in TAI nanoseconds since1970.
    pub fn tai_nanoseconds(&self) -> i128 {
        self.tai_ns
    }
    /// Original scale and its stated/assumed provenance.
    pub fn native_scale(&self) -> ScaleEvidence {
        self.scale
    }
    /// TAI minus native seconds: GPS19 is RDG-009 inferred; UTC uses the table.
    pub fn applied_seconds(&self) -> i32 {
        self.applied_seconds
    }
    /// Last known UTC offset was used beyond declared table coverage.
    pub fn possibly_out_of_date(&self) -> bool {
        self.possibly_out_of_date
    }
    /// Exact immutable table snapshot; final transition is not currency expiry.
    pub fn table(&self) -> &LeapSecondTable {
        &self.table
    }
    fn new(time: AbsoluteTime, scale: ScaleEvidence, table: LeapSecondTable) -> Result<Self> {
        let year = time.year().ok_or(TimeError::NeedsContext {
            context: "dated scale normalization",
        })?;
        let days = |y: u16| -> i128 {
            (0..y)
                .map(|y| {
                    if crate::util::is_leap_year(y) {
                        366
                    } else {
                        365
                    }
                })
                .sum()
        };
        let native_ns =
            (days(year) - days(1970)) * 86_400_000_000_000 + i128::from(time.as_total_ns());
        let utc = SignedUnixSeconds::new(native_ns.div_euclid(1_000_000_000));
        let (applied_seconds, possibly_out_of_date) = match scale.effective() {
            TimeScale::Tai => (0, false),
            TimeScale::Gps => (19, false),
            TimeScale::Utc => {
                let unsigned = utc.to_unsigned().ok_or(TimeError::NeedsContext {
                    context: "UTC normalization before supported table",
                })?;
                (
                    table.offset_at_unix(unsigned)?.get(),
                    !table.covers_unix(utc),
                )
            }
            TimeScale::Unknown => {
                return Err(TimeError::NeedsContext {
                    context: "unknown scale normalization",
                });
            }
        };
        let tai_ns = native_ns + i128::from(applied_seconds) * 1_000_000_000;
        Ok(Self {
            tai_ns,
            native_ns,
            scale,
            applied_seconds,
            possibly_out_of_date,
            table,
        })
    }
    fn shifted(&self, delta: i128) -> Result<CalendarReading> {
        let shifted = self
            .tai_ns
            .checked_add(delta)
            .ok_or(TimeError::QuantityOverflow {
                quantity: "normalized time",
            })?;
        let mut reading = if self.scale.effective() == TimeScale::Utc {
            let whole = shifted.div_euclid(1_000_000_000);
            let tai =
                TaiSeconds::new(u64::try_from(whole).map_err(|_| TimeError::NeedsContext {
                    context: "TAI outside table range",
                })?);
            let offset = self.table.offset_at_tai(tai)?;
            let unix = tai.to_signed_unix(offset);
            let leap = self.table.entries().windows(2).any(|p| {
                p[1].tai_utc_offset.get() - p[0].tai_utc_offset.get() == 1
                    && whole
                        == i128::from(p[1].effective_unix.get())
                            + i128::from(p[0].tai_utc_offset.get())
            });
            crate::leap::calendar(unix, shifted.rem_euclid(1_000_000_000) as u32, leap)?
        } else {
            let ns = self
                .native_ns
                .checked_add(delta)
                .ok_or(TimeError::QuantityOverflow {
                    quantity: "native time",
                })?;
            crate::leap::calendar(
                SignedUnixSeconds::new(ns.div_euclid(1_000_000_000)),
                ns.rem_euclid(1_000_000_000) as u32,
                false,
            )?
        };
        reading.scale = self.scale;
        Ok(reading)
    }
}
/// Full network evidence retained instead of fabricated Format1 SRC/FMT/ITS.
/// **Traces:** L3-F2COR-101 ← L2-F2COR-101 ← L1-F2COR-001
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkReference {
    reference: Format2Reference,
    context: NtpContext,
    table: LeapSecondTable,
    raw_network_rate: bool,
    normalization_issue: Option<TimeError>,
}
impl NetworkReference {
    /// Conversion, native scale/raw data and actual boundary policy snapshot.
    pub fn reference(&self) -> &Format2Reference {
        &self.reference
    }
    /// Why a unique physical coordinate could not be established, if applicable.
    pub fn normalization_issue(&self) -> Option<&TimeError> {
        self.normalization_issue.as_ref()
    }
    /// Explicit caller declaration of raw NTP/PTP frame-rate recording.
    pub fn raw_network_rate(&self) -> bool {
        self.raw_network_rate
    }
}
impl ReferenceInput {
    /// Select TAI for arithmetic with an explicit table; native answer scale
    /// and policy/stated provenance remain unchanged. Unknown year/scale fails.
    /// **Traces:** L3-F2COR-102 ← L2-F2COR-102 ← L1-F2COR-002
    pub fn normalize_to_tai(mut self, table: LeapSecondTable) -> Result<Self> {
        if self.network.is_some() {
            return Err(TimeError::NeedsContext {
                context: "reconvert network observation to change its normalization table",
            });
        }
        let ReferenceValue::Readable(t, _) = self.value else {
            return Err(TimeError::NeedsContext {
                context: "readable dated scale normalization",
            });
        };
        self.normalization = Some(Normalization::new(t, self.scale, table)?);
        Ok(self)
    }
    /// Convert a bounded network observation; engine admission subsequently
    /// refreshes the boundary with its actual policy. Raw-rate is caller evidence,
    /// never inferred from spacing. Native PTP remains TAI, not relabelled UTC.
    /// **Traces:** L3-F2COR-101 ← L2-F2COR-101 ← L1-F2COR-001
    pub fn format2(
        observation: Format2Observation,
        context: NtpContext,
        table: LeapSecondTable,
        raw_network_rate: bool,
    ) -> Result<Self> {
        let reference = observation.convert(&context, &table, AdmissionPolicy::default())?;
        let (calendar, ambiguous) = match reference.conversion() {
            NetworkConversion::Ntp(c) => (c.utc(), c.basis().leap == NtpLeapStatus::Ambiguous),
            NetworkConversion::Ptp(c) => {
                let mut native = crate::leap::calendar(
                    SignedUnixSeconds::new(c.raw().seconds.into()),
                    c.raw().nanoseconds,
                    false,
                )?;
                native.scale = observation.scale();
                (Some(native), false)
            }
            _ => (None, false),
        };
        let valid = observation.csdw().time_status() == network_time::TimeStatus::Valid;
        let value = if let Some(t) = calendar {
            ReferenceValue::Readable(legacy_calendar(t)?, valid)
        } else if observation.decode_error().is_some() {
            ReferenceValue::Malformed
        } else {
            ReferenceValue::NeedsContext
        };
        let mut input = Self {
            value,
            scale: observation.scale(),
            source: None,
            format: None,
            its: None,
            precision_ns: 1,
            wire: None,
            qualified: None,
            raw_csdw: Some(observation.csdw().as_raw()),
            year_origin: YearOrigin::NetworkEvidence,
            network: Some(Box::new(NetworkReference {
                reference,
                context,
                table: table.clone(),
                raw_network_rate,
                normalization_issue: None,
            })),
            normalization: None,
        };
        let normalized = if ambiguous {
            Err(TimeError::NeedsContext {
                context: "NTP leap ambiguity has no unique physical instant",
            })
        } else if let ReferenceValue::Readable(t, _) = input.value {
            Normalization::new(t, input.scale, table)
        } else {
            Err(TimeError::NeedsContext {
                context: "network calendar unavailable",
            })
        };
        match normalized {
            Ok(n) => input.normalization = Some(n),
            Err(error) => {
                if let Some(n) = input.network.as_mut() {
                    n.normalization_issue = Some(error)
                }
            }
        }
        Ok(input)
    }
}
pub(crate) fn legacy_calendar(t: CalendarReading) -> Result<AbsoluteTime> {
    AbsoluteTime::new(t.day_of_year, t.hours, t.minutes, t.seconds, t.nanoseconds)?
        .with_year(t.year)
}
/// Produce a native-scale calendar using physical arithmetic when qualified.
/// Ambiguous/unresolved network values are retained only at the capture point.
/// **Traces:** L3-F2COR-102 ← L2-F2COR-102 ← L1-F2COR-002
/// **Traces:** L3-ANS-105 ← L2-ANS-105 ← L1-ANS-005
pub(crate) fn shifted_reference(r: &TimelineReference, delta: i128) -> Result<CalendarReading> {
    if let Some(n) = &r.input.normalization {
        return n.shifted(delta);
    }
    if r.input.network.is_some() && delta != 0 {
        return Err(r
            .input
            .network
            .as_ref()
            .and_then(|n| n.normalization_issue.clone())
            .unwrap_or(TimeError::NeedsContext {
                context: "network time has no unique normalized instant",
            }));
    }
    let ReferenceValue::Readable(t, _) = r.input.value else {
        return Err(TimeError::NeedsContext {
            context: "no readable reference",
        });
    };
    let t = crate::timeline::shift(t, delta)?;
    Ok(CalendarReading {
        year: t.year(),
        day_of_year: t.day_of_year(),
        hours: t.hours(),
        minutes: t.minutes(),
        seconds: t.seconds(),
        nanoseconds: t.nanoseconds(),
        scale: r.input.scale,
    })
}
// Validate capture before ingestion state changes. Session checked after the
// counter/reset calculation has established the candidate session identity.
pub(crate) fn check_capture(packet: &PacketObservation) -> Result<()> {
    if let Some(n) = packet.reference.as_ref().and_then(|r| r.network.as_ref()) {
        let o = n.reference.observation();
        let c = o.capture();
        if c.packet != packet.id
            || c.channel != packet.channel
            || c.rtc != packet.rtc
            || o.declared_timein() != packet.declared_timein
        {
            return Err(TimeError::InvalidTimelineInput(
                "network capture/declaration differs from packet",
            ));
        }
    }
    Ok(())
}
pub(crate) fn admit_network(
    mut input: ReferenceInput,
    packet: &PacketObservation,
    key: PacketKey,
    policy: &OfflinePolicy,
) -> Result<(TimelineReference, Vec<TimelineFinding>)> {
    let n = input
        .network
        .as_mut()
        .ok_or(TimeError::InvalidTimelineInput("network evidence missing"))?;
    n.reference = n.reference.observation().convert(
        &n.context,
        &n.table,
        AdmissionPolicy {
            include_invalid: policy.include_invalid,
            include_undeclared: policy.include_undeclared,
            finding_levels: policy.finding_levels.clone(),
        },
    )?;
    let admitted = n.reference.is_eligible();
    let mut reasons = Vec::new();
    if !packet.declared_timein {
        reasons.push(Rejection::Undeclared)
    }
    if n.reference.observation().csdw().time_status() != network_time::TimeStatus::Valid {
        reasons.push(Rejection::Invalid)
    }
    match n.reference.conversion() {
        NetworkConversion::Malformed => reasons.push(Rejection::Malformed),
        NetworkConversion::UnsupportedFormat => reasons.push(Rejection::Reserved),
        NetworkConversion::Ntp(c) if c.status() == NtpConversionStatus::UnknownUnsynchronized => {
            reasons.push(Rejection::Invalid)
        }
        _ if !matches!(input.value, ReferenceValue::Readable(..)) => {
            reasons.push(Rejection::NeedsContext)
        }
        _ => (),
    }
    let mut findings: Vec<TimelineFinding> = n
        .reference
        .findings()
        .iter()
        .map(|f| {
            let evidence = f.evidence().clone();
            TimelineFinding {
                code: f.code().number(),
                severity: f.severity(),
                session: key.session,
                packet: key.packet,
                channel: Some(packet.channel),
                related: None,
                raw_rtc: evidence.raw_rtc(),
                delta_ns: None,
                evidence,
            }
        })
        .collect();
    if let Some(f) = normalization_warning(&input, key, packet.channel, policy)? {
        findings.push(f)
    }
    Ok((
        TimelineReference {
            key,
            channel: packet.channel,
            declared_timein: packet.declared_timein,
            declared_external: packet.declared_external,
            input,
            admitted,
            reasons,
        },
        findings,
    ))
}
/// Qualify a policy gap: the standard's 1 Hz requirement excludes explicitly
/// raw-network-rate Format2 frames. A configured policy gap remains TF-016.
/// **Traces:** L3-COR-108 ← L2-COR-108 ← L1-COR-008
pub(crate) fn qualify_gap(f: &mut TimelineFinding, a: &TimelineReference, b: &TimelineReference) {
    let raw = a.input.network.as_ref().is_some_and(|n| n.raw_network_rate)
        && b.input.network.as_ref().is_some_and(|n| n.raw_network_rate);
    f.evidence = f.evidence.clone().with_raw_field(findings::RawField {
        name: "standard_minimum_1hz_applies",
        observed: findings::RawValue::Unsigned(u128::from(!raw)),
        expected: None,
    });
}

// Conversions using a last-known UTC offset retain the value and actual operand.
// **Traces:** L3-TAI-104 ← L2-TAI-104 ← L1-TAI-004
pub(crate) fn normalization_warning(
    input: &ReferenceInput,
    key: PacketKey,
    channel: u16,
    policy: &OfflinePolicy,
) -> Result<Option<TimelineFinding>> {
    let Some(n) = input
        .normalization
        .as_ref()
        .filter(|n| n.possibly_out_of_date)
    else {
        return Ok(None);
    };
    let mut f = TimelineFinding::observed(
        21,
        policy.finding_levels.get(&21).copied(),
        key,
        Some(channel),
        None,
        None,
    )?;
    f.evidence =
        findings::FindingEvidence::new(f.evidence.location(), findings::ValueValidity::Kept)
            .with_raw_rtc(Rtc::from_raw((key.ticks & ((1i128 << 48) - 1)) as u64))
            .with_raw_field(findings::RawField {
                name: "tai_minus_native_seconds",
                observed: findings::RawValue::Signed(n.applied_seconds.into()),
                expected: None,
            })
            .with_raw_field(findings::RawField {
                name: "known_current_through_unix_seconds",
                observed: findings::RawValue::Unsigned(
                    n.table.metadata().known_current_through.get().into(),
                ),
                expected: None,
            });
    Ok(Some(f))
}
