//! `irigtime` — CLI tool for inspecting IRIG 106 Chapter 10 time data.
//!
//! Host-callable CLI implementation. `run` accepts arguments without argv[0].
//! Reads a Ch10 file, extracts all time packets, builds a correlation table,
//! and produces various diagnostic outputs.
//!
//! # Usage
//!
//! ```sh
//! irigtime summary  <file.ch10>           # Time span, channels, sources
//! irigtime channels <file.ch10>           # Per-channel time source inventory
//! irigtime jumps    <file.ch10> [--threshold-ms 1000]
//! irigtime timeline <file.ch10> [--limit 50]
//! irigtime csv      <file.ch10> [--output times.csv]
//! irigtime correlate <file.ch10> <rtc_hex>  # Resolve a single RTC value
//! ```

#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::dbg_macro,
    clippy::print_stdout,
    clippy::print_stderr
)]

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::io::{self, Write};
use std::path::Path;
use std::result::Result;

use irig106_time::bcd::{DayFormatTime, DmyFormatTime};
use irig106_time::csdw::{DateFormat, TimeF1Csdw};
use irig106_time::findings::Severity;
use irig106_time::network_time::{LeapSecondTable, TimeF2Csdw, parse_time_f2_payload};
use irig106_time::*;
/// Strict, filesystem-free argument parsing.
pub mod args;
mod clock_basis;
mod network_facts;
use network_facts::NetworkFacts;
mod correlate_report;
mod csv_spool;
/// Bounded raw packet transport; command integration follows in S16b.
pub mod input;
mod live_refs;
mod report;
mod report_json;
mod report_policy;
mod report_rows;
use report_rows::PacketRow as ResolvedPacket;
mod segment_answers;
mod segment_inventory;
mod target_refs;

// ────────────────────────────────────────────────────────────────────
// Constants
// ────────────────────────────────────────────────────────────────────

const DATA_TYPE_TIME_F1: u8 = 0x11;
const DATA_TYPE_TIME_F2: u8 = 0x12;
const FINDING_SAMPLE_LIMIT: usize = 20;
const JUMP_SAMPLE_LIMIT: usize = 20;
const REFERENCE_WINDOW_NS: u64 = 30_000_000_000;
const RESET_BACKWARD_TICKS: u64 = REFERENCE_WINDOW_NS / 100;
const RESET_RUN_REFERENCES: usize = 3;
// Resource policy, not a standard-imposed time/reference bound.
const REFERENCE_CAP: core::num::NonZeroUsize = match core::num::NonZeroUsize::new(1024) {
    Some(n) => n,
    None => core::num::NonZeroUsize::MIN,
};
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

// ────────────────────────────────────────────────────────────────────
// Minimal Ch10 header parsing (just enough for time extraction)
// ────────────────────────────────────────────────────────────────────

struct PktHeader {
    channel_id: u16,
    data_type: u8,
    rtc: Rtc,
}

// ────────────────────────────────────────────────────────────────────
// Time channel info
// ────────────────────────────────────────────────────────────────────

#[derive(Debug)]
struct TimeChannelInfo {
    channel_id: u16,
    packet_count: usize,
    source: Option<TimeSource>,
    format: Option<TimeFormat>,
    date_format: Option<DateFormat>,
    is_leap_year: Option<bool>,
    /// Raw Format 2 word: preserve NTF revision and TS even when conversion fails.
    network_word: Option<TimeF2Csdw>,
    first_basis: Option<&'static str>,
    last_basis: Option<&'static str>,
    year_rollovers: usize,
    first_time: Option<AbsoluteTime>,
    last_time: Option<AbsoluteTime>,
    first_rtc: Option<Rtc>,
    last_rtc: Option<Rtc>,
    // Last non-backward reference used for jump/gap pairs. Displayed Last
    // remains the latest arrival, including accepted late references.
    pair_baseline: Option<(Rtc, AbsoluteTime)>,
    late: Vec<ObservedReference>,
    segment: usize,
    active_segment: Option<segment_inventory::Active>,
    min_gap_ns: Option<u64>,
    max_gap_ns: Option<u64>,
}

impl TimeChannelInfo {
    fn new(channel_id: u16) -> Self {
        Self {
            channel_id,
            packet_count: 0,
            source: None,
            format: None,
            date_format: None,
            is_leap_year: None,
            network_word: None,
            first_basis: None,
            last_basis: None,
            year_rollovers: 0,
            first_time: None,
            last_time: None,
            first_rtc: None,
            last_rtc: None,
            pair_baseline: None,
            late: Vec::new(),
            segment: 1,
            active_segment: None,
            min_gap_ns: None,
            max_gap_ns: None,
        }
    }
}

// ────────────────────────────────────────────────────────────────────
// Resolved packet record (for timeline / CSV)
// ────────────────────────────────────────────────────────────────────

type RowSink<'a> = &'a mut dyn FnMut(&ResolvedPacket) -> io::Result<()>;

// ────────────────────────────────────────────────────────────────────
// File scanner
// ────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
struct ObservedReference {
    network_time: Option<NetworkFacts>,
    basis: &'static str,
    rtc: Rtc,
    time: AbsoluteTime,
    packet: usize,
    index: usize,
}

#[derive(Debug)]
struct FindingSample {
    level: Severity,
    code: Option<String>,
    message: String,
    channel: Option<u16>,
    packet: Option<usize>,
}

struct Ch10TimeScanner {
    segment_inventory: Option<segment_inventory::Archive>,
    finding_samples: Vec<FindingSample>,
    finding_location: Option<(u16, usize)>,
    correlator: StreamingTimeCorrelator,
    query_refs: Option<target_refs::TargetReferences>,
    query_any: Option<irig106_time::Result<AbsoluteTime>>,
    query_any_basis: Option<&'static str>,
    query_any_network: Option<NetworkFacts>,
    clock_basis: clock_basis::Window,
    query_answers: Vec<correlate_report::Answer>,
    segment_answers: Option<segment_answers::SegmentAnswers>,
    segment_error: Option<io::Error>,
    isolated: live_refs::LiveReferences,
    pending_channels: usize,
    reset_count: usize,
    references_seen: usize,
    jump_threshold: Option<u64>,
    jumps_seen: usize,
    jumps: Vec<TimeJump>,
    jump_year_days: Vec<(u16, u16)>,
    leap_table: LeapSecondTable,
    ntp_era: Option<i64>,
    time_channels: BTreeMap<u16, TimeChannelInfo>,
    total_packets: usize,
    time_packets: usize,
    data_type_counts: BTreeMap<u8, usize>,
    first_rtc: Option<Rtc>,
    last_rtc: Option<Rtc>,
    resolved: Vec<ResolvedPacket>,
    errors: Vec<String>,
    finding_counts: [u64; 3],
    highest: Option<Severity>,
}

