//! Streaming RTC-to-absolute-time correlator for live data.
//!
//! Unlike [`TimeCorrelator`](crate::correlation::TimeCorrelator) which is
//! designed for post-processing complete files, `StreamingTimeCorrelator`
//! handles live UDP streams where packets arrive in near-real-time,
//! potentially out of order, and the reference set must be bounded in memory.
//!
//! # Design
//!
//! - **Sliding window:** Reference points older than `max_age_ns` are evicted
//!   on each insert, keeping memory bounded.
//! - **Arrival-order insert:** Does not assume packets arrive in RTC order.
//!   Maintains RTC-sorted per-channel vecs via binary search insert.
//! - **Per-channel indexing:** Same O(log n) channel-filtered lookup as the
//!   batch correlator.
//!
//! # Requirement Traceability
//!
//! | Requirement | Description |
//! |-------------|-------------|
//! | P5-02       | Streaming correlator with sliding window |

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::absolute::AbsoluteTime;
use crate::error::{Result, TimeError};
use crate::rtc::Rtc;

/// A streaming correlator that maintains a bounded sliding window of
/// reference points for live UDP/network telemetry processing.
///
/// Reference points older than `max_age_ns` (measured in RTC nanoseconds
/// from the latest reference) are automatically evicted on each insert.
///
/// # Example
///
/// ```
/// use irig106_time::streaming::StreamingTimeCorrelator;
/// use irig106_time::{Rtc, AbsoluteTime};
///
/// // Keep a 60-second window
/// let mut sc = StreamingTimeCorrelator::new(60_000_000_000);
///
/// sc.add_reference(1, Rtc::from_raw(10_000_000),
///     AbsoluteTime::new(100, 12, 0, 0, 0).unwrap());
///
/// let resolved = sc.correlate(Rtc::from_raw(10_500_000), None).unwrap();
/// ```
///
/// **Traces:** P5-02
pub struct StreamingTimeCorrelator {
    /// Per-channel reference points, each vec sorted by RTC.
    channel_refs: BTreeMap<u16, Vec<StreamingRef>>,
    /// Maximum age in nanoseconds. References older than
    /// `latest_rtc - max_age_ns` are evicted.
    max_age_ns: u64,
    /// The highest RTC seen so far (for eviction).
    latest_rtc: Option<Rtc>,
    /// Total reference points across all channels.
    total_refs: usize,
    /// Total references evicted since creation.
    total_evicted: usize,
}

/// A reference point in the streaming correlator.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    /// `max_age_ns` is the maximum age in nanoseconds. Reference points
    /// whose RTC is more than this far behind the latest-seen RTC are
    /// evicted on the next insert.
    ///
    /// A typical value for 1 Hz time packets with 30 seconds of lookback:
    /// `30_000_000_000` (30 billion nanoseconds).
    pub fn new(max_age_ns: u64) -> Self {
        Self {
            channel_refs: BTreeMap::new(),
            max_age_ns,
            latest_rtc: None,
            total_refs: 0,
            total_evicted: 0,
        }
    }

    /// Insert a reference point and evict stale entries.
    ///
    /// The reference is inserted into the per-channel sorted vec. Then,
    /// any references on any channel whose RTC is older than
    /// `latest_rtc - max_age_ns` are removed.
    pub fn add_reference(&mut self, channel_id: u16, rtc: Rtc, time: AbsoluteTime) {
        // Update latest RTC
        match self.latest_rtc {
            Some(latest) if rtc > latest => self.latest_rtc = Some(rtc),
            None => self.latest_rtc = Some(rtc),
            _ => {}
        }

        // Insert into per-channel sorted vec
        let refs = self.channel_refs.entry(channel_id).or_default();
        let pos = refs
            .binary_search_by_key(&rtc, |r| r.rtc)
            .unwrap_or_else(|e| e);
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
    pub fn add_reference_f2(
        &mut self,
        channel_id: u16,
        rtc: Rtc,
        network_time: &crate::network_time::NetworkTime,
        leap_table: &crate::network_time::LeapSecondTable,
    ) -> Result<()> {
        let abs_time = match network_time {
            crate::network_time::NetworkTime::Ntp(ntp) => ntp.to_absolute()?,
            crate::network_time::NetworkTime::Ptp(ptp) => {
                let utc_secs = ptp.to_utc_seconds(leap_table.offset_at_tai(ptp.seconds));
                let (year, doy, hour, minute, second) =
                    crate::network_time::unix_seconds_to_ymd_pub(utc_secs);
                let mut abs = AbsoluteTime::new(doy, hour, minute, second, ptp.nanoseconds)?;
                abs.year = Some(year);
                abs
            }
        };
        self.add_reference(channel_id, rtc, abs_time);
        Ok(())
    }

    /// Correlate an RTC value to absolute time using the nearest reference
    /// in the sliding window.
    ///
    /// If `channel_id` is `Some(id)`, only references from that channel
    /// are considered.
    pub fn correlate(&self, target_rtc: Rtc, channel_id: Option<u16>) -> Result<AbsoluteTime> {
        let nearest = match channel_id {
            Some(id) => self.nearest_for_channel(target_rtc, id)?,
            None => self.nearest_any(target_rtc)?,
        };

        if target_rtc >= nearest.rtc {
            let delta_nanos = nearest.rtc.elapsed_nanos(target_rtc);
            Ok(nearest.time.add_nanos(delta_nanos))
        } else {
            let delta_nanos = target_rtc.elapsed_nanos(nearest.rtc);
            Ok(nearest.time.sub_nanos(delta_nanos))
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
            let idx = refs
                .binary_search_by_key(&target, |r| r.rtc)
                .unwrap_or_else(|e| e);

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

        let idx = refs
            .binary_search_by_key(&target, |r| r.rtc)
            .unwrap_or_else(|e| e);

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

        for refs in self.channel_refs.values_mut() {
            let before = refs.len();
            refs.retain(|r| r.rtc.as_raw() >= cutoff_raw);
            let evicted = before - refs.len();
            self.total_refs -= evicted;
            self.total_evicted += evicted;
        }
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
#[path = "streaming_tests.rs"]
mod streaming_tests;
