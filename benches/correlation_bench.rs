//! Criterion benchmarks for correlation engine performance.
//!
//! Measures the P4-01 channel-indexed optimization against various
//! reference point counts and channel configurations.
//!
//! # Run
//! ```sh
//! cargo bench --bench correlation_bench
//! ```

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use irig106_time::*;

fn build_correlator(n_refs: usize, n_channels: u16) -> TimeCorrelator {
    let mut c = TimeCorrelator::new();
    for i in 0..n_refs {
        let ch = (i as u16) % n_channels;
        c.add_reference(
            ch,
            Rtc::from_raw((i as u64 + 1) * 10_000_000),
            AbsoluteTime::new(100, 12, 0, i as u8 % 60, 0).unwrap(),
        );
    }
    c
}

fn bench_correlate_any(c: &mut Criterion) {
    let mut group = c.benchmark_group("correlate_any");
    for n in [10, 100, 1000, 3600] {
        let corr = build_correlator(n, 4);
        let mid = Rtc::from_raw((n as u64 / 2) * 10_000_000 + 5_000_000);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| corr.correlate(black_box(mid), None));
        });
    }
    group.finish();
}

fn bench_correlate_by_channel(c: &mut Criterion) {
    let mut group = c.benchmark_group("correlate_by_channel");
    for n in [10, 100, 1000, 3600] {
        let corr = build_correlator(n, 4);
        let mid = Rtc::from_raw((n as u64 / 2) * 10_000_000 + 5_000_000);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| corr.correlate(black_box(mid), Some(1)));
        });
    }
    group.finish();
}

fn bench_detect_time_jump(c: &mut Criterion) {
    let mut group = c.benchmark_group("detect_time_jump");
    for n in [100, 1000, 3600] {
        let corr = build_correlator(n, 4);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| corr.detect_time_jump(black_box(1), black_box(1_000_000)));
        });
    }
    group.finish();
}

fn bench_drift_ppm(c: &mut Criterion) {
    let mut group = c.benchmark_group("drift_ppm");
    for n in [100, 1000, 3600] {
        let corr = build_correlator(n, 4);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| corr.drift_ppm(black_box(1)));
        });
    }
    group.finish();
}

fn bench_add_reference(c: &mut Criterion) {
    c.bench_function("add_reference", |b| {
        b.iter(|| {
            let mut corr = TimeCorrelator::new();
            for i in 0..100u64 {
                corr.add_reference(
                    black_box(1),
                    black_box(Rtc::from_raw(i * 10_000_000)),
                    black_box(AbsoluteTime::new(100, 12, 0, 0, 0).unwrap()),
                );
            }
        });
    });
}

criterion_group!(
    benches,
    bench_correlate_any,
    bench_correlate_by_channel,
    bench_detect_time_jump,
    bench_drift_ppm,
    bench_add_reference,
);
criterion_main!(benches);