impl Ch10TimeScanner {
    fn new() -> Self {
        Self {
            correlator: StreamingTimeCorrelator::with_reference_limit(
                NanosDuration::new(REFERENCE_WINDOW_NS),
                REFERENCE_CAP,
            ),
            segment_inventory: None,
            finding_samples: Vec::new(),
            finding_location: None,
            query_refs: None,
            query_any: None,
            query_any_basis: None,
            query_any_network: None,
            clock_basis: clock_basis::Window::default(),
            query_answers: Vec::new(),
            segment_answers: None,
            segment_error: None,
            isolated: live_refs::LiveReferences::default(),
            pending_channels: 0,
            reset_count: 0,
            references_seen: 0,
            jump_threshold: None,
            jumps_seen: 0,
            jumps: Vec::new(),
            jump_year_days: Vec::new(),
            leap_table: LeapSecondTable::builtin(),
            ntp_era: None,
            time_channels: BTreeMap::new(),
            total_packets: 0,
            time_packets: 0,
            data_type_counts: BTreeMap::new(),
            first_rtc: None,
            last_rtc: None,
            resolved: Vec::new(),
            errors: Vec::new(),
            finding_counts: [0; 3],
            highest: None,
        }
    }

    fn configured(args: &args::Args) -> Self {
        let mut scanner = Self::new();
        scanner.ntp_era = args.ntp_era;
        scanner
    }

    fn scan(
        &mut self,
        source: impl io::Read,
        resolve_all: bool,
        limit: Option<usize>,
    ) -> Result<(), Failure> {
        self.scan_to(source, resolve_all, limit, None)
    }
    // Partial CLI-104: one transport/dispatch loop for every renderer.
    fn scan_to(
        &mut self,
        source: impl io::Read,
        resolve_all: bool,
        limit: Option<usize>,
        mut emit: Option<RowSink<'_>>,
    ) -> Result<(), Failure> {
        use input::{Event, PacketReader, ReaderLimits, TailKind};
        let mut reader = PacketReader::new(source, ReaderLimits::default())
            .map_err(|e| Failure::new(ExitClass::Input, e.to_string()))?;
        // Time bodies require at most CSDW + eight body bytes. Other payloads
        // are drained without capture; neither Data nor Packet Length allocates.
        while let Some(event) = reader
            .next_packet(|h| {
                if matches!(h.data_type, DATA_TYPE_TIME_F1 | DATA_TYPE_TIME_F2) {
                    12
                } else {
                    0
                }
            })
            .map_err(|e| {
                Failure::new(
                    ExitClass::Input,
                    format!("invalid packet sync or framing: {e}"),
                )
            })?
        {
            let packet = match event {
                Event::Packet(p) => p,
                Event::Tail {
                    offset,
                    bytes,
                    kind,
                } => {
                    if self.total_packets == 0 {
                        return Err(Failure::new(ExitClass::Input, "no valid Chapter 10 header"));
                    }
                    let (level, message) = match kind {
                        TailKind::PartialHeader => (
                            Severity::Warning,
                            format!("truncated final header at {offset:#X} ({bytes} bytes)"),
                        ),
                        TailKind::TruncatedPacket => (
                            Severity::Warning,
                            format!(
                                "truncated final packet at {offset:#X} ({bytes} available bytes)"
                            ),
                        ),
                        TailKind::Unsynchronized => (
                            Severity::Information,
                            format!("{bytes} trailing bytes without sync at offset {offset:#X}"),
                        ),
                    };
                    self.note(level, message);
                    break;
                }
            };
            if packet.recovered_bytes != 0 {
                self.note(
                    Severity::Warning,
                    format!(
                        "Sync lost at {:#X}, recovered at {:#X} ({} bytes skipped)",
                        packet.offset - packet.recovered_bytes,
                        packet.offset,
                        packet.recovered_bytes
                    ),
                );
            }
            let hdr = PktHeader {
                channel_id: packet.header.channel,
                data_type: packet.header.data_type,
                rtc: Rtc::from_le_bytes(packet.header.rtc),
            };
            if self.first_rtc.is_none() {
                self.first_rtc = Some(hdr.rtc);
            }
            self.last_rtc = Some(hdr.rtc);
            *self.data_type_counts.entry(hdr.data_type).or_insert(0) += 1;
            self.finding_location = Some((hdr.channel_id, self.total_packets));
            let prior_refs = self.references_seen;
            if hdr.data_type == DATA_TYPE_TIME_F1 {
                self.time_packets += 1;
                self.process_time_packet(&packet.body_prefix, &hdr);
            } else if hdr.data_type == DATA_TYPE_TIME_F2 {
                self.time_packets += 1;
                self.process_time_f2_packet(&packet.body_prefix, &hdr);
            }
            if let Some(error) = self.segment_error.take() {
                return Err(error.into());
            }
            if resolve_all && limit.is_none_or(|n| self.total_packets < n) {
                let decoded = self.references_seen != prior_refs;
                let abs_time = self.resolve_row(&hdr, decoded);
                let basis = abs_time.and_then(|_| {
                    if (self.reset_count != 0 || self.pending_channels != 0) && decoded {
                        self.time_channels
                            .get(&hdr.channel_id)
                            .and_then(|c| c.last_basis)
                            .and_then(clock_basis::known)
                    } else {
                        self.basis_for(hdr.rtc, None)
                    }
                });
                let row = ResolvedPacket {
                    packet_num: self.total_packets,
                    offset: packet.offset,
                    channel_id: hdr.channel_id,
                    data_type: hdr.data_type,
                    rtc: hdr.rtc,
                    abs_time,
                    basis,
                    network_time: (hdr.data_type == DATA_TYPE_TIME_F2)
                        .then(|| NetworkFacts::observed(&packet.body_prefix))
                        .flatten(),
                };
                if let Some(write_row) = emit.as_mut() {
                    write_row(&row)?;
                } else {
                    self.resolved.push(row);
                }
            }
            self.finding_location = None;
            self.total_packets += 1;
        }
        if self.total_packets == 0 {
            return Err(Failure::new(ExitClass::Input, "no valid Chapter 10 header"));
        }
        self.finish_pending();
        if self.time_packets == 0 {
            self.note(Severity::Warning, "TF-001: no time packets in session");
        }
        Ok(())
    }

