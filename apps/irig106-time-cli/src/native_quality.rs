//! Bounded summary reference quality, separate from legacy CLI correlation.
//! Partial CLI-104/106 and COR-113: packet-local years, no TMATS declarations.
use irig106_time::{
    Result, Rtc, TimeError,
    edition::EditionEvidence,
    format1::decode_qualified_time_f1,
    native_quality::OrderedQuality,
    scale::{ScaleOrigin, ScalePolicy, TimeScale},
    timeline::*,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

// Resource limits, not timing/admission defaults. Exhaustion labels a prefix;
// never discard input and then pretend to have complete quality statistics.
const CHANNEL_CAP: usize = 64;
const PENDING_CAP: usize = 1024;
#[derive(Default)]
pub(super) struct Quality {
    channels: BTreeMap<u16, OrderedQuality>,
    pending: Vec<TimelineReference>,
    unavailable: Option<String>,
    ntp_era: Option<i64>,
    issues: BTreeMap<u16, String>,
}
impl Quality {
    pub(super) fn new(ntp_era: Option<i64>) -> Self {
        Self {
            ntp_era,
            ..Default::default()
        }
    }

    pub(super) fn advance(&mut self, high: i128, late_ns: u64) {
        self.drain(Some(high - i128::from(late_ns) / 100));
    }
    pub(super) fn observe(
        &mut self,
        key: PacketKey,
        channel: u16,
        kind: u8,
        version: u8,
        payload: &[u8],
    ) {
        if self.unavailable.is_some() {
            return;
        }
        if !self.channels.contains_key(&channel) {
            if self.channels.len() == CHANNEL_CAP {
                self.unavailable = Some("native quality channel cap (64); prefix only".into());
                return;
            }
            match OrderedQuality::new(key.session, channel) {
                Ok(q) => {
                    self.channels.insert(channel, q);
                }
                Err(e) => {
                    self.unavailable = Some(e.to_string());
                    return;
                }
            }
        }
        match reference(key, channel, kind, version, payload, self.ntp_era) {
            Ok(r) => {
                if self.pending.len() == PENDING_CAP {
                    self.unavailable =
                        Some("native quality pending cap (1024); prefix only".into());
                } else {
                    self.pending.push(r);
                }
            }
            Err(error) => {
                self.issues.insert(channel, error.to_string());
                if let Some(q) = self.channels.get_mut(&channel) {
                    if let Err(e) = q.add_decode_rejections(1) {
                        self.unavailable = Some(e.to_string());
                    }
                }
            }
        }
    }
    fn drain(&mut self, cut: Option<i128>) {
        if self.unavailable.is_some() {
            return;
        }
        self.pending.sort_by_key(|r| (r.key.ticks, r.key.packet));
        let n = self
            .pending
            .partition_point(|r| cut.is_none_or(|t| r.key.ticks <= t));
        for r in self.pending.drain(..n) {
            if let Some(q) = self.channels.get_mut(&r.channel) {
                if let Err(e) = q.observe(&r) {
                    self.unavailable = Some(e.to_string());
                    break;
                }
            }
        }
    }
    pub(super) fn value(&self) -> Value {
        // Render the final late window without changing live state or collecting history.
        let mut final_state = Self {
            channels: self.channels.clone(),
            pending: self.pending.clone(),
            unavailable: self.unavailable.clone(),
            ntp_era: self.ntp_era,
            issues: self.issues.clone(),
        };
        final_state.drain(None);
        let rows: Vec<_> = final_state.channels.iter().map(|(&channel,q)| {
            let stats = q.snapshot();
            let endpoints = q.calendar_endpoints();
            let (first,last,calendar_issue) = match endpoints {
                Ok(Some((a,b))) => (calendar(a,q.first_reference()),calendar(b,q.last_reference()),None),
                Ok(None) => (Value::Null,Value::Null,None),
                Err(e) => (Value::Null,Value::Null,Some(e.to_string())),
            };
            json!({"channel":channel,"valid":stats.valid.to_string(),"admitted":stats.admitted.to_string(),"rejected":stats.rejected.to_string(),
                "interval_ns":stats.interval_ns.map(|n|n.to_string()),"min_spacing_ns":stats.min_spacing_ns.map(|n|n.to_string()),"max_spacing_ns":stats.max_spacing_ns.map(|n|n.to_string()),
                "references_per_second":stats.references_per_second,"drift_ppm":stats.drift_ppm,"drift_uses_stale_leap_table":q.drift_uses_stale_leap_table(),"unavailable_reason":stats.unavailable,
                "first":first,"last":last,"calendar_unavailable_reason":calendar_issue,"rejection_reason":final_state.issues.get(&channel)})
        }).collect();
        json!({"formats":"format1+format2","ntp_era":self.ntp_era.map(|n|n.to_string()),"ntp_era_source":if self.ntp_era.is_some(){"caller-declared"}else{"none"},"ntp_era_inference":"disabled","include_undeclared":true,"year_inference":"packet-only","channel_cap":CHANNEL_CAP.to_string(),"pending_cap":PENDING_CAP.to_string(),"status":if final_state.unavailable.is_some(){"needs-context"}else{"complete"},"unavailable_reason":final_state.unavailable,"channels":rows})
    }
}
fn calendar(t: irig106_time::format1::CalendarReading, r: Option<&TimelineReference>) -> Value {
    use irig106_time::{format2::NetworkConversion, ntp::NtpLeapStatus};
    let leap =
        r.and_then(|r| r.input.network.as_ref())
            .and_then(|n| match n.reference().conversion() {
                NetworkConversion::Ntp(c) => match c.basis().leap {
                    NtpLeapStatus::UnambiguousCovered => Some("covered"),
                    NtpLeapStatus::Ambiguous => Some("leap-ambiguous"),
                    NtpLeapStatus::UncertainCoverage => Some("coverage-unknown"),
                    NtpLeapStatus::NoValue => None,
                },
                _ => None,
            });
    json!({"ntp_leap_status":leap,"year":t.year,"day":t.day_of_year,"hours":t.hours,"minutes":t.minutes,"seconds":t.seconds,"nanoseconds":t.nanoseconds,
        "scale":match t.scale.effective(){TimeScale::Utc=>"UTC",TimeScale::Tai=>"TAI",TimeScale::Gps=>"GPS",TimeScale::Unknown=>"unknown"},
        "basis":match t.scale.origin(){ScaleOrigin::Standard=>"stated",ScaleOrigin::Policy=>"assumed",ScaleOrigin::Unspecified=>"unknown"}})
}
fn reference(
    key: PacketKey,
    channel: u16,
    kind: u8,
    version: u8,
    payload: &[u8],
    era: Option<i64>,
) -> Result<TimelineReference> {
    // Do NOT guess SRC/EXT from the header version or force a modern layout.
    // A one-packet native builder applies admission/calendar checks without
    // retaining recording history or claiming retrospective session year inference.
    if kind == 0x12 {
        use irig106_time::{
            format2::{Format2Input, ReferenceCapture},
            network_time::NetworkTimeProtocol,
            ntp::{EraDeclaration, NtpContext},
        };
        let observation = Format2Input {
            payload,
            data_length: payload.len(),
            capture: ReferenceCapture {
                session: key.session,
                packet: key.packet,
                channel,
                rtc: Rtc::from_raw(key.ticks as u64),
                byte_offset: None,
            },
            declared_edition: None,
            data_type_version: version,
            declared_timein: false,
        }
        .decode()?;
        // No implicit era0 or pivot guessing. The zero sentinel needs no era.
        if observation.csdw().time_protocol() == NetworkTimeProtocol::Ntp
            && era.is_none()
            && observation.raw_body().iter().any(|b| *b != 0)
        {
            return Err(TimeError::NeedsContext {
                context: "NTP era/basis required; supply --ntp-era <N>",
            });
        }
        let mut context = NtpContext::new(key.session);
        context.declared = era.map(|era| EraDeclaration {
            era,
            id: "--ntp-era".into(),
        });
        let (r, _) = irig106_time::reference::admit_format2_reference(
            observation,
            context,
            irig106_time::LeapSecondTable::builtin(),
            key,
            &OfflinePolicy {
                include_undeclared: true,
                ..Default::default()
            },
            false,
        )?;
        return Ok(r);
    }
    let reading = decode_qualified_time_f1(
        payload,
        payload.len(),
        EditionEvidence {
            data_type_version: Some(version),
            ..Default::default()
        },
        ScalePolicy::default(),
    )?;
    let mut input = ReferenceInput::qualified_format1(reading)?;
    if let Ok(normalized) = input
        .clone()
        .normalize_to_tai(irig106_time::LeapSecondTable::builtin())
    {
        input = normalized;
    }
    let mut b = TimelineBuilder::new(OfflinePolicy {
        include_undeclared: true,
        ..OfflinePolicy::default()
    })?;
    b.observe(
        PacketObservation {
            id: key.packet,
            channel,
            rtc: Rtc::from_raw(0),
            kind: PacketKind::Time,
            declared_timein: false,
            declared_external: false,
            flags: 0,
            reference: Some(input),
        },
        false,
    )?;
    let timeline = b.finish()?;
    let mut r = timeline
        .sessions()
        .first()
        .and_then(|s| s.references.first())
        .cloned()
        .ok_or(TimeError::InvalidTimelineInput(
            "missing native quality reference",
        ))?;
    r.key = key;
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn key(packet: u64, ticks: i128) -> PacketKey {
        PacketKey {
            session: 1,
            packet,
            ticks,
        }
    }
    #[test]
    fn packet_year_validity_decode_rejections_and_limits_are_explicit() {
        // Fig11-12/14 DATE1/FMT4,2017Jan2 00:00; no guessed SRC layout.
        let bytes = [0x41, 2, 0, 0, 0, 0, 0, 0, 2, 1, 0x17, 0x20];
        let mut q = Quality::default();
        q.observe(key(0, 0), 7, 0x11, 6, &bytes);
        q.observe(key(1, 10_000_000), 7, 0x11, 6, &[0; 2]);
        let v = q.value();
        let row = &v["channels"][0];
        assert_eq!(row["valid"], "1");
        assert_eq!(row["rejected"], "1");
        assert_eq!(row["first"]["year"], 2017);
        assert_eq!(row["first"]["basis"], "stated");
        for ch in 8..=71 {
            q.observe(key(u64::from(ch), 0), ch, 0x11, 6, &[]);
        }
        assert_eq!(q.channels.len(), CHANNEL_CAP);
        assert_eq!(q.value()["status"], "needs-context");
        assert!(
            q.value()["unavailable_reason"]
                .as_str()
                .is_some_and(|reason| reason.contains("channel cap"))
        );
        let mut q = Quality::default();
        for p in 0..=PENDING_CAP {
            q.observe(key(p as u64, 0), 7, 0x11, 6, &bytes);
        }
        assert_eq!(q.pending.len(), PENDING_CAP);
        assert_eq!(q.value()["status"], "needs-context");
        assert!(
            q.value()["unavailable_reason"]
                .as_str()
                .is_some_and(|reason| reason.contains("pending cap"))
        );
    }
}
