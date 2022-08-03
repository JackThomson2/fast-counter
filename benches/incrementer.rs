#![feature(thread_local)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rayon;
use rayon::prelude::*;
use std::sync::atomic::{AtomicIsize, Ordering};

const ITER: isize = 32 * 1024;
const CORES_TO_USE: [usize; 4] = [2, 4, 8, 16];

fn atomic_counter(c: &mut Criterion) {
    let mut group = c.benchmark_group("atomic_counter");
    group.throughput(Throughput::Elements(ITER as u64));

    for threads in CORES_TO_USE {
        group.bench_with_input(
            BenchmarkId::from_parameter(threads),
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
                        assert_eq!(ITER, counter.load(Ordering::Relaxed));
                    })
                });
            },
        );
    }

    group.finish();
}

use fast_counter::ConcurrentCounter as JackCounter;

fn fast_counter(c: &mut Criterion) {
    let mut group = c.benchmark_group("fast_counter_nightly");
    group.throughput(Throughput::Elements(ITER as u64));

    for threads in CORES_TO_USE {
        group.bench_with_input(
            BenchmarkId::from_parameter(threads),
            &threads,
            |b, &threads| {
                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(threads)
                    .build()
                    .unwrap();
                pool.install(|| {
                    b.iter(|| {
                        let counter = JackCounter::new(threads);
                        (0..ITER).into_par_iter().for_each(|_| {
                            counter.add(1);
                        });
                        assert_eq!(ITER, counter.sum());
                    })
                });
            },
        );
    }
}

use fast_counter::default::ConcurrentCounter as ConcurrentCounterTLMacro;

fn fast_counter_stable(c: &mut Criterion) {
    let mut group = c.benchmark_group("fast_counter_stable");
    group.throughput(Throughput::Elements(ITER as u64));

    for threads in CORES_TO_USE {
        group.bench_with_input(
            BenchmarkId::from_parameter(threads),
            &threads,
            |b, &threads| {
                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(threads)
                    .build()
                    .unwrap();
                pool.install(|| {
                    b.iter(|| {
                        let counter = ConcurrentCounterTLMacro::new(threads);
                        counter.reset_counter();
                        (0..ITER).into_par_iter().for_each(|_| {
                            counter.add(1);
                        });
                        assert_eq!(ITER, counter.sum());
                    })
                });
            },
        );
    }
}

criterion_group!(
    benches,
    atomic_counter,
    fast_counter,
    fast_counter_stable,
);
criterion_main!(benches);
