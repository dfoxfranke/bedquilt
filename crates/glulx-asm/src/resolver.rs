// SPDX-License-Identifier: Apache-2.0 WITH LLVM-Exception
// Copyright 2024 Daniel Fox Franke.

//! Defines a trait for address resolution.

use crate::error::AssemblerError;

/// Trait for a callback to resolve an address from a label.
/// 
/// This could just be a closure. Originally it was, but I was stumbling over an
/// obscure bug related to recursive trait resolution. I know now how to avoid
/// the bug and could change it back, but there's no reason to bother.
pub(crate) trait Resolver {
    /// The type of label that this resolver can resolve.
    type Label;

    /// Returns the address to which the label resolves.
    fn resolve(&self, label: &Self::Label) -> Result<ResolvedAddr, AssemblerError<Self::Label>>;
}

/// The result of address resolution.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ResolvedAddr {
    /// An address at the given absolute address.
    Rom(u32),
    /// An address at the given offset from the start of RAM.
    Ram(u32),
}
