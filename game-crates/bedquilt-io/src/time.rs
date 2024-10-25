//! Time and timers.

/// Named futures returned by time-related functions.
/// 
/// Since you rarely should need to refer to any of these types by name, they're
/// shuffled into this separate module in order to declutter things.
pub mod futures {
    pub use crate::reactor::TimerFuture;
}

use futures::TimerFuture;

use crate::reactor::GLOBAL_REACTOR;

/// Sets the interval between ticks.
///
/// Setting this to zero, or leaving it at its initial setting of zero, will
/// disable all timers.
pub use crate::sys::glk::set_tick_interval;

/// Returns the current UTC time and the local timezone's UTC offset.
///
/// The first element of the returned tuple is the duration since the POSIX
/// epoch of midnight, January 1, 1970, disregarding leap seconds. Keep in mind
/// that this is derived from the user's system clock, which may be reset an any
/// time and may resultingly move backward. Unfortunately, Glk does not provide
/// a monotonic clock.
///
/// The second element is the local timezone's UTC offset, in seconds. Keep in
/// mind that this offset is not a constant: the user may change timezones, or
/// DST may begin or end.
///
/// This the *only* function Bedquilt provides for dealing with system time. For
/// calendaring etc., pass what's returned from this function into your favorite
/// third-party crate such as `time` or `chrono`.
pub use crate::sys::glk::get_time_and_offset;

/// Sets a timer, returning a future that will become ready in the given number
/// of ticks.
pub fn sleep(ticks: u64) -> TimerFuture {
    GLOBAL_REACTOR.set_timer(GLOBAL_REACTOR.current_tick().saturating_add(ticks))
}
