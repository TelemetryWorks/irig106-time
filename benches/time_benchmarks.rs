//! Zero-dependency benchmarks for `irig106-time` hot paths.
//!
//! Uses `std::time::Instant` — no criterion, no rayon, no transitive deps.
//!
//! # Run
//! ```sh
//! cargo bench --release
//! ```
//!
//! # Performance Targets (at 10 Gbps Ch10 throughput)
//!
//! At 10 Gbps with average 512-byte packets → ~2.4M packets/sec.
//! Each packet needs at minimum:
//!   - 1x Rtc::from_le_bytes  (~1-2 ns target)
//!   - 1x correlator lookup   (~50-100 ns target)

use std::hint::black_box;
use std::time::{Duration, Instant};

use irig106_time::bcd::{DayFormatTime, DmyFormatTime};
use irig106_time::csdw::TimeF1Csdw;
use irig106_time::intra_packet::{parse_intra_packet_time, IntraPacketTimeFormat};
use irig106_time::secondary::{
    parse_secondary_header, validate_secondary_checksum, SecHdrTimeFormat,
};
use irig106_time::*;

const WARMUP: u64 = 1_000;
const ITERS: u64 = 100_000;

struct Bench {
    name: &'static str,
    total: Duration,
    iters: u64,
}

impl Bench {
    fn ns(&self) -> f64 {
        self.total.as_nanos() as f64 / self.iters as f64
    }
}

fn bench<F: FnMut()>(name: &'static str, mut f: F) -> Bench {
    for _ in 0..WARMUP {
        f();
    }
    let start = Instant::now();
    for _ in 0..ITERS {
        f();
    }
    Bench {
        name,
        total: start.elapsed(),
        iters: ITERS,
    }
}

fn day100_bytes() -> [u8; 8] {
    let mut b = [0u8; 8];
    b[0..2].copy_from_slice(&0x2534u16.to_le_bytes());
    b[2..4].copy_from_slice(&0x1230u16.to_le_bytes());
    b[4..6].copy_from_slice(&0x0100u16.to_le_bytes());
    b
}

fn dmy_bytes() -> [u8; 10] {
    let mut b = [0u8; 10];
    b[0..2].copy_from_slice(&0x3012u16.to_le_bytes());
    b[2..4].copy_from_slice(&0x0845u16.to_le_bytes());
    b[4..6].copy_from_slice(&0x0315u16.to_le_bytes());
    b[6..8].copy_from_slice(&0x2025u16.to_le_bytes());
    b
}

fn sec_hdr_buf() -> [u8; 12] {
    let mut b = [0u8; 12];
    b[0..4].copy_from_slice(&500_000_000u32.to_le_bytes());
    b[4..8].copy_from_slice(&1000u32.to_le_bytes());
    let mut sum: u32 = 0;
    for i in 0..5 {
        sum = sum.wrapping_add(u16::from_le_bytes([b[i * 2], b[i * 2 + 1]]) as u32);
    }
    b[10..12].copy_from_slice(&((sum & 0xFFFF) as u16).to_le_bytes());
    b
}

fn build_correlator(n: usize) -> TimeCorrelator {
    let mut c = TimeCorrelator::new();
    for i in 0..n {
        c.add_reference(
            1,
            Rtc::from_raw((i as u64) * 10_000_000),
            AbsoluteTime::new(1, (i % 24) as u8, 0, 0, 0).unwrap(),
        );
    }
    c
}

