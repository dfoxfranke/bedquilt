//! Random number generation
//! 
//! This module provides OS entropy source compatible with the
//! [`rand`](https://docs.rs/rand) crate. On Glulx, it is backed by Glulx's
//! `random` intrinsic. On other platforms, it is a re-export of
//! `rand_core::OsRng`. You can therefore declare your `rand` dependency
//! with `default-features = false` (thus avoiding a dependency on std)
//! and still have an entropy source available.

pub use crate::sys::random::OsRng;