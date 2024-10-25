//! Synchronization primitives.
//! 
//! The types defined in this module are based on [`lock_api`]. On platforms
//! other than Glulx, the raw locks are re-exports from
//! [`parking_lot`](https://docs.rs/parking_lot). On Glulx, since it is
//! single-threaded, they are trivial implementations whose `lock` method will
//! panic if the lock is not immediately available, since this is certain to
//! indicate a deadlock. The trivial locks still perform a useful function: they
//! allow you to keep mutable global state without writing unsafe code.
//! 
//! These are synchronous primitives, meaning that the lock methods are
//! synchronous, and guards cannot be held across an `.await` point. For
//! asynchronous communication, you can use any `no_std`, executor-agnostic
//! crate such as [`async-lock`](https://docs.rs/async-lock) or
//! [`async-channel`](https://docs.rs/async-channel).

pub use crate::sys::mutex::{RawFairMutexImpl, RawMutexImpl, RawRwLockImpl, RawThreadIdImpl};

/// A mutual exclusion primitive useful for protecting shared data.
pub type Mutex<T> = lock_api::Mutex<RawMutexImpl, T>;

/// An RAII implementation of a “scoped lock” of a mutex. When this structure is
/// dropped (falls out of scope), the lock will be unlocked.
pub type MutexGuard<'a, T> = lock_api::MutexGuard<'a, RawMutexImpl, T>;

/// An RAII mutex guard returned by `MutexGuard::map`, which can point to a
/// subfield of the protected data.
pub type MappedMutexGuard<'a, T> = lock_api::MappedMutexGuard<'a, RawMutexImpl, T>;

/// A mutual exclusive primitive that is always fair, useful for protecting
/// shared data.
pub type FairMutex<T> = lock_api::Mutex<RawFairMutexImpl, T>;

/// An RAII implementation of a “scoped lock” of a fair mutex. When this
/// structure is dropped (falls out of scope), the lock will be unlocked.
pub type FairMutexGuard<'a, T> = lock_api::MutexGuard<'a, RawFairMutexImpl, T>;

/// An RAII mutex guard returned by `FairMutexGuard::map`, which can point to a
/// subfield of the protected data.
pub type MappedFairMutexGuard<'a, T> = lock_api::MappedMutexGuard<'a, RawFairMutexImpl, T>;

/// A reader-writer lock.
pub type RwLock<'a, T> = lock_api::RwLock<RawRwLockImpl, T>;

/// RAII structure used to release the shared read access of a lock when
/// dropped.
pub type RwLockReadGuard<'a, T> = lock_api::RwLockReadGuard<'a, RawRwLockImpl, T>;

/// RAII structure used to release the exclusive write access of a lock when
/// dropped.
pub type RwLockWriteGuard<'a, T> = lock_api::RwLockWriteGuard<'a, RawRwLockImpl, T>;

/// An RAII read lock guard returned by `RwLockReadGuard::map`, which can point
/// to a subfield of the protected data.
pub type MappedRwLockReadGuard<'a, T> = lock_api::MappedRwLockReadGuard<'a, RawRwLockImpl, T>;

/// An RAII write lock guard returned by `RwLockWriteGuard::map`, which can
/// point to a subfield of the protected data.
pub type MappedRwLockWriteGuard<'a, T> = lock_api::MappedRwLockWriteGuard<'a, RawRwLockImpl, T>;