    // Coordinator-approved WARNING: missing retained context is not malformed
    // packet data. No new TF ID. A known target has its own retained brackets.
    fn correlate(&self, rtc: Rtc, channel: Option<u16>) -> irig106_time::Result<AbsoluteTime> {
        if let Some(ch) = channel {
            let info = self.time_channels.get(&ch);
            if info.is_some_and(|c| !c.late.is_empty()) {
                return Err(TimeError::NeedsContext {
                    context: "RTC reset classification pending on this channel",
                });
            }
            if self.query_refs.is_some() && info.is_some_and(|c| c.segment > 1) {
                return Err(TimeError::NeedsContext {
                    context: "RTC requires a labelled segment answer",
                });
            }
            if self.query_refs.as_ref().is_none_or(|q| q.target != rtc) {
                return self.isolated.correlate(rtc, ch);
            }
        } else if self.reset_count != 0 || self.pending_channels != 0 {
            return Err(TimeError::NeedsContext {
                context: "RTC source/channel association required",
            });
        }
        if let Some(query) = &self.query_refs {
            if query.target == rtc {
                return query.correlate(channel);
            }
        }
        self.correlator.correlate(rtc, channel)
    }
    fn basis_for(&self, rtc: Rtc, channel: Option<u16>) -> Option<&'static str> {
        if let Some(q) = &self.query_refs {
            if q.target == rtc {
                return q.basis(channel);
            }
        }
        if channel.is_none() {
            self.clock_basis.nearest(rtc)
        } else {
            None
        }
    }
    fn resolve(&mut self, rtc: Rtc, channel: Option<u16>) -> Option<AbsoluteTime> {
        match self.correlate(rtc, channel) {
            Ok(t) => Some(t),
            Err(e) => {
                self.note_decode(&e, format!("Correlation query: {e}"));
                None
            }
        }
    }

    fn resolve_row(&mut self, hdr: &PktHeader, decoded: bool) -> Option<AbsoluteTime> {
        if self.reset_count == 0 && self.pending_channels == 0 {
            return self.resolve(hdr.rtc, None);
        }
        // A decoded time packet states its own value; no counter bridge or
        // provisional segment decision is needed to report that observation.
        if decoded {
            return self
                .time_channels
                .get(&hdr.channel_id)
                .and_then(|c| c.last_time);
        }
        // A matching numeric ID is not a qualified data-to-time association.
        // S17 governing TMATS joining must select its time source/segment.
        self.resolve(hdr.rtc, None)
    }

    fn observe_reference(
        &mut self,
        hdr: &PktHeader,
        time: AbsoluteTime,
        basis: &'static str,
        network_time: Option<NetworkFacts>,
    ) {
        let point = ObservedReference {
            network_time,
            basis,
            rtc: hdr.rtc,
            time,
            packet: self.total_packets,
            index: self.references_seen,
        };
        self.references_seen += 1;
        let info = self
            .time_channels
            .entry(hdr.channel_id)
            .or_insert_with(|| TimeChannelInfo::new(hdr.channel_id));
        let baseline = info.pair_baseline.map(|(rtc, _)| rtc);
        let mut late = core::mem::take(&mut info.late);
        if !late.is_empty() {
            self.pending_channels -= 1;
        }
        if let Some(previous) = baseline.filter(|previous| point.rtc < *previous) {
            if previous.as_raw() - point.rtc.as_raw() > RESET_BACKWARD_TICKS {
                self.report_late(hdr.channel_id, late);
                self.start_reset(hdr.channel_id, previous, point);
                self.observe_pair(hdr.channel_id, point);
            } else {
                if late.last().is_some_and(|p| p.rtc >= point.rtc) {
                    self.report_late(hdr.channel_id, core::mem::take(&mut late));
                }
                late.push(point);
                if late.len() == RESET_RUN_REFERENCES {
                    if let Some(first) = late.first().copied() {
                        self.start_reset(hdr.channel_id, previous, first);
                    }
                    for p in late {
                        self.observe_pair(hdr.channel_id, p);
                    }
                } else if let Some(info) = self.time_channels.get_mut(&hdr.channel_id) {
                    info.late = late;
                    self.pending_channels += 1;
                }
            }
        } else {
            self.report_late(hdr.channel_id, late);
            self.observe_pair(hdr.channel_id, point);
        }
    }

    fn cache_reference(&mut self, channel: u16, point: ObservedReference) {
        if self.segment_inventory.is_some() {
            if let Some(info) = self.time_channels.get_mut(&channel) {
                if let Some(active) = &mut info.active_segment {
                    active.observe(point);
                } else {
                    info.active_segment = Some(segment_inventory::Active::new(info.segment, point));
                }
            }
        }
        self.correlator
            .add_reference(channel, point.rtc, point.time);
        self.clock_basis
            .observe(channel, point.rtc, point.basis, &self.correlator);
        self.isolated.observe(channel, point.rtc, point.time);
        if let Some(query) = self.query_refs.as_mut() {
            query.observe_tagged_at(
                channel,
                point.rtc,
                point.time,
                point.index,
                clock_basis::known(point.basis),
                point.network_time,
            );
        }
    }
    fn report_late(&mut self, channel: u16, late: Vec<ObservedReference>) {
        if let Some(previous) = self
            .time_channels
            .get(&channel)
            .and_then(|c| c.pair_baseline.map(|p| p.0))
        {
            for point in late {
                self.cache_reference(channel, point);
                self.note_at(
                    (channel, point.packet),
                    Severity::Warning,
                    None,
                    format!(
                        "Out-of-order time reference: channel {} RTC {:#X} -> {:#X} at packet {}",
                        channel,
                        previous.as_raw(),
                        point.rtc.as_raw(),
                        point.packet
                    ),
                );
            }
        }
    }
    fn finish_pending(&mut self) {
        if self.pending_channels == 0 {
            return;
        }
        let channels: Vec<_> = self
            .time_channels
            .iter()
            .filter(|(_, c)| !c.late.is_empty())
            .map(|(&ch, _)| ch)
            .collect();
        for channel in channels {
            if let Some(info) = self.time_channels.get_mut(&channel) {
                let late = core::mem::take(&mut info.late);
                self.pending_channels -= 1;
                self.report_late(channel, late);
            }
        }
    }
    fn start_reset(&mut self, channel: u16, previous: Rtc, first: ObservedReference) {
        self.note_at(
            (channel, first.packet),
            Severity::Warning,
            None,
            format!(
                "RTC reset on channel {} at packet {}: {:#X} -> {:#X}",
                channel,
                first.packet,
                previous.as_raw(),
                first.rtc.as_raw()
            ),
        );
        if let Some(archive) = &self.segment_inventory {
            if let Some(row) = self
                .time_channels
                .get_mut(&channel)
                .and_then(|c| c.active_segment.take())
            {
                if let Err(e) = archive.push(channel, row) {
                    self.segment_error = Some(e);
                }
            }
        }
        self.reset_count += 1;
        self.close_segment(channel, true);
        self.correlator.clear_channel(channel);
        self.clock_basis.clear(channel);
        self.isolated.clear(channel);
        if let Some(query) = self.query_refs.as_mut() {
            query.clear_channel(channel);
        }
        if let Some(info) = self.time_channels.get_mut(&channel) {
            info.pair_baseline = None;
            info.year_rollovers = 0;
            info.segment += 1;
        }
    }
    fn observe_pair(&mut self, channel: u16, point: ObservedReference) {
        self.cache_reference(channel, point);
        let hdr = PktHeader {
            channel_id: channel,
            data_type: 0,
            rtc: point.rtc,
        };
        let time = point.time;
        if let Some(info) = self.time_channels.get_mut(&hdr.channel_id) {
            if let Some((previous, _)) = info.pair_baseline {
                let gap = previous.elapsed_nanos(hdr.rtc);
                info.min_gap_ns = Some(info.min_gap_ns.map_or(gap, |old| old.min(gap)));
                info.max_gap_ns = Some(info.max_gap_ns.map_or(gap, |old| old.max(gap)));
            }
        }
        if let Some((rtc, previous)) = self
            .time_channels
            .get(&channel)
            .and_then(|c| c.pair_baseline)
        {
            let elapsed = rtc.elapsed_nanos(point.rtc);
            let leap = self
                .time_channels
                .get(&channel)
                .and_then(|c| c.is_leap_year);
            let year_days = report::year_boundary(previous, time, elapsed, leap);
            let expected_nanos = previous.as_total_ns() + elapsed;
            let actual_nanos =
                time.as_total_ns() + u64::from(year_days.unwrap_or(0)) * 86_400_000_000_000;
            if year_days.is_some() {
                if let Some(info) = self.time_channels.get_mut(&channel) {
                    info.year_rollovers += 1;
                    if let Some(row) = &mut info.active_segment {
                        row.years = info.year_rollovers;
                    }
                }
            }
            let delta_nanos = actual_nanos as i64 - expected_nanos as i64;
            if self
                .jump_threshold
                .is_some_and(|threshold| delta_nanos.unsigned_abs() > threshold)
            {
                self.jumps_seen += 1;
                if self.jumps.len() < JUMP_SAMPLE_LIMIT {
                    self.jump_year_days.push((
                        report::known_year_days(previous, leap)
                            .or(year_days)
                            .unwrap_or(0),
                        year_days.unwrap_or(0),
                    ));
                    self.jumps.push(TimeJump {
                        index: point.index,
                        channel_id: channel,
                        expected_nanos,
                        actual_nanos,
                        delta_nanos,
                    });
                }
                self.note_at(
                    (channel, point.packet),
                    Severity::Warning,
                    Some("TF-017"),
                    format!(
                        "Time jump on channel {} at reference {}",
                        channel, point.index
                    ),
                );
            }
        }
        self.time_channels
            .entry(hdr.channel_id)
            .or_insert_with(|| TimeChannelInfo::new(hdr.channel_id))
            .pair_baseline = Some((hdr.rtc, time));
    }

    fn close_segment(&mut self, channel: u16, labelled: bool) {
        if self.segment_error.is_some() {
            return;
        }
        let Some(query) = self.query_refs.as_ref() else {
            return;
        };
        if labelled && !query.covers(channel) {
            return;
        }
        let segment = self.time_channels.get(&channel).map_or(1, |c| c.segment);
        let answer = query.correlate(Some(channel));
        let basis = answer
            .as_ref()
            .ok()
            .and_then(|_| query.basis(Some(channel)));
        let network_time = answer
            .as_ref()
            .ok()
            .and_then(|_| query.network_time(Some(channel)));
        if let Err(error) = &answer {
            self.note(decode_level(error), format!("Segment query: {error}"));
        }
        if self.segment_answers.is_none() {
            match segment_answers::SegmentAnswers::new() {
                Ok(report) => self.segment_answers = Some(report),
                Err(e) => {
                    self.segment_error = Some(e);
                    return;
                }
            }
        }
        let Some(report) = self.segment_answers.as_mut() else {
            return;
        };
        match report.record_with_basis(channel, segment, labelled, answer, basis, network_time) {
            Ok(true) => self.note(
                Severity::Warning,
                format!("RTC is ambiguous across reset segments on channel {channel}"),
            ),
            Ok(false) => (),
            Err(e) => self.segment_error = Some(e),
        }
    }
    fn finish_segments(&mut self) -> Result<(), Failure> {
        let channels: Vec<_> = self
            .time_channels
            .iter()
            .filter(|(_, c)| c.first_rtc.is_some())
            .map(|(&ch, _)| ch)
            .collect();
        for ch in channels {
            let labelled = self.time_channels.get(&ch).is_some_and(|c| c.segment > 1);
            self.close_segment(ch, labelled);
            if !self
                .segment_answers
                .as_ref()
                .is_some_and(|a| a.contains(ch))
            {
                self.note(
                    Severity::Warning,
                    format!("No reset segment covers requested RTC on channel {ch}"),
                );
            }
        }
        if let Some(error) = self.segment_error.take() {
            return Err(error.into());
        }
        Ok(())
    }

    fn note(&mut self, level: Severity, message: impl Into<String>) {
        self.observe(level);
        // <=3 facts per complete packet plus terminal/query facts. Checked
        // u64 byte offsets and >=24-byte frames bound these counts below u64::MAX.
        self.finding_counts[usize::from(rank(level))] += 1;
        if self.errors.len() < FINDING_SAMPLE_LIMIT {
            let message = message.into();
            let label = match level {
                Severity::Error => "Error",
                Severity::Warning => "Warning",
                Severity::Information => "Information",
            };
            self.errors.push(format!("{label}: {message}"));
            let code = message
                .split(':')
                .next()
                .filter(|v| {
                    v.len() == 6
                        && v.starts_with("TF-")
                        && v[3..].bytes().all(|b| b.is_ascii_digit())
                })
                .map(str::to_owned);
            self.finding_samples.push(FindingSample {
                level,
                code,
                message,
                channel: self.finding_location.map(|p| p.0),
                packet: self.finding_location.map(|p| p.1),
            });
        }
    }

    fn note_at(
        &mut self,
        location: (u16, usize),
        level: Severity,
        code: Option<&str>,
        message: String,
    ) {
        let previous = self.finding_location.replace(location);
        self.note_code(level, code, message);
        self.finding_location = previous;
    }
    fn note_code(&mut self, level: Severity, code: Option<&str>, message: impl Into<String>) {
        let before = self.finding_samples.len();
        self.note(level, message);
        if self.finding_samples.len() > before {
            if let Some(sample) = self.finding_samples.last_mut() {
                sample.code = code.map(str::to_owned);
            }
        }
    }
    fn note_decode(&mut self, error: &TimeError, message: String) {
        let code = match error {
            TimeError::ReservedBitSet { .. } => Some("TF-005"),
            TimeError::InvalidTimeStatus { status: 0 } => Some("TF-003"),
            TimeError::InvalidTimeStatus { .. } => Some("TF-006"),
            TimeError::ReservedNetworkTimeFormat { .. } => Some("TF-006"),
            TimeError::InvalidBcdDigit { .. }
            | TimeError::OutOfRange { .. }
            | TimeError::BufferTooShort { .. } => Some("TF-004"),
            _ => None,
        };
        self.note_code(decode_level(error), code, message);
    }

    fn observe(&mut self, level: Severity) {
        if self.highest.is_none_or(|old| rank(level) > rank(old)) {
            self.highest = Some(level);
        }
    }

    fn report_findings(&self, stderr: &mut dyn Write) -> Result<(), Failure> {
        report::Report::from(self).report_findings(stderr)
    }

    fn process_time_packet(&mut self, payload: &[u8], hdr: &PktHeader) {
        let Some(csdw_bytes) = payload.get(..4) else {
            self.note_code(
                Severity::Error,
                Some("TF-004"),
                format!("Time packet at ch={} too short for CSDW", hdr.channel_id),
            );
            return;
        };
        let csdw =
            TimeF1Csdw::from_le_bytes([csdw_bytes[0], csdw_bytes[1], csdw_bytes[2], csdw_bytes[3]]);

        let bcd_start = 4;
        let abs_time = match csdw.date_format() {
            DateFormat::DayOfYear => {
                if bcd_start + 6 > payload.len() {
                    self.note_code(
                        Severity::Error,
                        Some("TF-004"),
                        format!(
                            "Time packet ch={}: buffer too short for DOY BCD",
                            hdr.channel_id
                        ),
                    );
                    return;
                }
                match DayFormatTime::from_le_bytes(&payload[bcd_start..]) {
                    Ok(t) => t.to_absolute(),
                    Err(e) => {
                        self.note_decode(
                            &e,
                            format!("Time packet ch={}: DOY BCD error: {}", hdr.channel_id, e),
                        );
                        return;
                    }
                }
            }
            DateFormat::DayMonthYear => {
                if bcd_start + 8 > payload.len() {
                    self.note_code(
                        Severity::Error,
                        Some("TF-004"),
                        format!(
                            "Time packet ch={}: buffer too short for DMY BCD",
                            hdr.channel_id
                        ),
                    );
                    return;
                }
                match DmyFormatTime::from_le_bytes(&payload[bcd_start..]) {
                    Ok(t) => match t.to_calendar_time() {
                        Ok(ct) => ct.into_absolute_time(),
                        Err(e) => {
                            self.note_decode(
                                &e,
                                format!(
                                    "Time packet ch={}: DMY date invalid: {}",
                                    hdr.channel_id, e
                                ),
                            );
                            return;
                        }
                    },
                    Err(e) => {
                        self.note_decode(
                            &e,
                            format!("Time packet ch={}: DMY BCD error: {}", hdr.channel_id, e),
                        );
                        return;
                    }
                }
            }
        };

        self.observe_reference(hdr, abs_time, report::f1_basis(csdw.time_format()), None);
        // Update channel info
        let info = self
            .time_channels
            .entry(hdr.channel_id)
            .or_insert_with(|| TimeChannelInfo::new(hdr.channel_id));
        info.packet_count += 1;
        info.network_word = None;
        info.source = Some(csdw.time_source());
        info.format = Some(csdw.time_format());
        info.date_format = Some(csdw.date_format());
        info.is_leap_year = Some(csdw.is_leap_year());
        let basis = report::f1_basis(csdw.time_format());
        if info.first_time.is_none() {
            info.first_time = Some(abs_time);
            info.first_basis = Some(basis);
            info.first_rtc = Some(hdr.rtc);
        }
        info.last_time = Some(abs_time);
        info.last_basis = Some(basis);
        info.last_rtc = Some(hdr.rtc);
    }

    fn convert_declared_ntp(
        &mut self,
        raw: NtpTime,
    ) -> irig106_time::Result<Option<(AbsoluteTime, &'static str)>> {
        use irig106_time::ntp::{
            EraDeclaration, NtpContext, NtpConversionStatus, NtpLeapStatus, convert_ntp,
        };
        // Only the zero sentinel is context-independent. Never call the API's
        // era0 fallback for a nonzero reading without an explicit declaration.
        if self.ntp_era.is_none() && (raw.seconds != 0 || raw.fraction != 0) {
            return Err(TimeError::NeedsContext {
                context: "NTP era/basis required; supply --ntp-era <N>",
            });
        }
        // Local conversion scope token, not an inferred recorder session. No
        // pivots are supplied; the explicit option applies to the entire input.
        let mut context = NtpContext::new(1);
        context.declared = self.ntp_era.map(|era| EraDeclaration {
            era,
            id: "--ntp-era".into(),
        });
        let result = convert_ntp(raw, &context, Some(&self.leap_table));
        if result.status() == NtpConversionStatus::UnknownUnsynchronized {
            self.note_code(Severity::Warning,Some("TF-029"),format!("TF-029: NTP unknown/unsynchronized; raw seconds={} fraction={}; no absolute value",raw.seconds,raw.fraction));
            return Ok(None);
        }
        if result.calendar_error().is_some() {
            return Err(TimeError::NeedsContext {
                context: "NTP era outside supported calendar range",
            });
        }
        let time = result
            .utc()
            .ok_or(TimeError::NeedsContext {
                context: "NTP declared era cannot supply civil time",
            })?
            .to_absolute()?
            .value;
        let basis = match result.basis().leap {
            NtpLeapStatus::Ambiguous => {
                self.note_code(Severity::Warning,Some("TF-030"),format!("TF-030: NTP leap-ambiguous, value kept; raw seconds={} fraction={}; caller-declared era={}; floor={}ns; table={}",raw.seconds,raw.fraction,self.ntp_era.map(|e|e.to_string()).unwrap_or_else(||"unknown".into()),result.nanoseconds(),self.leap_table.metadata().id));
                clock_basis::NTP_AMBIGUOUS
            }
            NtpLeapStatus::UncertainCoverage => clock_basis::NTP_UNCERTAIN,
            _ => clock_basis::NTP_DECLARED,
        };
        Ok(Some((time, basis)))
    }

    fn process_time_f2_packet(&mut self, payload: &[u8], hdr: &PktHeader) {
        // Inventory is factual even when TS/era/body prevents an absolute answer.
        if let Some(bytes) = payload.get(..4).and_then(|b| b.try_into().ok()) {
            let info = self
                .time_channels
                .entry(hdr.channel_id)
                .or_insert_with(|| TimeChannelInfo::new(hdr.channel_id));
            info.packet_count += 1;
            info.network_word = Some(TimeF2Csdw::from_le_bytes(bytes));
            info.source = None;
            info.format = None;
            info.date_format = None;
            info.is_leap_year = None;
        }
        let (_, network_time) = match parse_time_f2_payload(payload) {
            Ok(result) => result,
            Err(e) => {
                self.note_decode(
                    &e,
                    format!("Time F2 packet ch={}: parse error: {}", hdr.channel_id, e),
                );
                return;
            }
        };

        // Checked conversion before eviction; retain the table-currency warning
        // while keeping D7's last-known-offset UTC and exact original TAI.
        let converted = match network_time {
            irig106_time::network_time::NetworkTime::Ntp(raw) => match self.convert_declared_ntp(raw) {
                Ok(Some(time)) => Ok(time), Ok(None) => return, Err(e) => Err(e),
            },
            irig106_time::network_time::NetworkTime::Ptp(ptp) => {
                self.leap_table.convert_ptp(ptp).and_then(|result| {
                    let utc = result.utc().ok_or(TimeError::NeedsContext {
                        context: "no known applicable TAI-UTC offset",
                    })?.to_absolute()?.value;
                    if result.status() == irig106_time::leap::PtpConversionStatus::PossiblyOutOfDate {
                        self.note_code(Severity::Warning, Some("TF-021"), format!(
                            "TF-021: UTC {} possibly out of date (value kept); raw TAI={}s+{}ns; TAI-UTC={}s; table={}; known through Unix={}s",
                            fmt_abs_time(&utc), result.raw().seconds, result.raw().nanoseconds,
                            result.basis().offset.map(|o|o.get().to_string()).unwrap_or_else(||"unknown".into()),
                            result.basis().table.id, result.basis().table.known_current_through.get(),
                        ));
                    }
                    Ok((utc,if result.status()==irig106_time::leap::PtpConversionStatus::PossiblyOutOfDate {clock_basis::STALE} else {clock_basis::COVERED}))
                })
            }
        };
        let (abs_time, basis) = match converted {
            Ok(t) => t,
            Err(e) => {
                self.note_decode(
                    &e,
                    format!(
                        "Time F2 packet ch={}: correlation error: {}",
                        hdr.channel_id, e
                    ),
                );
                return;
            }
        };
        self.observe_reference(hdr, abs_time, basis, NetworkFacts::observed(payload));
        // Update channel info
        let info = self
            .time_channels
            .entry(hdr.channel_id)
            .or_insert_with(|| TimeChannelInfo::new(hdr.channel_id));
        if info.first_time.is_none() {
            info.first_time = Some(abs_time);
            info.first_basis = Some(basis);
            info.first_rtc = Some(hdr.rtc);
        }
        info.last_time = Some(abs_time);
        info.last_basis = Some(basis);
        info.last_rtc = Some(hdr.rtc);
    }
}

