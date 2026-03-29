//! Recording Event (Data Type 0x02) time extraction.
//!
//! Recording Event packets carry timestamps that can serve as additional
//! correlation reference points. Each event packet contains an RTC in its
//! primary header (like all packets) and may contain absolute time in its
//! secondary header if present.
//!
//! # Event Types
//!
//! | Index | Event | Time Relevance |
//! |-------|-------|---------------|
//! | 0x01  | Recording started | First RTC reference |
//! | 0x02  | Recording stopped | Last RTC reference |
//! | 0x03  | Recording overrun | Potential time gap |
//! | 0x04  | Index point | User-defined marker |
//!
//! # Usage
//!
//! Recording events don't carry their own time payload (unlike Type 0x11/0x12
//! time packets), but they can be used as supplementary reference points
//! when a secondary header with absolute time is present.
//!
//! # Requirement Traceability
//!
//! | Requirement | Description |
//! |-------------|-------------|
//! | GAP-08      | Recording Events (0x02) time data |

use crate::absolute::AbsoluteTime;
use crate::rtc::Rtc;

/// Recording event types from Data Type 0x02 packets.
///
/// **Traces:** GAP-08
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecordingEventType {
    /// Recording started (index 0x01).
    Started,
    /// Recording stopped (index 0x02).
    Stopped,
    /// Recording overrun — data may have been lost (index 0x03).
    Overrun,
    /// User-defined index point (index 0x04+).
    IndexPoint(u8),
    /// Reserved or unknown event type.
    Reserved(u8),
}

impl RecordingEventType {
    /// Decode the event type from the Recording Event Index field.
    pub fn from_index(index: u8) -> Self {
        match index {
            0x01 => RecordingEventType::Started,
            0x02 => RecordingEventType::Stopped,
            0x03 => RecordingEventType::Overrun,
            idx @ 0x04..=0x0F => RecordingEventType::IndexPoint(idx),
            other => RecordingEventType::Reserved(other),
        }
    }

    /// Returns `true` if this event type may indicate a time discontinuity.
    ///
    /// Stopped and Overrun events suggest the recorder's time reference
    /// may not be continuous across the boundary.
    pub fn may_cause_time_gap(self) -> bool {
        matches!(
            self,
            RecordingEventType::Stopped | RecordingEventType::Overrun
        )
    }
}

/// A parsed recording event with time context.
///
/// Combines the event type with the packet's RTC and optional secondary
/// header absolute time.
///
/// **Traces:** GAP-08
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordingEvent {
    /// The type of recording event.
    pub event_type: RecordingEventType,
    /// Channel ID from the packet header.
    pub channel_id: u16,
    /// RTC from the packet header.
    pub rtc: Rtc,
    /// Absolute time from the secondary header, if present.
    pub absolute_time: Option<AbsoluteTime>,
}

impl RecordingEvent {
    /// Create a recording event from packet header fields.
    ///
    /// `event_index` is the Recording Event Index from the CSDW.
    /// `abs_time` is the decoded secondary header time, if the packet
    /// has a secondary header (Packet Flag bit 0 set).
    pub fn new(
        event_index: u8,
        channel_id: u16,
        rtc: Rtc,
        abs_time: Option<AbsoluteTime>,
    ) -> Self {
        Self {
            event_type: RecordingEventType::from_index(event_index),
            channel_id,
            rtc,
            absolute_time: abs_time,
        }
    }

    /// Whether this event can serve as a correlation reference point.
    ///
    /// Only events with absolute time from a secondary header can be
    /// used as reference points for RTC-to-absolute-time correlation.
    pub fn has_reference_time(&self) -> bool {
        self.absolute_time.is_some()
    }
}

impl core::fmt::Display for RecordingEventType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RecordingEventType::Started => write!(f, "Recording Started"),
            RecordingEventType::Stopped => write!(f, "Recording Stopped"),
            RecordingEventType::Overrun => write!(f, "Recording Overrun"),
            RecordingEventType::IndexPoint(idx) => write!(f, "Index Point {}", idx),
            RecordingEventType::Reserved(v) => write!(f, "Reserved(0x{:02X})", v),
        }
    }
}

#[cfg(test)]
#[path = "recording_event_tests.rs"]
mod recording_event_tests;
