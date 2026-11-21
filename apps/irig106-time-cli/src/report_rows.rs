//! Shared packet observations projected by timeline and CSV.
//! CLI-105/106 partial clauses; no claim of qualified joining or JSON row coverage.
use super::{fmt_abs_time, fmt_data_type_short};
use irig106_time::{AbsoluteTime, Rtc};
use std::io::{self, Write};

pub(super) const CSV_HEADER: &str = "packet_num,offset_hex,channel_id,data_type_hex,data_type_name,rtc_raw,rtc_nanos,day_of_year,hours,minutes,seconds,nanoseconds,year,month,day";

pub(super) struct PacketRow {
    pub(super) packet_num: usize,
    pub(super) offset: u64,
    pub(super) channel_id: u16,
    pub(super) data_type: u8,
    pub(super) rtc: Rtc,
    pub(super) abs_time: Option<AbsoluteTime>,
}
impl PacketRow {
    pub(super) fn write_text(&self, out: &mut dyn Write) -> io::Result<()> {
        writeln!(
            out,
            "{:>8}  {:>#12X}  {:>6}  {:<14}  {:>14}  {:>28}",
            self.packet_num,
            self.offset,
            self.channel_id,
            fmt_data_type_short(self.data_type),
            self.rtc.as_raw(),
            self.abs_time
                .as_ref()
                .map(fmt_abs_time)
                .unwrap_or_else(|| "N/A".to_string()),
        )
    }

    pub(super) fn write_csv(&self, out: &mut dyn Write) -> io::Result<()> {
        write!(
            out,
            "{},{:#X},{},0x{:02X},",
            self.packet_num, self.offset, self.channel_id, self.data_type
        )?;
        field(out, fmt_data_type_short(self.data_type))?;
        write!(out, ",{},{},", self.rtc.as_raw(), self.rtc.to_nanos())?;
        if let Some(t) = self.abs_time {
            write!(
                out,
                "{},{},{},{},{},",
                t.day_of_year(),
                t.hours(),
                t.minutes(),
                t.seconds(),
                t.nanoseconds()
            )?;
            if let Some(year) = t.year() {
                write!(out, "{year}")?;
            }
            // Legacy columns reserved for civil month/day: not synthesized.
            writeln!(out, ",,")
        } else {
            // Eight absent calendar fields, including year/month/day.
            writeln!(out, ",,,,,,,")
        }
    }
}

// RFC 4180 §2: quote separators, CR/LF and quotes; double embedded quotes.
fn field(out: &mut dyn Write, value: &str) -> io::Result<()> {
    let quoted = value
        .bytes()
        .any(|b| matches!(b, b',' | b'"' | b'\r' | b'\n'));
    if quoted {
        out.write_all(b"\"")?;
    }
    for part in value.split_inclusive('"') {
        out.write_all(part.as_bytes())?;
        if part.ends_with('"') {
            out.write_all(b"\"")?;
        }
    }
    if quoted {
        out.write_all(b"\"")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn csv_field_escaping_is_exact() {
        for (value, expected) in [
            ("plain", "plain"),
            ("a,b", "\"a,b\""),
            ("a\"b", "\"a\"\"b\""),
            ("a\r\nb", "\"a\r\nb\""),
            ("a\rb", "\"a\rb\""),
            ("", ""),
        ] {
            let mut out = Vec::new();
            assert!(field(&mut out, value).is_ok());
            assert_eq!(out, expected.as_bytes());
        }
    }
}
