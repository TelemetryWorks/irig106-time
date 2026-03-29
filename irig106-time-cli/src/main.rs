//! `ch10time` — CLI tool for inspecting IRIG 106 Chapter 10 time data.
//!
//! Reads a Ch10 file, extracts all time packets, builds a correlation table,
//! and produces various diagnostic outputs.
//!
//! # Usage
//!
//! ```sh
//! ch10time summary  <file.ch10>           # Time span, channels, sources
//! ch10time channels <file.ch10>           # Per-channel time source inventory
//! ch10time jumps    <file.ch10> [--threshold-ms 1000]
//! ch10time timeline <file.ch10> [--limit 50]
//! ch10time csv      <file.ch10> [--output times.csv]
//! ch10time correlate <file.ch10> <rtc_hex>  # Resolve a single RTC value
//! ```

use memmap2::Mmap;
use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::process;

use irig106_time::*;
use irig106_time::bcd::{DayFormatTime, DmyFormatTime};
use irig106_time::csdw::{DateFormat, TimeF1Csdw};
use irig106_time::network_time::{
    parse_time_f2_payload, NetworkTimeProtocol,
    LeapSecondTable,
};

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

    fn has_secondary_header(&self) -> bool {
        (self.packet_flags & 0x04) != 0
    }

    fn data_offset(&self) -> usize {
        HEADER_SIZE + if self.has_secondary_header() { SECONDARY_HEADER_SIZE } else { 0 }
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

    fn scan(&mut self, mmap: &[u8], resolve_all: bool, limit: Option<usize>) {
        let mut offset = 0;
        let file_len = mmap.len();

        while offset + HEADER_SIZE <= file_len {
            if let Some(limit) = limit {
                if self.total_packets >= limit {
                    break;
                }
            }

            let hdr = match PktHeader::parse(&mmap[offset..]) {
                Some(h) => h,
                None => {
                    // Try to find next sync
                    if let Some(next) = find_next_sync(&mmap, offset + 1) {
                        self.errors.push(format!(
                            "Sync lost at {:#X}, recovered at {:#X} ({} bytes skipped)",
                            offset, next, next - offset
                        ));
                        offset = next;
                        continue;
                    } else {
                        break;
                    }
                }
            };

            let pkt_len = hdr.packet_length as usize;
            if pkt_len < HEADER_SIZE || offset + pkt_len > file_len {
                break;
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
                self.process_time_packet(&mmap[offset..offset + pkt_len], &hdr);
            } else if hdr.data_type == DATA_TYPE_TIME_F2 {
                self.time_packets += 1;
                self.process_time_f2_packet(&mmap[offset..offset + pkt_len], &hdr);
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
            offset += pkt_len;
        }
    }

    fn process_time_packet(&mut self, pkt_buf: &[u8], hdr: &PktHeader) {
        let data_start = hdr.data_offset();
        if data_start + 4 > pkt_buf.len() {
            self.errors.push(format!(
                "Time packet at ch={} too short for CSDW", hdr.channel_id
            ));
            return;
        }

        let csdw_bytes = &pkt_buf[data_start..data_start + 4];
        let csdw = TimeF1Csdw::from_le_bytes([
            csdw_bytes[0], csdw_bytes[1], csdw_bytes[2], csdw_bytes[3],
        ]);

        let bcd_start = data_start + 4;
        let abs_time = match csdw.date_format() {
            DateFormat::DayOfYear => {
                if bcd_start + 8 > pkt_buf.len() {
                    self.errors.push(format!(
                        "Time packet ch={}: buffer too short for DOY BCD", hdr.channel_id
                    ));
                    return;
                }
                match DayFormatTime::from_le_bytes(&pkt_buf[bcd_start..]) {
                    Ok(t) => t.to_absolute(),
                    Err(e) => {
                        self.errors.push(format!(
                            "Time packet ch={}: DOY BCD error: {}", hdr.channel_id, e
                        ));
                        return;
                    }
                }
            }
            DateFormat::DayMonthYear => {
                if bcd_start + 10 > pkt_buf.len() {
                    self.errors.push(format!(
                        "Time packet ch={}: buffer too short for DMY BCD", hdr.channel_id
                    ));
                    return;
                }
                match DmyFormatTime::from_le_bytes(&pkt_buf[bcd_start..]) {
                    Ok(t) => t.to_absolute(),
                    Err(e) => {
                        self.errors.push(format!(
                            "Time packet ch={}: DMY BCD error: {}", hdr.channel_id, e
                        ));
                        return;
                    }
                }
            }
        };

        // Add to correlator
        self.correlator.add_reference(hdr.channel_id, hdr.rtc, abs_time);

        // Update channel info
        let info = self.time_channels
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
        let data_start = hdr.data_offset();
        let payload = &pkt_buf[data_start..];

        let (csdw, network_time) = match parse_time_f2_payload(payload) {
            Ok(result) => result,
            Err(e) => {
                self.errors.push(format!(
                    "Time F2 packet ch={}: parse error: {}", hdr.channel_id, e
                ));
                return;
            }
        };

        // Use the library's add_reference_f2 which handles NTP/PTP→AbsoluteTime
        // conversion and leap-second offset application internally.
        if let Err(e) = self.correlator.add_reference_f2(
            hdr.channel_id, hdr.rtc, &network_time, &self.leap_table,
        ) {
            self.errors.push(format!(
                "Time F2 packet ch={}: correlation error: {}", hdr.channel_id, e
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
        let info = self.time_channels
            .entry(hdr.channel_id)
            .or_insert_with(|| TimeChannelInfo::new(hdr.channel_id));
        info.packet_count += 1;
        // Map network protocol to TimeSource/TimeFormat for display
        // (Gap 2: NTP/PTP identity is lost here — deferred to v0.3.0)
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
    let end = buf.len().saturating_sub(HEADER_SIZE);
    for i in start..end {
        if u16::from_le_bytes([buf[i], buf[i + 1]]) == SYNC_PATTERN {
            // Basic validation: check header checksum
            let mut sum: u32 = 0;
            for j in 0..11 {
                let word = u16::from_le_bytes([buf[i + j * 2], buf[i + j * 2 + 1]]);
                sum = sum.wrapping_add(word as u32);
            }
            let computed = (sum & 0xFFFF) as u16;
            let stored = u16::from_le_bytes([buf[i + 22], buf[i + 23]]);
            if computed == stored {
                return Some(i);
            }
        }
    }
    None
}

// ────────────────────────────────────────────────────────────────────
// Formatting helpers
// ────────────────────────────────────────────────────────────────────

fn fmt_abs_time(t: &AbsoluteTime) -> String {
    let ms = t.nanoseconds / 1_000_000;
    let us = (t.nanoseconds % 1_000_000) / 1_000;
    match (t.year, t.month, t.day_of_month) {
        (Some(y), Some(m), Some(d)) => {
            format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}.{:03}",
                y, m, d, t.hours, t.minutes, t.seconds, ms, us
            )
        }
        _ => {
            format!(
                "Day {:03} {:02}:{:02}:{:02}.{:03}.{:03}",
                t.day_of_year, t.hours, t.minutes, t.seconds, ms, us
            )
        }
    }
}

fn fmt_time_source(src: &TimeSource) -> &'static str {
    match src {
        TimeSource::Internal => "Internal",
        TimeSource::External => "External",
        TimeSource::InternalRtc => "Internal RTC",
        TimeSource::Gps => "GPS",
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

fn cmd_summary(mmap: &[u8], filename: &str) {
    let mut scanner = Ch10TimeScanner::new();
    scanner.scan(mmap, false, None);

    println!("================================================================");
    println!("ch10time — IRIG 106 Time Analysis");
    println!("================================================================");
    println!();
    println!("File               : {}", filename);
    println!("File Size          : {} bytes", fmt_comma(mmap.len()));
    println!("Total Packets      : {}", fmt_comma(scanner.total_packets));
    println!("Time Packets (0x11): {}", fmt_comma(scanner.time_packets));
    println!("Time Channels      : {}", scanner.time_channels.len());
    println!("Correlation Refs   : {}", scanner.correlator.len());
    println!();

    if let (Some(first), Some(last)) = (scanner.first_rtc, scanner.last_rtc) {
        let dur_ns = first.elapsed_nanos(last);
        let dur_s = dur_ns as f64 / 1_000_000_000.0;
        println!("RTC Range          : {:#014X} → {:#014X}", first.as_raw(), last.as_raw());
        println!("RTC Duration       : {:.3} seconds ({:.1} minutes)", dur_s, dur_s / 60.0);
    }
    println!();

    // Per-channel summary
    if !scanner.time_channels.is_empty() {
        println!("Time Channels");
        println!("─────────────");
        for (_, info) in &scanner.time_channels {
            println!(
                "  Channel {:>3}  │  Source: {:<12}  Format: {:<8}  Date: {:<4}  Leap: {}  Packets: {}",
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
            );
            if let Some(ref ft) = info.first_time {
                println!("                │  First: {}", fmt_abs_time(ft));
            }
            if let Some(ref lt) = info.last_time {
                println!("                │  Last:  {}", fmt_abs_time(lt));
            }
        }
        println!();
    }

    // Time jump detection
    let all_channels: Vec<u16> = scanner.time_channels.keys().cloned().collect();
    let mut any_jumps = false;
    for ch in &all_channels {
        let jumps = scanner.correlator.detect_time_jump(*ch, 1_000_000_000); // 1s threshold
        if !jumps.is_empty() {
            if !any_jumps {
                println!("Time Jumps Detected (threshold: 1 second)");
                println!("──────────────────────────────────────────");
                any_jumps = true;
            }
            for j in &jumps {
                let delta_ms = j.delta_nanos as f64 / 1_000_000.0;
                println!(
                    "  Channel {:>3}  │  Ref #{:<4}  Delta: {:>+10.1} ms  ({})",
                    j.channel_id,
                    j.index,
                    delta_ms,
                    if j.delta_nanos > 0 { "jump forward" } else { "jump backward" }
                );
            }
        }
    }
    if !any_jumps && !all_channels.is_empty() {
        println!("Time Jumps         : None detected (threshold: 1 second)");
    }
    println!();

    // Data type breakdown
    println!("Data Type Breakdown");
    println!("───────────────────");
    for (dt, count) in &scanner.data_type_counts {
        println!(
            "  0x{:02X} {:<14}  {:>10}",
            dt, fmt_data_type_short(*dt), fmt_comma(*count)
        );
    }
    println!();

    if !scanner.errors.is_empty() {
        println!("Errors ({})", scanner.errors.len());
        println!("──────");
        for (i, e) in scanner.errors.iter().enumerate().take(20) {
            println!("  {}. {}", i + 1, e);
        }
        if scanner.errors.len() > 20 {
            println!("  ... and {} more", scanner.errors.len() - 20);
        }
    }
}

fn cmd_channels(mmap: &[u8]) {
    let mut scanner = Ch10TimeScanner::new();
    scanner.scan(mmap, false, None);

    println!("{:<8}  {:<12}  {:<10}  {:<5}  {:<5}  {:>8}  {:>28}  {:>28}",
        "Channel", "Source", "Format", "Date", "Leap", "Packets", "First Time", "Last Time"
    );
    println!("{}", "─".repeat(120));

    for (_, info) in &scanner.time_channels {
        println!("{:>7}  {:<12}  {:<10}  {:<5}  {:<5}  {:>8}  {:>28}  {:>28}",
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
            info.first_time.as_ref().map(|t| fmt_abs_time(t)).unwrap_or_default(),
            info.last_time.as_ref().map(|t| fmt_abs_time(t)).unwrap_or_default(),
        );
    }
}

fn cmd_jumps(mmap: &[u8], threshold_ms: u64) {
    let mut scanner = Ch10TimeScanner::new();
    scanner.scan(mmap, false, None);

    let threshold_ns = threshold_ms * 1_000_000;
    let channels: Vec<u16> = scanner.time_channels.keys().cloned().collect();
    let mut total_jumps = 0;

    println!("Time Jump Detection (threshold: {} ms)", threshold_ms);
    println!("{}", "═".repeat(70));

    for ch in &channels {
        let jumps = scanner.correlator.detect_time_jump(*ch, threshold_ns);
        if !jumps.is_empty() {
            println!();
            println!("Channel {} — {} jump(s):", ch, jumps.len());
            for j in &jumps {
                let delta_ms = j.delta_nanos as f64 / 1_000_000.0;
                println!(
                    "  Ref #{:<4}  Expected: {:>15} ns  Actual: {:>15} ns  Delta: {:>+12.3} ms",
                    j.index, j.expected_nanos, j.actual_nanos, delta_ms
                );
            }
            total_jumps += jumps.len();
        }
    }

    if total_jumps == 0 {
        println!();
        println!("No time jumps detected across {} channel(s).", channels.len());
    } else {
        println!();
        println!("Total: {} jump(s) across {} channel(s).", total_jumps, channels.len());
    }
}

fn cmd_timeline(mmap: &[u8], limit: usize) {
    let mut scanner = Ch10TimeScanner::new();
    scanner.scan(mmap, true, Some(limit));

    println!("{:>8}  {:>12}  {:>6}  {:<14}  {:>14}  {:>28}",
        "Pkt#", "Offset", "Ch", "Type", "RTC", "Absolute Time"
    );
    println!("{}", "─".repeat(100));

    for r in &scanner.resolved {
        println!("{:>8}  {:>#12X}  {:>6}  {:<14}  {:>14}  {:>28}",
            r.packet_num,
            r.offset,
            r.channel_id,
            fmt_data_type_short(r.data_type),
            r.rtc.as_raw(),
            r.abs_time.as_ref().map(|t| fmt_abs_time(t)).unwrap_or_else(|| "N/A".to_string()),
        );
    }

    if scanner.total_packets > limit {
        println!("... showing first {} of {} packets", limit, scanner.total_packets);
    }
}

fn cmd_csv(mmap: &[u8], output_path: Option<&str>) {
    let mut scanner = Ch10TimeScanner::new();
    scanner.scan(mmap, true, None);

    let mut out: Box<dyn std::io::Write> = match output_path {
        Some(path) => {
            let f = std::fs::File::create(path).expect("Failed to create output file");
            Box::new(std::io::BufWriter::new(f))
        }
        None => Box::new(std::io::stdout()),
    };

    writeln!(out, "packet_num,offset_hex,channel_id,data_type_hex,data_type_name,rtc_raw,rtc_nanos,day_of_year,hours,minutes,seconds,nanoseconds,year,month,day").unwrap();

    for r in &scanner.resolved {
        match &r.abs_time {
            Some(t) => {
                writeln!(out,
                    "{},{:#X},{},0x{:02X},{},{},{},{},{},{},{},{},{},{},{}",
                    r.packet_num, r.offset, r.channel_id,
                    r.data_type, fmt_data_type_short(r.data_type),
                    r.rtc.as_raw(), r.rtc.to_nanos(),
                    t.day_of_year, t.hours, t.minutes, t.seconds, t.nanoseconds,
                    t.year.map(|y| y.to_string()).unwrap_or_default(),
                    t.month.map(|m| m.to_string()).unwrap_or_default(),
                    t.day_of_month.map(|d| d.to_string()).unwrap_or_default(),
                ).unwrap();
            }
            None => {
                writeln!(out,
                    "{},{:#X},{},0x{:02X},{},{},{},,,,,,,",
                    r.packet_num, r.offset, r.channel_id,
                    r.data_type, fmt_data_type_short(r.data_type),
                    r.rtc.as_raw(), r.rtc.to_nanos(),
                ).unwrap();
            }
        }
    }

    if let Some(path) = output_path {
        eprintln!("Wrote {} rows to {}", scanner.resolved.len(), path);
    }
}

fn cmd_correlate(mmap: &[u8], rtc_hex: &str) {
    let rtc_val = u64::from_str_radix(rtc_hex.trim_start_matches("0x").trim_start_matches("0X"), 16)
        .expect("Invalid hex RTC value");
    let target = Rtc::from_raw(rtc_val);

    let mut scanner = Ch10TimeScanner::new();
    scanner.scan(mmap, false, None);

    println!("Resolving RTC {:#014X} ({} ticks, {} ns)", target.as_raw(), target.as_raw(), target.to_nanos());
    println!();

    if scanner.correlator.is_empty() {
        eprintln!("Error: No time reference points found in file.");
        process::exit(1);
    }

    // Correlate with all channels
    match scanner.correlator.correlate(target, None) {
        Ok(t) => println!("  Any channel  → {}", fmt_abs_time(&t)),
        Err(e) => println!("  Any channel  → Error: {}", e),
    }

    // Per-channel
    for ch in scanner.time_channels.keys() {
        match scanner.correlator.correlate(target, Some(*ch)) {
            Ok(t) => println!("  Channel {:>3}  → {}", ch, fmt_abs_time(&t)),
            Err(_) => {}
        }
    }
}

// ────────────────────────────────────────────────────────────────────
// Main / CLI dispatch
// ────────────────────────────────────────────────────────────────────

fn print_usage() {
    println!("ch10time v{} — IRIG 106 Chapter 10 Time Inspector", PKG_VERSION);
    println!();
    println!("Usage:");
    println!("  ch10time summary   <file.ch10>                    File time summary");
    println!("  ch10time channels  <file.ch10>                    Time channel inventory");
    println!("  ch10time jumps     <file.ch10> [--threshold-ms N] Time jump detection (default: 1000 ms)");
    println!("  ch10time timeline  <file.ch10> [--limit N]        Per-packet timeline (default: 100)");
    println!("  ch10time csv       <file.ch10> [--output file]    Export all timestamps to CSV");
    println!("  ch10time correlate <file.ch10> <rtc_hex>          Resolve a single RTC value");
    println!();
    println!("Examples:");
    println!("  ch10time summary flight_042.ch10");
    println!("  ch10time jumps   flight_042.ch10 --threshold-ms 500");
    println!("  ch10time csv     flight_042.ch10 --output times.csv");
    println!("  ch10time correlate flight_042.ch10 0x00009896800");
}

fn open_file(path: &str) -> Mmap {
    let file = File::open(path).unwrap_or_else(|e| {
        eprintln!("Error: cannot open '{}': {}", path, e);
        process::exit(1);
    });
    unsafe { Mmap::map(&file) }.unwrap_or_else(|e| {
        eprintln!("Error: cannot mmap '{}': {}", path, e);
        process::exit(1);
    })
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(0);
    }

    let command = args[1].as_str();

    match command {
        "summary" => {
            if args.len() < 3 { eprintln!("Usage: ch10time summary <file>"); process::exit(1); }
            let mmap = open_file(&args[2]);
            cmd_summary(&mmap, &args[2]);
        }
        "channels" => {
            if args.len() < 3 { eprintln!("Usage: ch10time channels <file>"); process::exit(1); }
            let mmap = open_file(&args[2]);
            cmd_channels(&mmap);
        }
        "jumps" => {
            if args.len() < 3 { eprintln!("Usage: ch10time jumps <file> [--threshold-ms N]"); process::exit(1); }
            let mmap = open_file(&args[2]);
            let threshold = args.iter().position(|a| a == "--threshold-ms")
                .and_then(|i| args.get(i + 1))
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(1000);
            cmd_jumps(&mmap, threshold);
        }
        "timeline" => {
            if args.len() < 3 { eprintln!("Usage: ch10time timeline <file> [--limit N]"); process::exit(1); }
            let mmap = open_file(&args[2]);
            let limit = args.iter().position(|a| a == "--limit")
                .and_then(|i| args.get(i + 1))
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(100);
            cmd_timeline(&mmap, limit);
        }
        "csv" => {
            if args.len() < 3 { eprintln!("Usage: ch10time csv <file> [--output path]"); process::exit(1); }
            let mmap = open_file(&args[2]);
            let output = args.iter().position(|a| a == "--output")
                .and_then(|i| args.get(i + 1))
                .map(|s| s.as_str());
            cmd_csv(&mmap, output);
        }
        "correlate" => {
            if args.len() < 4 { eprintln!("Usage: ch10time correlate <file> <rtc_hex>"); process::exit(1); }
            let mmap = open_file(&args[2]);
            cmd_correlate(&mmap, &args[3]);
        }
        "-h" | "--help" | "help" => {
            print_usage();
        }
        "-V" | "--version" => {
            println!("ch10time {}", PKG_VERSION);
        }
        _ => {
            eprintln!("Unknown command: '{}'", command);
            eprintln!();
            print_usage();
            process::exit(1);
        }
    }
}

use std::io::Write;
