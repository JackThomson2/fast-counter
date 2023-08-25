//! Fast-counter is a sharded concurrent atomic counter
//!
//! The library works by sharding the atomic numbers between multiple values, each thread will
//! attempt to read from a different cell. This helps with cache-thrashing and contention. This
//! benefit is seen when there is a greater number of threads competing over a single cell, on my
//! machine with 16 cores attempting to update the value it can be nearly 60x faster. The price you
//! pay for this is higher memory usage as we have multiple Atomic numbers which are cache-padded
//! so can be significantly higher.
//!
//! # Usage
//!
//! Usage of the library is simple, create a new ConcurrentCounter with then number of shards you
//! wish to have, internally the library will use the next power of two as the number of cells for
//! a faster modulus.
//!
//! ```rust
//! use fast_counter::ConcurrentCounter;
//!
//! let counter = ConcurrentCounter::new(10);
//!
//! counter.add(1);
//!
//! let sum = counter.sum();
//!
//! assert_eq!(sum, 1);
//! ```
//!
//! # Performance considerations
//!
//! The library will perform best when the threads are accessing their own cell consistently. This
//! can helped by making sure more than enough cells are allocated for the number of threads which
//! are going to be writing to the cell.
//!
//! Due to the sharding behaviour the time to call the `sum()` method does slow down with the
//! increase in shards, if this becomes a bottleneck it may be worth investigating running with a
//! lower shard counter
//!

#[macro_use]
mod safe_getters;
mod utils;

pub mod counter;
pub use counter::*;
