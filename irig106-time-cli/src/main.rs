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
use irig106_time::network_time::{LeapSecondTable, NetworkTimeProtocol, parse_time_f2_payload};
use irig106_time::*;

// ────────────────────────────────────────────────────────────────────
// Constants
// ────────────────────────────────────────────────────────────────────

const SYNC_PATTERN: u16 = 0xEB25;
const HEADER_SIZE: usize = 24;
const SECONDARY_HEADER_SIZE: usize = 12;
const DATA_TYPE_TIME_F1: u8 = 0x11;
const DATA_TYPE_TIME_F2: u8 = 0x12;
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

// ────────────────────────────────────────────────────────────────────
// Minimal Ch10 header parsing (just enough for time extraction)
// ────────────────────────────────────────────────────────────────────

struct PktHeader {
    channel_id: u16,
    packet_length: u32,
    _data_length: u32,
    packet_flags: u8,
    data_type: u8,
    rtc: Rtc,
}

impl PktHeader {
    fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() < HEADER_SIZE {
            return None;
        }
        let sync = u16::from_le_bytes([buf[0], buf[1]]);
        if sync != SYNC_PATTERN {
            return None;
        }
        Some(PktHeader {
            channel_id: u16::from_le_bytes([buf[2], buf[3]]),
            packet_length: u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]),
            _data_length: u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]),
            packet_flags: buf[14],
            data_type: buf[15],
            rtc: Rtc::from_le_bytes([buf[16], buf[17], buf[18], buf[19], buf[20], buf[21]]),
        })
    }

    fn payload<'a>(&self, packet: &'a [u8]) -> Option<&'a [u8]> {
        packet
            .get(self.data_offset()..)?
            .get(..self._data_length as usize)
    }

    fn has_secondary_header(&self) -> bool {
        (self.packet_flags & 0x04) != 0
    }

    fn data_offset(&self) -> usize {
        HEADER_SIZE
            + if self.has_secondary_header() {
                SECONDARY_HEADER_SIZE
            } else {
                0
            }
    }
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
        }
    }
}

// ────────────────────────────────────────────────────────────────────
// Resolved packet record (for timeline / CSV)
// ────────────────────────────────────────────────────────────────────

struct ResolvedPacket {
    packet_num: usize,
    offset: usize,
    channel_id: u16,
    data_type: u8,
    rtc: Rtc,
    abs_time: Option<AbsoluteTime>,
}

// ────────────────────────────────────────────────────────────────────
// File scanner
// ────────────────────────────────────────────────────────────────────

struct Ch10TimeScanner {
    correlator: TimeCorrelator,
    leap_table: LeapSecondTable,
    time_channels: BTreeMap<u16, TimeChannelInfo>,
    total_packets: usize,
    time_packets: usize,
    data_type_counts: BTreeMap<u8, usize>,
    first_rtc: Option<Rtc>,
    last_rtc: Option<Rtc>,
    resolved: Vec<ResolvedPacket>,
    errors: Vec<String>,
}

