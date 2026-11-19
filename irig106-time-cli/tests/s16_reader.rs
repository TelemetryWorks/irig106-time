//! Partial CLI-104 transport clauses only; joining/report coverage remains S16b.
use irig106_time_cli::input::{Event, PacketReader, ReaderLimits, TailKind};
use std::io::{self, Read};

// 106-24R1 §11.2.1.1, 24-byte primary header, little-endian
// word checksum over bytes 0..22. Bit 7, not bit 2, selects the secondary.
fn packet(flags: u8, data: &[u8]) -> Vec<u8> {
    let start = 24 + if flags & 0x80 != 0 { 12 } else { 0 };
    let length = (start + data.len()).next_multiple_of(4);
    let mut bytes = vec![0xAA; length];
    bytes[..24].fill(0);
    bytes[..2].copy_from_slice(&[0x25, 0xEB]);
    bytes[4..8].copy_from_slice(&(length as u32).to_le_bytes());
    bytes[8..12].copy_from_slice(&(data.len() as u32).to_le_bytes());
    bytes[14] = flags;
    bytes[15] = 9;
    let sum = bytes[..22].chunks_exact(2).fold(0u16, |s, w| {
        s.wrapping_add(u16::from_le_bytes([w[0], w[1]]))
    });
    bytes[22..24].copy_from_slice(&sum.to_le_bytes());
    bytes[start..start + data.len()].copy_from_slice(data);
    bytes
}
fn reader(bytes: &[u8]) -> PacketReader<&[u8]> {
    PacketReader::new(
        bytes,
        ReaderLimits {
            buffer_bytes: 32,
            capture_bytes: 16,
        },
    )
    .unwrap()
}
#[test]
fn bounded_reader_crosses_small_buffers_without_exposing_filler() {
    struct Chunked {
        bytes: std::fs::File,
        largest: usize,
    }
    impl Read for Chunked {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            self.largest = self.largest.max(out.len());
            let n = out.len().min(3);
            self.bytes.read(&mut out[..n])
        }
    }
    let data = packet(0, &[1, 2, 3, 4, 5]).repeat(100);
    let path = std::env::temp_dir().join(format!("irigtime-small-buffer-{}", std::process::id()));
    std::fs::write(&path, data).unwrap();
    let input = Chunked {
        bytes: std::fs::File::open(&path).unwrap(),
        largest: 0,
    };
    let mut r = PacketReader::new(
        input,
        ReaderLimits {
            buffer_bytes: 32,
            capture_bytes: 16,
        },
    )
    .unwrap();
    for i in 0..100 {
        let Some(Event::Packet(p)) = r.next_packet(|_| 16).unwrap() else {
            panic!("packet")
        };
        assert_eq!(p.offset, i * 32);
        assert_eq!(p.body_prefix, [1, 2, 3, 4, 5]);
        assert_eq!(p.header.data_length, 5);
        assert!(p.secondary.is_none());
    }
    assert!(r.next_packet(|_| 0).unwrap().is_none());
    assert_eq!(r.position(), 3200);
    assert!(r.into_inner().largest <= 32);
    std::fs::remove_file(path).unwrap();
}
#[test]
fn secondary_bit_and_capture_cap_are_explicit() {
    for flags in [4, 0x80] {
        let bytes = packet(flags, &[1, 2, 3]);
        let mut r = reader(&bytes);
        let Some(Event::Packet(p)) = r.next_packet(|_| 2).unwrap() else {
            panic!("packet")
        };
        assert_eq!(p.secondary, (flags == 0x80).then_some([0xAA; 12]));
        assert_eq!(p.body_prefix, [1, 2]);
    }
    let bytes = packet(0, &[0; 20]);
    assert_eq!(
        reader(&bytes).next_packet(|_| 17).unwrap_err().kind(),
        io::ErrorKind::InvalidInput
    );
    assert!(
        PacketReader::new(
            &[][..],
            ReaderLimits {
                buffer_bytes: 0,
                capture_bytes: 0
            }
        )
        .is_err()
    );
}
#[test]
fn framing_recovery_and_final_tails_remain_distinct() {
    let good = packet(0, &[1, 2, 3, 4]);
    let mut bytes = vec![0xFF; 29];
    bytes.extend_from_slice(&good);
    bytes.extend_from_slice(&[0; 512]);
    let mut r = reader(&bytes);
    let Some(Event::Packet(p)) = r.next_packet(|_| 0).unwrap() else {
        panic!("packet")
    };
    assert_eq!((p.offset, p.recovered_bytes), (29, 29));
    assert!(matches!(
        r.next_packet(|_| 0).unwrap(),
        Some(Event::Tail {
            kind: TailKind::Unsynchronized,
            bytes: 512,
            offset: 57
        })
    ));
    assert!(r.next_packet(|_| 0).unwrap().is_none());
    assert_eq!(
        reader(&[0; 512]).next_packet(|_| 0).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    for (cut, kind) in [
        (13, TailKind::PartialHeader),
        (26, TailKind::TruncatedPacket),
    ] {
        assert!(
            matches!(reader(&good[..cut]).next_packet(|_| 4).unwrap(), Some(Event::Tail { kind: k, bytes: n, .. }) if k == kind && n == cut as u64)
        );
    }
    for (flags, length) in [(0, 0u32), (0, 25), (0x80, 24)] {
        let mut bad = good.clone();
        bad[4..8].copy_from_slice(&length.to_le_bytes());
        bad[14] = flags;
        let sum = bad[..22].chunks_exact(2).fold(0u16, |s, w| {
            s.wrapping_add(u16::from_le_bytes([w[0], w[1]]))
        });
        bad[22..24].copy_from_slice(&sum.to_le_bytes());
        let mut r = reader(&bad);
        assert_eq!(
            r.next_packet(|_| 0).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        assert!(r.next_packet(|_| 0).unwrap().is_none());
    }
    // A valid primary checksum cannot excuse impossible Data Length.
    let mut bad = good.clone();
    bad[8] = 5;
    let sum = 0xEB25u16 + 28 + 5 + 0x0900;
    bad[22..24].copy_from_slice(&sum.to_le_bytes());
    assert_eq!(
        reader(&bad).next_packet(|_| 0).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
}
#[test]
#[ignore = "CI-only generated 384 MiB input; invoked by check-reader-rss.py"]
fn rss_reader_worker() {
    let file = std::fs::File::open(std::env::var_os("IRIGTIME_RSS_INPUT").unwrap()).unwrap();
    let length = file.metadata().unwrap().len();
    let mut r = PacketReader::new(file, ReaderLimits::default()).unwrap();
    let mut count = 0;
    while let Some(event) = r.next_packet(|_| 0).unwrap() {
        assert!(matches!(event, Event::Packet(_)));
        count += 1;
    }
    assert_eq!(r.position(), length);
    assert_eq!(count, length / 16384);
}