// ────────────────────────────────────────────────────────────────────
// Formatting helpers
// ────────────────────────────────────────────────────────────────────

fn fmt_abs_time(t: &AbsoluteTime) -> String {
    report::clock_value(t)
}

fn fmt_time_source(src: &TimeSource) -> &'static str {
    match src {
        TimeSource::Internal => "Internal",
        TimeSource::External => "External",
        TimeSource::InternalFromRmm => "Internal from RMM",
        TimeSource::None => "None",
        TimeSource::Reserved(_) => "Reserved",
    }
}

fn fmt_time_format(fmt: &TimeFormat) -> &'static str {
    match fmt {
        TimeFormat::IrigB => "IRIG-B",
        TimeFormat::IrigA => "IRIG-A",
        TimeFormat::IrigG => "IRIG-G",
        TimeFormat::Rtc => "RTC",
        TimeFormat::Utc => "UTC",
        TimeFormat::Gps => "GPS",
        TimeFormat::None => "None",
        TimeFormat::Reserved(_) => "Reserved",
    }
}

fn fmt_data_type_short(dt: u8) -> &'static str {
    match dt {
        0x00 => "CompGen-0",
        0x01 => "TMATS",
        0x02 => "RecEvent",
        0x03 => "RecIndex",
        0x11 => "Time-F1",
        0x12 => "Time-F2",
        0x09 => "PCM-1",
        0x19 => "1553-1",
        0x1A => "1553-2",
        0x21 => "Analog-1",
        0x29 => "Discrete-1",
        0x30 => "Message-0",
        0x38 => "ARINC429",
        0x40 => "Video-0",
        0x50 => "UART-0",
        0x68 => "Ethernet-0",
        0x69 => "Ethernet-1",
        _ => "Other",
    }
}