impl Ch10TimeScanner {
    fn new() -> Self {
        Self {
            correlator: TimeCorrelator::new(),
            leap_table: LeapSecondTable::builtin(),
            time_channels: BTreeMap::new(),
            total_packets: 0,
            time_packets: 0,
            data_type_counts: BTreeMap::new(),
            first_rtc: None,
            last_rtc: None,
            resolved: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn scan(
        &mut self,
        mmap: &[u8],
        resolve_all: bool,
        limit: Option<usize>,
    ) -> Result<(), Failure> {
        let mut offset = 0;
        let file_len = mmap.len();

        while offset < file_len {
            if let Some(limit) = limit {
                if self.total_packets >= limit {
                    break;
                }
            }

            let remaining = mmap
                .get(offset..)
                .ok_or_else(|| Failure::new(ExitClass::Input, "invalid packet offset"))?;
            if remaining.len() < HEADER_SIZE {
                self.errors.push(format!(
                    "truncated final header at {offset:#X} ({} bytes)",
                    remaining.len()
                ));
                break;
            }
            let hdr = match PktHeader::parse(remaining) {
                Some(h) => h,
                None => {
                    // Try to find next sync
                    if let Some(next) = find_next_sync(mmap, offset + 1) {
                        self.errors.push(format!(
                            "Sync lost at {:#X}, recovered at {:#X} ({} bytes skipped)",
                            offset,
                            next,
                            next - offset
                        ));
                        offset = next;
                        continue;
                    } else if self.total_packets > 0 {
                        self.errors.push(format!(
                            "{} trailing bytes without sync at offset {offset:#X}",
                            remaining.len()
                        ));
                        break;
                    } else {
                        return Err(Failure::new(ExitClass::Input, "invalid packet sync"));
                    }
                }
            };

            let pkt_len = hdr.packet_length as usize;
            // Chapter11 §11.2.1.4: filler makes the packet a multiple of4.
            if pkt_len < hdr.data_offset() || pkt_len % 4 != 0 {
                return Err(Failure::new(ExitClass::Input, "invalid packet length"));
            }
            if hdr._data_length as usize > pkt_len - hdr.data_offset() {
                return Err(Failure::new(
                    ExitClass::Input,
                    "Data Length exceeds packet body",
                ));
            }
            let Some(packet) = remaining.get(..pkt_len) else {
                self.errors.push(format!(
                    "truncated final packet at {offset:#X}: declared {pkt_len}, available {} bytes",
                    remaining.len()
                ));
                break;
            };
            if hdr.payload(packet).is_none() {
                return Err(Failure::new(
                    ExitClass::Input,
                    "Data Length exceeds packet body",
                ));
            }

            // Track RTC range
            if self.first_rtc.is_none() {
                self.first_rtc = Some(hdr.rtc);
            }
            self.last_rtc = Some(hdr.rtc);

            *self.data_type_counts.entry(hdr.data_type).or_insert(0) += 1;

            // Process time packets
            if hdr.data_type == DATA_TYPE_TIME_F1 {
                self.time_packets += 1;
                self.process_time_packet(packet, &hdr);
            } else if hdr.data_type == DATA_TYPE_TIME_F2 {
                self.time_packets += 1;
                self.process_time_f2_packet(packet, &hdr);
            }

            // Optionally resolve all packets
            if resolve_all {
                let abs = self.correlator.correlate(hdr.rtc, None).ok();
                self.resolved.push(ResolvedPacket {
                    packet_num: self.total_packets,
                    offset,
                    channel_id: hdr.channel_id,
                    data_type: hdr.data_type,
                    rtc: hdr.rtc,
                    abs_time: abs,
                });
            }

            self.total_packets += 1;
            offset += packet.len(); // Bounded by remaining.len(), so cannot overflow.
        }
        Ok(())
    }

    fn report_findings(&self, stderr: &mut dyn Write) -> Result<(), Failure> {
        for error in &self.errors {
            writeln!(stderr, "Finding: {error}")?;
        }
        Ok(())
    }

    fn process_time_packet(&mut self, pkt_buf: &[u8], hdr: &PktHeader) {
        let Some(payload) = hdr.payload(pkt_buf) else {
            self.errors.push("invalid Format 1 body bounds".into());
            return;
        };
        let Some(csdw_bytes) = payload.get(..4) else {
            self.errors.push(format!(
                "Time packet at ch={} too short for CSDW",
                hdr.channel_id
            ));
            return;
        };
        let csdw =
            TimeF1Csdw::from_le_bytes([csdw_bytes[0], csdw_bytes[1], csdw_bytes[2], csdw_bytes[3]]);

        let bcd_start = 4;
        let abs_time = match csdw.date_format() {
            DateFormat::DayOfYear => {
                if bcd_start + 6 > payload.len() {
                    self.errors.push(format!(
                        "Time packet ch={}: buffer too short for DOY BCD",
                        hdr.channel_id
                    ));
                    return;
                }
                match DayFormatTime::from_le_bytes(&payload[bcd_start..]) {
                    Ok(t) => t.to_absolute(),
                    Err(e) => {
                        self.errors.push(format!(
                            "Time packet ch={}: DOY BCD error: {}",
                            hdr.channel_id, e
                        ));
                        return;
                    }
                }
            }
            DateFormat::DayMonthYear => {
                if bcd_start + 8 > payload.len() {
                    self.errors.push(format!(
                        "Time packet ch={}: buffer too short for DMY BCD",
                        hdr.channel_id
                    ));
                    return;
                }
                match DmyFormatTime::from_le_bytes(&payload[bcd_start..]) {
                    Ok(t) => match t.to_calendar_time() {
                        Ok(ct) => ct.into_absolute_time(),
                        Err(e) => {
                            self.errors.push(format!(
                                "Time packet ch={}: DMY date invalid: {}",
                                hdr.channel_id, e
                            ));
                            return;
                        }
                    },
                    Err(e) => {
                        self.errors.push(format!(
                            "Time packet ch={}: DMY BCD error: {}",
                            hdr.channel_id, e
                        ));
                        return;
                    }
                }
            }
        };

        // Add to correlator
        self.correlator
            .add_reference(hdr.channel_id, hdr.rtc, abs_time);

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

    fn process_time_f2_packet(&mut self, pkt_buf: &[u8], hdr: &PktHeader) {
        let Some(payload) = hdr.payload(pkt_buf) else {
            self.errors.push("invalid Format 2 body bounds".into());
            return;
        };

        let (csdw, network_time) = match parse_time_f2_payload(payload) {
            Ok(result) => result,
            Err(e) => {
                self.errors.push(format!(
                    "Time F2 packet ch={}: parse error: {}",
                    hdr.channel_id, e
                ));
                return;
            }
        };

        // Use the library's add_reference_f2 which handles NTP/PTP→AbsoluteTime
        // conversion and leap-second offset application internally.
        if let Err(e) = self.correlator.add_reference_f2(
            hdr.channel_id,
            hdr.rtc,
            &network_time,
            &self.leap_table,
        ) {
            self.errors.push(format!(
                "Time F2 packet ch={}: correlation error: {}",
                hdr.channel_id, e
            ));
            return;
        }

        // Retrieve the resolved absolute time for channel info tracking.
        // We just inserted a reference at this exact RTC, so correlate is an exact match.
        let abs_time = match self.correlator.correlate(hdr.rtc, Some(hdr.channel_id)) {
            Ok(t) => t,
            Err(_) => return,
        };

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

fn find_next_sync(buf: &[u8], start: usize) -> Option<usize> {
    let tail = buf.get(start..)?;
    for (offset, header) in tail.windows(HEADER_SIZE).enumerate() {
        if u16::from_le_bytes([header[0], header[1]]) == SYNC_PATTERN {
            let sum = header[..22].chunks_exact(2).fold(0u16, |sum, word| {
                sum.wrapping_add(u16::from_le_bytes([word[0], word[1]]))
            });
            if sum == u16::from_le_bytes([header[22], header[23]]) {
                return start.checked_add(offset);
            }
        }
    }
    None
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

fn fmt_comma(n: usize) -> String {
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

fn cmd_summary(mmap: &[u8], filename: &Path, stdout: &mut dyn Write) -> Result<(), Failure> {
    let mut scanner = Ch10TimeScanner::new();
    scanner.scan(mmap, false, None)?;

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
    writeln!(
        stdout,
        "File Size          : {} bytes",
        fmt_comma(mmap.len())
    )?;
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
    writeln!(stdout)?;

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
        }
        writeln!(stdout)?;
    }

    // Time jump detection
    let all_channels: Vec<u16> = scanner.time_channels.keys().cloned().collect();
    let mut any_jumps = false;
    for ch in &all_channels {
        let jumps = scanner
            .correlator
            .detect_time_jump(*ch, NanosDuration::new(1_000_000_000));
        if !jumps.is_empty() {
            if !any_jumps {
                writeln!(stdout, "Time Jumps Detected (threshold: 1 second)")?;
                writeln!(stdout, "──────────────────────────────────────────")?;
                any_jumps = true;
            }
            for j in &jumps {
                let delta_ms = j.delta_nanos as f64 / 1_000_000.0;
                writeln!(
                    stdout,
                    "  Channel {:>3}  │  Ref #{:<4}  Delta: {:>+10.1} ms  ({})",
                    j.channel_id,
                    j.index,
                    delta_ms,
                    if j.delta_nanos > 0 {
                        "jump forward"
                    } else {
                        "jump backward"
                    }
                )?;
            }
        }
    }
    if !any_jumps && !all_channels.is_empty() {
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

    if !scanner.errors.is_empty() {
        writeln!(stdout, "Errors ({})", scanner.errors.len())?;
        writeln!(stdout, "──────")?;
        for (i, e) in scanner.errors.iter().enumerate().take(20) {
            writeln!(stdout, "  {}. {}", i + 1, e)?;
        }
        if scanner.errors.len() > 20 {
            writeln!(stdout, "  ... and {} more", scanner.errors.len() - 20)?;
        }
    }
    Ok(())
}

fn cmd_channels(
    mmap: &[u8],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), Failure> {
    let mut scanner = Ch10TimeScanner::new();
    scanner.scan(mmap, false, None)?;
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
    Ok(())
}

fn cmd_jumps(
    mmap: &[u8],
    threshold_ms: u64,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), Failure> {
    let mut scanner = Ch10TimeScanner::new();
    scanner.scan(mmap, false, None)?;
    scanner.report_findings(stderr)?;

    let threshold_ns = threshold_ms.checked_mul(1_000_000).ok_or_else(|| {
        Failure::new(
            ExitClass::Usage,
            "threshold milliseconds cannot be represented in nanoseconds",
        )
    })?;
    let channels: Vec<u16> = scanner.time_channels.keys().cloned().collect();
    let mut total_jumps = 0;

    writeln!(
        stdout,
        "Time Jump Detection (threshold: {} ms)",
        threshold_ms
    )?;
    writeln!(stdout, "{}", "═".repeat(70))?;

    for ch in &channels {
        let jumps = scanner
            .correlator
            .detect_time_jump(*ch, NanosDuration::new(threshold_ns));
        if !jumps.is_empty() {
            writeln!(stdout)?;
            writeln!(stdout, "Channel {} — {} jump(s):", ch, jumps.len())?;
            for j in &jumps {
                let delta_ms = j.delta_nanos as f64 / 1_000_000.0;
                writeln!(
                    stdout,
                    "  Ref #{:<4}  Expected: {:>15} ns  Actual: {:>15} ns  Delta: {:>+12.3} ms",
                    j.index, j.expected_nanos, j.actual_nanos, delta_ms
                )?;
            }
            total_jumps += jumps.len();
        }
    }

    if total_jumps == 0 {
        writeln!(stdout)?;
        writeln!(
            stdout,
            "No time jumps detected across {} channel(s).",
            channels.len()
        )?;
    } else {
        writeln!(stdout)?;
        writeln!(
            stdout,
            "Total: {} jump(s) across {} channel(s).",
            total_jumps,
            channels.len()
        )?;
    }
    Ok(())
}

fn cmd_timeline(
    mmap: &[u8],
    limit: usize,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), Failure> {
    let mut scanner = Ch10TimeScanner::new();
    scanner.scan(mmap, true, Some(limit))?;
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
    Ok(())
}

fn cmd_csv(
    mmap: &[u8],
    output_path: Option<&Path>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), Failure> {
    let mut scanner = Ch10TimeScanner::new();
    scanner.scan(mmap, true, None)?;
    scanner.report_findings(stderr)?;

    let mut out: Box<dyn Write + '_> = match output_path {
        Some(path) => {
            let f = std::fs::File::create(path)?;
            Box::new(std::io::BufWriter::new(f))
        }
        None => Box::new(stdout),
    };

    writeln!(
        out,
        "packet_num,offset_hex,channel_id,data_type_hex,data_type_name,rtc_raw,rtc_nanos,day_of_year,hours,minutes,seconds,nanoseconds,year,month,day"
    )?;

    for r in &scanner.resolved {
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
    }

    out.flush()?;
    if let Some(path) = output_path {
        writeln!(
            stderr,
            "Wrote {} rows to {}",
            scanner.resolved.len(),
            path.display()
        )?;
    }
    Ok(())
}

fn cmd_correlate(
    mmap: &[u8],
    rtc_hex: &str,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), Failure> {
    let rtc_val = u64::from_str_radix(
        rtc_hex.trim_start_matches("0x").trim_start_matches("0X"),
        16,
    )
    .map_err(|e| Failure::new(ExitClass::Usage, format!("invalid hex RTC value: {e}")))?;
    let target = Rtc::from_raw(rtc_val);

    let mut scanner = Ch10TimeScanner::new();
    scanner.scan(mmap, false, None)?;
    scanner.report_findings(stderr)?;

    writeln!(
        stdout,
        "Resolving RTC {:#014X} ({} ticks, {} ns)",
        target.as_raw(),
        target.as_raw(),
        target.to_nanos()
    )?;
    writeln!(stdout)?;

    if scanner.correlator.is_empty() {
        return Err(Failure::new(
            ExitClass::Findings,
            "No time reference points found in file.",
        ));
    }

    // Correlate with all channels
    match scanner.correlator.correlate(target, None) {
        Ok(t) => writeln!(stdout, "  Any channel  → {}", fmt_abs_time(&t))?,
        Err(e) => writeln!(stdout, "  Any channel  → Error: {}", e)?,
    }

    // Per-channel
    for ch in scanner.time_channels.keys() {
        if let Ok(t) = scanner.correlator.correlate(target, Some(*ch)) {
            writeln!(stdout, "  Channel {:>3}  → {}", ch, fmt_abs_time(&t))?;
        }
    }
    Ok(())
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
/// S16a3 replaces the permissive legacy dispatcher with strict args validation.
/// Input still uses the S15b whole-file snapshot until S16b bounded streaming.
/// Traces: L3-CLI-101 <- L2-CLI-101 <- L1-CLI-001;
/// L3-CLI-102 <- L2-CLI-102 <- L1-CLI-002 (host/writer clauses only).
pub fn run<S: AsRef<OsStr>>(
    args: &[S],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> ExitClass {
    let result = dispatch(args, stdout, stderr).and_then(|()| {
        stdout.flush()?;
        stderr.flush()?;
        Ok(())
    });
    match result {
        Ok(()) => ExitClass::Success,
        Err(error) => {
            // Preserve the first input/output failure if its diagnostic also fails.
            if writeln!(stderr, "Error: {}", error.message)
                .and_then(|()| stderr.flush())
                .is_err()
                && error.class == ExitClass::Findings
            {
                ExitClass::Output
            } else {
                error.class
            }
        }
    }
}
fn dispatch<S: AsRef<OsStr>>(
    args: &[S],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), Failure> {
    let command = args
        .first()
        .map(AsRef::as_ref)
        .unwrap_or(OsStr::new("--help"))
        .to_str()
        .ok_or_else(|| Failure::new(ExitClass::Usage, "command is not UTF-8"))?;
    for (index, argument) in args.iter().enumerate().skip(1) {
        let input_path = index == 1
            && matches!(
                command,
                "summary" | "channels" | "jumps" | "timeline" | "csv" | "correlate"
            );
        let output_path = command == "csv"
            && args
                .get(index - 1)
                .is_some_and(|a| a.as_ref() == "--output");
        let option_name = argument.as_ref().as_encoded_bytes().starts_with(b"-") && !output_path;
        if argument.as_ref().to_str().is_none() && ((!input_path && !output_path) || option_name) {
            return Err(Failure::new(
                ExitClass::Usage,
                "command argument or option name is not UTF-8",
            ));
        }
    }
    if matches!(command, "-h" | "--help" | "help") {
        return print_usage(stdout);
    }
    if matches!(command, "-V" | "--version") {
        writeln!(stdout, "irigtime {}", PKG_VERSION)?;
        return Ok(());
    }
    if !matches!(
        command,
        "summary" | "channels" | "jumps" | "timeline" | "csv" | "correlate"
    ) {
        return Err(Failure::new(
            ExitClass::Usage,
            format!("Unknown command: '{command}'"),
        ));
    }
    let path = Path::new(
        args.get(1)
            .ok_or_else(|| Failure::new(ExitClass::Usage, "missing input file"))?
            .as_ref(),
    );
    let rtc = if command == "correlate" {
        Some(
            args.get(2)
                .and_then(|s| s.as_ref().to_str())
                .ok_or_else(|| Failure::new(ExitClass::Usage, "missing RTC"))?,
        )
    } else {
        None
    };
    // Safe owned snapshot; S16b will replace this with bounded streaming.
    let data = std::fs::read(path).map_err(|e| {
        Failure::new(
            ExitClass::Input,
            format!("cannot read '{}': {e}", path.display()),
        )
    })?;
    match command {
        "summary" => cmd_summary(&data, path, stdout),
        "channels" => cmd_channels(&data, stdout, stderr),
        "jumps" => {
            let threshold = args
                .iter()
                .position(|a| a.as_ref() == "--threshold-ms")
                .and_then(|i| args.get(i + 1))
                .and_then(|v| v.as_ref().to_str())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(1000);
            cmd_jumps(&data, threshold, stdout, stderr)
        }
        "timeline" => {
            let limit = args
                .iter()
                .position(|a| a.as_ref() == "--limit")
                .and_then(|i| args.get(i + 1))
                .and_then(|v| v.as_ref().to_str())
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(100);
            cmd_timeline(&data, limit, stdout, stderr)
        }
        "csv" => {
            let output = args
                .iter()
                .position(|a| a.as_ref() == "--output")
                .and_then(|i| args.get(i + 1))
                .map(|s| Path::new(s.as_ref()));
            cmd_csv(&data, output, stdout, stderr)
        }
        "correlate" => cmd_correlate(&data, rtc.unwrap_or_default(), stdout, stderr),
        _ => Err(Failure::new(ExitClass::Usage, "unknown command")),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Test assertions only.
#[path = "main_tests.rs"]
mod main_tests;
