//! Allocation-free native session/span tracking in recording order.
//!
//! Uses the same nearest48-bit unwrap and strict late-bound reset calculation
//! as TimelineBuilder. It holds one active span; callers may stream closed spans
//! to a report sink. No reference admission, channel association, calendar/year
//! inference or quality/drift assessment is performed by this counter-only API.
use crate::{
    Result, Rtc, TimeError,
    rtc::{CounterOrder, SessionRtc},
    timeline::{OfflinePolicy, PacketKey},
};

/// Why a reader began this session; inferred boundaries are policy, not wire facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStart {
    /// First observed packet.
    Initial,
    /// Caller supplied a boundary.
    Explicit,
    /// Backward displacement strictly exceeded the configured late bound.
    Inferred,
}
/// Observed header span across all channels in one native session.
/// First/last are file-order observations; low/high are unwrapped extrema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionSpan {
    /// Boundary provenance; does not establish data/time-channel association.
    pub start: SessionStart,
    /// First observed header in this session, including its one-based session ID.
    pub first: PacketKey,
    /// Latest observed header, including late packets.
    pub last: PacketKey,
    /// Minimum observed unwrapped RTC tick (100ns units).
    pub low_ticks: i128,
    /// High-water unwrapped RTC tick, never lowered by late packets.
    pub high_ticks: i128,
    /// All observed packets, not just time packets or admitted references.
    pub packets: u64,
}
/// One atomic progress result; a closed span is returned once at a boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionUpdate {
    /// Native query position of this observation.
    pub key: PacketKey,
    /// Prior completed span, if this packet began a new session.
    pub closed: Option<SessionSpan>,
}
/// Bounded session-only projection for recording readers.
///
/// Snapshot the native policy's late bound (default1100ms) and infer_sessions.
/// No whole recording, seen-ID set, references or closed spans are retained.
/// IDs must strictly increase, as for StreamTimeline. Errors leave state intact.
/// The CLI's per-channel30s/run reset heuristic is a separate compatibility view.
/// Partial Traces: L3-COR-105/107; L3-CLI-104 (bounded session spans only).
#[derive(Debug, Clone)]
pub struct SessionClock {
    active: Option<SessionSpan>,
    late_ns: u64,
    infer_sessions: bool,
}
impl SessionClock {
    /// Validate the supplied native policy; freeze its two session settings.
    pub fn new(policy: &OfflinePolicy) -> Result<Self> {
        policy.validate()?;
        Ok(Self {
            active: None,
            late_ns: policy.late_ns,
            infer_sessions: policy.infer_sessions,
        })
    }
    /// Actual frozen late-packet bound in ns.
    pub fn late_ns(&self) -> u64 {
        self.late_ns
    }
    /// Whether backward displacements beyond the bound infer new sessions.
    pub fn infer_sessions(&self) -> bool {
        self.infer_sessions
    }
    /// Current span, including the final span at EOF; None before first packet.
    pub fn active(&self) -> Option<&SessionSpan> {
        self.active.as_ref()
    }
    /// Observe a header; explicit caller boundaries take priority over inference.
    pub fn observe(&mut self, id: u64, rtc: Rtc, boundary: bool) -> Result<SessionUpdate> {
        if self.active.is_some_and(|s| id <= s.last.packet) {
            return Err(TimeError::InvalidTimelineInput(
                "packet identities must increase",
            ));
        }
        let next = position(
            self.active.map(|s| (s.first.session, s.high_ticks)),
            rtc,
            id,
            boundary,
            self.late_ns,
            self.infer_sessions,
        )?;
        let key = next.key;
        let starts = self.active.is_none_or(|s| s.first.session != key.session);
        let span = if starts {
            SessionSpan {
                start: if self.active.is_none() {
                    SessionStart::Initial
                } else if boundary {
                    SessionStart::Explicit
                } else {
                    SessionStart::Inferred
                },
                first: key,
                last: key,
                low_ticks: key.ticks,
                high_ticks: key.ticks,
                packets: 1,
            }
        } else {
            let mut span = self
                .active
                .ok_or(TimeError::InvalidTimelineInput("missing session span"))?;
            span.last = key;
            span.low_ticks = span.low_ticks.min(key.ticks);
            span.high_ticks = span.high_ticks.max(key.ticks);
            span.packets = span
                .packets
                .checked_add(1)
                .ok_or(TimeError::QuantityOverflow {
                    quantity: "session packet count",
                })?;
            span
        };
        let closed = if starts { self.active } else { None };
        self.active = Some(span);
        Ok(SessionUpdate { key, closed })
    }
}

pub(crate) struct Position {
    pub key: PacketKey,
    pub late: Option<i128>,
    pub reset: bool,
}
// Shared native calculation; inference never mistakes a short forward wrap
// for a reset. Caller performs admission and commits high-water state afterward.
pub(crate) fn position(
    previous: Option<(usize, i128)>,
    rtc: Rtc,
    packet: u64,
    boundary: bool,
    late_ns: u64,
    infer: bool,
) -> Result<Position> {
    let raw = i128::from(rtc.as_raw());
    let mut ticks = raw;
    let mut late = None;
    if let Some((session, high)) = previous.filter(|_| !boundary) {
        let delta = SessionRtc {
            session,
            rtc: Rtc::from_raw((high & ((1i128 << 48) - 1)) as u64),
        }
        .delta_to(SessionRtc { session, rtc }, CounterOrder::Nearest)?
        .get();
        ticks = high.checked_add(delta).ok_or(TimeError::QuantityOverflow {
            quantity: "unwrapped RTC ticks",
        })?;
        if delta < 0 && -delta * 100 > i128::from(late_ns) {
            late = Some(delta * 100);
        }
    }
    let reset = late.is_some() && infer;
    let session = if previous.is_none() || boundary || reset {
        ticks = raw;
        previous
            .map_or(0, |p| p.0)
            .checked_add(1)
            .ok_or(TimeError::QuantityOverflow {
                quantity: "session identity",
            })?
    } else {
        previous.map_or(1, |p| p.0)
    };
    Ok(Position {
        key: PacketKey {
            session,
            ticks,
            packet,
        },
        late,
        reset,
    })
}
