//! # Legacy compatibility API
//! Not recommended for new recording answers: this shape omits session/scale/basis.
//! Use [`crate::timeline`] offline or [`crate::stream`] live with shared policy.
//!
//! Streaming RTC-to-absolute-time correlator for live data.
//!
//! Unlike [`TimeCorrelator`] which is
//! designed for post-processing complete files, `StreamingTimeCorrelator`
//! handles live UDP streams where packets arrive in near-real-time,
//! potentially out of order, and the reference set must be bounded in memory.
//!
//! # Design
//!
//! - **Sliding window:** Reference points older than `max_age_ns` are evicted
//!   on each insert. Age alone does not cap frozen-counter duplicates; use
//!   `with_reference_limit` for a global count/storage bound.
//! - **Arrival-order insert:** Does not assume packets arrive in RTC order.
//!   Maintains RTC-sorted per-channel deques via partition-point insert.
//! - **Per-channel indexing:** Same O(log n) channel-filtered lookup as the
//!   batch correlator.
//!
//! # Requirement Traceability
//!
//! | Requirement | Description |
//! |-------------|-------------|
//! | P5-02       | Streaming correlator with sliding window |

extern crate alloc;
use alloc::collections::{BTreeMap, BTreeSet, VecDeque};
use alloc::vec::Vec;
use core::num::NonZeroUsize;

use crate::absolute::AbsoluteTime;
use crate::error::{Result, TimeError};
use crate::rtc::Rtc;
use irig106_types::NanosDuration;

/// A streaming correlator that maintains a bounded sliding window of
/// reference points for live UDP/network telemetry processing.
///
/// `new` limits age only; `with_reference_limit` also caps count and storage.
/// Reference points older than `max_age_ns` (measured in RTC nanoseconds
/// from the latest reference) are automatically evicted on each insert.
///
/// # Example
///
/// ```
/// use irig106_time::streaming::StreamingTimeCorrelator;
/// use irig106_time::{AbsoluteTime, NanosDuration, Rtc};
///
/// // Keep a 60-second window
/// let mut sc = StreamingTimeCorrelator::new(NanosDuration::new(60_000_000_000));
///
/// sc.add_reference(1, Rtc::from_raw(10_000_000),
///     AbsoluteTime::new(100, 12, 0, 0, 0).unwrap());
///
/// let resolved = sc.correlate(Rtc::from_raw(10_500_000), None).unwrap();
/// ```
///
/// **Traces:** P5-02
pub struct StreamingTimeCorrelator {
    /// Per-channel reference points, each deque sorted by RTC.
    channel_refs: BTreeMap<u16, VecDeque<StreamingRef>>,
    /// Maximum age in nanoseconds. References older than
    /// `latest_rtc - max_age_ns` are evicted.
    max_age_ns: u64,
    /// Loss history, bounded by the u16 channel domain; contains no anchors.
    evicted_channels: BTreeSet<u16>,
    /// Optional global count limit, including equal-RTC references.
    max_references: Option<NonZeroUsize>,
    /// The highest RTC seen so far (for eviction).
    latest_rtc: Option<Rtc>,
    /// Total reference points across all channels.
    total_refs: usize,
    /// Total references evicted since creation.
    total_evicted: usize,
}

/// A reference point in the streaming correlator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StreamingRef {
    /// Channel ID of the time source.
    pub channel_id: u16,
    /// RTC value at the instant the absolute time was valid.
    pub rtc: Rtc,
    /// Decoded absolute time from the time packet.
    pub time: AbsoluteTime,
}

impl StreamingTimeCorrelator {
    /// Create a new streaming correlator with the given maximum age window.
    ///
    /// Reference points whose RTC is more than `max_age` behind the
    /// latest-seen RTC are evicted on the next insert.
    ///
    /// This constructor has no count limit: equal-RTC input can grow without
    /// bound. Use [`Self::with_reference_limit`] for a hard resource cap.
    ///
    /// A typical value for 1 Hz time packets with 30 seconds of lookback:
    /// `NanosDuration::from_secs(30)` (checked, returning Option).
    pub fn new(max_age: NanosDuration) -> Self {
        Self {
            channel_refs: BTreeMap::new(),
            max_age_ns: max_age.get(),
            evicted_channels: BTreeSet::new(),
            max_references: None,
            latest_rtc: None,
            total_refs: 0,
            total_evicted: 0,
        }
    }

