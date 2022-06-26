#![feature(thread_local)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rayon;
use rayon::prelude::*;
use std::{sync::atomic::{AtomicIsize, Ordering, AtomicU8, AtomicUsize}, cell::UnsafeCell};

const ITER: isize = 32 * 1024;

#[derive(Debug)]
struct ConcurrentCounter {
    base: AtomicIsize,
    cells: Vec<AtomicIsize>,
}


const CORES_TO_USE: [usize; 3] = [1, 4, 8];

impl ConcurrentCounter {
    fn new() -> Self {
        Self {
            base: AtomicIsize::new(0),
            cells: (0..num_cpus::get())
                .into_iter()
                .map(|_| AtomicIsize::new(0))
                .collect(),
        }
    }

    fn add(&self, value: isize) {
        let mut base = self.base.load(Ordering::SeqCst);
        let mut index = base + value;

        loop {
            match self.base.compare_exchange(
                base,
                base + value,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(b) => base = b,
            }

            let c = &self.cells[index as usize % self.cells.len()];
            let cv = c.load(Ordering::SeqCst);
            index += cv;

            if c.compare_exchange(cv, cv + value, Ordering::SeqCst, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
    }

    fn sum(&self, ordering: Ordering) -> isize {
        let sum: isize = self.cells.iter().map(|c| c.load(ordering)).sum();

        self.base.load(ordering) + sum
    }
}

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

fn concurrent_counter(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_counter");
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
                        let counter = ConcurrentCounter::new();
                        (0..ITER).into_par_iter().for_each(|_| {
                            counter.add(1);
                        });
                        assert_eq!(ITER, counter.sum(Ordering::Relaxed));
                    })
                });
            },
        );
    }

    group.finish();
}


#[derive(Debug)]
struct JackConcurrentCounter {
    cells: Vec<AtomicIsize>,
}

static THREAD_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[thread_local]
static mut THREAD_ID: usize = 0;

impl JackConcurrentCounter {
    fn new() -> Self {
        Self {
            cells: (0..num_cpus::get().next_power_of_two())
                .into_iter()
                .map(|_| AtomicIsize::new(0))
                .collect(),
        }
    }

    fn thread_id(&self) -> usize {
        unsafe {
            if THREAD_ID == 0 {
                THREAD_ID = THREAD_COUNTER.fetch_add(1, Ordering::Relaxed)
            }
            THREAD_ID
        }
    }

    fn add(&self, value: isize) {
        let c = unsafe { self.cells.get_unchecked(self.thread_id() & (self.cells.len() - 1))} ;
        c.fetch_add(value, Ordering::SeqCst);
    }

    fn sum(&self, ordering: Ordering) -> isize {
       self.cells.iter().map(|c| c.load(ordering)).sum()
    }
}

fn jack_counter(c: &mut Criterion) {
    let mut group = c.benchmark_group("jack_counter");
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
                        let counter = JackConcurrentCounter::new();
                        (0..ITER).into_par_iter().for_each(|_| {
                            counter.add(1);
                        });
                        assert_eq!(ITER, counter.sum(Ordering::Relaxed));
                    })
                });
            },
        );
    }
}



criterion_group!(
    benches,
    jack_counter,
    concurrent_counter,
    atomic_counter,
);
criterion_main!(benches);