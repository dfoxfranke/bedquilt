// SPDX-License-Identifier: Apache-2.0 WITH LLVM-Exception
// Copyright 2024 Daniel Fox Franke.

//! Some utilities for safely dealing with overflow and numeric conversion.

use crate::error::AssemblerError;

/// Workalike of `.cast_signed()` from `std`, which is not yet stable.
pub(crate) trait CastSign<T> {
    /// Returns the bit pattern of self reinterpreted as a signed integer of the
    /// same size.
    ///
    /// This produces the same result as an `as` cast, but ensures that the
    /// bit-width remains the same.
    fn cast_sign(self) -> T;
}

impl CastSign<i32> for u32 {
    #[allow(clippy::as_conversions, clippy::cast_possible_wrap)]
    #[inline]
    fn cast_sign(self) -> i32 {
        self as i32
    }
}

/// Trait for converting various `Result` types from overflow errors into the
/// one we want.
pub(crate) trait Overflow<T> {
    /// Replace `Self`'s error  with [`AssemblerError::Overflow`].
    fn overflow<L>(self) -> Result<T, AssemblerError<L>>;
}

impl <T> Overflow<T> for Option<T> {
    #[inline]
    fn overflow<L>(self) -> Result<T, AssemblerError<L>> {
        self.ok_or(AssemblerError::Overflow)
    }
}

impl <T> Overflow<T> for Result<T, std::num::TryFromIntError> {
    #[inline]
    fn overflow<L>(self) -> Result<T, AssemblerError<L>> {
        self.or(Err(AssemblerError::Overflow))
    }
}

/// Like ``.next_multiple_of()`` on primitive types, but error rather than
/// panicking on overflow.
#[inline]
pub(crate) fn checked_next_multiple_of<L>(n: u32, m: u32) -> Result<u32, AssemblerError<L>> {
    if u32::MAX - m + 1 < n {
        Err(AssemblerError::Overflow)
    } else {
        Ok(n.next_multiple_of(m))
    }
}