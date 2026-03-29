//! RTC-to-absolute-time correlation engine.
//!
//! Correlates 48-bit RTC values to absolute clock time using reference points
//! from Time Data Format 1 packets that pair each RTC with an absolute time.
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

extern crate alloc;
use alloc::vec::Vec;

use crate::absolute::AbsoluteTime;
use crate::error::{Result, TimeError};
use crate::rtc::Rtc;

/// A reference point pairing an RTC value with an absolute time on a channel.
///
/// **Traces:** L3-COR-001 ← L2-COR-001
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

/// Correlates RTC values to absolute time using time data reference points.
///
/// Reference points are kept sorted by RTC for efficient nearest-point lookup.
///
/// **Traces:** L3-COR-002 ← L2-COR-001
pub struct TimeCorrelator {
    references: Vec<ReferencePoint>,
}

impl TimeCorrelator {
    /// Create a new empty correlator.
    pub fn new() -> Self {
        Self {
            references: Vec::new(),
        }
    }

    /// Number of reference points currently stored.
    pub fn len(&self) -> usize {
        self.references.len()
    }

    /// Whether the correlator has no reference points.
    pub fn is_empty(&self) -> bool {
        self.references.is_empty()
    }

    /// Insert a reference point, maintaining RTC sort order.
    ///
    /// **Traces:** L3-COR-003 ← L2-COR-002
    pub fn add_reference(&mut self, channel_id: u16, rtc: Rtc, time: AbsoluteTime) {
        let point = ReferencePoint {
            channel_id,
            rtc,
            time,
        };
        let pos = self
            .references
            .binary_search_by_key(&rtc, |r| r.rtc)
            .unwrap_or_else(|e| e);
        self.references.insert(pos, point);
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
                let utc_secs = ptp.to_utc_seconds(
                    leap_table.offset_at_tai(ptp.seconds),
                );
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
    /// are considered.
    ///
    /// **Traces:** L3-COR-004, L3-COR-005 ← L2-COR-003..L2-COR-005
    pub fn correlate(
        &self,
        target_rtc: Rtc,
        channel_id: Option<u16>,
    ) -> Result<AbsoluteTime> {
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
    fn nearest_any(&self, target: Rtc) -> Result<&ReferencePoint> {
        if self.references.is_empty() {
            return Err(TimeError::NoReferencePoint);
        }

        let idx = self
            .references
            .binary_search_by_key(&target, |r| r.rtc)
            .unwrap_or_else(|e| e);

        // Check the insertion point and its neighbor for closest
        if idx == 0 {
            Ok(&self.references[0])
        } else if idx >= self.references.len() {
            Ok(&self.references[self.references.len() - 1])
        } else {
            // Check both idx-1 and idx
            Ok(self.closer_of(idx - 1, idx, target))
        }
    }

    /// Find the nearest reference point for a specific channel.
    ///
    /// **Traces:** L3-COR-005
    fn nearest_for_channel(&self, target: Rtc, channel_id: u16) -> Result<&ReferencePoint> {
        let filtered: Vec<&ReferencePoint> = self
            .references
            .iter()
            .filter(|r| r.channel_id == channel_id)
            .collect();

        if filtered.is_empty() {
            return Err(TimeError::NoReferencePoint);
        }

        let mut best = filtered[0];
        let mut best_dist = best.rtc.elapsed_ticks(target).min(target.elapsed_ticks(best.rtc));

        for &r in &filtered[1..] {
            let dist = r.rtc.elapsed_ticks(target).min(target.elapsed_ticks(r.rtc));
            if dist < best_dist {
                best = r;
                best_dist = dist;
            }
        }

        Ok(best)
    }

    /// Return whichever of two indexed reference points is closer to target.
    fn closer_of(&self, a: usize, b: usize, target: Rtc) -> &ReferencePoint {
        let da = self.references[a]
            .rtc
            .elapsed_ticks(target)
            .min(target.elapsed_ticks(self.references[a].rtc));
        let db = self.references[b]
            .rtc
            .elapsed_ticks(target)
            .min(target.elapsed_ticks(self.references[b].rtc));
        if da <= db {
            &self.references[a]
        } else {
            &self.references[b]
        }
    }

    /// Detect time jumps on a specific channel.
    ///
    /// A jump is flagged when the absolute time difference between consecutive
    /// reference points on the same channel deviates from the RTC-predicted
    /// delta by more than `threshold_ns` nanoseconds.
    ///
    /// **Traces:** L3-COR-007 ← L2-COR-006
    pub fn detect_time_jump(&self, channel_id: u16, threshold_ns: u64) -> Vec<TimeJump> {
        let filtered: Vec<(usize, &ReferencePoint)> = self
            .references
            .iter()
            .enumerate()
            .filter(|(_, r)| r.channel_id == channel_id)
            .collect();

        let mut jumps = Vec::new();

        for window in filtered.windows(2) {
            let (idx_prev, prev) = window[0];
            let (idx_curr, curr) = window[1];
            let _ = idx_prev; // prev index not needed in output

            let rtc_delta_nanos = prev.rtc.elapsed_nanos(curr.rtc);
            let expected_nanos = prev.time.total_nanos_of_day() + rtc_delta_nanos;
            let actual_nanos = curr.time.total_nanos_of_day();

            let delta = actual_nanos as i64 - expected_nanos as i64;
            if delta.unsigned_abs() > threshold_ns {
                jumps.push(TimeJump {
                    index: idx_curr,
                    channel_id,
                    expected_nanos,
                    actual_nanos,
                    delta_nanos: delta,
                });
            }
        }

        jumps
    }

    /// Access all reference points (sorted by RTC).
    pub fn references(&self) -> &[ReferencePoint] {
        &self.references
    }

    /// Estimate the clock drift in parts-per-million between two reference points
    /// on the given channel.
    ///
    /// Compares the RTC progression against absolute time progression for
    /// consecutive same-channel reference pairs and returns the average drift.
    /// A positive value means the RTC is running fast relative to the reference
    /// clock; negative means slow.
    ///
    /// Returns `None` if fewer than 2 reference points exist for the channel.
    pub fn drift_ppm(&self, channel_id: u16) -> Option<f64> {
        let channel_refs: Vec<&ReferencePoint> = self
            .references
            .iter()
            .filter(|r| r.channel_id == channel_id)
            .collect();

        if channel_refs.len() < 2 {
            return None;
        }

        let mut total_drift = 0.0f64;
        let mut pair_count = 0u64;

        for pair in channel_refs.windows(2) {
            let r0 = pair[0];
            let r1 = pair[1];

            let rtc_delta_ns = r0.rtc.elapsed_nanos(r1.rtc) as f64;
            if rtc_delta_ns == 0.0 {
                continue;
            }

            // Compute expected absolute time delta in nanoseconds
            let abs_ns_0 = ((r0.time.day_of_year as u64).saturating_sub(1))
                * 86_400_000_000_000
                + r0.time.total_nanos_of_day();
            let abs_ns_1 = ((r1.time.day_of_year as u64).saturating_sub(1))
                * 86_400_000_000_000
                + r1.time.total_nanos_of_day();
            let abs_delta_ns = abs_ns_1 as f64 - abs_ns_0 as f64;

            if abs_delta_ns == 0.0 {
                continue;
            }

            // drift = (rtc_elapsed - abs_elapsed) / abs_elapsed * 1_000_000
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
}

impl Default for TimeCorrelator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "correlation_tests.rs"]
mod correlation_tests;
