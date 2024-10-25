//! Spawn and run tasks (a.k.a. green threads).

use core::future::IntoFuture;

pub use crate::reactor::JoinHandle;
use crate::reactor::GLOBAL_REACTOR;

/// Spawns an asynchronous task.
pub fn spawn<F>(fut: F) -> JoinHandle<F::Output>
where
    F: IntoFuture,
    F::IntoFuture: Send + 'static,
    F::Output: Send + 'static,
{
    GLOBAL_REACTOR.spawn(fut)
}

/// Runs all spawned tasks to completion.
pub fn run() {
    GLOBAL_REACTOR.run();
}
