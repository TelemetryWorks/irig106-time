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

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Private report fixtures only.
mod tests {
    use super::*;
    use std::io::{Read, Seek, SeekFrom};
    #[test]
    fn segment_spool_keeps_structured_time_before_rendering() {
        let mut answers = SegmentAnswers::new().unwrap();
        let time = AbsoluteTime::new(100, 12, 0, 5, 123)
            .unwrap()
            .with_year(Some(2025))
            .unwrap();
        answers.record(7, 2, true, Ok(time)).unwrap();
        let file = answers.spool.file().unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();
        let mut length = [0; 4];
        file.read_exact(&mut length).unwrap();
        let length = u32::from_le_bytes(length) as usize;
        assert!(
            length <= 4096,
            "spool stores bounded structured records, not rendered lines"
        );
        let mut bytes = vec![0; length];
        file.read_exact(&mut bytes).unwrap();
        let row: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(row["channel"], 7);
        assert_eq!(row["segment"], "2");
        assert_eq!(row["time"], serde_json::json!([100, 12, 0, 5, 123, 2025]));
        assert!(row["issue"].is_null());
    }
}
