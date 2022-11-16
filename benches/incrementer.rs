use criterion::{criterion_group, BenchmarkId, Criterion, Throughput};
use rayon;
use rayon::prelude::*;
use std::sync::atomic::{AtomicIsize, Ordering};

const ITER: isize = 1024 * 1024;
const CORES_TO_USE: [usize; 5] = [1, 2, 4, 8, 16];

use fast_counter::ConcurrentCounter as ConcurrentCounterTLMacro;

fn atomic_counter(c: &mut Criterion) {
    let mut group = c.benchmark_group("atomic_addition");
    group.throughput(Throughput::Elements(ITER as u64));

    for threads in CORES_TO_USE {
        group.bench_with_input(
            BenchmarkId::new("atomic_isize", threads),
            &threads,
            |b, &threads| {
                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(threads)
                    .build()
                    .unwrap();
                pool.install(|| {
                    b.iter(|| {
                        let counter = AtomicIsize::new(0);
                        (0..ITER).into_par_iter().for_each(|_| {
                            counter.fetch_add(1, Ordering::Relaxed);
                        });
                    })
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("fast_counter", threads),
            &threads,
            |b, &threads| {
                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(threads)
                    .build()
                    .unwrap();
                pool.install(|| {
                    b.iter(|| {
                        let counter = ConcurrentCounterTLMacro::new(threads);
                        (0..ITER).into_par_iter().for_each(|_| {
                            counter.add(1);
                        });
                    })
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    atomic_counter,
);
