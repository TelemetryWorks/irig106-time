//! Target-specific bracketing: two RTC groups per channel, independent of live
//! cache eviction. Original arrival order is retained for library tie handling.
use irig106_time::{AbsoluteTime, ReferencePoint, Rtc, TimeCorrelator, TimeError};
use std::collections::BTreeMap;

#[derive(Clone, Copy)]
struct Candidate {
    point: ReferencePoint,
    arrival: usize,
}
struct Group {
    first: Candidate,
    second: Option<Candidate>,
    last: Option<Candidate>,
}
impl Group {
    fn candidates(&self) -> impl Iterator<Item = &Candidate> {
        core::iter::once(&self.first)
            .chain(self.second.iter())
            .chain(self.last.iter())
    }
}
#[derive(Default)]
struct Bracket {
    before: Option<Group>,
    after: Option<Group>,
}
impl Bracket {
    fn observe(&mut self, target: Rtc, candidate: Candidate) {
        let preceding = candidate.point.rtc <= target;
        let slot = if preceding {
            &mut self.before
        } else {
            &mut self.after
        };
        if slot.as_ref().is_none_or(|old| {
            if preceding {
                candidate.point.rtc > old.first.point.rtc
            } else {
                candidate.point.rtc < old.first.point.rtc
            }
        }) {
            *slot = Some(Group {
                first: candidate,
                second: None,
                last: None,
            });
        } else if let Some(group) = slot {
            // For a duplicate run, library boundary lookups can select the
            // first, second or latest arrival (before/after/exact queries).
            // Keep those representatives, not the unbounded run.
            if candidate.point.rtc == group.first.point.rtc {
                if group.second.is_none() {
                    group.second = Some(candidate);
                } else {
                    group.last = Some(candidate);
                }
            }
        }
    }
    fn candidates(&self) -> impl Iterator<Item = &Candidate> {
        self.before
            .iter()
            .chain(self.after.iter())
            .flat_map(Group::candidates)
    }
}

pub(super) struct TargetReferences {
    pub(super) target: Rtc,
    arrivals: usize,
    channels: BTreeMap<u16, Bracket>,
}
impl TargetReferences {
    pub(super) fn new(target: Rtc) -> Self {
        Self {
            target,
            arrivals: 0,
            channels: BTreeMap::new(),
        }
    }
    pub(super) fn observe(&mut self, channel_id: u16, rtc: Rtc, time: AbsoluteTime) {
        let candidate = Candidate {
            point: ReferencePoint {
                channel_id,
                rtc,
                time,
            },
            arrival: self.arrivals,
        };
        self.arrivals += 1;
        self.channels
            .entry(channel_id)
            .or_default()
            .observe(self.target, candidate);
    }
    pub(super) fn correlate(&self, channel: Option<u16>) -> Result<AbsoluteTime, TimeError> {
        let mut candidates: Vec<_> = match channel {
            Some(id) => self
                .channels
                .get(&id)
                .ok_or(TimeError::NoReferencePoint)?
                .candidates()
                .collect(),
            None => self
                .channels
                .values()
                .flat_map(Bracket::candidates)
                .collect(),
        };
        candidates.sort_unstable_by_key(|c| c.arrival);
        // At most six/channel (two RTC groups with duplicate boundary values).
        // Reuse the existing
        // library's nearest/tie/checked-shift semantics, including cross-channel
        // equal RTCs. The temporary library index is bounded by channel count.
        let mut pair = TimeCorrelator::new();
        for candidate in candidates {
            let p = candidate.point;
            pair.add_reference(p.channel_id, p.rtc, p.time);
        }
        pair.correlate(self.target, channel)
    }
}
