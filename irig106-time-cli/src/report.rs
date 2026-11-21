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
    jump_year_days: &'a [(u16, u16)],
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
            jump_year_days: &s.jump_year_days,
            jumps_seen: s.jumps_seen,
            errors: &s.errors,
            finding_counts: s.finding_counts,
            highest: s.highest,
        }
    }
}
impl Report<'_> {
    pub(super) fn report_jump_samples(&self, out: &mut dyn Write) -> io::Result<()> {
        for (index, j) in self.jumps.iter().enumerate() {
            writeln!(
                out,
                "  Channel {:>3} Ref #{:<4} Expected: {} Actual: {} Delta: {:+.3} ms",
                j.channel_id,
                j.index,
                clock_count(
                    j.expected_nanos,
                    self.jump_year_days.get(index).copied().unwrap_or((0, 0)).0
                ),
                clock_count(
                    j.actual_nanos,
                    self.jump_year_days.get(index).copied().unwrap_or((0, 0)).1
                ),
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
fn clock_count(ns: u64, days: u16) -> String {
    let span = u64::from(days) * 86_400_000_000_000;
    if (span != 0 && ns >= 2 * span) || (span == 0 && ns / 86_400_000_000_000 >= 366) {
        return "calendar context required".into();
    }
    let rolled = span != 0 && ns >= span;
    let ns = if rolled { ns - span } else { ns };
    let seconds = ns / 1_000_000_000;
    format!(
        "{:03}:{:02}:{:02}:{:02}.{:09}{}",
        seconds / 86400 + 1,
        seconds / 3600 % 24,
        seconds / 60 % 60,
        seconds % 60,
        ns % 1_000_000_000,
        if rolled { " (+1 year)" } else { "" }
    )
}

// Coordinator's report policy: unwrap an end-year -> day1 pair only when the
// cyclic explanation is closer to forward RTC than the linear clock jump.
// Known year/LY constrains the span; unknown context considers both lengths.
pub(super) fn year_boundary(
    previous: AbsoluteTime,
    time: AbsoluteTime,
    elapsed: u64,
    leap: Option<bool>,
) -> Option<u16> {
    if elapsed == 0 || previous.day_of_year() < 365 || time.day_of_year() != 1 {
        return None;
    }
    let known_days = known_year_days(previous, leap);
    let linear =
        (i128::from(time.as_total_ns()) - i128::from(previous.as_total_ns()) - i128::from(elapsed))
            .unsigned_abs();
    [365u16, 366]
        .into_iter()
        .filter(|&days| days >= previous.day_of_year() && known_days.is_none_or(|d| days == d))
        .filter_map(|days| {
            let span =
                time.as_total_ns() + u64::from(days) * 86_400_000_000_000 - previous.as_total_ns();
            let residual = span.abs_diff(elapsed);
            (u128::from(residual) < linear && residual <= 15_768_000_000_000_000)
                .then_some((residual, days))
        })
        .min()
        .map(|(_, days)| days)
}
pub(super) fn known_year_days(previous: AbsoluteTime, leap: Option<bool>) -> Option<u16> {
    previous
        .year()
        .map(|y| y % 4 == 0 && (y % 100 != 0 || y % 400 == 0))
        .or(leap)
        .map(|l| if l { 366 } else { 365 })
}
pub(super) fn f1_basis(format: TimeFormat) -> &'static str {
    match format {
        TimeFormat::Gps => "GPS (stated FMT)",
        TimeFormat::Utc => "UTC (stated FMT)",
        _ => "scale unknown",
    }
}
fn display_clock(time: AbsoluteTime, basis: Option<&str>, years: usize) -> String {
    let year = time.year().map_or(String::new(), |y| format!("{y:04} "));
    let offset = if years == 0 {
        String::new()
    } else {
        format!(" (+{years} year{})", if years == 1 { "" } else { "s" })
    };
    format!(
        "{year}{}{offset} {}",
        clock_count(time.as_total_ns(), 0),
        basis.unwrap_or("scale unknown")
    )
}
pub(super) fn first_clock(info: &TimeChannelInfo) -> String {
    info.first_time
        .map_or(String::new(), |t| display_clock(t, info.first_basis, 0))
}
pub(super) fn last_clock(info: &TimeChannelInfo) -> String {
    info.last_time.map_or(String::new(), |t| {
        display_clock(t, info.last_basis, info.year_rollovers)
    })
}
pub(super) fn columns(info: &TimeChannelInfo) -> (String, String, &'static str) {
    use irig106_time::network_time::TimeStatus;
    match info.network_word {
        Some(word) => {
            let (proto, scale) = match word.time_format() {
                NetworkTimeFormat::NtpV3 => ("NTP".into(), "UTC"),
                NetworkTimeFormat::Ptp2002 => ("IEEE1588-2002".into(), "TAI"),
                NetworkTimeFormat::Ptp2008 => ("IEEE1588-2008".into(), "TAI"),
                NetworkTimeFormat::Reserved(n) => (format!("RSVD-{n}"), "unknown"),
            };
            let status = match word.time_status() {
                TimeStatus::Valid => "Valid".into(),
                TimeStatus::NotValid => "NotValid".into(),
                TimeStatus::Reserved(n) => format!("RSVD-{n}"),
            };
            (proto, status, scale)
        }
        None => (
            "—".into(),
            "—".into(),
            match info.format {
                Some(TimeFormat::Gps) => "GPS",
                Some(TimeFormat::Utc) => "UTC",
                _ => "unknown",
            },
        ),
    }
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
