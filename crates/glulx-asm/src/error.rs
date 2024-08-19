// SPDX-License-Identifier: Apache-2.0 WITH LLVM-Exception
// Copyright 2024 Daniel Fox Franke.

//! Definition and impls for [`AssemblerError`].

use core::fmt::{Debug,Display};

#[derive(Debug,Copy,Clone)]
/// Errors that can occur during assembly.
pub enum AssemblerError<L> {
    /// Assembly would overflow Glulx's 4 GiB address space.
    Overflow,
    /// An operand referenced a label which was not defined.
    UndefinedLabel(L),
    /// A label was defined in multiple places.
    DuplicateLabel(L),
}

impl <L> Display for AssemblerError<L> where L: Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssemblerError::Overflow => write!(f, "address space overflow"),
            AssemblerError::UndefinedLabel(l) => write!(f, "undefined label {l}"),
            AssemblerError::DuplicateLabel(l) => write!(f, "duplicate label {l}"),
        }
    }
}

#[cfg(feature = "std")]
impl <L> std::error::Error for AssemblerError<L> where L: Debug + Display {}