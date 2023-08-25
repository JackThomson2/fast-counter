use std::cell::Cell;
use std::fmt;
use std::iter::repeat_with;
use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};

use crate::safe_getters::SafeGetters;
use crate::utils::{make_new_padded_counter, CachePadded};

static THREAD_COUNTER: AtomicUsize = AtomicUsize::new(1);

thread_local! {
    static THREAD_ID: Cell<usize> = Cell::new(THREAD_COUNTER.fetch_add(1, Ordering::SeqCst));
}

/// A sharded atomic counter
///
/// `ConcurrentCounter` shards cacheline aligned `AtomicIsizes` across a vector for faster updates in
/// a high contention scenarios.
pub struct ConcurrentCounter {
    cells: Vec<CachePadded<AtomicIsize>>,
}

impl fmt::Debug for ConcurrentCounter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConcurrentCounter")
            .field("sum", &self.sum())
            .field("cells", &self.cells.len())
            .finish()
    }
}

impl ConcurrentCounter {
    /// Creates a new `ConcurrentCounter` with a minimum of the `count` cells. Concurrent counter
    /// will align the `count` to the next power of two for better speed when doing the modulus.
    ///
    /// # Examples
    ///
    /// ```
    /// use fast_counter::ConcurrentCounter;
    ///
    /// let counter = ConcurrentCounter::new(10);
    /// ```
    #[inline]
    pub fn new(count: usize) -> Self {
        let count = count.next_power_of_two();
        Self {
            cells: repeat_with(make_new_padded_counter).take(count).collect(),
        }
    }

    #[inline]
    fn thread_id(&self) -> usize {
        THREAD_ID.with(|id| id.get())
    }

    /// Adds the value to the counter, internally with is using `add_with_ordering` with a
    /// `Ordering::Relaxed` and is mainly for convenience.
    ///
    /// `ConcurrentCounter` will identify a cell to add the `value` too with using a `thread_local`
    /// which will try to aleviate the contention on a single number
    ///
    /// # Examples
    ///
    /// ```
    /// use fast_counter::ConcurrentCounter;
    ///
    /// let counter = ConcurrentCounter::new(10);
    /// counter.add(1);
    /// counter.add(-1);
    /// ```
    #[inline]
    pub fn add(&self, value: isize) {
        self.add_with_ordering(value, Ordering::Relaxed)
    }

    /// `ConcurrentCounter` will identify a cell to add the `value` too with using a `thread_local`
    /// which will try to aleviate the contention on a single number. The cell will be updated
    /// atomically using the ordering provided in `ordering`
    ///
    /// # Examples
    ///
    /// ```
    /// use fast_counter::ConcurrentCounter;
    /// use std::sync::atomic::Ordering;
    ///
    /// let counter = ConcurrentCounter::new(10);
    /// counter.add_with_ordering(1, Ordering::SeqCst);
    /// counter.add_with_ordering(-1, Ordering::Relaxed);
    /// ```
    #[inline]
    pub fn add_with_ordering(&self, value: isize, ordering: Ordering) {
        let c = self
            .cells
            .safely_get(self.thread_id() & (self.cells.len() - 1));
        c.value.fetch_add(value, ordering);
    }

    /// This will fetch the sum of the concurrent counter be iterating through each of the cells
    /// and loading the values. Internally this uses `sum_with_ordering` with a `Relaxed` ordering.
    ///
    /// Due to the fact the cells are sharded and the concurrent nature of the library this sum
    /// may be slightly inaccurate. For example if used in a concurrent map and using
    /// `ConcurrentCounter` to track the length, depending on the ordering the length may be returned
    /// as a negative value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fast_counter::ConcurrentCounter;
    ///
    /// let counter = ConcurrentCounter::new(10);
    ///
    /// counter.add(1);
    ///
    /// let sum = counter.sum();
    ///
    /// assert_eq!(sum, 1);
    /// ```
    #[inline]
    pub fn sum(&self) -> isize {
        self.sum_with_ordering(Ordering::Relaxed)
    }

    /// This will fetch the sum of the concurrent counter be iterating through each of the cells
    /// and loading the values with the ordering defined by `ordering`.
    ///
    /// Due to the fact the cells are sharded and the concurrent nature of the library this sum
    /// may be slightly inaccurate. For example if used in a concurrent map and using
    /// `ConcurrentCounter` to track the length, depending on the ordering the length may be returned
    /// as a negative value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::atomic::Ordering;
    /// use fast_counter::ConcurrentCounter;
    ///
    /// let counter = ConcurrentCounter::new(10);
    ///
    /// counter.add(1);
    ///
    /// let sum = counter.sum_with_ordering(Ordering::SeqCst);
    ///
    /// assert_eq!(sum, 1);
    /// ```
    #[inline]
    pub fn sum_with_ordering(&self, ordering: Ordering) -> isize {
        self.cells.iter().map(|c| c.value.load(ordering)).sum()
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

    #[test]
    fn debug_works_as_expected() {
        const WRITE_COUNT: isize = 1_000_000;
        const THREAD_COUNT: isize = 8;
        // Spin up two threads that increment the counter concurrently
        let counter = ConcurrentCounter::new(THREAD_COUNT as usize);

        for _ in 0..WRITE_COUNT {
            counter.add(1);
        }

        assert_eq!(counter.sum(), WRITE_COUNT);

        assert_eq!(
            format!("Counter is: {counter:?}"),
            "Counter is: ConcurrentCounter { sum: 1000000, cells: 8 }"
        )
    }
}