    /// Limit both RTC age and the global retained reference count.
    ///
    /// Age alone does not bound duplicate/frozen-counter input. This opt-in
    /// count bound evicts the smallest raw RTC first, then the smallest channel
    /// ID on a tie; equal-RTC entries on one channel evict in arrival order.
    /// A quiet channel can lose all references: no per-channel anchor is kept.
    /// Such queries return NoReferencePoint. Empty channels and excess deque
    /// storage are reclaimed. Insertion can
    /// temporarily hold one extra reference before eviction.
    ///
    /// This compatibility API still has no session/scale/basis semantics;
    /// recording answers should use `stream`/`timeline` with shared policy.
    pub fn with_reference_limit(max_age: NanosDuration, max_references: NonZeroUsize) -> Self {
        let mut this = Self::new(max_age);
        this.max_references = Some(max_references);
        this
    }

    /// Insert a reference point and evict stale entries.
    ///
    /// The reference is inserted into the per-channel sorted deque. Then,
    /// any references on any channel whose RTC is older than
    /// `latest_rtc - max_age_ns` are removed.
    pub fn add_reference(&mut self, channel_id: u16, rtc: Rtc, time: AbsoluteTime) {
        // Update latest RTC
        match self.latest_rtc {
            Some(latest) if rtc > latest => self.latest_rtc = Some(rtc),
            None => self.latest_rtc = Some(rtc),
            _ => {}
        }

        // Insert after equal RTCs in arrival order.
        let refs = self.channel_refs.entry(channel_id).or_default();
        let pos = refs.partition_point(|r| r.rtc <= rtc);
        refs.insert(
            pos,
            StreamingRef {
                channel_id,
                rtc,
                time,
            },
        );
        self.total_refs += 1;

        // Evict stale references
        self.evict();
    }

    /// Insert a Format 2 (Network Time) reference point.
    ///
    /// Converts NTP or PTP time to `AbsoluteTime` and delegates to
    /// `add_reference`.
    /// Compatibility result omits table/currency basis; retain convert_ptp
    /// results for qualified answers. Leap labels need the new adapter.
    /// **Traces:** L3-PTP-104 ← L2-PTP-104 ← L1-PTP-004
    pub fn add_reference_f2(
        &mut self,
        channel_id: u16,
        rtc: Rtc,
        network_time: &crate::network_time::NetworkTime,
        leap_table: &crate::network_time::LeapSecondTable,
    ) -> Result<()> {
        let abs_time = match network_time {
            crate::network_time::NetworkTime::Ntp(_) => {
                return Err(crate::TimeError::NeedsContext {
                    context: "NTP era/basis required; use ntp::convert_ntp",
                });
            }
            crate::network_time::NetworkTime::Ptp(ptp) => {
                // Compatibility calendar cannot hold second60. Canonical callers
                // retain convert_ptp's raw TAI/currency/basis; this legacy API
                // still returns only an ordinary calendar (S09 adapter remains).
                leap_table
                    .convert_ptp(*ptp)?
                    .utc()
                    .ok_or(crate::TimeError::NeedsContext {
                        context: "no known applicable TAI-UTC offset",
                    })?
                    .to_absolute()?
                    .value
            }
        };
        self.add_reference(channel_id, rtc, abs_time);
        Ok(())
    }

