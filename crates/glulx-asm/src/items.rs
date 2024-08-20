//! [`Item`] and related types.

// SPDX-License-Identifier: Apache-2.0 WITH LLVM-Exception
// Copyright 2024 Daniel Fox Franke.

use bytes::{BufMut, Bytes};
use core::num::NonZeroU32;
use never::Never;

use crate::{
    cast::Overflow,
    decoding_table::DecodeNode,
    error::AssemblerError,
    instr_def::Instr,
    resolver::{ResolvedAddr, Resolver},
    strings::{MysteryString, Utf32String},
};

/// A reference to an item.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ItemRef<L> {
    /// Reference to an item by its label, plus an offset.
    Label(L, i32),
    /// Reference to an item by raw address.
    ///
    /// This variant is a leaked implementation detail; there is never any
    /// reason for users to construct it.
    Resolved(u32),
}

/// An item of top-level content in a story file assembly.
#[derive(Debug, Clone)]
pub enum Item<L> {
    /// Generates padding such that the next item is aligned to a multiple of
    /// the given `NonZeroU32`, which will likely be a power of two but
    /// arbitrary values are accepted. Glulx itself never requires any item in
    /// main memory to be aligned, but you can use this if you are generating
    /// code which assumes some alignment.
    Align(NonZeroU32),
    /// A string decoding table.
    DecodingTable(DecodeNode<L>),
    /// A header for a function, specifying its calling convention and how many
    /// locals it allocates. Since one- and two-byte locals have been deprecated
    /// since 2010, this assembler does not support them and all locals are taken
    /// to be four bytes.
    FnHeader(CallingConvention, u32),
    /// An instruction.
    Instr(Instr<L>),
    /// An `E0` string (usually Latin-1).
    MysteryString(MysteryString),
    /// An `E1` string of Huffman-coded data, decompressed via a decoding table.
    /// No validity checks are performed.
    CompressedString(Bytes),
    /// An `E2` (Unicode) string.
    Utf32String(Utf32String),
    /// Some arbitrary bytes to be serialized verbatim.
    Blob(Bytes),
    /// Four bytes representing the absolute adddress of the given label.
    Label(L),
}

/// Placeholder for space in RAM that shoud be allocated at startup with
/// initially-zeroed content.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ZeroItem {
    /// Reserves the given amount of space, in bytes.
    Space(u32),
    /// Generates padding such that the next item is aligned to a multiple of
    /// the given `NonZeroU32`, which will likely be a power of two but
    /// arbitrary values are accepted. Glulx itself never requires any item in
    /// main memory to be aligned, but you can use this if you are generating
    /// code which assumes some alignment.
    Align(NonZeroU32),
}

/// Specifies how a function receives its arguments.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CallingConvention {
    /// Arguments are placed on the stack.
    ArgsOnStack,
    /// Arguments are placed in local variables.
    ArgsInLocals,
}

impl<L> Item<L> {
    /// Applies the given mapping function to all labels within the item.
    pub fn map<F, M>(self, mut f: F) -> Item<M>
    where
        F: FnMut(L) -> M,
    {
        match self {
            Item::Align(a) => Item::Align(a),
            Item::DecodingTable(t) => Item::DecodingTable(t.map(&mut f)),
            Item::FnHeader(t, n) => Item::FnHeader(t, n),
            Item::Instr(i) => Item::Instr(i.map(f)),
            Item::MysteryString(s) => Item::MysteryString(s),
            Item::Utf32String(s) => Item::Utf32String(s),
            Item::CompressedString(s) => Item::CompressedString(s),
            Item::Blob(b) => Item::Blob(b),
            Item::Label(l) => Item::Label(f(l)),
        }
    }

    pub(crate) fn worst_len(&self) -> usize {
        match self {
            Item::Align(_) => 0,
            Item::DecodingTable(t) => 12 + t.len(),
            Item::FnHeader(_, n) => {
                let n_records: usize = n
                    .div_ceil(255)
                    .try_into()
                    .expect("u32 should fit in a usize");
                2 * n_records + 3
            }
            Item::Instr(i) => i.worst_len(),
            Item::MysteryString(s) => s.len() + 2,
            Item::Utf32String(s) => s.byte_len() + 8,
            Item::CompressedString(s) => 1 + s.len(),
            Item::Blob(b) => b.len(),
            Item::Label(_) => 4,
        }
    }

    pub(crate) fn align(&self) -> u32 {
        match self {
            Item::Align(a) => (*a).into(),
            _ => 1,
        }
    }

