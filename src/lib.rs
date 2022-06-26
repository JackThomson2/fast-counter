#![feature(thread_local)]

use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};

#[derive(Debug)]
pub struct ConcurrentCounter {
    cells: Vec<AtomicIsize>,
}

static THREAD_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[thread_local]
static mut THREAD_ID: usize = 0;

impl ConcurrentCounter {
    pub fn new(count: usize) -> Self {
        Self {
            cells: (0..count)
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

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
        let counter = Arc::new(ConcurrentCounter::new(2));

        let mut threads = Vec::new();

        for _ in 0..2 {
            let counter = counter.clone();
            threads.push(std::thread::spawn(move || {
                counter.add(1);
            }));
        }

        for i in threads {
            i.join().unwrap();
        }

        assert_eq!(counter.sum(), 2);
    }

    #[test]
    fn two_threads_incrementing_multiple_times_concurrently() {
        const WRITE_COUNT: isize = 100_000;
        // Spin up two threads that increment the counter concurrently
        let counter = Arc::new(ConcurrentCounter::new(2));

        let mut threads = Vec::new();

        for _ in 0..2 {
            let counter = counter.clone();
            threads.push(std::thread::spawn(move || {
                for _ in 0..WRITE_COUNT {
                    counter.add(1);
                }
            }));
        }

        for i in threads {
            i.join().unwrap();
        }

        assert_eq!(counter.sum(), 2 * WRITE_COUNT);
    }

    #[test]
    fn multple_threads_incrementing_multiple_times_concurrently() {
        const WRITE_COUNT: isize = 1_000_000;
        const THREAD_COUNT: isize = 20;
        // Spin up two threads that increment the counter concurrently
        let counter = Arc::new(ConcurrentCounter::new(THREAD_COUNT as usize));

        let mut threads = Vec::with_capacity(THREAD_COUNT as usize);

        for _ in 0..THREAD_COUNT {
            let counter = counter.clone();
            threads.push(std::thread::spawn(move || {
                for _ in 0..WRITE_COUNT {
                    counter.add(1);
                }
            }));
        }

        for i in threads {
            i.join().unwrap();
        }

        assert_eq!(counter.sum(), THREAD_COUNT * WRITE_COUNT);
    }
}
