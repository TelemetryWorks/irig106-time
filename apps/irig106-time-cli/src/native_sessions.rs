//! Summary-only native spans: one active session, closed rows in a private spool.
//! Partial Traces: L3-CLI-104/106; reference quality and TMATS remain separate.
use super::csv_spool::CsvSpool;
use irig106_time::{
    Rtc,
    session::{SessionClock, SessionSpan, SessionStart},
    timeline::OfflinePolicy,
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    io::{self, Read, Seek, SeekFrom, Write},
};

const TEXT_SESSION_LIMIT: usize = 20;

pub(super) struct NativeSessions {
    clock: SessionClock,
    archive: RefCell<CsvSpool>,
    unavailable: Option<String>,
}
impl NativeSessions {
    pub(super) fn new() -> io::Result<Self> {
        Ok(Self {
            clock: SessionClock::new(&OfflinePolicy::default()).map_err(io::Error::other)?,
            archive: RefCell::new(CsvSpool::new(None)?),
            unavailable: None,
        })
    }
    pub(super) fn observe(&mut self, packet: u64, rtc: Rtc) -> io::Result<Option<String>> {
        if self.unavailable.is_some() {
            return Ok(None);
        }
        let update = match self.clock.observe(packet, rtc, false) {
            Ok(update) => update,
            Err(error) => {
                let reason = error.to_string();
                self.unavailable = Some(reason.clone());
                return Ok(Some(reason));
            }
        };
        if let Some(span) = update.closed {
            let bytes = serde_json::to_vec(&value(span)?)?;
            let mut spool = self.archive.borrow_mut();
            let file = spool.file()?;
            file.seek(SeekFrom::End(0))?;
            file.write_all(&(bytes.len() as u32).to_le_bytes())?;
            file.write_all(&bytes)?;
        }
        Ok(None)
    }
    pub(super) fn metadata(&self) -> Value {
        json!({"count":self.clock.active().map_or(0,|s|s.first.session).to_string(),
            "late_bound_ns":self.clock.late_ns().to_string(),"infer_sessions":self.clock.infer_sessions(),
            "status":if self.unavailable.is_some() {"needs-context"} else {"complete"},
            "unavailable_reason":self.unavailable,"spans":[]}) // Marker: never collect rows.
    }
    pub(super) fn each(&self, mut emit: impl FnMut(&Value) -> io::Result<()>) -> io::Result<()> {
        let mut spool = self.archive.borrow_mut();
        let file = spool.file()?;
        let end = file.seek(SeekFrom::End(0))?;
        file.rewind()?;
        while file.stream_position()? < end {
            let mut len = [0; 4];
            file.read_exact(&mut len)?;
            let len = u32::from_le_bytes(len) as usize;
            if len > 2048 {
                return Err(io::Error::other("invalid private native span record"));
            }
            let mut bytes = vec![0; len];
            file.read_exact(&mut bytes)?;
            emit(&serde_json::from_slice(&bytes)?)?;
        }
        if let Some(span) = self.clock.active() {
            emit(&value(*span)?)?;
        }
        Ok(())
    }
    pub(super) fn text(&self, out: &mut dyn Write) -> io::Result<()> {
        writeln!(out, "Recording-wide native sessions")?;
        writeln!(
            out,
            "  Native RTC policy: late bound {} ns; infer sessions: {}",
            self.clock.late_ns(),
            self.clock.infer_sessions()
        )?;
        writeln!(
            out,
            "  File-order RTC range/duration above is not a native-session span."
        )?;
        if let Some(reason) = &self.unavailable {
            writeln!(
                out,
                "  Needs context; spans cover known prefix only: {reason}"
            )?;
        }
        let total = self.clock.active().map_or(0, |s| s.first.session);
        writeln!(
            out,
            "  Native sessions: {total} (showing {})",
            total.min(TEXT_SESSION_LIMIT)
        )?;
        let mut shown = 0;
        self.each(|v| {
            if shown >= TEXT_SESSION_LIMIT {
                return Ok(());
            }
            let ns: u128 = field(v, "counter_span_ns")?
                .parse()
                .map_err(io::Error::other)?;
            writeln!(
                out,
                "  Session {} [{}]: packets {}..{} ({}); span {}.{:09} s",
                field(v, "index")?,
                field(v, "start")?,
                field(v, "first_packet")?,
                field(v, "last_packet")?,
                field(v, "packets")?,
                ns / 1_000_000_000,
                ns % 1_000_000_000
            )?;
            shown += 1;
            Ok(())
        })?;
        if total > shown {
            writeln!(out, "  ... and {} more native sessions", total - shown)?;
        }
        Ok(())
    }
}
fn field<'a>(v: &'a Value, key: &str) -> io::Result<&'a str> {
    v[key]
        .as_str()
        .ok_or_else(|| io::Error::other("invalid private native span field"))
}
fn value(s: SessionSpan) -> io::Result<Value> {
    let ns = s
        .high_ticks
        .checked_sub(s.low_ticks)
        .and_then(|n| n.checked_mul(100))
        .ok_or_else(|| io::Error::other("native counter span overflow"))?;
    Ok(
        json!({"index":s.first.session.to_string(),"start":match s.start {
        SessionStart::Initial=>"initial",SessionStart::Explicit=>"explicit",SessionStart::Inferred=>"inferred"},
        "first_packet":s.first.packet.to_string(),"last_packet":s.last.packet.to_string(),
        "first_ticks":s.first.ticks.to_string(),"last_ticks":s.last.ticks.to_string(),
        "low_ticks":s.low_ticks.to_string(),"high_ticks":s.high_ticks.to_string(),
        "counter_span_ns":ns.to_string(),"packets":s.packets.to_string()}),
    )
}
