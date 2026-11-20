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
use irig106_time::network_time::{LeapSecondTable, NetworkTimeProtocol, parse_time_f2_payload};
use irig106_time::*;
/// Strict, filesystem-free argument parsing.
pub mod args;
mod csv_spool;
/// Bounded raw packet transport; command integration follows in S16b.
pub mod input;
mod target_refs;

// ────────────────────────────────────────────────────────────────────
// Constants
// ────────────────────────────────────────────────────────────────────

const DATA_TYPE_TIME_F1: u8 = 0x11;
const DATA_TYPE_TIME_F2: u8 = 0x12;
const FINDING_SAMPLE_LIMIT: usize = 20;
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
    /// For Format 2 channels: the specific network protocol (NTP or PTP).
    network_protocol: Option<NetworkTimeProtocol>,
    first_time: Option<AbsoluteTime>,
    last_time: Option<AbsoluteTime>,
    first_rtc: Option<Rtc>,
    last_rtc: Option<Rtc>,
    // Last non-backward reference used for jump/gap pairs. Displayed Last
    // remains the latest arrival, including accepted late references.
    pair_baseline: Option<(Rtc, AbsoluteTime)>,
    late: Vec<ObservedReference>,
    segment: usize,
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
            network_protocol: None,
            first_time: None,
            last_time: None,
            first_rtc: None,
            last_rtc: None,
            pair_baseline: None,
            late: Vec::new(),
            segment: 1,
            min_gap_ns: None,
            max_gap_ns: None,
        }
    }
}

// ────────────────────────────────────────────────────────────────────
// Resolved packet record (for timeline / CSV)
// ────────────────────────────────────────────────────────────────────

struct ResolvedPacket {
    packet_num: usize,
    offset: u64,
    channel_id: u16,
    data_type: u8,
    rtc: Rtc,
    abs_time: Option<AbsoluteTime>,
}

type RowSink<'a> = &'a mut dyn FnMut(&ResolvedPacket) -> io::Result<()>;

// ────────────────────────────────────────────────────────────────────
// File scanner
// ────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
struct ObservedReference {
    rtc: Rtc,
    time: AbsoluteTime,
    packet: usize,
    index: usize,
}

