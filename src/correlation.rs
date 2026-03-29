//! RTC-to-absolute-time correlation engine.
//!
//! Correlates 48-bit RTC values to absolute clock time using reference points
//! from Time Data Format 1 packets that pair each RTC with an absolute time.
//!
//! # Performance
//!
//! Reference points are stored in two structures:
//! - A flat `Vec` sorted by RTC for efficient any-channel nearest-point lookup (O(log n))
//! - A per-channel `BTreeMap<u16, Vec<ReferencePoint>>` for O(log n) channel-filtered lookup
//!
//! This eliminates the O(n) linear scans from v0.4.0 for `nearest_for_channel`,
//! `detect_time_jump`, `drift_ppm`, and `detect_rtc_resets`.
//!
//! # Requirement Traceability
//!
//! | Requirement | Description |
//! |-------------|-------------|
//! | L3-COR-001  | `ReferencePoint` struct |
//! | L3-COR-002  | `TimeCorrelator` struct |
//! | L3-COR-003  | Sorted insert via binary search |
//! | L3-COR-004  | `correlate` with nearest-point interpolation |
//! | L3-COR-005  | Channel-filtered correlation |
//! | L3-COR-006  | `TimeJump` detection struct |
//! | L3-COR-007  | `detect_time_jump` algorithm |
//! | P4-01       | Channel-indexed O(log n) lookup |
//! | P4-02       | Per-channel cached access |

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::absolute::AbsoluteTime;
use crate::error::{Result, TimeError};
use crate::rtc::Rtc;

/// A reference point pairing an RTC value with an absolute time on a channel.
///
/// **Traces:** L3-COR-001 ← L2-COR-001
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReferencePoint {
    /// Channel ID of the time source.
    pub channel_id: u16,
    /// RTC value at the instant the absolute time was valid.
    pub rtc: Rtc,
    /// Decoded absolute time from the time packet.
    pub time: AbsoluteTime,
}

/// A detected discontinuity in absolute time progression.
///
/// **Traces:** L3-COR-006 ← L2-COR-006
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeJump {
    /// Index of the later reference point in the correlator's list.
    pub index: usize,
    /// Channel ID where the jump was detected.
    pub channel_id: u16,
    /// Expected absolute time (nanoseconds of day) based on RTC progression.
    pub expected_nanos: u64,
    /// Actual absolute time (nanoseconds of day) from the reference point.
    pub actual_nanos: u64,
    /// Signed difference: actual − expected (positive = jump forward).
    pub delta_nanos: i64,
}

/// A detected RTC counter reset (as opposed to a 48-bit wrap).
///
/// A reset is flagged when the RTC value decreases by more than a plausible
/// wrap amount while absolute time continues to advance normally.
///
/// **Traces:** GAP-07
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtcReset {
    /// Index of the reference point after the reset.
    pub index: usize,
    /// Channel ID where the reset was detected.
    pub channel_id: u16,
    /// RTC value before the reset.
    pub rtc_before: Rtc,
    /// RTC value after the reset.
    pub rtc_after: Rtc,
    /// Absolute time before the reset.
    pub time_before: AbsoluteTime,
    /// Absolute time after the reset.
    pub time_after: AbsoluteTime,
}

/// Correlates RTC values to absolute time using time data reference points.
///
/// Reference points are stored in a flat RTC-sorted vec (for any-channel lookup)
/// and a per-channel index (for O(log n) channel-filtered operations).
///
/// **Traces:** L3-COR-002 ← L2-COR-001, P4-01
pub struct TimeCorrelator {
    /// All reference points sorted by RTC (for `nearest_any`).
    references: Vec<ReferencePoint>,
    /// Per-channel reference points, each vec sorted by RTC.
    /// **Traces:** P4-01, P4-02
    channel_index: BTreeMap<u16, Vec<ReferencePoint>>,
    /// Maximum out-of-order window in nanoseconds.
    ooo_window_ns: Option<u64>,
}

impl TimeCorrelator {
    /// Create a new empty correlator with unbounded out-of-order tolerance.
    pub fn new() -> Self {
        Self {
            references: Vec::new(),
            channel_index: BTreeMap::new(),
            ooo_window_ns: None,
        }
    }

    /// Create a correlator with a bounded out-of-order window.
    ///
    /// Pre-105 files may have packets 5+ seconds out of order, so pass `None`
    /// for unbounded. Post-105 files are guaranteed within ~1.1 seconds
    /// (100 ms buffer + 1 sec write deadline), so 2 seconds is a safe default.
    ///
    /// **Traces:** P2-02
    pub fn with_ooo_window(ooo_window_ns: Option<u64>) -> Self {
        Self {
            references: Vec::new(),
            channel_index: BTreeMap::new(),
            ooo_window_ns,
        }
    }

    /// Default out-of-order window for post-105 files: 2 seconds in nanoseconds.
    pub const DEFAULT_OOO_WINDOW_NS: u64 = 2_000_000_000;

