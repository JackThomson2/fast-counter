#![feature(thread_local)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rayon;
use rayon::prelude::*;
use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};

const ITER: isize = 32 * 1024;

#[derive(Debug)]
struct ConcurrentCounter {
    base: AtomicIsize,
    cells: Vec<AtomicIsize>,
}

const CORES_TO_USE: [usize; 2] = [2, 4];

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

use fastcounter::ConcurrentCounter as JackCounter;

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

use std::cell::UnsafeCell;

pub struct ConcurrentCounterTLMacro {
    cells: Vec<AtomicIsize>,
}

thread_local! {
    static THREAD_ID_LOCAL: UnsafeCell<usize> = UnsafeCell::new(THREAD_COUNTER_TL.fetch_add(1, Ordering::Relaxed));
}

static THREAD_COUNTER_TL: AtomicUsize = AtomicUsize::new(1);

impl ConcurrentCounterTLMacro {
    pub fn new(count: usize) -> Self {
        let count = count.next_power_of_two();
        Self {
            cells: (0..count)
                .into_iter()
                .map(|_| AtomicIsize::new(0))
                .collect(),
        }
    }

    fn thread_id(&self) -> usize {
        unsafe { THREAD_ID_LOCAL.with(|id| *id.get()) }
    }

    pub fn add(&self, value: isize) {
        let c = unsafe {
            self.cells
                .get_unchecked(self.thread_id() & (self.cells.len() - 1))
        };
        c.fetch_add(value, Ordering::SeqCst);
    }

    pub fn sum(&self) -> isize {
        self.cells.iter().map(|c| c.load(Ordering::Acquire)).sum()
    }
}

fn jack_counter_thread_local(c: &mut Criterion) {
    let mut group = c.benchmark_group("jack_counter thread local macro");
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
    concurrent_counter,
    atomic_counter,
    jack_counter,
    jack_counter_thread_local,
);
criterion_main!(benches);
