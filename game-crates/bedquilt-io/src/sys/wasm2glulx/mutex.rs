use core::{
    num::NonZeroUsize,
    sync::atomic::{AtomicBool, AtomicUsize},
};
use std::sync::atomic::Ordering;

#[derive(Debug, Default)]
pub struct RawMutexImpl(AtomicBool);
pub type RawFairMutexImpl = RawMutexImpl;
pub struct RawRwLockImpl(AtomicUsize, AtomicBool);
pub struct RawThreadIdImpl;

unsafe impl lock_api::RawMutex for RawMutexImpl {
    type GuardMarker = lock_api::GuardSend;
    #[allow(clippy::declare_interior_mutable_const)]
    const INIT: Self = RawMutexImpl(AtomicBool::new(false));

    fn try_lock(&self) -> bool {
        !self.0.swap(true, Ordering::Acquire)
    }

    fn lock(&self) {
        if !self.try_lock() {
            panic!("Deadlock")
        }
    }

    unsafe fn unlock(&self) {
        self.0.store(false, Ordering::Release);
    }
}

unsafe impl lock_api::RawMutexFair for RawMutexImpl {
    unsafe fn unlock_fair(&self) {
        lock_api::RawMutex::unlock(self);
    }
}

unsafe impl lock_api::RawRwLock for RawRwLockImpl {
    #[allow(clippy::declare_interior_mutable_const)]
    const INIT: Self = RawRwLockImpl(AtomicUsize::new(0), AtomicBool::new(false));

    type GuardMarker = lock_api::GuardSend;

    fn lock_shared(&self) {
        if !self.try_lock_shared() {
            panic!("Deadlock")
        }
    }

    fn try_lock_shared(&self) -> bool {
        if self.1.swap(true, Ordering::Acquire) {
            false
        } else {
            self.0.fetch_add(1, Ordering::Acquire);
            self.1.store(false, Ordering::Release);
            true
        }
    }

    unsafe fn unlock_shared(&self) {
        self.0.fetch_sub(1, Ordering::Release);
    }

    fn lock_exclusive(&self) {
        if !self.try_lock_exclusive() {
            panic!("Deadlock")
        }
    }

    fn try_lock_exclusive(&self) -> bool {
        if self.1.swap(true, Ordering::Acquire) {
            false
        } else if self.0.load(Ordering::Acquire) != 0 {
            self.1.store(false, Ordering::Release);
            false
        } else {
            true
        }
    }

    unsafe fn unlock_exclusive(&self) {
        self.1.store(false, Ordering::Release);
    }
}

unsafe impl lock_api::GetThreadId for RawThreadIdImpl {
    const INIT: Self = RawThreadIdImpl;

    fn nonzero_thread_id(&self) -> NonZeroUsize {
        NonZeroUsize::new(1).unwrap()
    }
}
