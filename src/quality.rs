//! Time quality metrics for correlation health assessment.
//!
//! Computes quality indicators from a set of reference points to help
//! assess the reliability of time correlation.
//!
//! # Requirement Traceability
//!
//! | Requirement | Description |
//! |-------------|-------------|
//! | P5-04       | Time quality metrics |

use crate::correlation::ReferencePoint;

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/// Quality assessment of a correlator's reference point set.
///
/// Computed from a snapshot of reference points to help assess whether
/// correlation results are trustworthy.
///
/// **Traces:** P5-04
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct TimeQuality {
    /// Total number of reference points.
    pub total_refs: usize,
    /// Number of distinct channels with reference points.
    pub channel_count: usize,
    /// Per-channel reference point counts.
    pub refs_per_channel: Vec<(u16, usize)>,
    /// Maximum RTC gap between consecutive reference points (in nanoseconds)
    /// across all channels. `None` if fewer than 2 reference points.
    pub max_rtc_gap_ns: Option<u64>,
    /// Minimum RTC gap between consecutive reference points (in nanoseconds).
    /// `None` if fewer than 2 reference points.
    pub min_rtc_gap_ns: Option<u64>,
    /// Average reference point density in references per second,
    /// computed from the total RTC span. `None` if fewer than 2 reference points.
    pub ref_density_per_sec: Option<f64>,
    /// Per-channel drift estimates in parts-per-million.
    /// Only populated for channels with 2+ reference points.
    pub drift_ppm_per_channel: Vec<(u16, f64)>,
    /// RTC span covered by reference points (latest - earliest, in nanoseconds).
    /// `None` if fewer than 2 reference points.
    pub rtc_span_ns: Option<u64>,
}

/// Compute quality metrics from a set of reference points.
///
/// This function works with reference points from either
/// [`TimeCorrelator::references()`](crate::correlation::TimeCorrelator::references)
/// or any other sorted reference point slice.
///
/// # Example
///
/// ```
/// use irig106_time::*;
/// use irig106_time::quality::compute_quality;
///
/// let mut c = TimeCorrelator::new();
/// c.add_reference(1, Rtc::from_raw(10_000_000),
///     AbsoluteTime::new(100, 12, 0, 0, 0).unwrap());
/// c.add_reference(1, Rtc::from_raw(20_000_000),
///     AbsoluteTime::new(100, 12, 0, 1, 0).unwrap());
///
/// let q = compute_quality(c.references());
/// assert_eq!(q.total_refs, 2);
/// assert_eq!(q.channel_count, 1);
/// ```
pub fn compute_quality(refs: &[ReferencePoint]) -> TimeQuality {
    if refs.is_empty() {
        return TimeQuality {
            total_refs: 0,
            channel_count: 0,
            refs_per_channel: Vec::new(),
            max_rtc_gap_ns: None,
            min_rtc_gap_ns: None,
            ref_density_per_sec: None,
            drift_ppm_per_channel: Vec::new(),
            rtc_span_ns: None,
        };
    }

    // Per-channel grouping
    let mut by_channel: BTreeMap<u16, Vec<&ReferencePoint>> = BTreeMap::new();
    for r in refs {
        by_channel.entry(r.channel_id).or_default().push(r);
    }

    let refs_per_channel: Vec<(u16, usize)> =
        by_channel.iter().map(|(&ch, v)| (ch, v.len())).collect();

    // Global RTC span
    let min_rtc = refs.iter().map(|r| r.rtc).min().unwrap();
    let max_rtc = refs.iter().map(|r| r.rtc).max().unwrap();
    let span_ticks = min_rtc.elapsed_ticks(max_rtc);
    let span_ns = span_ticks * 100;

    // Gaps (across all consecutive pairs in RTC order)
    let mut max_gap_ns: Option<u64> = None;
    let mut min_gap_ns: Option<u64> = None;

    if refs.len() >= 2 {
        for window in refs.windows(2) {
            let gap = window[0].rtc.elapsed_nanos(window[1].rtc);
            max_gap_ns = Some(max_gap_ns.map_or(gap, |m: u64| m.max(gap)));
            min_gap_ns = Some(min_gap_ns.map_or(gap, |m: u64| m.min(gap)));
        }
    }

    // Density
    let ref_density = if span_ns > 0 && refs.len() >= 2 {
        Some((refs.len() as f64) / (span_ns as f64 / 1_000_000_000.0))
    } else {
        None
    };

    // Per-channel drift
    let mut drift_ppm_per_channel = Vec::new();
    for (&ch, ch_refs) in &by_channel {
        if ch_refs.len() < 2 {
            continue;
        }
        let mut total_drift = 0.0f64;
        let mut count = 0u64;
        for pair in ch_refs.windows(2) {
            let r0 = pair[0];
            let r1 = pair[1];
            let rtc_ns = r0.rtc.elapsed_nanos(r1.rtc) as f64;
            if rtc_ns == 0.0 {
                continue;
            }
            let abs_0 = ((r0.time.day_of_year as u64).saturating_sub(1)) * 86_400_000_000_000
                + r0.time.total_nanos_of_day();
            let abs_1 = ((r1.time.day_of_year as u64).saturating_sub(1)) * 86_400_000_000_000
                + r1.time.total_nanos_of_day();
            let abs_ns = abs_1 as f64 - abs_0 as f64;
            if abs_ns == 0.0 {
                continue;
            }
            total_drift += (rtc_ns - abs_ns) / abs_ns * 1_000_000.0;
            count += 1;
        }
        if count > 0 {
            drift_ppm_per_channel.push((ch, total_drift / count as f64));
        }
    }

    TimeQuality {
        total_refs: refs.len(),
        channel_count: by_channel.len(),
        refs_per_channel,
        max_rtc_gap_ns: max_gap_ns,
        min_rtc_gap_ns: min_gap_ns,
        ref_density_per_sec: ref_density,
        drift_ppm_per_channel,
        rtc_span_ns: if refs.len() >= 2 { Some(span_ns) } else { None },
    }
}

#[cfg(test)]
#[path = "quality_tests.rs"]
mod quality_tests;
