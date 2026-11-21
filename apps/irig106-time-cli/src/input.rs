//! Bounded Chapter 10 transport (partial CLI-104). No time interpretation or I/O
//! in the time library. 106-24R1 §11.2.1.1: lengths, bit 7,
//! primary word checksum.
//! Traces: L3-CLI-104 <- L2-CLI-104 <- L1-CLI-004 (transport clause only). Joining and report-state bounds are separate work.
use std::io::{self, BufRead, BufReader, Read};

/// Caller-owned transport bounds; no allocation follows Packet Length.
#[derive(Debug, Clone, Copy)]
pub struct ReaderLimits {
    pub buffer_bytes: usize,
    pub capture_bytes: usize,
}
impl Default for ReaderLimits {
    fn default() -> Self {
        Self {
            buffer_bytes: 8192,
            capture_bytes: 65536,
        }
    }
}
/// Raw primary header fields, before edition/format qualification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub channel: u16,
    pub packet_length: u32,
    pub data_length: u32,
    pub data_type_version: u8,
    pub sequence: u8,
    pub flags: u8,
    pub data_type: u8,
    pub rtc: [u8; 6],
}
/// Captured prefix of DATA ONLY. Compare its length to `header.data_length`
/// before passing it to any decoder requiring a complete body. Filler/checksum
/// bytes are drained, never exposed as data. Secondary bytes remain raw.
#[derive(Debug, PartialEq, Eq)]
pub struct Packet {
    pub offset: u64,
    pub header: Header,
    pub secondary: Option<[u8; 12]>,
    pub body_prefix: Vec<u8>,
    pub recovered_bytes: u64,
}
/// Coordinator EOF policy: cut final packets/headers are warnings; unsynced
/// padding after an established stream is information. The caller grades them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TailKind {
    PartialHeader,
    TruncatedPacket,
    Unsynchronized,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    Packet(Packet),
    Tail {
        offset: u64,
        bytes: u64,
        kind: TailKind,
    },
}
/// Constant transport space: buffer + 24-byte window + caller-bounded capture.
/// Returned packets belong to the caller; retaining them is its responsibility.
/// Errors are terminal. A truncated final packet is not rescanned: its consumed
/// bytes (possibly including later syncs) are reported in the tail count.
#[derive(Debug)]
pub struct PacketReader<R> {
    input: BufReader<R>,
    limits: ReaderLimits,
    position: u64,
    established: bool,
    done: bool,
}
fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn valid_header(b: &[u8; 24]) -> bool {
    b[..2] == [0x25, 0xEB]
        && b[..22].chunks_exact(2).fold(0u16, |s, w| {
            s.wrapping_add(u16::from_le_bytes([w[0], w[1]]))
        }) == u16::from_le_bytes([b[22], b[23]])
}
impl<R: Read> PacketReader<R> {
    pub fn new(input: R, limits: ReaderLimits) -> io::Result<Self> {
        if limits.buffer_bytes == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "zero reader buffer",
            ));
        }
        Ok(Self {
            input: BufReader::with_capacity(limits.buffer_bytes, input),
            limits,
            position: 0,
            established: false,
            done: false,
        })
    }
    pub fn position(&self) -> u64 {
        self.position
    }
    /// Returns the underlying reader, discarding any buffered read-ahead.
    pub fn into_inner(self) -> R {
        self.input.into_inner()
    }
    fn advance(&mut self, n: usize) -> io::Result<()> {
        self.position = self
            .position
            .checked_add(n as u64)
            .ok_or_else(|| invalid("file offset overflow"))?;
        Ok(())
    }
    fn read_counted(&mut self, out: &mut [u8]) -> io::Result<usize> {
        let mut total = 0;
        while total < out.len() {
            match self.input.read(&mut out[total..]) {
                Ok(0) => break,
                Ok(n) => {
                    self.advance(n)?;
                    total += n;
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            }
        }
        Ok(total)
    }
    fn drain(&mut self, mut n: u64) -> io::Result<bool> {
        while n > 0 {
            let available = match self.input.fill_buf() {
                Ok(b) => b.len(),
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            };
            if available == 0 {
                return Ok(false);
            }
            let take = available.min(n.try_into().unwrap_or(usize::MAX));
            self.input.consume(take);
            self.advance(take)?;
            n -= take as u64;
        }
        Ok(true)
    }
    fn tail(&mut self, offset: u64, kind: TailKind) -> Event {
        self.done = true;
        Event::Tail {
            offset,
            bytes: self.position - offset,
            kind,
        }
    }
    /// Capture at most this many DATA bytes, selected after header validation.
    /// Requests are clamped to Data Length, then checked against `capture_bytes`.
    /// Invalid framing/no established sync is InvalidData; exceeding the caller's
    /// capture cap is InvalidInput. Read errors retain their original kind.
    pub fn next_packet(
        &mut self,
        capture: impl FnOnce(&Header) -> usize,
    ) -> io::Result<Option<Event>> {
        if self.done {
            return Ok(None);
        }
        let result = self.next_inner(capture);
        if result.is_err() {
            self.done = true;
        }
        result
    }
    fn next_inner(&mut self, capture: impl FnOnce(&Header) -> usize) -> io::Result<Option<Event>> {
        let start = self.position;
        let mut b = [0; 24];
        match self.read_counted(&mut b)? {
            0 => {
                self.done = true;
                return Ok(None);
            }
            24 => (),
            _ => return Ok(Some(self.tail(start, TailKind::PartialHeader))),
        }
        // Sliding fixed-size checksum-qualified sync window, including short reads.
        while !valid_header(&b) {
            b.copy_within(1.., 0);
            if self.read_counted(&mut b[23..])? == 0 {
                return if self.established {
                    Ok(Some(self.tail(start, TailKind::Unsynchronized)))
                } else {
                    Err(invalid("no valid Chapter 10 sync/header"))
                };
            }
        }
        let offset = self.position - 24;
        let header = Header {
            channel: u16::from_le_bytes([b[2], b[3]]),
            packet_length: u32::from_le_bytes([b[4], b[5], b[6], b[7]]),
            data_length: u32::from_le_bytes([b[8], b[9], b[10], b[11]]),
            data_type_version: b[12],
            sequence: b[13],
            flags: b[14],
            data_type: b[15],
            rtc: [b[16], b[17], b[18], b[19], b[20], b[21]],
        };
        let with_secondary = header.flags & 0x80 != 0;
        let header_size = if with_secondary { 36 } else { 24 };
        if header.packet_length < header_size
            || header.packet_length % 4 != 0
            || header.data_length > header.packet_length - header_size
        {
            return Err(invalid("invalid packet length or Data Length"));
        }
        let wanted = capture(&header).min(header.data_length as usize);
        if wanted > self.limits.capture_bytes {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "data capture exceeds caller cap",
            ));
        }
        let secondary = if with_secondary {
            let mut raw = [0; 12];
            if self.read_counted(&mut raw)? != 12 {
                return Ok(Some(self.tail(offset, TailKind::TruncatedPacket)));
            }
            Some(raw)
        } else {
            None
        };
        let mut body_prefix = Vec::new();
        body_prefix
            .try_reserve_exact(wanted)
            .map_err(|_| io::Error::other("data capture allocation failed"))?;
        body_prefix.resize(wanted, 0);
        if self.read_counted(&mut body_prefix)? != wanted
            || !self.drain(u64::from(header.packet_length - header_size) - wanted as u64)?
        {
            return Ok(Some(self.tail(offset, TailKind::TruncatedPacket)));
        }
        self.established = true;
        Ok(Some(Event::Packet(Packet {
            offset,
            header,
            secondary,
            body_prefix,
            recovered_bytes: offset - start,
        })))
    }
}