    /// Correlate an RTC value to absolute time using the nearest reference
    /// in the sliding window.
    ///
    /// If `channel_id` is `Some(id)`, only references from that channel
    /// are considered. After any eviction in that set, a target older than its
    /// oldest retained reference returns QueryBeforeRetainedWindow. A channel
    /// with no remaining references returns NoReferencePoint. Before any loss,
    /// pre-reference extrapolation remains available. Duplicate RTCs choose the
    /// latest arrival on that channel for exact matches in both constructor modes.
    pub fn correlate(&self, target_rtc: Rtc, channel_id: Option<u16>) -> Result<AbsoluteTime> {
        let oldest = match channel_id {
            Some(id) => self.channel_refs.get(&id).and_then(|r| r.front()),
            None => self
                .channel_refs
                .values()
                .filter_map(|r| r.front())
                .min_by_key(|r| r.rtc),
        }
        .ok_or(TimeError::NoReferencePoint)?;
        let lost = match channel_id {
            Some(id) => self.evicted_channels.contains(&id),
            None => self.total_evicted != 0,
        };
        if lost && target_rtc < oldest.rtc {
            return Err(TimeError::QueryBeforeRetainedWindow);
        }
        let nearest = match channel_id {
            Some(id) => self.nearest_for_channel(target_rtc, id)?,
            None => self.nearest_any(target_rtc)?,
        };

        if target_rtc >= nearest.rtc {
            let delta_nanos = nearest.rtc.elapsed_nanos(target_rtc);
            nearest.time.add_nanos(delta_nanos)
        } else {
            let delta_nanos = target_rtc.elapsed_nanos(nearest.rtc);
            nearest.time.sub_nanos(delta_nanos)
        }
    }

    /// Find the nearest reference across all channels.
    fn nearest_any(&self, target: Rtc) -> Result<&StreamingRef> {
        let mut best: Option<&StreamingRef> = None;
        let mut best_dist = u64::MAX;

        for refs in self.channel_refs.values() {
            if refs.is_empty() {
                continue;
            }
            let idx = refs.partition_point(|r| r.rtc <= target);

            // Check idx and idx-1 for closest
            for &check_idx in &[idx.saturating_sub(1), idx.min(refs.len() - 1)] {
                if check_idx < refs.len() {
                    let dist = refs[check_idx]
                        .rtc
                        .elapsed_ticks(target)
                        .min(target.elapsed_ticks(refs[check_idx].rtc));
                    if dist < best_dist {
                        best_dist = dist;
                        best = Some(&refs[check_idx]);
                    }
                }
            }
        }

        best.ok_or(TimeError::NoReferencePoint)
    }

    /// Find the nearest reference for a specific channel.
    fn nearest_for_channel(&self, target: Rtc, channel_id: u16) -> Result<&StreamingRef> {
        let refs = self
            .channel_refs
            .get(&channel_id)
            .ok_or(TimeError::NoReferencePoint)?;

        if refs.is_empty() {
            return Err(TimeError::NoReferencePoint);
        }

        let idx = refs.partition_point(|r| r.rtc <= target);

        if idx == 0 {
            Ok(&refs[0])
        } else if idx >= refs.len() {
            Ok(&refs[refs.len() - 1])
        } else {
            let da = refs[idx - 1]
                .rtc
                .elapsed_ticks(target)
                .min(target.elapsed_ticks(refs[idx - 1].rtc));
            let db = refs[idx]
                .rtc
                .elapsed_ticks(target)
                .min(target.elapsed_ticks(refs[idx].rtc));
            if da <= db {
                Ok(&refs[idx - 1])
            } else {
                Ok(&refs[idx])
            }
        }
    }

