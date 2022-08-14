#![cfg_attr(feature = "nightly", feature(thread_local))]

#[macro_use]
mod safe_getters;

pub mod default;

#[cfg(feature = "nightly")]
mod nightly;
#[cfg(feature = "nightly")]
pub use nightly::*;

#[cfg(not(feature = "nightly"))]
pub use default::*;