//! Bounded segment inventory: closed records on disk, one active row per channel.
//! No JSON is loaded from input; the private spool contains our own projection.
use super::{ObservedReference, csv_spool::CsvSpool, report_json::clock};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    io::{self, Read, Seek, SeekFrom, Write},
};

#[derive(Debug)]
pub(super) struct Active {
    pub(super) index: usize,
    first: ObservedReference,
    last: ObservedReference,
    pub(super) years: usize,
}
impl Active {
    pub(super) fn new(index: usize, point: ObservedReference) -> Self {
        Self {
            index,
            first: point,
            last: point,
            years: 0,
        }
    }
    pub(super) fn observe(&mut self, point: ObservedReference) {
        // First/last mean file observation order, including late references.
        if point.packet < self.first.packet {
            self.first = point;
        }
        if point.packet >= self.last.packet {
            self.last = point;
        }
    }
    pub(super) fn value(&self) -> Value {
        let mut value = json!({"index": self.index.to_string(),
            "first": clock(self.first.time, Some(self.first.basis), 0),
            "last": clock(self.last.time, Some(self.last.basis), self.years),
            "first_rtc": self.first.rtc.as_raw().to_string(),
            "last_rtc": self.last.rtc.as_raw().to_string()});
        value.sort_all_objects();
        value
    }
}
const END: u64 = u64::MAX;
struct Storage {
    spool: CsvSpool,
    links: BTreeMap<u16, (u64, u64)>,
}
pub(super) struct Archive(RefCell<Storage>);
impl Archive {
    pub(super) fn new() -> io::Result<Self> {
        Ok(Self(RefCell::new(Storage {
            spool: CsvSpool::new(None)?,
            links: BTreeMap::new(),
        })))
    }
    pub(super) fn push(&self, channel: u16, row: Active) -> io::Result<()> {
        let bytes = serde_json::to_vec(&row.value())?;
        let mut s = self.0.borrow_mut();
        let file = s.spool.file()?;
        let offset = file.seek(SeekFrom::End(0))?;
        file.write_all(&END.to_le_bytes())?;
        file.write_all(&(bytes.len() as u32).to_le_bytes())?;
        file.write_all(&bytes)?;
        if let Some((_, previous)) = s.links.get(&channel).copied() {
            let file = s.spool.file()?;
            file.seek(SeekFrom::Start(previous))?;
            file.write_all(&offset.to_le_bytes())?;
        }
        s.links
            .entry(channel)
            .and_modify(|v| v.1 = offset)
            .or_insert((offset, offset));
        Ok(())
    }
    pub(super) fn each(
        &self,
        channel: u16,
        mut emit: impl FnMut(&Value) -> io::Result<()>,
    ) -> io::Result<()> {
        let mut s = self.0.borrow_mut();
        let mut offset = s.links.get(&channel).map_or(END, |v| v.0);
        let file = s.spool.file()?;
        while offset != END {
            file.seek(SeekFrom::Start(offset))?;
            let mut next = [0; 8];
            let mut len = [0; 4];
            file.read_exact(&mut next)?;
            file.read_exact(&mut len)?;
            let len = u32::from_le_bytes(len) as usize;
            if len > 4096 {
                return Err(io::Error::other("invalid private segment record"));
            }
            let mut bytes = vec![0; len];
            file.read_exact(&mut bytes)?;
            emit(&serde_json::from_slice(&bytes)?)?;
            offset = u64::from_le_bytes(next);
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Private archive fixtures only.
mod tests {
    use super::*;
    use irig106_time::{AbsoluteTime, Rtc};
    #[test]
    fn linked_records_keep_each_channels_segment_order_and_clock_basis() {
        let archive = Archive::new().unwrap();
        for n in 1..=3 {
            for channel in [7, 8] {
                let first = ObservedReference {
                    network_time: None,
                    basis: "UTC (from TAI via leap table)",
                    time: AbsoluteTime::new(100, 12, 0, n as u8, 123).unwrap(),
                    rtc: Rtc::from_raw(n as u64),
                    packet: n,
                    index: n,
                };
                let mut active = Active::new(n, first);
                let mut last = first;
                last.packet += 1;
                last.rtc = Rtc::from_raw(n as u64 + 10);
                active.observe(last);
                archive.push(channel, active).unwrap();
            }
        }
        for channel in [7, 8] {
            let mut indexes = Vec::new();
            archive
                .each(channel, |row| {
                    indexes.push(row["index"].as_str().unwrap().to_owned());
                    assert_eq!(row["first"]["scale"], "UTC");
                    assert_eq!(row["first"]["basis"], "derived, leap table");
                    assert_eq!(row["last"]["basis"], "derived, leap table");
                    assert_eq!(row["last"]["nanosecond"], 123);
                    Ok(())
                })
                .unwrap();
            assert_eq!(indexes, ["1", "2", "3"]);
        }
        let mut count = 0;
        archive
            .each(9, |_| {
                count += 1;
                Ok(())
            })
            .unwrap();
        assert_eq!(count, 0);
    }
}
