// SPDX-License-Identifier: Apache-2.0 WITH LLVM-Exception
// Copyright 2024 Daniel Fox Franke.

//! Defines a trait for address resolution.

use crate::error::AssemblerError;

/// Trait for a callback to resolve an address from a label.
pub(crate) trait Resolver {
    /// The type of label that this resolver can resolve.
    type Label;

    /// Returns the address to which the label resolves.
    fn resolve(&self, label: &Self::Label) -> Result<ResolvedAddr, AssemblerError<Self::Label>>;

    /// Returns the absolute address to which the label resolves.
    fn resolve_absolute(&self, label: &Self::Label) -> Result<u32, AssemblerError<Self::Label>>;
}

/// The result of address resolution.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ResolvedAddr {
    /// An address at the given absolute address.
    Rom(u32),
    /// An address at the given offset from the start of RAM.
    Ram(u32),
}
