//! Target-specific bracketing: two RTC groups per channel, independent of live
//! cache eviction. Original arrival order is retained for library tie handling.
use irig106_time::{AbsoluteTime, ReferencePoint, Rtc, TimeCorrelator, TimeError};
use std::collections::BTreeMap;

#[derive(Clone, Copy)]
struct Candidate {
    point: ReferencePoint,
    arrival: usize,
    basis: Option<&'static str>,
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
    range: Option<(Rtc, Rtc)>,
    before: Option<Group>,
    after: Option<Group>,
}
impl Bracket {
    fn observe(&mut self, target: Rtc, candidate: Candidate) {
        let rtc = candidate.point.rtc;
        self.range = Some(
            self.range
                .map_or((rtc, rtc), |(lo, hi)| (lo.min(rtc), hi.max(rtc))),
        );
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
    #[cfg(test)]
    pub(super) fn observe(&mut self, channel_id: u16, rtc: Rtc, time: AbsoluteTime) {
        self.observe_at(channel_id, rtc, time, self.arrivals);
    }
    #[cfg(test)]
    pub(super) fn observe_at(
        &mut self,
        channel_id: u16,
        rtc: Rtc,
        time: AbsoluteTime,
        arrival: usize,
    ) {
        self.observe_tagged_at(channel_id, rtc, time, arrival, None);
    }
    pub(super) fn observe_tagged_at(
        &mut self,
        channel_id: u16,
        rtc: Rtc,
        time: AbsoluteTime,
        arrival: usize,
        basis: Option<&'static str>,
    ) {
        let candidate = Candidate {
            point: ReferencePoint {
                channel_id,
                rtc,
                time,
            },
            arrival,
            basis,
        };
        self.arrivals = self.arrivals.max(arrival + 1);
        self.channels
            .entry(channel_id)
            .or_default()
            .observe(self.target, candidate);
    }
    pub(super) fn basis(&self, channel: Option<u16>) -> Option<&'static str> {
        let mut arrivals: Vec<_> = self
            .channels
            .iter()
            .filter(|(ch, _)| channel.is_none_or(|id| id == **ch))
            .flat_map(|(_, b)| b.candidates())
            .collect();
        arrivals.sort_unstable_by_key(|c| c.arrival);
        let mut refs: Vec<&Candidate> = Vec::new();
        // Same insertion/search/tie ordering as the TimeCorrelator replay below.
        for c in arrivals {
            let i = refs
                .binary_search_by_key(&c.point.rtc, |r| r.point.rtc)
                .unwrap_or_else(|i| i);
            refs.insert(i, c);
        }
        if refs.is_empty() {
            return None;
        }
        let i = refs
            .binary_search_by_key(&self.target, |r| r.point.rtc)
            .unwrap_or_else(|i| i);
        let selected = if i == 0 {
            0
        } else if i == refs.len() {
            i - 1
        } else if super::clock_basis::distance(refs[i - 1].point.rtc, self.target)
            <= super::clock_basis::distance(refs[i].point.rtc, self.target)
        {
            i - 1
        } else {
            i
        };
        refs[selected].basis
    }
    pub(super) fn covers(&self, channel: u16) -> bool {
        self.channels
            .get(&channel)
            .and_then(|b| b.range)
            .is_some_and(|(lo, hi)| lo <= self.target && self.target <= hi)
    }
    pub(super) fn clear_channel(&mut self, channel: u16) {
        self.channels.remove(&channel);
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

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Small replay fixture.
mod basis_tests {
    use super::*;
    #[test]
    fn target_basis_matches_numeric_replay_with_late_duplicate_candidates() {
        use crate::clock_basis::{COVERED, STALE};
        for target in (0..21).step_by(3).map(Rtc::from_raw) {
            let mut query = TargetReferences::new(target);
            for n in 0..60usize {
                let marker = (n / 6) % 3;
                let basis = [Some(COVERED), Some(STALE), None][marker];
                // Non-monotonic arrivals plus equal counters exercise the same
                // before/after duplicate representatives and insertion indices.
                let rtc = Rtc::from_raw(3 * ((n * 7) % 6) as u64);
                query.observe_tagged_at(
                    (n % 2) as u16,
                    rtc,
                    AbsoluteTime::new(100, 12, 0, 0, marker as u32).unwrap(),
                    n,
                    basis,
                );
            }
            for channel in [None, Some(0), Some(1)] {
                let value = query.correlate(channel).unwrap();
                assert_eq!(
                    query.basis(channel),
                    [Some(COVERED), Some(STALE), None][(value.as_total_ns() % 3) as usize]
                );
            }
        }
    }
}
