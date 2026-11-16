//! Checked policy wire DTOs. No live engine state or source defaults are loaded.
use super::*;
use crate::{
    declarations::*, ntp::*, own_time::*, policy::*, secondary_packet::TimeApplicability,
    stream::*, timeline::*,
};
use alloc::{boxed::Box, vec::Vec};

/// Complete clock configuration, including all shared qualifier/table settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyValue {
    /// Offline query/session/admission settings.
    Offline(OfflinePolicy),
    /// Stream settings, caller caps and exact embedded offline/shared settings.
    Stream(StreamPolicy),
}
/// Versioned policy snapshot. Loading calls every native configuration validator.
/// Defaults are materialized; no field is silently filled from a newer library.
/// **Traces:** L3-POL-103 ← L2-POL-103 ← L1-POL-003
/// **Traces:** L3-API-110 ← L2-API-110 ← L1-API-010
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicySnapshot(PolicyValue);
impl PolicySnapshot {
    /// Validate and retain a complete offline policy.
    pub fn offline(value: OfflinePolicy) -> Result<Self> {
        value.validate()?;
        Ok(Self(PolicyValue::Offline(value)))
    }
    /// Validate and retain a complete stream policy.
    pub fn stream(value: StreamPolicy) -> Result<Self> {
        value.validate()?;
        Ok(Self(PolicyValue::Stream(value)))
    }
    /// Read-only exact settings, including the prior snapshot identity.
    pub fn value(&self) -> &PolicyValue {
        &self.0
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PolicyEnvelope {
    schema: String,
    version: u16,
    defaults_version: u16,
    #[serde(deserialize_with = "super::unit_map")]
    units: BTreeMap<String, String>,
    value: ValueWire,
}
fn policy_units() -> BTreeMap<String, String> {
    units(&[
        ("duration", "ns"),
        ("counter", "rtc_ticks"),
        ("unix", "unix_s"),
        ("identity", "count"),
        ("buffer_bytes", "byte"),
        ("ntp_fraction", "2^-32 s"),
    ])
}
impl Serialize for PolicySnapshot {
    fn serialize<S: Serializer>(&self, s: S) -> core::result::Result<S::Ok, S::Error> {
        PolicyEnvelope {
            schema: "irig106-time/policy".into(),
            version: 1,
            defaults_version: 1,
            units: policy_units(),
            value: match &self.0 {
                PolicyValue::Offline(p) => ValueWire::Offline(OfflineWire::from(p)),
                PolicyValue::Stream(p) => ValueWire::Stream(StreamWire::from(p)),
            },
        }
        .serialize(s)
    }
}
impl<'de> Deserialize<'de> for PolicySnapshot {
    fn deserialize<D: Deserializer<'de>>(d: D) -> core::result::Result<Self, D::Error> {
        let w: PolicyEnvelope = object(d)?;
        header(&w.schema, w.version, "irig106-time/policy").map_err(de::Error::custom)?;
        if w.defaults_version != 1 || w.units != policy_units() {
            return Err(de::Error::custom(invalid()));
        }
        let value = match w.value {
            ValueWire::Offline(p) => Self::offline(p.try_into().map_err(de::Error::custom)?),
            ValueWire::Stream(p) => Self::stream(p.try_into().map_err(de::Error::custom)?),
        };
        value.map_err(de::Error::custom)
    }
}
#[derive(Serialize, Deserialize)]
enum ValueWire {
    Offline(OfflineWire),
    Stream(StreamWire),
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OfflineWire {
    #[serde(deserialize_with = "super::required")]
    shared: Option<SharedWire>,
    gap_interval_ns: Decimal<u64>,
    version: u16,
    late_ns: Decimal<u64>,
    infer_sessions: bool,
    #[serde(deserialize_with = "super::required")]
    channel: Option<u16>,
    channel_preference: Vec<u16>,
    include_undeclared: bool,
    include_invalid: bool,
    strategy: ReferenceStrategy,
    preceding_on_tie: bool,
    back_extrapolate: bool,
    stop_on_loss: bool,
    gap_tolerance_ns: Decimal<u64>,
    #[serde(deserialize_with = "super::required")]
    jump_threshold_ns: Option<Decimal<u64>>,
    finding_levels: Vec<(u8, crate::findings::Severity)>,
}
impl From<&OfflinePolicy> for OfflineWire {
    fn from(p: &OfflinePolicy) -> Self {
        Self {
            shared: p.shared.as_deref().map(SharedWire::from),
            gap_interval_ns: Decimal::new(p.gap_interval_ns),
            version: p.version,
            late_ns: Decimal::new(p.late_ns),
            infer_sessions: p.infer_sessions,
            channel: p.channel,
            channel_preference: p.channel_preference.clone(),
            include_undeclared: p.include_undeclared,
            include_invalid: p.include_invalid,
            strategy: p.strategy,
            preceding_on_tie: p.preceding_on_tie,
            back_extrapolate: p.back_extrapolate,
            stop_on_loss: p.stop_on_loss,
            gap_tolerance_ns: Decimal::new(p.gap_tolerance_ns),
            jump_threshold_ns: p.jump_threshold_ns.map(Decimal::new),
            finding_levels: p.finding_levels.iter().map(|(k, v)| (*k, *v)).collect(),
        }
    }
}
impl TryFrom<OfflineWire> for OfflinePolicy {
    type Error = TimeError;
    fn try_from(w: OfflineWire) -> Result<Self> {
        let mut levels = BTreeMap::new();
        for (id, level) in w.finding_levels {
            if levels.insert(id, level).is_some() {
                return Err(invalid());
            }
        }
        let p = Self {
            shared: w
                .shared
                .map(SharedPolicy::try_from)
                .transpose()?
                .map(Box::new),
            gap_interval_ns: w.gap_interval_ns.get(),
            version: w.version,
            late_ns: w.late_ns.get(),
            infer_sessions: w.infer_sessions,
            channel: w.channel,
            channel_preference: w.channel_preference,
            include_undeclared: w.include_undeclared,
            include_invalid: w.include_invalid,
            strategy: w.strategy,
            preceding_on_tie: w.preceding_on_tie,
            back_extrapolate: w.back_extrapolate,
            stop_on_loss: w.stop_on_loss,
            gap_tolerance_ns: w.gap_tolerance_ns.get(),
            jump_threshold_ns: w.jump_threshold_ns.map(Decimal::get),
            finding_levels: levels,
        };
        p.validate()?;
        Ok(p)
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CapsWire {
    packets: Decimal<usize>,
    bytes: Decimal<usize>,
    references: Decimal<usize>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StreamWire {
    id: Decimal<u64>,
    mode: StreamMode,
    offline: OfflineWire,
    hold_tolerance_ns: Decimal<u64>,
    hold_interval_ns: Decimal<u64>,
    raw_network_rate: bool,
    caps: CapsWire,
}
impl From<&StreamPolicy> for StreamWire {
    fn from(p: &StreamPolicy) -> Self {
        Self {
            id: Decimal::new(p.id),
            mode: p.mode,
            offline: OfflineWire::from(&p.offline),
            hold_tolerance_ns: Decimal::new(p.hold_tolerance_ns),
            hold_interval_ns: Decimal::new(p.hold_interval_ns),
            raw_network_rate: p.raw_network_rate,
            caps: CapsWire {
                packets: Decimal::new(p.caps.packets),
                bytes: Decimal::new(p.caps.bytes),
                references: Decimal::new(p.caps.references),
            },
        }
    }
}
impl TryFrom<StreamWire> for StreamPolicy {
    type Error = TimeError;
    fn try_from(w: StreamWire) -> Result<Self> {
        let p = Self {
            id: w.id.get(),
            mode: w.mode,
            offline: w.offline.try_into()?,
            hold_tolerance_ns: w.hold_tolerance_ns.get(),
            hold_interval_ns: w.hold_interval_ns.get(),
            raw_network_rate: w.raw_network_rate,
            caps: BufferCaps {
                packets: w.caps.packets.get(),
                bytes: w.caps.bytes.get(),
                references: w.caps.references.get(),
            },
        };
        p.validate()?;
        Ok(p)
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SharedWire {
    id: Decimal<u64>,
    #[serde(deserialize_with = "super::required")]
    caller_year: Option<u16>,
    year_order: Vec<YearSource>,
    scales: crate::scale::ScalePolicy,
    #[serde(deserialize_with = "super::required")]
    ieee: Option<EpochWire>,
    #[serde(deserialize_with = "super::required")]
    ch4: Option<EpochWire>,
    table: LeapTableSnapshot,
    counter_applicability: TimeApplicability,
    #[serde(deserialize_with = "super::required")]
    own_tolerance: Option<Decimal<u64>>,
    declarations: DeclarationWire,
    use_tfmt_default: bool,
    ntp: Vec<NtpWire>,
}
impl From<&SharedPolicy> for SharedWire {
    fn from(p: &SharedPolicy) -> Self {
        let s = p.settings();
        Self {
            id: Decimal::new(p.id()),
            caller_year: s.caller_year,
            year_order: s.year_order.sources().to_vec(),
            scales: s.scales,
            ieee: s.own.ieee.as_ref().map(EpochWire::from),
            ch4: s.own.ch4.as_ref().map(EpochWire::from),
            table: LeapTableSnapshot::new(s.own.table.clone()),
            counter_applicability: s.own.counter_applicability.clone(),
            own_tolerance: s.own_tolerance.map(|n| Decimal::new(n.get())),
            declarations: match &s.declarations {
                None => DeclarationWire::CallerPrepared,
                Some(d) => match d.snapshots() {
                    None => DeclarationWire::Absent,
                    Some(all) => {
                        DeclarationWire::Present(all.iter().map(SetupWire::from).collect())
                    }
                },
            },
            use_tfmt_default: s.use_tfmt_default,
            ntp: s
                .ntp
                .iter()
                .map(|(key, c)| NtpWire {
                    key: Decimal::new(*key),
                    session: Decimal::new(c.session),
                    declared: c.declared.as_ref().map(|d| EraWire {
                        era: Decimal::new(d.era),
                        id: d.id.clone(),
                    }),
                    pivots: c.pivots.clone(),
                })
                .collect(),
        }
    }
}
impl TryFrom<SharedWire> for SharedPolicy {
    type Error = TimeError;
    fn try_from(w: SharedWire) -> Result<Self> {
        let mut ntp = BTreeMap::new();
        for c in w.ntp {
            let context = NtpContext {
                session: c.session.get(),
                declared: c.declared.map(|d| EraDeclaration {
                    era: d.era.get(),
                    id: d.id,
                }),
                pivots: c.pivots,
            };
            if ntp.insert(c.key.get(), context).is_some() {
                return Err(invalid());
            }
        }
        let declarations = match w.declarations {
            DeclarationWire::CallerPrepared => None,
            DeclarationWire::Absent => Some(Declarations::absent()),
            DeclarationWire::Present(all) => Some(Declarations::present(
                all.into_iter()
                    .map(SetupDeclarations::try_from)
                    .collect::<Result<Vec<_>>>()?,
            )?),
        };
        Self::new(
            w.id.get(),
            SharedSettings {
                caller_year: w.caller_year,
                year_order: YearOrder::new(w.year_order)?,
                scales: w.scales,
                own: OwnContext {
                    ieee: w.ieee.map(EpochContext::from),
                    ch4: w.ch4.map(EpochContext::from),
                    table: w.table.value().clone(),
                    counter_applicability: w.counter_applicability,
                },
                own_tolerance: w.own_tolerance.map(|n| NanosDuration::new(n.get())),
                declarations,
                use_tfmt_default: w.use_tfmt_default,
                ntp,
            },
        )
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EraWire {
    era: Decimal<i64>,
    id: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NtpWire {
    key: Decimal<usize>,
    session: Decimal<usize>,
    #[serde(deserialize_with = "super::required")]
    declared: Option<EraWire>,
    pivots: Vec<NtpPivot>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SetupWire {
    session: Decimal<usize>,
    start: Decimal<i128>,
    #[serde(deserialize_with = "super::required")]
    end: Option<Decimal<i128>>,
    setup_packet: Decimal<u64>,
    provenance: String,
    #[serde(deserialize_with = "super::required")]
    declared_version: Option<String>,
    #[serde(deserialize_with = "super::required")]
    ri4: Option<String>,
    #[serde(deserialize_with = "super::required")]
    recording_year: Option<u16>,
    channels: Vec<ChannelDeclaration>,
}
impl From<&SetupDeclarations> for SetupWire {
    fn from(s: &SetupDeclarations) -> Self {
        Self {
            session: Decimal::new(s.range.session()),
            start: Decimal::new(s.range.start().get()),
            end: s.range.end().map(|n| Decimal::new(n.get())),
            setup_packet: Decimal::new(s.setup_packet),
            provenance: s.provenance.clone(),
            declared_version: s.declared_version.clone(),
            ri4: s.ri4.clone(),
            recording_year: s.recording_year,
            channels: s.channels.clone(),
        }
    }
}
impl TryFrom<SetupWire> for SetupDeclarations {
    type Error = TimeError;
    fn try_from(s: SetupWire) -> Result<Self> {
        Ok(Self {
            range: GoverningRange::new(
                s.session.get(),
                SignedRtcTicks::new(s.start.get()),
                s.end.map(|n| SignedRtcTicks::new(n.get())),
            )?,
            setup_packet: s.setup_packet.get(),
            provenance: s.provenance,
            declared_version: s.declared_version,
            ri4: s.ri4,
            recording_year: s.recording_year,
            channels: s.channels,
        })
    }
}

#[derive(Serialize, Deserialize)]
enum DeclarationWire {
    CallerPrepared,
    Absent,
    Present(Vec<SetupWire>),
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EpochWire {
    origin: CalendarSnapshot,
    counting: EpochCounting,
    source: String,
}
impl From<&EpochContext> for EpochWire {
    fn from(e: &EpochContext) -> Self {
        Self {
            origin: CalendarSnapshot(e.origin),
            counting: e.counting,
            source: e.source.clone(),
        }
    }
}
impl From<EpochWire> for EpochContext {
    fn from(e: EpochWire) -> Self {
        Self {
            origin: e.origin.value(),
            counting: e.counting,
            source: e.source,
        }
    }
}