fn main() {
    let mut r: Vec<Bench> = Vec::new();

    // RTC
    let bytes: [u8; 6] = [0xAB, 0xCD, 0xEF, 0x12, 0x34, 0x56];
    r.push(bench("rtc_from_le_bytes", || {
        black_box(Rtc::from_le_bytes(black_box(bytes)));
    }));
    r.push(bench("rtc_from_raw", || {
        black_box(Rtc::from_raw(black_box(0xABCDEF123456u64)));
    }));
    let rtc = Rtc::from_raw(12345678);
    r.push(bench("rtc_to_nanos", || {
        black_box(black_box(rtc).to_nanos());
    }));
    let (a, b2) = (Rtc::from_raw(1_000_000), Rtc::from_raw(2_000_000));
    r.push(bench("rtc_elapsed_ticks", || {
        black_box(black_box(a).elapsed_ticks(black_box(b2)));
    }));
    r.push(bench("rtc_elapsed_nanos", || {
        black_box(black_box(a).elapsed_nanos(black_box(b2)));
    }));

    // BCD
    let db = day100_bytes();
    r.push(bench("bcd_day_parse", || {
        let _ = black_box(DayFormatTime::from_le_bytes(black_box(&db)));
    }));
    let dmb = dmy_bytes();
    r.push(bench("bcd_dmy_parse", || {
        let _ = black_box(DmyFormatTime::from_le_bytes(black_box(&dmb)));
    }));
    let dt = DayFormatTime::from_le_bytes(&db).unwrap();
    r.push(bench("bcd_day_to_absolute", || {
        black_box(black_box(dt).to_absolute());
    }));
    r.push(bench("bcd_day_full_pipeline", || {
        let t = DayFormatTime::from_le_bytes(black_box(&db)).unwrap();
        black_box(t.to_absolute());
    }));

    // CSDW
    let cb = 0x0351u32.to_le_bytes();
    r.push(bench("csdw_from_le_bytes", || {
        black_box(TimeF1Csdw::from_le_bytes(black_box(cb)));
    }));
    let csdw = TimeF1Csdw::from_raw(0x0351);
    r.push(bench("csdw_all_fields", || {
        let c = black_box(csdw);
        black_box(c.time_source());
        black_box(c.time_format());
        black_box(c.is_leap_year());
        black_box(c.date_format());
    }));

    // Secondary header
    let sb = sec_hdr_buf();
    r.push(bench("sec_checksum_validate", || {
        let _ = black_box(validate_secondary_checksum(black_box(&sb)));
    }));
    r.push(bench("sec_parse_ieee1588", || {
        let _ = black_box(parse_secondary_header(
            black_box(&sb),
            SecHdrTimeFormat::Ieee1588,
        ));
    }));

    // Intra-packet
    let mut rb = [0u8; 8];
    rb[0..6].copy_from_slice(&0xABCD1234u64.to_le_bytes()[0..6]);
    r.push(bench("ipt_parse_rtc48", || {
        let _ = black_box(parse_intra_packet_time(
            black_box(&rb),
            IntraPacketTimeFormat::Rtc48,
        ));
    }));
    let mut ib = [0u8; 8];
    ib[0..4].copy_from_slice(&123_456_789u32.to_le_bytes());
    ib[4..8].copy_from_slice(&42u32.to_le_bytes());
    r.push(bench("ipt_parse_ieee1588", || {
        let _ = black_box(parse_intra_packet_time(
            black_box(&ib),
            IntraPacketTimeFormat::Ieee1588,
        ));
    }));

    // Correlation
    let sm = build_correlator(100);
    let mt = Rtc::from_raw(50 * 10_000_000 + 5_000_000);
    r.push(bench("corr_100refs_any", || {
        let _ = black_box(sm.correlate(black_box(mt), None));
    }));
    r.push(bench("corr_100refs_by_ch", || {
        let _ = black_box(sm.correlate(black_box(mt), Some(1)));
    }));
    let lg = build_correlator(3600);
    let lt = Rtc::from_raw(3500 * 10_000_000 + 7_500_000);
    r.push(bench("corr_3600refs_any", || {
        let _ = black_box(lg.correlate(black_box(lt), None));
    }));
    // Use reduced iterations for O(n) scan
    let start = Instant::now();
    for _ in 0..1_000 {
        black_box(lg.detect_time_jump(black_box(1), black_box(1_000_000_000)));
    }
    r.push(Bench {
        name: "corr_detect_jumps_3600",
        total: start.elapsed(),
        iters: 1_000,
    });

    // AbsoluteTime
    let t = AbsoluteTime::new(100, 12, 30, 25, 340_000_000).unwrap();
    r.push(bench("abs_add_nanos", || {
        black_box(black_box(t).add_nanos(black_box(15_000_000)));
    }));
    r.push(bench("abs_sub_nanos", || {
        black_box(black_box(t).sub_nanos(black_box(15_000_000)));
    }));
    r.push(bench("abs_total_nanos_of_day", || {
        black_box(black_box(t).total_nanos_of_day());
    }));

    // HOT PATH: the exact code path for every data packet
    let corr = build_correlator(100);
    let hp: [u8; 6] = [0x80, 0x96, 0x98, 0x00, 0x00, 0x00];
    r.push(bench("HOT_rtc_to_absolute", || {
        let rtc = Rtc::from_le_bytes(black_box(hp));
        black_box(corr.correlate(rtc, None).unwrap());
    }));

    // NTP
    let mut ntp_buf = [0u8; 8];
    ntp_buf[0..4].copy_from_slice(&3_944_678_400u32.to_le_bytes());
    ntp_buf[4..8].copy_from_slice(&(1u32 << 31).to_le_bytes());
    r.push(bench("ntp_from_le_bytes", || {
        let _ = black_box(irig106_time::network_time::NtpTime::from_le_bytes(
            black_box(&ntp_buf),
        ));
    }));
    let ntp = irig106_time::network_time::NtpTime::from_le_bytes(&ntp_buf).unwrap();
    r.push(bench("ntp_to_absolute", || {
        let _ = black_box(black_box(ntp).to_absolute());
    }));

    // PTP
    let mut ptp_buf = [0u8; 10];
    ptp_buf[0..6].copy_from_slice(&1_735_689_637u64.to_le_bytes()[0..6]);
    ptp_buf[6..10].copy_from_slice(&500_000_000u32.to_le_bytes());
    r.push(bench("ptp_from_le_bytes", || {
        let _ = black_box(irig106_time::network_time::PtpTime::from_le_bytes(
            black_box(&ptp_buf),
        ));
    }));
    let ptp = irig106_time::network_time::PtpTime::from_le_bytes(&ptp_buf).unwrap();
    r.push(bench("ptp_to_absolute", || {
        let _ = black_box(black_box(ptp).to_absolute(black_box(37)));
    }));

    // Leap second table lookup
    let lst = irig106_time::network_time::LeapSecondTable::builtin();
    r.push(bench("leap_table_lookup", || {
        black_box(lst.offset_at_unix(black_box(1_718_409_600)));
    }));

    // Full F2 payload parse
    let mut f2_payload = [0u8; 12];
    f2_payload[0..4].copy_from_slice(&0u32.to_le_bytes());
    f2_payload[4..8].copy_from_slice(&3_944_678_400u32.to_le_bytes());
    r.push(bench("f2_ntp_payload_parse", || {
        let _ = black_box(irig106_time::network_time::parse_time_f2_payload(
            black_box(&f2_payload),
        ));
    }));

    // Report
    println!();
    println!("================================================================");
    println!("  irig106-time Benchmarks ({} iterations each)", ITERS);
    println!("================================================================");
    println!();
    println!("  {:<32} {:>10} {:>12}", "Benchmark", "ns/iter", "ops/sec");
    println!("  {}", "─".repeat(58));
    for b in &r {
        let ns = b.ns();
        let ops = if ns > 0.0 { 1e9 / ns } else { f64::INFINITY };
        let os = if ops > 1e6 {
            format!("{:.1}M", ops / 1e6)
        } else if ops > 1e3 {
            format!("{:.1}K", ops / 1e3)
        } else {
            format!("{:.0}", ops)
        };
        println!("  {:<32} {:>8.1} ns {:>10}/s", b.name, ns, os);
    }
    println!();
}
