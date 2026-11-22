//! Partial CLI-104/106, COR-110/113: private raw-wire spool, native year policy.
//! Sources: 106-24R1 Figures 11-13/11-14; contract §5.3 / D9 / RDG-010. No TMATS inference.
use super::{
    csv_spool::CsvSpool,
    native_quality::{self, Quality},
};
use irig106_time::{
    timeline::*,
    years::{SessionYearCursor, SessionYearResolver},
};
use serde_json::Value;
use std::{
    cell::RefCell,
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
};
const SIZE: u64 = 62;

#[derive(Default)]
pub(super) struct Notes {
    pub counts: [u64; 3],
    pub samples: Vec<TimelineFinding>,
}
impl Notes {
    fn push(&mut self, f: TimelineFinding) {
        self.counts[usize::from(super::rank(f.severity))] += 1;
        if self.samples.len() < super::FINDING_SAMPLE_LIMIT {
            self.samples.push(f);
        }
    }
    pub fn merge(&mut self, other: Self) {
        for (a, b) in self.counts.iter_mut().zip(other.counts) {
            *a += b;
        }
        self.samples.extend(
            other
                .samples
                .into_iter()
                .take(super::FINDING_SAMPLE_LIMIT - self.samples.len()),
        );
    }
}
// Fixed private record, NOT a public snapshot. Raw prefix excludes Data Length filler.
struct Record {
    rejected: bool,
    kind: u8,
    version: u8,
    len: u8,
    channel: u16,
    key: PacketKey,
    high: i128,
    body: [u8; 16],
}
impl Record {
    fn read(file: &mut File, position: u64, session: usize) -> io::Result<Self> {
        file.seek(SeekFrom::Start(position))?;
        let mut b = [0u8; SIZE as usize];
        file.read_exact(&mut b)?;
        if b[0] > 1 || b[3] > 16 {
            return Err(io::Error::other("invalid private year record"));
        }
        let mut packet = [0; 8];
        packet.copy_from_slice(&b[6..14]);
        let mut ticks = [0; 16];
        ticks.copy_from_slice(&b[14..30]);
        let mut high = [0; 16];
        high.copy_from_slice(&b[30..46]);
        let mut body = [0; 16];
        body.copy_from_slice(&b[46..62]);
        Ok(Self {
            rejected: b[0] != 0,
            kind: b[1],
            version: b[2],
            len: b[3],
            channel: u16::from_le_bytes([b[4], b[5]]),
            key: PacketKey {
                session,
                packet: u64::from_le_bytes(packet),
                ticks: i128::from_le_bytes(ticks),
            },
            high: i128::from_le_bytes(high),
            body,
        })
    }
    fn reference(&self, era: Option<i64>) -> irig106_time::Result<TimelineReference> {
        let mut r = native_quality::reference(
            self.key,
            self.channel,
            self.kind,
            self.version,
            &self.body[..usize::from(self.len)],
            era,
        )?;
        if self.rejected {
            r.input.value = ReferenceValue::Malformed;
            r.admitted = false;
            if !r.reasons.contains(&Rejection::Malformed) {
                r.reasons.push(Rejection::Malformed);
            }
        }
        Ok(r)
    }
}
pub(super) struct Replay {
    raw: RefCell<CsvSpool>,
    session: usize,
    era: Option<i64>,
    high: i128,
}
impl Replay {
    pub fn new(era: Option<i64>) -> io::Result<Self> {
        Ok(Self {
            raw: RefCell::new(CsvSpool::new(None)?),
            session: 0,
            era,
            high: 0,
        })
    }
    pub fn advance(&mut self, high: i128) {
        self.high = high;
    }
    pub fn observe(
        &mut self,
        key: PacketKey,
        channel: u16,
        kind: u8,
        version: u8,
        payload: &[u8],
    ) -> io::Result<()> {
        if self.session != 0 && self.session != key.session {
            return Err(io::Error::other("year spool session mismatch"));
        }
        self.session = key.session;
        let len = payload.len().min(16);
        let mut b = [0u8; SIZE as usize];
        b[1] = kind;
        b[2] = version;
        b[3] = len as u8;
        b[4..6].copy_from_slice(&channel.to_le_bytes());
        b[6..14].copy_from_slice(&key.packet.to_le_bytes());
        b[14..30].copy_from_slice(&key.ticks.to_le_bytes());
        b[30..46].copy_from_slice(&self.high.to_le_bytes());
        b[46..46 + len].copy_from_slice(&payload[..len]);
        let mut raw = self.raw.borrow_mut();
        let file = raw.file()?;
        file.seek(SeekFrom::End(0))?;
        file.write_all(&b)
    }
    pub fn finish(&self) -> io::Result<(Value, Notes)> {
        let mut raw = self.raw.borrow_mut();
        let file = raw.file()?;
        let end = file.seek(SeekFrom::End(0))?;
        if end % SIZE != 0 {
            return Err(io::Error::other("partial private year record"));
        }
        loop {
            let mut quality = Quality::new(self.era);
            let mut notes = Notes::default();
            let mut changed = false;
            if end != 0 {
                let mut cursor = SessionYearCursor::new(self.session, OfflinePolicy::default())
                    .map_err(io::Error::other)?;
                let mut start = 0;
                for pos in (0..end).step_by(SIZE as usize) {
                    let rec = Record::read(file, pos, self.session)?;
                    if let Ok(r) = rec.reference(self.era) {
                        let (_, closed) = cursor.observe(r).map_err(io::Error::other)?;
                        if let Some(resolver) = closed {
                            changed |= self.segment(
                                file,
                                start,
                                pos,
                                Some(&resolver),
                                &mut quality,
                                &mut notes,
                            )?;
                            start = pos;
                        }
                    }
                }
                changed |= self.segment(
                    file,
                    start,
                    end,
                    cursor.freeze().as_ref(),
                    &mut quality,
                    &mut notes,
                )?;
            }
            // Flags are monotone 0->1: at most one new rejection per wire record.
            // Discard derived statistics/findings until the complete session stabilizes.
            if !changed {
                let mut value = quality.value();
                value["year_inference"] = "session-dmy".into();
                return Ok((value, notes));
            }
        }
    }
    fn segment(
        &self,
        file: &mut File,
        start: u64,
        end: u64,
        resolver: Option<&SessionYearResolver>,
        quality: &mut Quality,
        notes: &mut Notes,
    ) -> io::Result<bool> {
        let mut cursor = resolver.map(SessionYearResolver::replay);
        let mut changed = false;
        for pos in (start..end).step_by(SIZE as usize) {
            let rec = Record::read(file, pos, self.session)?;
            quality.advance(rec.high, OfflinePolicy::default().late_ns);
            let original = rec.reference(self.era);
            let result = match (original, cursor.as_mut(), resolver) {
                (Ok(r), Some(cursor), Some(resolver)) => {
                    let previous = r.input.year_basis.clone();
                    let (stamp, closed) = cursor.observe(r).map_err(io::Error::other)?;
                    if closed.is_some() {
                        return Err(io::Error::other("year replay segment changed within pass"));
                    }
                    let (mut resolved, findings, bad) = match resolver.resolve(stamp) {
                        Ok(result) => result,
                        Err(error @ irig106_time::TimeError::NeedsContext { .. }) => {
                            use irig106_time::findings::{
                                FindingCode, FindingEvidence, FindingLocation, ValueValidity,
                            };
                            let code = FindingCode::from_number(20).map_err(io::Error::other)?;
                            notes.push(TimelineFinding {
                                code: 20,
                                severity: code.resolve_level(None).map_err(io::Error::other)?,
                                session: rec.key.session,
                                packet: rec.key.packet,
                                channel: Some(rec.channel),
                                related: None,
                                raw_rtc: Some(irig106_time::Rtc::from_raw(rec.key.ticks as u64)),
                                delta_ns: None,
                                evidence: FindingEvidence::new(
                                    FindingLocation {
                                        session: Some(rec.key.session),
                                        packet: Some(rec.key.packet),
                                        channel: Some(rec.channel),
                                        byte_offset: None,
                                    },
                                    ValueValidity::Unusable,
                                )
                                .with_raw_rtc(irig106_time::Rtc::from_raw(rec.key.ticks as u64)),
                            });
                            quality.prepared(rec.key, rec.channel, Err(error));
                            continue;
                        }
                        Err(error) => return Err(io::Error::other(error)),
                    };
                    if bad && !rec.rejected {
                        file.seek(SeekFrom::Start(pos))?;
                        file.write_all(&[1])?;
                        changed = true;
                    }
                    // Baseline packet decode findings remain the scanner's responsibility.
                    // Emit only newly derived year facts; no duplicate jump/rate evaluation.
                    for f in findings {
                        if matches!(f.code, 4 | 7 | 32) && resolved.input.year_basis != previous
                            || f.code == 20 && resolved.input.year_basis.gap_ticks().is_some()
                        {
                            notes.push(f);
                        }
                    }
                    if resolved.input.network.is_none() {
                        if let Ok(input) = resolved
                            .input
                            .clone()
                            .normalize_to_tai(irig106_time::LeapSecondTable::builtin())
                        {
                            resolved.input = input;
                        }
                    }
                    Ok(resolved)
                }
                (result, _, _) => result,
            };
            quality.prepared(rec.key, rec.channel, result);
        }
        Ok(changed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejected_label_flags_persist_and_private_record_errors_are_fallible() -> io::Result<()> {
        let mut replay = Replay::new(None)?;
        let bodies: [&[u8]; 4] = [
            &[0x41, 1, 0, 0, 0, 0x59, 0x59, 0x23, 0x65, 3],
            &[0x41, 1, 0, 0, 0, 0x60, 0x59, 0x23, 0x65, 3],
            &[0x41, 1, 0, 0, 0, 0, 0, 0, 0x66, 3],
            &[0x41, 3, 0, 0, 0, 1, 0, 0, 0x31, 0x12, 0x16, 0x20],
        ];
        for (packet, body) in bodies.iter().enumerate() {
            let ticks = packet as i128 * 10_000_000;
            replay.advance(ticks);
            replay.observe(
                PacketKey {
                    session: 1,
                    packet: packet as u64,
                    ticks,
                },
                7,
                0x11,
                6,
                body,
            )?;
        }
        let first = replay.finish()?;
        let second = replay.finish()?;
        assert_eq!(first.0, second.0);
        assert_eq!(first.1.counts, second.1.counts);
        let mut spool = replay.raw.borrow_mut();
        let file = spool.file()?;
        let rejected = Record::read(file, SIZE, 1)?;
        assert!(rejected.rejected);
        assert!(matches!(
            rejected
                .reference(None)
                .map_err(io::Error::other)?
                .input
                .value,
            ReferenceValue::Malformed
        ));
        // A corrupted rejection flag is distinct from an oversized payload.
        file.seek(SeekFrom::Start(SIZE))?;
        file.write_all(&[2])?;
        assert!(Record::read(file, SIZE, 1).is_err());
        file.seek(SeekFrom::Start(SIZE))?;
        file.write_all(&[1])?;
        file.seek(SeekFrom::Start(SIZE + 3))?;
        file.write_all(&[17])?;
        assert!(Record::read(file, SIZE, 1).is_err());
        file.set_len(SIZE + 1)?;
        drop(spool);
        assert!(replay.finish().is_err());
        Ok(())
    }
}

#[cfg(test)]
mod note_tests {
    use super::*;
    #[test]
    fn one_pass_caps_samples_before_any_session_merge() {
        let mut notes = Notes::default();
        for packet in 0..30 {
            use irig106_time::findings::{
                FindingEvidence, FindingLocation, Severity, ValueValidity,
            };
            notes.push(TimelineFinding {
                code: 32,
                severity: Severity::Warning,
                session: 1,
                packet,
                channel: Some(7),
                related: None,
                raw_rtc: None,
                delta_ns: None,
                evidence: FindingEvidence::new(
                    FindingLocation {
                        session: Some(1),
                        packet: Some(packet),
                        channel: Some(7),
                        byte_offset: None,
                    },
                    ValueValidity::Kept,
                ),
            });
        }
        assert_eq!(notes.counts, [0, 30, 0]);
        assert_eq!(notes.samples.len(), super::super::FINDING_SAMPLE_LIMIT);
    }
}
