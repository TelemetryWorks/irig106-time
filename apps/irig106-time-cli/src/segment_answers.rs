//! Closed-segment report rows on disk; only per-channel counts remain in RAM.
use super::{csv_spool::CsvSpool, fmt_abs_time};
use irig106_time::{AbsoluteTime, TimeError};
use std::{
    collections::BTreeMap,
    io::{self, Write},
};

pub(super) struct SegmentAnswers {
    spool: CsvSpool,
    matches: BTreeMap<u16, usize>,
}
impl SegmentAnswers {
    pub(super) fn new() -> io::Result<Self> {
        Ok(Self {
            spool: CsvSpool::new(None)?,
            matches: BTreeMap::new(),
        })
    }
    pub(super) fn record(
        &mut self,
        ch: u16,
        segment: usize,
        labelled: bool,
        answer: Result<AbsoluteTime, TimeError>,
    ) -> io::Result<bool> {
        let count = self.matches.entry(ch).or_default();
        *count += 1;
        let value = match answer {
            Ok(time) => fmt_abs_time(&time),
            Err(e) => format!("unresolved: {e}"),
        };
        if labelled {
            writeln!(
                self.spool,
                "  Channel {:>3} Segment {:>3}  → {}",
                ch, segment, value
            )?;
        } else {
            writeln!(self.spool, "  Channel {:>3}  → {}", ch, value)?;
        }
        Ok(labelled && *count == 2)
    }
    pub(super) fn contains(&self, ch: u16) -> bool {
        self.matches.contains_key(&ch)
    }
    pub(super) fn copy_to(&mut self, out: &mut dyn Write) -> io::Result<()> {
        self.spool.copy_stdout(out)
    }
}