fn fmt_comma(n: impl std::fmt::Display) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let mut result = String::new();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*b as char);
    }
    result
}

// ────────────────────────────────────────────────────────────────────
// Command implementations
// ────────────────────────────────────────────────────────────────────

fn cmd_summary(
    scanner: &report::Report<'_>,
    file_len: u64,
    filename: &Path,
    stdout: &mut dyn Write,
) -> Result<Option<Severity>, Failure> {
    writeln!(
        stdout,
        "================================================================"
    )?;
    writeln!(stdout, "irigtime — IRIG 106 Time Analysis")?;
    writeln!(
        stdout,
        "================================================================"
    )?;
    writeln!(stdout)?;
    writeln!(stdout, "File               : {}", filename.display())?;
    writeln!(stdout, "File Size          : {} bytes", fmt_comma(file_len))?;
    writeln!(
        stdout,
        "Total Packets      : {}",
        fmt_comma(scanner.total_packets)
    )?;
    writeln!(
        stdout,
        "Time Packets (0x11/0x12): {}",
        fmt_comma(scanner.time_packets)
    )?;
    writeln!(
        stdout,
        "Time Channels      : {}",
        scanner.time_channels.len()
    )?;
    writeln!(stdout, "Correlation Refs   : {}", scanner.reference_count)?;
    writeln!(stdout, "References Seen    : {}", scanner.references_seen)?;
    writeln!(
        stdout,
        "References Evicted : {}",
        scanner.references_evicted
    )?;
    writeln!(stdout)?;

    if scanner.reset_count != 0 {
        writeln!(
            stdout,
            "RTC Duration       : unavailable across reset segments"
        )?;
    } else {
        if let (Some(first), Some(last)) = (scanner.first_rtc, scanner.last_rtc) {
            let dur_ns = first.elapsed_nanos(last);
            let dur_s = dur_ns as f64 / 1_000_000_000.0;
            writeln!(
                stdout,
                "RTC Range          : {:#014X} → {:#014X}",
                first.as_raw(),
                last.as_raw()
            )?;
            writeln!(
                stdout,
                "RTC Duration       : {:.3} seconds ({:.1} minutes)",
                dur_s,
                dur_s / 60.0
            )?;
        }
    }
    writeln!(stdout)?;

    // Per-channel summary
    if !scanner.time_channels.is_empty() {
        writeln!(stdout, "Time Channels")?;
        writeln!(stdout, "─────────────")?;
        for info in scanner.time_channels.values() {
            let proto_str = report::network_label(info);
            writeln!(
                stdout,
                "  Channel {:>3}  │  Source: {:<12}  Format: {:<8}  Date: {:<4}  Leap: {}  Packets: {}{}",
                info.channel_id,
                info.source.as_ref().map(fmt_time_source).unwrap_or("?"),
                report::format_label(info),
                match info.date_format {
                    Some(DateFormat::DayOfYear) => "DOY",
                    Some(DateFormat::DayMonthYear) => "DMY",
                    None => "?",
                },
                match info.is_leap_year {
                    Some(true) => "Yes",
                    Some(false) => "No",
                    None => "?",
                },
                fmt_comma(info.packet_count),
                proto_str,
            )?;
            if info.first_time.is_some() {
                writeln!(
                    stdout,
                    "                │  First: {}",
                    report::first_clock(info)
                )?;
            }
            if info.last_time.is_some() {
                writeln!(
                    stdout,
                    "                │  Last:  {}",
                    report::last_clock(info)
                )?;
            }
            if let Some((min, max)) = info.min_gap_ns.zip(info.max_gap_ns) {
                writeln!(
                    stdout,
                    "                │  Observed RTC gaps: min={min} ns max={max} ns"
                )?;
            }
        }
        writeln!(stdout)?;
    }

    if scanner.jumps_seen != 0 {
        writeln!(stdout, "Time Jumps Detected (threshold: 1 second)")?;
        scanner.report_jump_samples(stdout)?;
        writeln!(stdout, "Total: {} jump(s).", scanner.jumps_seen)?;
    } else if !scanner.time_channels.is_empty() {
        writeln!(
            stdout,
            "Time Jumps         : None detected (threshold: 1 second)"
        )?;
    }
    writeln!(stdout)?;

    // Data type breakdown
    writeln!(stdout, "Data Type Breakdown")?;
    writeln!(stdout, "───────────────────")?;
    for (dt, count) in scanner.data_type_counts {
        writeln!(
            stdout,
            "  0x{:02X} {:<14}  {:>10}",
            dt,
            fmt_data_type_short(*dt),
            fmt_comma(*count)
        )?;
    }
    writeln!(stdout)?;

    if scanner.finding_total() != 0 {
        writeln!(stdout, "Errors ({})", scanner.finding_total())?;
        scanner.report_finding_counts(stdout)?;
        writeln!(stdout, "──────")?;
        for (i, e) in scanner.errors.iter().enumerate() {
            writeln!(stdout, "  {}. {}", i + 1, e)?;
        }
        if scanner.omitted_findings() != 0 {
            writeln!(
                stdout,
                "  ... and {} more findings",
                scanner.omitted_findings()
            )?;
        }
    }
    Ok(scanner.highest)
}