struct Ch10TimeScanner {
    correlator: StreamingTimeCorrelator,
    query_refs: Option<target_refs::TargetReferences>,
    pending_channels: usize,
    reset_count: usize,
    references_seen: usize,
    jump_threshold: Option<u64>,
    jumps_seen: usize,
    jumps: Vec<TimeJump>,
    leap_table: LeapSecondTable,
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
                NanosDuration::new(30_000_000_000),
                REFERENCE_CAP,
            ),
            query_refs: None,
            pending_channels: 0,
            reset_count: 0,
            references_seen: 0,
            jump_threshold: None,
            jumps_seen: 0,
            jumps: Vec::new(),
            leap_table: LeapSecondTable::builtin(),
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
            if hdr.data_type == DATA_TYPE_TIME_F1 {
                self.time_packets += 1;
                self.process_time_packet(&packet.body_prefix, &hdr);
            } else if hdr.data_type == DATA_TYPE_TIME_F2 {
                self.time_packets += 1;
                self.process_time_f2_packet(&packet.body_prefix, &hdr);
            }
            if resolve_all && limit.is_none_or(|n| self.total_packets < n) {
                let row = ResolvedPacket {
                    packet_num: self.total_packets,
                    offset: packet.offset,
                    channel_id: hdr.channel_id,
                    data_type: hdr.data_type,
                    rtc: hdr.rtc,
                    abs_time: self.resolve(hdr.rtc, None),
                };
                if let Some(write_row) = emit.as_mut() {
                    write_row(&row)?;
                } else {
                    self.resolved.push(row);
                }
            }
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
        if self.reset_count != 0 {
            return Err(TimeError::NeedsContext {
                context: "RTC reset: segment-qualified answers unavailable",
            });
        }
        if self.pending_channels != 0 {
            return Err(TimeError::NeedsContext {
                context: "RTC reset classification pending",
            });
        }
        if let Some(query) = &self.query_refs {
            if query.target == rtc {
                return query.correlate(channel);
            }
        }
        self.correlator.correlate(rtc, channel)
    }
    fn resolve(&mut self, rtc: Rtc, channel: Option<u16>) -> Option<AbsoluteTime> {
        match self.correlate(rtc, channel) {
            Ok(t) => Some(t),
            Err(e) => {
                self.note(decode_level(&e), format!("Correlation query: {e}"));
                None
            }
        }
    }

    fn observe_reference(&mut self, hdr: &PktHeader, time: AbsoluteTime) {
        let point = ObservedReference {
            rtc: hdr.rtc,
            time,
            packet: self.total_packets,
            index: self.references_seen,
        };
        self.references_seen += 1;
        self.cache_reference(hdr.channel_id, point);
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
            if previous.as_raw() - point.rtc.as_raw() > 300_000_000 {
                self.report_late(hdr.channel_id, late);
                self.start_reset(hdr.channel_id, previous, point);
                self.cache_reference(hdr.channel_id, point);
                self.observe_pair(hdr.channel_id, point);
            } else {
                if late.last().is_some_and(|p| p.rtc >= point.rtc) {
                    self.report_late(hdr.channel_id, core::mem::take(&mut late));
                }
                late.push(point);
                if late.len() == 3 {
                    if let Some(first) = late.first().copied() {
                        self.start_reset(hdr.channel_id, previous, first);
                    }
                    for p in late {
                        self.cache_reference(hdr.channel_id, p);
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
        self.correlator
            .add_reference(channel, point.rtc, point.time);
        if let Some(query) = self.query_refs.as_mut() {
            query.observe(channel, point.rtc, point.time);
        }
    }
    fn report_late(&mut self, channel: u16, late: Vec<ObservedReference>) {
        if let Some(previous) = self
            .time_channels
            .get(&channel)
            .and_then(|c| c.pair_baseline.map(|p| p.0))
        {
            for point in late {
                self.note(
                    Severity::Warning,
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
        self.note(
            Severity::Warning,
            format!(
                "RTC reset on channel {} at packet {}: {:#X} -> {:#X}",
                channel,
                first.packet,
                previous.as_raw(),
                first.rtc.as_raw()
            ),
        );
        self.reset_count += 1;
        self.correlator.clear_channel(channel);
        if let Some(query) = self.query_refs.as_mut() {
            query.clear_channel(channel);
        }
        if let Some(info) = self.time_channels.get_mut(&channel) {
            info.pair_baseline = None;
            info.segment += 1;
        }
    }
    fn observe_pair(&mut self, channel: u16, point: ObservedReference) {
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
        if let Some(threshold) = self.jump_threshold {
            if let Some((rtc, previous)) = self
                .time_channels
                .get(&hdr.channel_id)
                .and_then(|c| c.pair_baseline)
            {
                // Adjacent non-backward observations: online state never sorts a file.
                let expected_nanos = previous.as_total_ns() + rtc.elapsed_nanos(hdr.rtc);
                let actual_nanos = time.as_total_ns();
                let delta_nanos = actual_nanos as i64 - expected_nanos as i64;
                if delta_nanos.unsigned_abs() > threshold {
                    self.jumps_seen += 1;
                    if self.jumps.len() < FINDING_SAMPLE_LIMIT {
                        self.jumps.push(TimeJump {
                            index: point.index,
                            channel_id: hdr.channel_id,
                            expected_nanos,
                            actual_nanos,
                            delta_nanos,
                        });
                    }
                    self.note(
                        Severity::Warning,
                        format!(
                            "Time jump on channel {} at reference {}",
                            hdr.channel_id, point.index
                        ),
                    );
                }
            }
        }
        self.time_channels
            .entry(hdr.channel_id)
            .or_insert_with(|| TimeChannelInfo::new(hdr.channel_id))
            .pair_baseline = Some((hdr.rtc, time));
    }

    fn report_jump_samples(&self, out: &mut dyn Write) -> io::Result<()> {
        for j in &self.jumps {
            writeln!(
                out,
                "  Channel {:>3} Ref #{:<4} Expected: {} ns Actual: {} ns Delta: {:+.3} ms",
                j.channel_id,
                j.index,
                j.expected_nanos,
                j.actual_nanos,
                j.delta_nanos as f64 / 1_000_000.0
            )?;
        }
        if self.jumps_seen > self.jumps.len() {
            writeln!(
                out,
                "  ... and {} more jump details",
                self.jumps_seen - self.jumps.len()
            )?;
        }
        Ok(())
    }

    fn note(&mut self, level: Severity, message: impl Into<String>) {
        self.observe(level);
        // <=3 facts per complete packet plus terminal/query facts. Checked
        // u64 byte offsets and >=24-byte frames bound these counts below u64::MAX.
        self.finding_counts[usize::from(rank(level))] += 1;
        if self.errors.len() < FINDING_SAMPLE_LIMIT {
            self.errors.push(format!("{level:?}: {}", message.into()));
        }
    }

    fn observe(&mut self, level: Severity) {
        if self.highest.is_none_or(|old| rank(level) > rank(old)) {
            self.highest = Some(level);
        }
    }

    fn finding_total(&self) -> u64 {
        self.finding_counts.iter().sum()
    }
    fn omitted_findings(&self) -> u64 {
        self.finding_total() - self.errors.len() as u64
    }
    fn report_finding_counts(&self, out: &mut dyn Write) -> io::Result<()> {
        writeln!(
            out,
            "Findings: error={}, warning={}, info={} (total={})",
            self.finding_counts[2],
            self.finding_counts[1],
            self.finding_counts[0],
            self.finding_total()
        )
    }
    fn report_findings(&self, stderr: &mut dyn Write) -> Result<(), Failure> {
        if self.finding_total() != 0 {
            self.report_finding_counts(stderr)?;
        }
        for error in &self.errors {
            writeln!(stderr, "Finding: {error}")?;
        }
        if self.omitted_findings() != 0 {
            writeln!(
                stderr,
                "Finding: ... and {} more findings",
                self.omitted_findings()
            )?;
        }
        Ok(())
    }

    fn process_time_packet(&mut self, payload: &[u8], hdr: &PktHeader) {
        let Some(csdw_bytes) = payload.get(..4) else {
            self.note(
                Severity::Error,
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
                    self.note(
                        Severity::Error,
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
                        self.note(
                            decode_level(&e),
                            format!("Time packet ch={}: DOY BCD error: {}", hdr.channel_id, e),
                        );
                        return;
                    }
                }
            }
            DateFormat::DayMonthYear => {
                if bcd_start + 8 > payload.len() {
                    self.note(
                        Severity::Error,
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
                            self.note(
                                decode_level(&e),
                                format!(
                                    "Time packet ch={}: DMY date invalid: {}",
                                    hdr.channel_id, e
                                ),
                            );
                            return;
                        }
                    },
                    Err(e) => {
                        self.note(
                            decode_level(&e),
                            format!("Time packet ch={}: DMY BCD error: {}", hdr.channel_id, e),
                        );
                        return;
                    }
                }
            }
        };

        self.observe_reference(hdr, abs_time);
        // Update channel info
        let info = self
            .time_channels
            .entry(hdr.channel_id)
            .or_insert_with(|| TimeChannelInfo::new(hdr.channel_id));
        info.packet_count += 1;
        info.source = Some(csdw.time_source());
        info.format = Some(csdw.time_format());
        info.date_format = Some(csdw.date_format());
        info.is_leap_year = Some(csdw.is_leap_year());
        if info.first_time.is_none() {
            info.first_time = Some(abs_time);
            info.first_rtc = Some(hdr.rtc);
        }
        info.last_time = Some(abs_time);
        info.last_rtc = Some(hdr.rtc);
    }

    fn process_time_f2_packet(&mut self, payload: &[u8], hdr: &PktHeader) {
        let (csdw, network_time) = match parse_time_f2_payload(payload) {
            Ok(result) => result,
            Err(e) => {
                self.note(
                    decode_level(&e),
                    format!("Time F2 packet ch={}: parse error: {}", hdr.channel_id, e),
                );
                return;
            }
        };

        // Same compatibility conversion as StreamingTimeCorrelator, performed
        // before eviction so late PTP references can still bracket a known query.
        let converted = match network_time {
            irig106_time::network_time::NetworkTime::Ntp(_) => Err(TimeError::NeedsContext {
                context: "NTP era/basis required; use ntp::convert_ntp",
            }),
            irig106_time::network_time::NetworkTime::Ptp(ptp) => {
                self.leap_table.convert_ptp(ptp).and_then(|result| {
                    result
                        .utc()
                        .ok_or(TimeError::NeedsContext {
                            context: "no known applicable TAI-UTC offset",
                        })?
                        .to_absolute()
                        .map(|t| t.value)
                })
            }
        };
        let abs_time = match converted {
            Ok(t) => t,
            Err(e) => {
                self.note(
                    decode_level(&e),
                    format!(
                        "Time F2 packet ch={}: correlation error: {}",
                        hdr.channel_id, e
                    ),
                );
                return;
            }
        };
        self.observe_reference(hdr, abs_time);
        // Update channel info
        let info = self
            .time_channels
            .entry(hdr.channel_id)
            .or_insert_with(|| TimeChannelInfo::new(hdr.channel_id));
        info.packet_count += 1;
        // Map network protocol to TimeSource/TimeFormat for display
        info.network_protocol = Some(csdw.time_protocol());
        info.source = Some(match csdw.time_protocol() {
            NetworkTimeProtocol::Ntp => TimeSource::External,
            NetworkTimeProtocol::Ptp => TimeSource::External,
            NetworkTimeProtocol::Reserved(_) => TimeSource::None,
        });
        info.format = Some(match csdw.time_protocol() {
            NetworkTimeProtocol::Ntp => TimeFormat::Utc,
            NetworkTimeProtocol::Ptp => TimeFormat::Gps, // closest analog
            NetworkTimeProtocol::Reserved(_) => TimeFormat::Reserved(0xFF),
        });
        info.date_format = Some(DateFormat::DayOfYear); // F2 doesn't use BCD date format
        info.is_leap_year = Some(false); // not applicable for F2
        if info.first_time.is_none() {
            info.first_time = Some(abs_time);
            info.first_rtc = Some(hdr.rtc);
        }
        info.last_time = Some(abs_time);
        info.last_rtc = Some(hdr.rtc);
    }
}

// ────────────────────────────────────────────────────────────────────
// Formatting helpers
// ────────────────────────────────────────────────────────────────────

fn fmt_abs_time(t: &AbsoluteTime) -> String {
    let ms = t.nanoseconds() / 1_000_000;
    let us = (t.nanoseconds() % 1_000_000) / 1_000;
    match t.year() {
        Some(y) => {
            format!(
                "{:04} Day {:03} {:02}:{:02}:{:02}.{:03}.{:03}",
                y,
                t.day_of_year(),
                t.hours(),
                t.minutes(),
                t.seconds(),
                ms,
                us
            )
        }
        None => {
            format!(
                "Day {:03} {:02}:{:02}:{:02}.{:03}.{:03}",
                t.day_of_year(),
                t.hours(),
                t.minutes(),
                t.seconds(),
                ms,
                us
            )
        }
    }
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
    input: std::fs::File,
    file_len: u64,
    filename: &Path,
    stdout: &mut dyn Write,
) -> Result<Option<Severity>, Failure> {
    let mut scanner = Ch10TimeScanner::new();
    scanner.jump_threshold = Some(1_000_000_000);
    scanner.scan(input, false, None)?;

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
        "Time Packets (0x11): {}",
        fmt_comma(scanner.time_packets)
    )?;
    writeln!(
        stdout,
        "Time Channels      : {}",
        scanner.time_channels.len()
    )?;
    writeln!(stdout, "Correlation Refs   : {}", scanner.correlator.len())?;
    writeln!(stdout, "References Seen    : {}", scanner.references_seen)?;
    writeln!(
        stdout,
        "References Evicted : {}",
        scanner.correlator.total_evicted()
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
            let proto_str = match info.network_protocol {
                Some(NetworkTimeProtocol::Ntp) => "  Proto: NTP",
                Some(NetworkTimeProtocol::Ptp) => "  Proto: PTP",
                Some(NetworkTimeProtocol::Reserved(_)) => "  Proto: ?",
                None => "",
            };
            writeln!(
                stdout,
                "  Channel {:>3}  │  Source: {:<12}  Format: {:<8}  Date: {:<4}  Leap: {}  Packets: {}{}",
                info.channel_id,
                info.source.as_ref().map(fmt_time_source).unwrap_or("?"),
                info.format.as_ref().map(fmt_time_format).unwrap_or("?"),
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
            if let Some(ref ft) = info.first_time {
                writeln!(stdout, "                │  First: {}", fmt_abs_time(ft))?;
            }
            if let Some(ref lt) = info.last_time {
                writeln!(stdout, "                │  Last:  {}", fmt_abs_time(lt))?;
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
    for (dt, count) in &scanner.data_type_counts {
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
    input: std::fs::File,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<Option<Severity>, Failure> {
    let mut scanner = Ch10TimeScanner::new();
    scanner.scan(input, false, None)?;
    scanner.report_findings(stderr)?;

    writeln!(
        stdout,
        "{:<8}  {:<12}  {:<10}  {:<5}  {:<5}  {:<5}  {:>8}  {:>28}  {:>28}",
        "Channel",
        "Source",
        "Format",
        "Date",
        "Leap",
        "Proto",
        "Packets",
        "First Time",
        "Last Time"
    )?;
    writeln!(stdout, "{}", "─".repeat(126))?;

    for info in scanner.time_channels.values() {
        let proto = match info.network_protocol {
            Some(NetworkTimeProtocol::Ntp) => "NTP",
            Some(NetworkTimeProtocol::Ptp) => "PTP",
            Some(NetworkTimeProtocol::Reserved(_)) => "?",
            None => "—",
        };
        writeln!(
            stdout,
            "{:>7}  {:<12}  {:<10}  {:<5}  {:<5}  {:<5}  {:>8}  {:>28}  {:>28}",
            info.channel_id,
            info.source.as_ref().map(fmt_time_source).unwrap_or("?"),
            info.format.as_ref().map(fmt_time_format).unwrap_or("?"),
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
            proto,
            fmt_comma(info.packet_count),
            info.first_time
                .as_ref()
                .map(fmt_abs_time)
                .unwrap_or_default(),
            info.last_time
                .as_ref()
                .map(fmt_abs_time)
                .unwrap_or_default(),
        )?;
    }
    Ok(scanner.highest)
}

fn cmd_jumps(
    input: std::fs::File,
    threshold_ms: u64,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<Option<Severity>, Failure> {
    let threshold = threshold_ms.checked_mul(1_000_000).ok_or_else(|| {
        Failure::new(
            ExitClass::Usage,
            "threshold milliseconds cannot be represented in nanoseconds",
        )
    })?;
    let mut scanner = Ch10TimeScanner::new();
    scanner.jump_threshold = Some(threshold);
    scanner.scan(input, false, None)?;
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
            scanner.time_channels.len()
        )?;
    } else {
        writeln!(
            stdout,
            "Total: {} jump(s) across {} channel(s).",
            scanner.jumps_seen,
            scanner.time_channels.len()
        )?;
    }
    scanner.report_findings(stderr)?;
    Ok(scanner.highest)
}

fn cmd_timeline(
    input: std::fs::File,
    limit: usize,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<Option<Severity>, Failure> {
    let mut scanner = Ch10TimeScanner::new();
    scanner.scan(input, true, Some(limit))?;
    scanner.report_findings(stderr)?;

    writeln!(
        stdout,
        "{:>8}  {:>12}  {:>6}  {:<14}  {:>14}  {:>28}",
        "Pkt#", "Offset", "Ch", "Type", "RTC", "Absolute Time"
    )?;
    writeln!(stdout, "{}", "─".repeat(100))?;

    for r in &scanner.resolved {
        writeln!(
            stdout,
            "{:>8}  {:>#12X}  {:>6}  {:<14}  {:>14}  {:>28}",
            r.packet_num,
            r.offset,
            r.channel_id,
            fmt_data_type_short(r.data_type),
            r.rtc.as_raw(),
            r.abs_time
                .as_ref()
                .map(fmt_abs_time)
                .unwrap_or_else(|| "N/A".to_string()),
        )?;
    }

    if scanner.total_packets > limit {
        writeln!(
            stdout,
            "... showing first {} of {} packets",
            limit, scanner.total_packets
        )?;
    }
    Ok(scanner.highest)
}

fn cmd_csv(
    input: std::fs::File,
    output_path: Option<&Path>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    pending: &mut Option<csv_spool::CsvSpool>,
) -> Result<Option<Severity>, Failure> {
    let mut spool = csv_spool::CsvSpool::new(output_path)?;
    writeln!(
        spool,
        "packet_num,offset_hex,channel_id,data_type_hex,data_type_name,rtc_raw,rtc_nanos,day_of_year,hours,minutes,seconds,nanoseconds,year,month,day"
    )?;
    let mut scanner = Ch10TimeScanner::new();
    let mut emit = |row: &ResolvedPacket| write_csv_row(&mut spool, row);
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

fn write_csv_row(out: &mut dyn Write, r: &ResolvedPacket) -> io::Result<()> {
    match &r.abs_time {
        Some(t) => {
            writeln!(
                out,
                "{},{:#X},{},0x{:02X},{},{},{},{},{},{},{},{},{},,",
                r.packet_num,
                r.offset,
                r.channel_id,
                r.data_type,
                fmt_data_type_short(r.data_type),
                r.rtc.as_raw(),
                r.rtc.to_nanos(),
                t.day_of_year(),
                t.hours(),
                t.minutes(),
                t.seconds(),
                t.nanoseconds(),
                t.year().map(|y| y.to_string()).unwrap_or_default(),
            )?;
        }
        None => {
            writeln!(
                out,
                "{},{:#X},{},0x{:02X},{},{},{},,,,,,,",
                r.packet_num,
                r.offset,
                r.channel_id,
                r.data_type,
                fmt_data_type_short(r.data_type),
                r.rtc.as_raw(),
                r.rtc.to_nanos(),
            )?;
        }
    }
    Ok(())
}

fn cmd_correlate(
    input: std::fs::File,
    target: Rtc,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<Option<Severity>, Failure> {
    let mut scanner = Ch10TimeScanner::new();
    scanner.query_refs = Some(target_refs::TargetReferences::new(target));
    scanner.scan(input, false, None)?;

    writeln!(
        stdout,
        "Resolving RTC {:#014X} ({} ticks, {} ns)",
        target.as_raw(),
        target.as_raw(),
        target.to_nanos()
    )?;
    writeln!(stdout)?;

    if scanner.reset_count != 0 {
        scanner.note(
            Severity::Warning,
            "RTC reset: segment-qualified answers unavailable",
        );
        scanner.report_findings(stderr)?;
        return Ok(scanner.highest);
    }
    if scanner.correlator.is_empty() {
        scanner.note(Severity::Warning, "No time reference points found in file.");
        scanner.report_findings(stderr)?;
        return Ok(scanner.highest);
    }

    // Correlate with all channels
    match scanner.correlate(target, None) {
        Ok(t) => writeln!(stdout, "  Any channel  → {}", fmt_abs_time(&t))?,
        Err(e) => {
            let level = decode_level(&e);
            writeln!(stdout, "  Any channel  → {level:?}: {e}")?;
            scanner.note(level, format!("Correlation query: {e}"));
        }
    }

    // Per-channel
    let channels: Vec<u16> = scanner.time_channels.keys().copied().collect();
    for ch in channels {
        if let Some(t) = scanner.resolve(target, Some(ch)) {
            writeln!(stdout, "  Channel {:>3}  → {}", ch, fmt_abs_time(&t))?;
        }
    }
    scanner.report_findings(stderr)?;
    Ok(scanner.highest)
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
        "Correlation: 1024 references globally / 30 seconds RTC; old/empty-window answers: warning (coordinator-approved)."
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
        "summary" => cmd_summary(input, file_len, &args.input, stdout),
        "channels" => cmd_channels(input, stdout, stderr),
        "jumps" => cmd_jumps(input, args.threshold_ms, stdout, stderr),
        "timeline" => cmd_timeline(input, args.limit, stdout, stderr),
        "csv" => cmd_csv(input, args.output.as_deref(), stdout, stderr, pending),
        "correlate" => cmd_correlate(input, args.rtc, stdout, stderr),
        _ => Err(Failure::new(ExitClass::Usage, "unknown command")),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Test assertions only.
#[path = "main_tests.rs"]
mod main_tests;
