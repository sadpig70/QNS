//! Drift scan benchmark.
//!
//! Target: <10ms for default scanner

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use qns_profiler::DriftScanner;

fn bench_drift_scan_default(c: &mut Criterion) {
    let mut scanner = DriftScanner::with_defaults();

    c.bench_function("drift_scan_default", |b| {
        b.iter(|| scanner.scan(0).unwrap())
    });
}

fn bench_drift_scan_fast(c: &mut Criterion) {
    let mut scanner = DriftScanner::fast();

    c.bench_function("drift_scan_fast", |b| b.iter(|| scanner.scan(0).unwrap()));
}

fn bench_drift_scan_accurate(c: &mut Criterion) {
    let mut scanner = DriftScanner::accurate();

    c.bench_function("drift_scan_accurate", |b| {
        b.iter(|| scanner.scan(0).unwrap())
    });
}

fn bench_drift_scan_batch(c: &mut Criterion) {
    let mut scanner = DriftScanner::fast();
    let qubit_ids: Vec<usize> = (0..5).collect();

    c.bench_function("drift_scan_batch_5", |b| {
        b.iter(|| scanner.scan_batch(&qubit_ids).unwrap())
    });
}

fn bench_drift_scan_with_history(c: &mut Criterion) {
    let mut group = c.benchmark_group("drift_scan_history");

    for history_size in [0, 10, 50, 100] {
        let mut scanner = DriftScanner::fast();

        // Build up history
        for _ in 0..history_size {
            scanner.scan(0).unwrap();
        }

        group.bench_with_input(
            BenchmarkId::new("history_size", history_size),
            &history_size,
            |b, _| b.iter(|| scanner.scan(0).unwrap()),
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_drift_scan_default,
    bench_drift_scan_fast,
    bench_drift_scan_accurate,
    bench_drift_scan_batch,
    bench_drift_scan_with_history,
);
criterion_main!(benches);
