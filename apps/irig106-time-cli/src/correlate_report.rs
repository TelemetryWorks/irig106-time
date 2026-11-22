//! Shared native counter-query report; text/JSON CLI-106 clauses only.
use super::{decode_level, fmt_abs_time, segment_answers::SegmentAnswers};
use irig106_time::{AbsoluteTime, Rtc, TimeError};
use serde_json::{Value, json};
use std::io::{self, Write};

pub(super) struct Answer {
    pub(super) channel: u16,
    pub(super) segment: usize,
    pub(super) labelled: bool,
    pub(super) time: Option<AbsoluteTime>,
    pub(super) issue: Option<String>,
}
impl Answer {
    pub(super) fn new(
        channel: u16,
        segment: usize,
        labelled: bool,
        answer: Result<AbsoluteTime, TimeError>,
    ) -> Self {
        let (time, issue) = match answer {
            Ok(t) => (Some(t), None),
            Err(e) => (None, Some(e.to_string())),
        };
        Self {
            channel,
            segment,
            labelled,
            time,
            issue,
        }
    }
    pub(super) fn write_text(&self, out: &mut dyn Write) -> io::Result<()> {
        let value = self.time.as_ref().map(fmt_abs_time).unwrap_or_else(|| {
            format!("unresolved: {}", self.issue.as_deref().unwrap_or_default())
        });
        if self.labelled {
            writeln!(
                out,
                "  Channel {:>3} Segment {:>3}  → {}",
                self.channel, self.segment, value
            )
        } else {
            writeln!(out, "  Channel {:>3}  → {}", self.channel, value)
        }
    }
    // Internal, exclusively-created scratch records, not a published JSON format
    // or a report loader. One bounded record is decoded at a time in the CLI.
    pub(super) fn private_value(&self) -> Value {
        json!({"channel":self.channel, "segment":self.segment.to_string(), "labelled":self.labelled,
            "time":self.time.map(|t| json!([t.day_of_year(),t.hours(),t.minutes(),t.seconds(),t.nanoseconds(),t.year()])), "issue":self.issue})
    }
    pub(super) fn from_private(v: Value) -> io::Result<Self> {
        let invalid = || io::Error::other("invalid private query record");
        let time = if v["time"].is_null() {
            None
        } else {
            let t = &v["time"];
            if t.as_array().is_none_or(|a| a.len() != 6) {
                return Err(invalid());
            }
            let year = if t[5].is_null() {
                None
            } else {
                Some(small(&t[5])?)
            };
            Some(
                AbsoluteTime::new(
                    small(&t[0])?,
                    small(&t[1])?,
                    small(&t[2])?,
                    small(&t[3])?,
                    small(&t[4])?,
                )
                .and_then(|t| t.with_year(year))
                .map_err(io::Error::other)?,
            )
        };
        let issue = if v["issue"].is_null() {
            None
        } else {
            Some(v["issue"].as_str().ok_or_else(invalid)?.to_owned())
        };
        let segment = v["segment"]
            .as_str()
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|&n| n != 0)
            .ok_or_else(invalid)?;
        if time.is_some() == issue.is_some() {
            return Err(invalid());
        }
        Ok(Self {
            channel: small(&v["channel"])?,
            segment,
            labelled: v["labelled"].as_bool().ok_or_else(invalid)?,
            time,
            issue,
        })
    }
}
fn small<T: TryFrom<u64>>(v: &Value) -> io::Result<T> {
    v.as_u64()
        .and_then(|n| T::try_from(n).ok())
        .ok_or_else(|| io::Error::other("invalid private query number"))
}

pub(super) struct Query<'a> {
    pub(super) target: Rtc,
    pub(super) any: Option<&'a Result<AbsoluteTime, TimeError>>,
    pub(super) answers: &'a [Answer],
    pub(super) segments: Option<&'a SegmentAnswers>,
}
impl Query<'_> {
    pub(super) fn write_header(&self, out: &mut dyn Write) -> io::Result<()> {
        writeln!(
            out,
            "Resolving RTC {:#014X} ({} ticks, {} ns)\n",
            self.target.as_raw(),
            self.target.as_raw(),
            self.target.to_nanos()
        )
    }
    pub(super) fn write_answers(&self, out: &mut dyn Write) -> io::Result<()> {
        if let Some(answer) = self.any {
            match answer {
                Ok(t) => writeln!(out, "  Any channel  → {}", fmt_abs_time(t))?,
                Err(e) => writeln!(out, "  Any channel  → {:?}: {e}", decode_level(e))?,
            }
        }
        for row in self.answers {
            row.write_text(out)?;
        }
        if let Some(segments) = self.segments {
            segments.each(|row| row.write_text(out))?;
        }
        Ok(())
    }
}