fn cmd_channels(
    scanner: &report::Report<'_>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<Option<Severity>, Failure> {
    scanner.report_findings(stderr)?;

    let header = format!(
        "{:>7}  {:<18}  {:<10}  {:<4}  {:<4}  {:<13}  {:<8}  {:<7}  {:>8}  {:<70}  {}",
        "Channel",
        "Source",
        "Format",
        "Date",
        "Leap",
        "Proto",
        "Status",
        "Scale",
        "Packets",
        "First Time",
        "Last Time"
    );
    writeln!(stdout, "{header}")?;
    writeln!(stdout, "{}", "─".repeat(header.chars().count()))?;
    for info in scanner.time_channels.values() {
        let (proto, status, scale) = report::columns(info);
        writeln!(
            stdout,
            "{:>7}  {:<18}  {:<10}  {:<4}  {:<4}  {:<13}  {:<8}  {:<7}  {:>8}  {:<70}  {}",
            info.channel_id,
            info.source.as_ref().map(fmt_time_source).unwrap_or("?"),
            if info.network_word.is_some() {
                "Time-F2"
            } else {
                report::format_label(info)
            },
            match info.date_format {
                Some(DateFormat::DayOfYear) => "DOY",
                Some(DateFormat::DayMonthYear) => "DMY",
                None => "—",
            },
            match info.is_leap_year {
                Some(true) => "Yes",
                Some(false) => "No",
                None => "—",
            },
            proto,
            status,
            scale,
            fmt_comma(info.packet_count),
            report::first_clock(info),
            report::last_clock(info)
        )?;
    }
    Ok(scanner.highest)
}

