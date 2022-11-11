use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};
use std::cell::Cell;

use crate::safe_getters::SafeGetters;

pub struct ConcurrentCounter {
    cells: Vec<AtomicIsize>,
}

static THREAD_COUNTER: AtomicUsize = AtomicUsize::new(1);

thread_local! {
    static THREAD_ID: Cell<usize> = Cell::new(THREAD_COUNTER.fetch_add(1, Ordering::SeqCst));
}

impl ConcurrentCounter {
    #[inline]
    pub fn new(count: usize) -> Self {
        let count = count.next_power_of_two();
        Self {
            cells: (0..count)
                .into_iter()
                .map(|_| AtomicIsize::new(0))
                .collect(),
        }
    }

    #[inline]
    fn thread_id(&self) -> usize {
        THREAD_ID.with(|id| {
            id.get()
        })
    }

    #[inline]
    pub fn add(&self, value: isize) {
        let c = self.cells.safely_get(self.thread_id() & (self.cells.len() - 1));
        c.fetch_add(value, Ordering::Relaxed);
    }

    #[inline]
    pub fn sum(&self) -> isize {
        self.cells.iter().map(|c| c.load(Ordering::Relaxed)).sum()
    }
}

#[cfg(test)]
mod tests {
    use crate::ConcurrentCounter;

    #[test]
    fn basic_test() {
        let counter = ConcurrentCounter::new(1);
        counter.add(1);
        assert_eq!(counter.sum(), 1);
    }

    #[test]
    fn increment_multiple_times() {
        let counter = ConcurrentCounter::new(1);
        counter.add(1);
        counter.add(1);
        counter.add(1);
        assert_eq!(counter.sum(), 3);
    }

    #[test]
    fn two_threads_incrementing_concurrently() {
        // Spin up two threads that increment the counter concurrently
        let counter = ConcurrentCounter::new(2);

        std::thread::scope(|s| {
            for _ in 0..2 {
                s.spawn(|| {
                    counter.add(1);
                });
            }
        });


        assert_eq!(counter.sum(), 2);
    }

    #[test]
    fn two_threads_incrementing_multiple_times_concurrently() {
        const WRITE_COUNT: isize = 100_000;
        // Spin up two threads that increment the counter concurrently
        let counter = ConcurrentCounter::new(2);

        std::thread::scope(|s| {
            for _ in 0..2 {
                s.spawn(|| {
                    for _ in 0..WRITE_COUNT {
                        counter.add(1);
                    }
                });
            }
        });


        assert_eq!(counter.sum(), 2 * WRITE_COUNT);
    }

    #[test]
    fn multple_threads_incrementing_multiple_times_concurrently() {
        const WRITE_COUNT: isize = 1_000_000;
        const THREAD_COUNT: isize = 8;
        // Spin up two threads that increment the counter concurrently
        let counter = ConcurrentCounter::new(THREAD_COUNT as usize);

        std::thread::scope(|s| {
            for _ in 0..THREAD_COUNT {
                s.spawn(|| {
                    for _ in 0..WRITE_COUNT {
                        counter.add(1);
                    }
                });
            }
        });


        assert_eq!(counter.sum(), THREAD_COUNT * WRITE_COUNT);
    }
}
