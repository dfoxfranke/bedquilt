pub use crate::sys::mutex::{RawFairMutexImpl, RawMutexImpl, RawRwLockImpl, RawThreadIdImpl};

pub type Mutex<T> = lock_api::Mutex<RawMutexImpl, T>;
pub type MutexGuard<'a, T> = lock_api::MutexGuard<'a, RawMutexImpl, T>;
pub type MappedMutexGuard<'a, T> = lock_api::MappedMutexGuard<'a, RawMutexImpl, T>;
pub type FairMutexGuard<'a, T> = lock_api::MutexGuard<'a, RawFairMutexImpl, T>;
pub type MappedFairMutexGuard<'a, T> = lock_api::MappedMutexGuard<'a, RawFairMutexImpl, T>;
pub type RwLock<'a, T> = lock_api::RwLock<RawRwLockImpl, T>;
pub type RwLockReadGuard<'a, T> = lock_api::RwLockReadGuard<'a, RawRwLockImpl, T>;
pub type RwLockWriteGuard<'a, T> = lock_api::RwLockWriteGuard<'a, RawRwLockImpl, T>;
pub type MappedRwLockReadGuard<'a, T> = lock_api::MappedRwLockReadGuard<'a, RawRwLockImpl, T>;
pub type MappedRwLockWriteGuard<'a, T> = lock_api::MappedRwLockWriteGuard<'a, RawRwLockImpl, T>;
