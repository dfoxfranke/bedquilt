use core::future::IntoFuture;

pub use crate::reactor::JoinHandle;
use crate::reactor::GLOBAL_REACTOR;

pub fn spawn<F>(fut: F) -> JoinHandle<F::Output>
where
    F: IntoFuture,
    F::IntoFuture: Send + 'static,
    F::Output: Send + 'static,
{
    GLOBAL_REACTOR.spawn(fut)
}

pub fn run() {
    GLOBAL_REACTOR.run();
}