    pub(crate) fn resolved_len<R>(
        &self,
        position: u32,
        ramstart: u32,
        resolver: &R,
    ) -> Result<usize, AssemblerError<L>>
    where
        R: Resolver<Label = L>,
    {
        Ok(match self {
            Item::Instr(i) => i.resolve(position, ramstart, resolver)?.len(),
            _ => self.worst_len(),
        })
    }

    pub(crate) fn serialize<R, B>(
        &self,
        position: u32,
        ramstart: u32,
        resolver: &R,
        mut buf: B,
    ) -> Result<(), AssemblerError<L>>
    where
        R: Resolver<Label = L>,
        B: BufMut,
    {
        match self {
            Item::Align(x) => {
                let align: u32 = (*x).into();
                let modulus = position % align;
                let padding = if modulus == 0 { 0 } else { align - modulus };
                buf.put_bytes(
                    0,
                    padding
                        .try_into()
                        .expect("u32 to usize conversion should succeed"),
                );
            }
            Item::DecodingTable(table) => {
                let resolved = table.resolve(ramstart, resolver)?;
                let count = u32::try_from(resolved.count_nodes()).overflow()?;
                let length =
                    u32::try_from(resolved.len().checked_add(12).overflow()?).overflow()?;
                let root = position.checked_add(12).overflow()?;
                buf.put_u32(length);
                buf.put_u32(count);
                buf.put_u32(root);
                resolved.serialize(0, &mut buf);
            }
            Item::FnHeader(cc, args) => {
                match cc {
                    CallingConvention::ArgsOnStack => buf.put_u8(0xc0),
                    CallingConvention::ArgsInLocals => buf.put_u8(0xc1),
                }

                for _ in 0..(*args / 255) {
                    buf.put_u8(4);
                    buf.put_u8(255);
                }

                if *args % 255 != 0 {
                    buf.put_u8(4);
                    buf.put_u8(
                        u8::try_from(*args % 255).expect("a number modulo 255 should fit in a u8"),
                    );
                }

                buf.put_bytes(0, 2);
            }
            Item::Instr(instr) => {
                let resolved = instr.resolve(position, ramstart, resolver)?;
                resolved.serialize(buf);
            }
            Item::MysteryString(s) => {
                buf.put_u8(0xe0);
                buf.put(s.to_bytes());
                buf.put_u8(0);
            }
            Item::Utf32String(s) => {
                buf.put_u32(0xe2000000);
                buf.put(s.to_bytes());
                buf.put_u32(0);
            }
            Item::CompressedString(bytes) => {
                buf.put_u8(0xe1);
                buf.put(bytes.clone());
            }
            Item::Blob(blob) => {
                buf.put(blob.clone());
            }
            Item::Label(l) => match resolver.resolve(l)? {
                ResolvedAddr::Rom(addr) => {
                    buf.put_u32(addr);
                }
                ResolvedAddr::Ram(addr) => buf.put_u32(addr.checked_add(ramstart).overflow()?),
            },
        }
        Ok(())
    }
}

impl ZeroItem {
    pub(crate) fn len(&self) -> u32 {
        match self {
            ZeroItem::Space(x) => *x,
            ZeroItem::Align(_) => 0,
        }
    }

    pub(crate) fn align(&self) -> u32 {
        match self {
            ZeroItem::Space(_) => 1,
            ZeroItem::Align(a) => (*a).into(),
        }
    }
}

impl<L> ItemRef<L> {
    /// Applies the given mapping function to the label (if any) within the item reference.
    pub fn map<F, M>(self, mut f: F) -> ItemRef<M>
    where
        F: FnMut(L) -> M,
    {
        match self {
            ItemRef::Label(l, offset) => ItemRef::Label(f(l), offset),
            ItemRef::Resolved(x) => ItemRef::Resolved(x),
        }
    }

    pub(crate) fn resolve<R>(
        &self,
        ramstart: u32,
        resolver: &R,
    ) -> Result<ItemRef<Never>, AssemblerError<L>>
    where
        R: Resolver<Label = L>,
    {
        Ok(match self {
            ItemRef::Label(l, offset) => match resolver.resolve(l)? {
                ResolvedAddr::Rom(addr) => {
                    ItemRef::Resolved(addr.checked_add_signed(*offset).overflow()?)
                }
                ResolvedAddr::Ram(addr) => ItemRef::Resolved(
                    addr.checked_add(ramstart)
                        .overflow()?
                        .checked_add_signed(*offset)
                        .overflow()?,
                ),
            },
            ItemRef::Resolved(addr) => ItemRef::Resolved(*addr),
        })
    }
}

impl From<ItemRef<Never>> for u32 {
    fn from(value: ItemRef<Never>) -> Self {
        match value {
            ItemRef::Label(l, _) => l.into_any(),
            ItemRef::Resolved(addr) => addr,
        }
    }
}
