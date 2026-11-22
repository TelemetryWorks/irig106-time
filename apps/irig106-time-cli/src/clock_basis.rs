//! Bounded metadata alongside the legacy numeric correlators, not a new policy.
//! Partial Traces: L3-CLI-106 <- L2-CLI-106 <- L1-CLI-006.
use irig106_time::{AbsoluteTime, Rtc, StreamingTimeCorrelator};
use std::collections::{BTreeMap, VecDeque};
pub(super) const COVERED: &str = "UTC (from TAI via leap table)";
pub(super) const STALE: &str = "UTC (from TAI via stale leap table, possibly out of date)";
pub(super) const NTP_DECLARED: &str = "UTC (caller-declared era)";
pub(super) const NTP_AMBIGUOUS: &str = "UTC (caller-declared era, leap-ambiguous)";
pub(super) const NTP_UNCERTAIN: &str = "UTC (caller-declared era, leap coverage uncertain)";
pub(super) fn known(label: &str) -> Option<&'static str> {
    match label {
        COVERED => Some(COVERED),
        STALE => Some(STALE),
        NTP_DECLARED => Some(NTP_DECLARED),
        NTP_AMBIGUOUS => Some(NTP_AMBIGUOUS),
        NTP_UNCERTAIN => Some(NTP_UNCERTAIN),
        "UTC (assumed, RDG-009)" => Some("UTC (assumed, RDG-009)"),
        "UTC (stated FMT)" => Some("UTC (stated FMT)"),
        "GPS (stated FMT)" => Some("GPS (stated FMT)"),
        "scale unknown" => Some("scale unknown"),
        _ => None,
    }
}
pub(super) fn text(time: &AbsoluteTime, basis: Option<&str>) -> String {
    let clock = super::fmt_abs_time(time);
    match basis {
        Some(b) => format!("{clock} {b}"),
        None => clock,
    }
}
pub(super) fn distance(a: Rtc, b: Rtc) -> u64 {
    a.elapsed_ticks(b).min(b.elapsed_ticks(a))
}
#[derive(Default)]
pub(super) struct Window(BTreeMap<u16, VecDeque<(Rtc, Option<&'static str>)>>);
impl Window {
    pub(super) fn observe(
        &mut self,
        ch: u16,
        rtc: Rtc,
        label: &str,
        core: &StreamingTimeCorrelator,
    ) {
        let refs = self.0.entry(ch).or_default();
        let index = refs.partition_point(|r| r.0 <= rtc);
        refs.insert(index, (rtc, known(label)));
        // Core evicts only fronts (age/count). Matching its post-insert channel
        // lengths preserves equal-RTC arrival order without retaining lost refs.
        self.0.retain(|ch, refs| {
            while refs.len() > core.channel_len(*ch) {
                refs.pop_front();
            }
            if refs.capacity() > 4 && refs.len() < refs.capacity() / 2 {
                *refs = refs.iter().copied().collect();
            }
            !refs.is_empty()
        });
    }
    pub(super) fn clear(&mut self, ch: u16) {
        self.0.remove(&ch);
    }
    pub(super) fn nearest(&self, target: Rtc) -> Option<&'static str> {
        let (mut best, mut basis) = (u64::MAX, None);
        // Mirror StreamingTimeCorrelator::nearest_any: channel order, preceding
        // before following, latest equal RTC, first channel wins equal distance.
        for refs in self.0.values() {
            let index = refs.partition_point(|r| r.0 <= target);
            for i in [index.saturating_sub(1), index.min(refs.len() - 1)] {
                let d = distance(refs[i].0, target);
                if d < best {
                    best = d;
                    basis = refs[i].1;
                }
            }
        }
        basis
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Small deterministic metadata fixture.
mod tests {
    use super::*;
    #[test]
    fn live_basis_tracks_native_ties_duplicates_age_and_count_eviction() {
        let mut core = StreamingTimeCorrelator::with_reference_limit(
            irig106_time::NanosDuration::new(1500),
            std::num::NonZeroUsize::new(16).unwrap(),
        );
        let mut tags = Window::default();
        for n in 0..120u64 {
            let channel = (n % 3) as u16;
            let rtc = Rtc::from_raw(3 * (n / 4));
            let marker = (n / 3) % 3;
            let label = [COVERED, STALE, "scale unknown"][marker as usize];
            let time = AbsoluteTime::new(100, 12, 0, 0, marker as u32).unwrap();
            core.add_reference(channel, rtc, time);
            tags.observe(channel, rtc, label, &core);
            assert_eq!(
                tags.0.values().map(VecDeque::len).sum::<usize>(),
                core.len()
            );
            assert!(tags.0.values().map(VecDeque::capacity).sum::<usize>() <= 4 * core.len());
            for target in (0..100).step_by(3).map(Rtc::from_raw) {
                if let Ok(value) = core.correlate(target, None) {
                    let expected = [Some(COVERED), Some(STALE), Some("scale unknown")]
                        [(value.as_total_ns() % 3) as usize];
                    assert_eq!(tags.nearest(target), expected);
                }
            }
        }
        core.clear_channel(1);
        tags.clear(1);
        assert_eq!(
            tags.0.values().map(VecDeque::len).sum::<usize>(),
            core.len()
        );
    }
}
