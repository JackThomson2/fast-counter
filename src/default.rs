use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};
use std::cell::UnsafeCell;

pub struct ConcurrentCounter {
    cells: Vec<AtomicIsize>,
}

static THREAD_COUNTER: AtomicUsize = AtomicUsize::new(1);

thread_local! {
    static THREAD_ID: UnsafeCell<usize> = UnsafeCell::new(0);
}

impl ConcurrentCounter {
    pub fn new(count: usize) -> Self {
        let count = count.next_power_of_two();
        Self {
            cells: (0..count)
                .into_iter()
                .map(|_| AtomicIsize::new(0))
                .collect(),
        }
    }

    pub fn reset_counter(&self) {
        THREAD_COUNTER.store(1, Ordering::SeqCst)
    }
    
    fn thread_id(&self) -> usize {
        unsafe { THREAD_ID.with(|id| {
            let mut val = *id.get();
            if val == 0 {
                val = THREAD_COUNTER.fetch_add(1, Ordering::SeqCst);
                *id.get().as_mut().unwrap_unchecked() = val;
            }
            val
        }) }
    }

    pub fn add(&self, value: isize) {
        let c = unsafe {
            self.cells
                .get_unchecked(self.thread_id() & (self.cells.len() - 1))
        };
        c.fetch_add(value, Ordering::Relaxed);
    }

    pub fn sum(&self) -> isize {
        self.cells.iter().map(|c| c.load(Ordering::Relaxed)).sum()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, atomic::Ordering};

    use crate::default::ConcurrentCounter;

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

    #[test]
    fn checking_good_distribution() {
        const WRITE_COUNT: isize = 1_000_000;
        const THREAD_COUNT: isize = 16;
        // Spin up two threads that increment the counter concurrently
        let counter = Arc::new(ConcurrentCounter::new(THREAD_COUNT as usize));
        counter.reset_counter();

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

        println!("{:?}", counter.cells);

        for cell in counter.cells.iter() {
            assert_eq!(cell.load(Ordering::Relaxed), WRITE_COUNT);
        }
    }
}