fn cmd_jumps(
    scanner: &report::Report<'_>,
    threshold_ms: u64,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<Option<Severity>, Failure> {
    writeln!(
        stdout,
        "Time Jump Detection (threshold: {} ms)",
        threshold_ms
    )?;
    scanner.report_jump_samples(stdout)?;
    if scanner.jumps_seen == 0 {
        writeln!(
            stdout,
            "No time jumps detected across {} channel(s).",
            scanner
                .time_channels
                .values()
                .filter(|c| c.first_rtc.is_some())
                .count()
        )?;
    } else {
        writeln!(
            stdout,
            "Total: {} jump(s) across {} channel(s).",
            scanner.jumps_seen,
            scanner
                .time_channels
                .values()
                .filter(|c| c.first_rtc.is_some())
                .count()
        )?;
    }
    scanner.report_findings(stderr)?;
    Ok(scanner.highest)
}

fn cmd_timeline(
    input: std::fs::File,
    args: &args::Args,
    size: u64,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<Option<Severity>, Failure> {
    let mut scanner = Ch10TimeScanner::configured(args);
    let limit = args.limit;
    if args.json {
        scanner.segment_inventory = Some(segment_inventory::Archive::new()?);
    }
    scanner.scan(input, true, Some(limit))?;
    let report = report::Report::from(&scanner);
    report.report_findings(stderr)?;
    if args.json {
        report_json::write(&report, args, size, stdout)?;
        return Ok(report.highest);
    }

    writeln!(
        stdout,
        "{:>8}  {:>12}  {:>6}  {:<14}  {:>14}  {:>28}",
        "Pkt#", "Offset", "Ch", "Type", "RTC", "Absolute Time"
    )?;
    writeln!(stdout, "{}", "─".repeat(100))?;

    for r in report.packet_rows {
        r.write_text(stdout)?;
    }

    if report.total_packets > limit {
        writeln!(
            stdout,
            "... showing first {} of {} packets",
            limit, report.total_packets
        )?;
    }
    Ok(report.highest)
}

fn cmd_csv(
    input: std::fs::File,
    args: &args::Args,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    pending: &mut Option<csv_spool::CsvSpool>,
) -> Result<Option<Severity>, Failure> {
    let output_path = args.output.as_deref();
    let mut spool = csv_spool::CsvSpool::new(output_path)?;
    writeln!(spool, "{}", report_rows::CSV_HEADER)?;
    let mut scanner = Ch10TimeScanner::configured(args);
    let mut emit = |row: &ResolvedPacket| row.write_csv(&mut spool);
    scanner.scan_to(input, true, None, Some(&mut emit))?;
    spool.flush()?;
    scanner.report_findings(stderr)?;
    if let Some(path) = output_path {
        writeln!(
            stderr,
            "Prepared {} rows for {}; file is replaced on exit 0 or 1.",
            scanner.total_packets,
            path.display()
        )?;
        *pending = Some(spool);
    } else {
        spool.copy_stdout(stdout)?;
    }
    Ok(scanner.highest)
}

fn cmd_correlate(
    input: std::fs::File,
    args: &args::Args,
    size: u64,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<Option<Severity>, Failure> {
    let mut scanner = Ch10TimeScanner::configured(args);
    let target = args.rtc;
    scanner.query_refs = Some(target_refs::TargetReferences::new(target));
    if args.json {
        scanner.segment_inventory = Some(segment_inventory::Archive::new()?);
    }
    scanner.scan(input, false, None)?;

    if scanner.reset_count != 0 {
        scanner.finish_segments()?;
    } else if scanner.correlator.is_empty() {
        scanner.note(Severity::Warning, "No time reference points found in file.");
    } else {
        let any = scanner.correlate(target, None);
        if let Err(e) = &any {
            scanner.note(decode_level(e), format!("Correlation query: {e}"));
        }
        scanner.query_any_basis = any
            .as_ref()
            .ok()
            .and_then(|_| scanner.basis_for(target, None));
        scanner.query_any_network = any.as_ref().ok().and_then(|_| {
            scanner
                .query_refs
                .as_ref()
                .and_then(|q| q.network_time(None))
        });
        scanner.query_any = Some(any);
        let channels: Vec<_> = scanner
            .time_channels
            .iter()
            .filter(|(_, c)| c.first_rtc.is_some())
            .map(|(&ch, _)| ch)
            .collect();
        for ch in channels {
            if let Some(time) = scanner.resolve(target, Some(ch)) {
                let mut answer = correlate_report::Answer::new(ch, 1, false, Ok(time));
                answer.basis = scanner.basis_for(target, Some(ch));
                answer.network_time = scanner
                    .query_refs
                    .as_ref()
                    .and_then(|q| q.network_time(Some(ch)));
                scanner.query_answers.push(answer);
            }
        }
    }
    let report = report::Report::from(&scanner);
    if args.json {
        report.report_findings(stderr)?;
        report_json::write(&report, args, size, stdout)?;
        return Ok(report.highest);
    }
    if let Some(query) = &report.query {
        query.write_header(stdout)?;
    }
    if report.reset_count != 0 {
        report.report_findings(stderr)?;
    }
    if let Some(query) = &report.query {
        query.write_answers(stdout)?;
    }
    if report.reset_count == 0 {
        report.report_findings(stderr)?;
    }
    Ok(report.highest)
}

fn print_usage(stdout: &mut dyn Write) -> Result<(), Failure> {
    writeln!(
        stdout,
        "irigtime v{} — IRIG 106 Chapter 10 Time Inspector",
        PKG_VERSION
    )?;
    writeln!(stdout)?;
    writeln!(stdout, "Usage:")?;
    writeln!(
        stdout,
        "  irigtime summary   <file.ch10>                    File time summary"
    )?;
    writeln!(
        stdout,
        "  irigtime channels  <file.ch10>                    Time channel inventory"
    )?;
    writeln!(
        stdout,
        "  irigtime jumps     <file.ch10> [--threshold-ms N] Time jump detection (default: 1000 ms)"
    )?;
    writeln!(
        stdout,
        "  irigtime timeline  <file.ch10> [--limit N]        Per-packet timeline (default: 100)"
    )?;
    writeln!(
        stdout,
        "  irigtime csv       <file.ch10> [--output file]    Export all timestamps to CSV"
    )?;
    writeln!(
        stdout,
        "  irigtime correlate <file.ch10> <rtc_hex>          Resolve a single RTC value"
    )?;
    writeln!(stdout)?;
    writeln!(
        stdout,
        "Use -- before dash-leading paths; option values are separate arguments."
    )?;
    writeln!(
        stdout,
        "--limit: positive decimal; --threshold-ms: unsigned decimal, checked in ns."
    )?;
    writeln!(
        stdout,
        "Shared irig106 exits: usage2, first fatal input3/output4, findings1, success0."
    )?;
    writeln!(
        stdout,
        "--fail-on <level>: error (default), warning, info, never. Inclusive; lowercase, separate value, no duplicates."
    )?;
    writeln!(
        stdout,
        "Accepted on all commands, including help/version with no effect there."
    )?;
    writeln!(
        stdout,
        "Finding levels: recovered sync / cut final packet or header / missing context / no usable reference: warning;"
    )?;
    writeln!(
        stdout,
        "trailing unsynced bytes after a stream: info; malformed/short/range decode and unusable correlation: error;"
    )?;
    writeln!(
        stdout,
        "reserved BCD bits: warning (TF-005, owner-approved); invalid TS/reserved NTF: warning (TF-003/006); jumps: warning (TF-017)."
    )?;
    writeln!(
        stdout,
        "Correlation caches: compatibility 1024; isolated 1024 arrivals / 30s per channel. Old/empty-window answers: warning (coordinator-approved)."
    )?;
    writeln!(
        stdout,
        "--format <text|json>: summary/channels/jumps/timeline/correlate; default text. Schema stable v1, revision 8."
    )?;
    writeln!(
        stdout,
        "--ntp-era <N>: explicit decimal era -14..=59 on all file commands; no era guessing; caller-declared era labels."
    )?;
    writeln!(stdout, "Examples:")?;
    writeln!(stdout, "  irigtime summary flight_042.ch10")?;
    writeln!(
        stdout,
        "  irigtime jumps   flight_042.ch10 --threshold-ms 500"
    )?;
    writeln!(
        stdout,
        "  irigtime csv     flight_042.ch10 --output times.csv"
    )?;
    writeln!(stdout, "  irigtime correlate flight_042.ch10 0x00009896800")?;
    Ok(())
}

/// Shared exit classes; numeric assignments agree with irig106 tmats.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ExitClass {
    /// Completed successfully.
    Success = 0,
    /// Findings meet the selected threshold.
    Findings = 1,
    /// Invalid command-line usage.
    Usage = 2,
    /// Input could not be read or recognized.
    Input = 3,
    /// Output could not be written or flushed.
    Output = 4,
}
struct Failure {
    class: ExitClass,
    message: String,
}
impl Failure {
    fn new(class: ExitClass, message: impl ToString) -> Self {
        Self {
            class,
            message: message.to_string(),
        }
    }
}
impl From<io::Error> for Failure {
    fn from(error: io::Error) -> Self {
        Self::new(ExitClass::Output, error)
    }
}

/// Run with arguments after the program name (or after the host's `time`).
///
/// Arguments may be OsString/OsStr or strings. Paths retain native encoding;
/// non-UTF-8 command/option syntax returns usage2.
/// All output uses these writers; no global stdout/stderr or process exit.
/// All syntax is validated before input opens; input/output failures precede
/// finding thresholds. The default failure threshold is error (shared irig106 CLI convention).
/// Input is buffered; remaining report/reference bounds belong to S16b.
/// Traces: L3-CLI-101 <- L2-CLI-101 <- L1-CLI-001;
/// L3-CLI-102 <- L2-CLI-102 <- L1-CLI-002 (host/writer clauses only).
/// Traces: L3-CLI-107 <- L2-CLI-107 <- L1-CLI-007.
pub fn run<S: AsRef<OsStr>>(
    raw: &[S],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> ExitClass {
    let mut pending = None;
    let result = args::parse(raw)
        .map_err(|e| Failure::new(ExitClass::Usage, e))
        .and_then(|request| {
            let fail_on = match &request {
                args::Request::File(a) => a.fail_on,
                _ => None,
            };
            let highest = dispatch(request, stdout, stderr, &mut pending)?;
            stdout.flush()?;
            stderr.flush()?;
            let class = if highest
                .zip(fail_on)
                .is_some_and(|(level, threshold)| rank(level) >= rank(threshold))
            {
                ExitClass::Findings
            } else {
                ExitClass::Success
            };
            if let Some(spool) = pending.as_mut() {
                spool.commit()?;
            }
            Ok(class)
        });
    match result {
        Ok(class) => class,
        Err(error) => {
            // Usage wins preflight; otherwise preserve the first fatal operational failure.
            let _ = writeln!(stderr, "Error: {}", error.message).and_then(|()| stderr.flush());
            error.class
        }
    }
}
fn rank(level: Severity) -> u8 {
    match level {
        Severity::Information => 0,
        Severity::Warning => 1,
        Severity::Error => 2,
    }
}
fn decode_level(error: &TimeError) -> Severity {
    match error {
        TimeError::ReservedBitSet { .. }
        | TimeError::NeedsContext { .. }
        | TimeError::InvalidTimeStatus { .. }
        | TimeError::ReservedNetworkTimeFormat { .. }
        // Coordinator-approved WARNING for unavailable retained context (no TF level change).
        | TimeError::QueryBeforeRetainedWindow
        | TimeError::NoReferencePoint => Severity::Warning,
        _ => Severity::Error,
    }
}
fn dispatch(
    request: args::Request,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    pending: &mut Option<csv_spool::CsvSpool>,
) -> Result<Option<Severity>, Failure> {
    let args = match request {
        args::Request::Help => {
            print_usage(stdout)?;
            return Ok(None);
        }
        args::Request::Version => {
            writeln!(stdout, "irigtime {}", PKG_VERSION)?;
            return Ok(None);
        }
        args::Request::File(args) => args,
    };
    let input = std::fs::File::open(&args.input).map_err(|e| {
        Failure::new(
            ExitClass::Input,
            format!("cannot read '{}': {e}", args.input.display()),
        )
    })?;
    let file_len = input
        .metadata()
        .map_err(|e| Failure::new(ExitClass::Input, e.to_string()))?
        .len();
    match args.command.as_str() {
        "summary" | "channels" | "jumps" => {
            let mut scanner = Ch10TimeScanner::configured(&args);
            scanner.jump_threshold = match args.command.as_str() {
                "summary" => Some(1_000_000_000),
                "jumps" => Some(
                    args.threshold_ms
                        .checked_mul(1_000_000)
                        .ok_or_else(|| Failure::new(ExitClass::Usage, "threshold overflows ns"))?,
                ),
                _ => None,
            };
            if args.json {
                scanner.segment_inventory = Some(segment_inventory::Archive::new()?);
            }
            scanner.scan(input, false, None)?;
            let report = report::Report::from(&scanner);
            if args.json {
                if args.command == "channels" {
                    report.report_findings(stderr)?;
                }
                report_json::write(&report, &args, file_len, stdout)?;
                if args.command == "jumps" {
                    report.report_findings(stderr)?;
                }
                Ok(report.highest)
            } else {
                match args.command.as_str() {
                    "summary" => cmd_summary(&report, file_len, &args.input, stdout),
                    "channels" => cmd_channels(&report, stdout, stderr),
                    _ => cmd_jumps(&report, args.threshold_ms, stdout, stderr),
                }
            }
        }
        "timeline" => cmd_timeline(input, &args, file_len, stdout, stderr),
        "csv" => cmd_csv(input, &args, stdout, stderr, pending),
        "correlate" => cmd_correlate(input, &args, file_len, stdout, stderr),
        _ => Err(Failure::new(ExitClass::Usage, "unknown command")),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Test assertions only.
#[path = "main_tests.rs"]
mod main_tests;
