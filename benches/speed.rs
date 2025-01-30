//! speed benchmark
use criterion::{criterion_group, criterion_main, Criterion};
use paymentlib::run_from_csv;

macro_rules! test_csv {
    ($fname:expr) => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/src/tests/data/", $fname)
    };
}
/// Benchmark speed of the application
pub fn speed_benchmark(c: &mut Criterion) {
    c.bench_function("speed", |b| {
        b.iter(|| run_from_csv(test_csv!("bench.csv")))
    });
}

criterion_group!(benches, speed_benchmark);
criterion_main!(benches);