    /// Evict references older than `max_age_ns` from the latest RTC.
    fn evict(&mut self) {
        let latest = match self.latest_rtc {
            Some(r) => r,
            None => return,
        };

        let cutoff_ticks = self.max_age_ns / 100; // ns → ticks (100 ns/tick)
        let cutoff_raw = latest.as_raw().saturating_sub(cutoff_ticks);

        for (&channel, refs) in &mut self.channel_refs {
            let before = refs.len();
            while refs.front().is_some_and(|r| r.rtc.as_raw() < cutoff_raw) {
                refs.pop_front();
            }
            let evicted = before - refs.len();
            self.total_refs -= evicted;
            self.total_evicted += evicted;
            if evicted != 0 {
                self.evicted_channels.insert(channel);
                Self::compact(refs);
            }
        }
        self.channel_refs.retain(|_, refs| !refs.is_empty());
        if let Some(limit) = self.max_references {
            while self.total_refs > limit.get() {
                let oldest = self
                    .channel_refs
                    .iter()
                    .filter_map(|(&ch, refs)| refs.front().map(|r| (r.rtc, ch)))
                    .min();
                let Some((_, channel)) = oldest else { break };
                if let Some(refs) = self.channel_refs.get_mut(&channel) {
                    // The selected entry is nonempty, with exclusive access.
                    if !refs.is_empty() {
                        refs.pop_front();
                        self.evicted_channels.insert(channel);
                        self.total_refs -= 1;
                        self.total_evicted += 1;
                        Self::compact(refs);
                    }
                }
                self.channel_refs.retain(|_, refs| !refs.is_empty());
            }
        }
    }

    fn compact(refs: &mut VecDeque<StreamingRef>) {
        if refs.capacity() > 4 && refs.len() < refs.capacity() / 2 {
            // A fresh allocation releases an old large backing store even if
            // one anchor remains. Empty deques are removed from the map.
            *refs = refs.iter().copied().collect();
        }
    }

    /// Discard the earliest-arriving retained match for a channel/RTC pair.
    /// Used by caller resource queues; counts as loss for pre-window guards.
    /// Out-of-order matches do not discard another reference or channel.
    pub fn discard_reference(&mut self, channel: u16, rtc: Rtc) -> bool {
        let Some(refs) = self.channel_refs.get_mut(&channel) else {
            return false;
        };
        let Some(index) = refs.iter().position(|r| r.rtc == rtc) else {
            return false;
        };
        refs.remove(index);
        Self::compact(refs);
        self.total_refs -= 1;
        self.total_evicted += 1;
        self.evicted_channels.insert(channel);
        self.channel_refs.retain(|_, refs| !refs.is_empty());
        true
    }

    /// Discard one channel's retained references, returning the removed count.
    /// Other channels survive; the age high-water mark is recomputed from them.
    /// An empty cache can therefore accept a lower-RTC epoch. This is a resource
    /// operation, not reset inference: callers must qualify segments themselves.
    /// Removed entries count as evictions; pre-window loss guards remain active.
    pub fn clear_channel(&mut self, channel_id: u16) -> usize {
        let removed = self.channel_refs.remove(&channel_id).map_or(0, |r| r.len());
        if removed != 0 {
            self.total_refs -= removed;
            self.total_evicted += removed;
            self.evicted_channels.insert(channel_id);
        }
        self.latest_rtc = self
            .channel_refs
            .values()
            .filter_map(|r| r.back())
            .map(|r| r.rtc)
            .max();
        removed
    }

    /// Total number of reference points currently in the window.
    pub fn len(&self) -> usize {
        self.total_refs
    }

    /// Whether the correlator has no reference points.
    pub fn is_empty(&self) -> bool {
        self.total_refs == 0
    }

    /// Total number of reference points evicted since creation.
    pub fn total_evicted(&self) -> usize {
        self.total_evicted
    }

    /// The maximum age window in nanoseconds.
    pub fn max_age_ns(&self) -> u64 {
        self.max_age_ns
    }

    /// The latest RTC value seen.
    pub fn latest_rtc(&self) -> Option<Rtc> {
        self.latest_rtc
    }

    /// Channel IDs that currently have reference points in the window.
    pub fn channel_ids(&self) -> Vec<u16> {
        self.channel_refs
            .keys()
            .copied()
            .filter(|ch| {
                self.channel_refs
                    .get(ch)
                    .map(|r| !r.is_empty())
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Number of reference points for a specific channel.
    pub fn channel_len(&self, channel_id: u16) -> usize {
        self.channel_refs
            .get(&channel_id)
            .map(|r| r.len())
            .unwrap_or(0)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)] // Test assertions only; production retains root denies.
#[path = "streaming_tests.rs"]
mod streaming_tests;
