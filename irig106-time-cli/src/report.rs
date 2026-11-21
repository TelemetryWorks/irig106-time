//! Bounded immutable report projection shared by summary/channels/jumps.
//! Partial Traces: L3-CLI-106 <- L2-CLI-106 <- L1-CLI-006 (text model only).
use super::*;
use irig106_time::network_time::NetworkTimeFormat;

pub(super) struct Report<'a> {
    pub(super) total_packets: usize,
    pub(super) time_packets: usize,
    pub(super) references_seen: usize,
    pub(super) reference_count: usize,
    pub(super) references_evicted: usize,
    pub(super) reset_count: usize,
    pub(super) first_rtc: Option<Rtc>,
    pub(super) last_rtc: Option<Rtc>,
    pub(super) time_channels: &'a BTreeMap<u16, TimeChannelInfo>,
    pub(super) data_type_counts: &'a BTreeMap<u8, usize>,
    pub(super) jumps: &'a [TimeJump],
    pub(super) jumps_seen: usize,
    pub(super) errors: &'a [String],
    pub(super) finding_counts: [u64; 3],
    pub(super) highest: Option<Severity>,
}
impl<'a> From<&'a Ch10TimeScanner> for Report<'a> {
    fn from(s: &'a Ch10TimeScanner) -> Self {
        Self {
            total_packets: s.total_packets,
            time_packets: s.time_packets,
            references_seen: s.references_seen,
            reference_count: s.correlator.len(),
            references_evicted: s.correlator.total_evicted(),
            reset_count: s.reset_count,
            first_rtc: s.first_rtc,
            last_rtc: s.last_rtc,
            time_channels: &s.time_channels,
            data_type_counts: &s.data_type_counts,
            jumps: &s.jumps,
            jumps_seen: s.jumps_seen,
            errors: &s.errors,
            finding_counts: s.finding_counts,
            highest: s.highest,
        }
    }
}
impl Report<'_> {
    pub(super) fn report_jump_samples(&self, out: &mut dyn Write) -> io::Result<()> {
        for j in self.jumps {
            writeln!(
                out,
                "  Channel {:>3} Ref #{:<4} Expected: {} Actual: {} Delta: {:+.3} ms",
                j.channel_id,
                j.index,
                clock_count(j.expected_nanos),
                clock_count(j.actual_nanos),
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

    pub(super) fn finding_total(&self) -> u64 {
        self.finding_counts.iter().sum()
    }
    pub(super) fn omitted_findings(&self) -> u64 {
        self.finding_total() - self.errors.len() as u64
    }
    pub(super) fn report_finding_counts(&self, out: &mut dyn Write) -> io::Result<()> {
        writeln!(
            out,
            "Findings: error={}, warning={}, info={} (total={})",
            self.finding_counts[2],
            self.finding_counts[1],
            self.finding_counts[0],
            self.finding_total()
        )
    }
    pub(super) fn report_findings(&self, stderr: &mut dyn Write) -> Result<(), Failure> {
        if self.finding_total() != 0 {
            self.report_finding_counts(stderr)?;
        }
        for error in self.errors {
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
}

// TimeJump stores a day-one-relative native count, not a Unix/UTC timestamp.
// Present the clock components without inventing a year or scale conversion.
fn clock_count(ns: u64) -> String {
    let seconds = ns / 1_000_000_000;
    format!(
        "{:03}:{:02}:{:02}:{:02}.{:09}",
        seconds / 86400 + 1,
        seconds / 3600 % 24,
        seconds / 60 % 60,
        seconds % 60,
        ns % 1_000_000_000
    )
}

pub(super) fn format_label(info: &TimeChannelInfo) -> &'static str {
    match info.network_word.map(TimeF2Csdw::time_format) {
        Some(NetworkTimeFormat::NtpV3) => "NTP",
        Some(NetworkTimeFormat::Ptp2002) => "IEEE 1588-2002",
        Some(NetworkTimeFormat::Ptp2008) => "IEEE 1588-2008",
        Some(NetworkTimeFormat::Reserved(_)) => "Reserved NTF",
        None => info.format.as_ref().map(fmt_time_format).unwrap_or("?"),
    }
}
pub(super) fn network_label(info: &TimeChannelInfo) -> String {
    match info.network_word {
        Some(word) => {
            let scale = match word.time_format() {
                NetworkTimeFormat::NtpV3 => "UTC (standard)",
                NetworkTimeFormat::Ptp2002 | NetworkTimeFormat::Ptp2008 => "TAI (standard)",
                NetworkTimeFormat::Reserved(_) => "unknown scale",
            };
            format!(
                "  Proto: {} Status: {:?} Native scale: {scale}",
                format_label(info),
                word.time_status()
            )
        }
        None => String::new(),
    }
}