    /// Number of reference points currently stored.
    pub fn len(&self) -> usize {
        self.references.len()
    }

    /// Whether the correlator has no reference points.
    pub fn is_empty(&self) -> bool {
        self.references.is_empty()
    }

    /// Insert a reference point, maintaining RTC sort order in both the
    /// flat list and the per-channel index.
    ///
    /// **Traces:** L3-COR-003 ← L2-COR-002, P4-01
    pub fn add_reference(&mut self, channel_id: u16, rtc: Rtc, time: AbsoluteTime) {
        let point = ReferencePoint {
            channel_id,
            rtc,
            time,
        };

        // Insert into flat sorted list
        let pos = self
            .references
            .binary_search_by_key(&rtc, |r| r.rtc)
            .unwrap_or_else(|e| e);
        self.references.insert(pos, point);

        // Insert into per-channel sorted list
        let channel_refs = self.channel_index.entry(channel_id).or_default();
        let ch_pos = channel_refs
            .binary_search_by_key(&rtc, |r| r.rtc)
            .unwrap_or_else(|e| e);
        channel_refs.insert(ch_pos, point);
    }

    /// Insert a Format 2 (Network Time) reference point.
    ///
    /// Converts the NTP or PTP time to `AbsoluteTime` and delegates to
    /// `add_reference`. For PTP sources, the leap-second table is used
    /// to convert TAI to UTC.
    ///
    /// **Traces:** L3-F2-014 ← L2-F2COR-001..003
    pub fn add_reference_f2(
        &mut self,
        channel_id: u16,
        rtc: Rtc,
        network_time: &crate::network_time::NetworkTime,
        leap_table: &crate::network_time::LeapSecondTable,
    ) -> crate::error::Result<()> {
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

    /// Correlate an RTC value to absolute time using the nearest reference point.
    ///
    /// If `channel_id` is `Some(id)`, only reference points from that channel
    /// are considered (O(log n) via per-channel index).
    ///
    /// **Traces:** L3-COR-004, L3-COR-005 ← L2-COR-003..L2-COR-005
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

    /// Find the nearest reference point (any channel) by RTC.
    ///
    /// O(log n) via binary search on the flat sorted list.
    fn nearest_any(&self, target: Rtc) -> Result<&ReferencePoint> {
        if self.references.is_empty() {
            return Err(TimeError::NoReferencePoint);
        }

        let idx = self
            .references
            .binary_search_by_key(&target, |r| r.rtc)
            .unwrap_or_else(|e| e);

        if idx == 0 {
            Ok(&self.references[0])
        } else if idx >= self.references.len() {
            Ok(&self.references[self.references.len() - 1])
        } else {
            Ok(self.closer_ref(&self.references[idx - 1], &self.references[idx], target))
        }
    }

    /// Find the nearest reference point for a specific channel.
    ///
    /// O(log n) via binary search on the per-channel sorted list.
    ///
    /// **Traces:** L3-COR-005, P4-01
    fn nearest_for_channel(&self, target: Rtc, channel_id: u16) -> Result<&ReferencePoint> {
        let ch_refs = self
            .channel_index
            .get(&channel_id)
            .ok_or(TimeError::NoReferencePoint)?;

        if ch_refs.is_empty() {
            return Err(TimeError::NoReferencePoint);
        }

        let idx = ch_refs
            .binary_search_by_key(&target, |r| r.rtc)
            .unwrap_or_else(|e| e);

        if idx == 0 {
            Ok(&ch_refs[0])
        } else if idx >= ch_refs.len() {
            Ok(&ch_refs[ch_refs.len() - 1])
        } else {
            Ok(self.closer_ref(&ch_refs[idx - 1], &ch_refs[idx], target))
        }
    }

    /// Find the global index of a reference point in the flat `references` list.
    ///
    /// Matches on all fields (channel_id, rtc, AND time) to correctly handle
    /// duplicate RTC values on the same channel.
    fn global_index_of(&self, point: &ReferencePoint) -> usize {
        self.references
            .iter()
            .position(|r| {
                r.channel_id == point.channel_id
                    && r.rtc == point.rtc
                    && r.time == point.time
            })
            .unwrap_or(0)
    }

    /// Return whichever of two reference points is closer to the target RTC.
    fn closer_ref<'a>(
        &self,
        a: &'a ReferencePoint,
        b: &'a ReferencePoint,
        target: Rtc,
    ) -> &'a ReferencePoint {
        let da = a.rtc.elapsed_ticks(target).min(target.elapsed_ticks(a.rtc));
        let db = b.rtc.elapsed_ticks(target).min(target.elapsed_ticks(b.rtc));
        if da <= db {
            a
        } else {
            b
        }
    }

    /// Detect time jumps on a specific channel.
    ///
    /// Uses the per-channel index for O(m) iteration where m is the number
    /// of reference points on this channel (no cross-channel filtering).
    ///
    /// **Traces:** L3-COR-007 ← L2-COR-006, P4-02
    pub fn detect_time_jump(&self, channel_id: u16, threshold_ns: u64) -> Vec<TimeJump> {
        let ch_refs = match self.channel_index.get(&channel_id) {
            Some(refs) => refs,
            None => return Vec::new(),
        };

        let mut jumps = Vec::new();

        for window in ch_refs.windows(2) {
            let prev = &window[0];
            let curr = &window[1];

            let rtc_delta_nanos = prev.rtc.elapsed_nanos(curr.rtc);
            let expected_nanos = prev.time.total_nanos_of_day() + rtc_delta_nanos;
            let actual_nanos = curr.time.total_nanos_of_day();

            let delta = actual_nanos as i64 - expected_nanos as i64;
            if delta.unsigned_abs() > threshold_ns {
                let global_idx = self.global_index_of(curr);

                jumps.push(TimeJump {
                    index: global_idx,
                    channel_id,
                    expected_nanos,
                    actual_nanos,
                    delta_nanos: delta,
                });
            }
        }

        jumps
    }

    /// Access all reference points (sorted by RTC across all channels).
    pub fn references(&self) -> &[ReferencePoint] {
        &self.references
    }

    /// Access reference points for a specific channel (sorted by RTC).
    ///
    /// Returns an empty slice if no reference points exist for the channel.
    ///
    /// **Traces:** P4-01
    pub fn channel_references(&self, channel_id: u16) -> &[ReferencePoint] {
        self.channel_index
            .get(&channel_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Returns the set of channel IDs that have reference points.
    pub fn channel_ids(&self) -> Vec<u16> {
        self.channel_index.keys().copied().collect()
    }

    /// Estimate the clock drift in parts-per-million between two reference points
    /// on the given channel.
    ///
    /// Uses the per-channel index for direct iteration (no filtering).
    ///
    /// Returns `None` if fewer than 2 reference points exist for the channel.
    ///
    /// **Traces:** P4-02
    pub fn drift_ppm(&self, channel_id: u16) -> Option<f64> {
        let ch_refs = self.channel_index.get(&channel_id)?;

        if ch_refs.len() < 2 {
            return None;
        }

        let mut total_drift = 0.0f64;
        let mut pair_count = 0u64;

        for pair in ch_refs.windows(2) {
            let r0 = &pair[0];
            let r1 = &pair[1];

            let rtc_delta_ns = r0.rtc.elapsed_nanos(r1.rtc) as f64;
            if rtc_delta_ns == 0.0 {
                continue;
            }

            let abs_ns_0 = ((r0.time.day_of_year as u64).saturating_sub(1)) * 86_400_000_000_000
                + r0.time.total_nanos_of_day();
            let abs_ns_1 = ((r1.time.day_of_year as u64).saturating_sub(1)) * 86_400_000_000_000
                + r1.time.total_nanos_of_day();
            let abs_delta_ns = abs_ns_1 as f64 - abs_ns_0 as f64;

            if abs_delta_ns == 0.0 {
                continue;
            }

            let drift = (rtc_delta_ns - abs_delta_ns) / abs_delta_ns * 1_000_000.0;
            total_drift += drift;
            pair_count += 1;
        }

        if pair_count == 0 {
            None
        } else {
            Some(total_drift / pair_count as f64)
        }
    }

    /// Detect RTC counter resets on a specific channel.
    ///
    /// Uses the per-channel index, sorted by absolute time, to detect
    /// points where absolute time advances but RTC goes backward.
    ///
    /// **Traces:** GAP-07, P4-02
    pub fn detect_rtc_resets(&self, channel_id: u16) -> Vec<RtcReset> {
        let ch_refs = match self.channel_index.get(&channel_id) {
            Some(refs) => refs,
            None => return Vec::new(),
        };

        // Sort by absolute time to see temporal order
        let mut sorted: Vec<&ReferencePoint> = ch_refs.iter().collect();
        sorted.sort_by_key(|r| {
            ((r.time.day_of_year as u64).saturating_sub(1)) * 86_400_000_000_000
                + r.time.total_nanos_of_day()
        });

        let mut resets = Vec::new();

        for window in sorted.windows(2) {
            let prev = window[0];
            let curr = window[1];

            if curr.rtc < prev.rtc {
                resets.push(RtcReset {
                    index: self.global_index_of(curr),
                    channel_id,
                    rtc_before: prev.rtc,
                    rtc_after: curr.rtc,
                    time_before: prev.time,
                    time_after: curr.time,
                });
            }
        }

        resets
    }

    /// Returns the configured out-of-order window, if any.
    pub fn ooo_window_ns(&self) -> Option<u64> {
        self.ooo_window_ns
    }
}

impl Default for TimeCorrelator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "correlation_tests.rs"]
mod correlation_tests;
