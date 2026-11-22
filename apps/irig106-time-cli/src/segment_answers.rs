//! Closed-segment native report records on disk; only per-channel counts in RAM.
use super::{correlate_report::Answer, csv_spool::CsvSpool};
use irig106_time::{AbsoluteTime, TimeError};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    io::{self, Read, Seek, SeekFrom, Write},
};

pub(super) struct SegmentAnswers {
    spool: RefCell<CsvSpool>,
    matches: BTreeMap<u16, usize>,
}
impl SegmentAnswers {
    pub(super) fn new() -> io::Result<Self> {
        Ok(Self {
            spool: RefCell::new(CsvSpool::new(None)?),
            matches: BTreeMap::new(),
        })
    }
    #[cfg(test)]
    pub(super) fn record(
        &mut self,
        ch: u16,
        segment: usize,
        labelled: bool,
        answer: Result<AbsoluteTime, TimeError>,
    ) -> io::Result<bool> {
        self.record_with_basis(ch, segment, labelled, answer, None)
    }
    pub(super) fn record_with_basis(
        &mut self,
        ch: u16,
        segment: usize,
        labelled: bool,
        answer: Result<AbsoluteTime, TimeError>,
        basis: Option<&'static str>,
    ) -> io::Result<bool> {
        let mut row = Answer::new(ch, segment, labelled, answer);
        row.basis = basis;
        let bytes = serde_json::to_vec(&row.private_value())?;
        if bytes.len() > 4096 {
            return Err(io::Error::other("private query record too large"));
        }
        let spool = self.spool.get_mut();
        spool.write_all(&(bytes.len() as u32).to_le_bytes())?;
        spool.write_all(&bytes)?;
        let count = self.matches.entry(ch).or_default();
        *count += 1;
        Ok(labelled && *count == 2)
    }
    pub(super) fn contains(&self, ch: u16) -> bool {
        self.matches.contains_key(&ch)
    }
    pub(super) fn each(&self, mut emit: impl FnMut(&Answer) -> io::Result<()>) -> io::Result<()> {
        let mut spool = self.spool.borrow_mut();
        let file = spool.file()?;
        let end = file.seek(SeekFrom::End(0))?;
        file.seek(SeekFrom::Start(0))?;
        while file.stream_position()? < end {
            let mut length = [0; 4];
            file.read_exact(&mut length)?;
            let length = u32::from_le_bytes(length) as usize;
            if length > 4096 {
                return Err(io::Error::other("invalid private query record length"));
            }
            let mut bytes = vec![0; length];
            file.read_exact(&mut bytes)?;
            emit(&Answer::from_private(serde_json::from_slice(&bytes)?)?)?;
        }
        Ok(())
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
        let file = answers.spool.get_mut().file().unwrap();
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
    #[test]
    fn structured_answers_preserve_order_year_zero_unknown_year_and_error() {
        let mut answers = SegmentAnswers::new().unwrap();
        let known = AbsoluteTime::new(1, 0, 0, 0, 999_999_999)
            .unwrap()
            .with_year(Some(0))
            .unwrap();
        let partial = AbsoluteTime::new(366, 23, 59, 59, 123).unwrap();
        answers.record(8, 1, false, Ok(known)).unwrap();
        answers.record(7, 2, true, Ok(partial)).unwrap();
        answers
            .record(
                7,
                3,
                true,
                Err(TimeError::NeedsContext {
                    context: "query fixture",
                }),
            )
            .unwrap();
        for _ in 0..2 {
            let mut rows = Vec::new();
            answers
                .each(|r| {
                    rows.push((r.channel, r.segment, r.labelled, r.time, r.issue.clone()));
                    Ok(())
                })
                .unwrap();
            assert_eq!(
                rows,
                vec![
                    (8, 1, false, Some(known), None),
                    (7, 2, true, Some(partial), None),
                    (
                        7,
                        3,
                        true,
                        None,
                        Some("time needs context: query fixture".into())
                    )
                ]
            );
        }
    }
    #[test]
    fn private_record_limits_and_truncation_fail_before_emission() {
        let row = Answer::new(7, 1, false, Ok(AbsoluteTime::new(1, 0, 0, 0, 0).unwrap()));
        let mut oversized = serde_json::to_vec(&row.private_value()).unwrap();
        oversized.resize(4097, b' '); // Valid JSON: only the length guard rejects it.
        for bytes in [
            [4097u32.to_le_bytes().as_slice(), &oversized].concat(),
            vec![1, 0, 0],
            vec![10, 0, 0, 0, b'{'],
            vec![1, 0, 0, 0, b'?'],
        ] {
            let mut answers = SegmentAnswers::new().unwrap();
            answers.spool.get_mut().write_all(&bytes).unwrap();
            let mut emitted = 0;
            assert!(
                answers
                    .each(|_| {
                        emitted += 1;
                        Ok(())
                    })
                    .is_err()
            );
            assert_eq!(emitted, 0);
        }
    }
    #[test]
    fn oversized_write_is_rejected_before_spool_or_match_state_changes() {
        // Bounded 4KiB fixture; TimeError requires a static context string.
        let context = Box::leak("x".repeat(4096).into_boxed_str());
        let mut answers = SegmentAnswers::new().unwrap();
        let error = answers
            .record(7, 1, true, Err(TimeError::NeedsContext { context }))
            .unwrap_err();
        assert_eq!(error.to_string(), "private query record too large");
        assert!(!answers.contains(7));
        assert_eq!(
            answers
                .spool
                .get_mut()
                .file()
                .unwrap()
                .metadata()
                .unwrap()
                .len(),
            0
        );
        assert!(
            !answers
                .record(7, 1, true, Ok(AbsoluteTime::new(1, 0, 0, 0, 0).unwrap()))
                .unwrap()
        );
    }
}
