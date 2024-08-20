// SPDX-License-Identifier: Apache-2.0 WITH LLVM-Exception
// Copyright 2024 Daniel Fox Franke.

//! Definition and impls for [`DecodeNode`] and related types.

use crate::cast::{CastSign, Overflow};
use crate::error::AssemblerError;
use crate::items::LabelRef;
use crate::resolver::Resolver;
use crate::strings::{MysteryString, Utf32String};
use bytes::BufMut;

/// A node in a decoding table.
#[derive(Debug, Clone)]
pub enum DecodeNode<L> {
    /// Branch left on 0, right on 1.
    Branch(Box<DecodeNode<L>>, Box<DecodeNode<L>>),
    /// Terminate decoding.
    StringTerminator,
    /// Emit a character whose encoding is unspecified and determined by the IO
    /// system (but probably Latin-1).
    MysteryChar(u8),
    /// Emit a string whose encoding is unspecified and determined by the IO
    /// system (but probably Latin-1).
    MysteryString(MysteryString),
    /// Emit a Unicode character.
    UnicodeChar(char),
    /// Emit a Unicode string.
    Utf32String(Utf32String),
    /// Emit the string or call the function found by dereferencing the given
    /// address.
    IndirectRef(LabelRef<L>),
    /// Emit the string or call the function found by doubly dereferencing the
    /// given address.
    DoubleIndirectRef(LabelRef<L>),
    /// Call the function found by derefencing the given address, passing it the
    /// given arguments.
    IndirectRefWithArgs(LabelRef<L>, Vec<DecodeArg<L>>),
    /// Call the function found by doubly derefencing the given address, passing
    /// it the given arguments.
    DoubleIndirectRefWithArgs(LabelRef<L>, Vec<DecodeArg<L>>),
}

pub(crate) enum ResolvedDecodeNode {
    Branch(Box<ResolvedDecodeNode>, Box<ResolvedDecodeNode>),
    StringTerminator,
    MysteryChar(u8),
    MysteryString(MysteryString),
    UnicodeChar(char),
    Utf32String(Utf32String),
    IndirectRef(u32),
    DoubleIndirectRef(u32),
    IndirectRefWithArgs(u32, Vec<i32>),
    DoubleIndirectRefWithArgs(u32, Vec<i32>),
}

/// Argument to a function invoked from a decoding table.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DecodeArg<L> {
    /// Argument is the absolute address of the given label+offset.
    Label(LabelRef<L>),
    /// Argument is the given literal.
    Literal(i32),
}

impl<L> DecodeArg<L> {
    /// Applies the given mapping function to the label within the argument, if any.
    pub fn map<F, M>(self, f: F) -> DecodeArg<M>
    where
        F: FnMut(L) -> M,
    {
        match self {
            DecodeArg::Label(l) => DecodeArg::Label(l.map(f)),
            DecodeArg::Literal(x) => DecodeArg::Literal(x),
        }
    }

    pub(crate) fn resolve<R>(&self, ramstart: u32, resolver: &R) -> Result<i32, AssemblerError<L>>
    where
        R: Resolver<Label = L>,
    {
        Ok(match self {
            DecodeArg::Label(l) => l.resolve_absolute(ramstart, resolver)?.cast_sign(),
            DecodeArg::Literal(x) => *x,
        })
    }
}

impl<L> DecodeNode<L> {
    /// Applies the given mapping function to all labels within the node.
    pub fn map<F, M>(self, mut f: F) -> DecodeNode<M>
    where
        F: FnMut(L) -> M,
    {
        self.map_inner(&mut f)
    }

    // This method is recursive. This hack of taking a &mut F instead of an F is
    // necessary in order have a type we can reborrow and avoid infinite
    // recursion during trait resolution.
    fn map_inner<F, M>(self, f: &mut F) -> DecodeNode<M>
    where
        F: FnMut(L) -> M,
    {
        match self {
            DecodeNode::Branch(left, right) => DecodeNode::Branch(
                Box::new(left.map_inner(&mut *f)),
                Box::new(right.map_inner(&mut *f)),
            ),
            DecodeNode::StringTerminator => DecodeNode::StringTerminator,
            DecodeNode::MysteryChar(x) => DecodeNode::MysteryChar(x),
            DecodeNode::MysteryString(x) => DecodeNode::MysteryString(x),
            DecodeNode::UnicodeChar(x) => DecodeNode::UnicodeChar(x),
            DecodeNode::Utf32String(x) => DecodeNode::Utf32String(x),
            DecodeNode::IndirectRef(r) => DecodeNode::IndirectRef(r.map(f)),
            DecodeNode::DoubleIndirectRef(r) => DecodeNode::DoubleIndirectRef(r.map(f)),
            DecodeNode::IndirectRefWithArgs(r, args) => DecodeNode::IndirectRefWithArgs(
                r.map(&mut *f),
                args.into_iter().map(|arg| arg.map(&mut *f)).collect(),
            ),
            DecodeNode::DoubleIndirectRefWithArgs(r, args) => {
                DecodeNode::DoubleIndirectRefWithArgs(
                    r.map(&mut *f),
                    args.into_iter().map(|arg| arg.map(&mut *f)).collect(),
                )
            }
        }
    }

    pub(crate) fn len(&self) -> usize {
        match self {
            DecodeNode::Branch(left, right) => left.len() + right.len() + 9,
            DecodeNode::StringTerminator => 1,
            DecodeNode::MysteryChar(_) => 2,
            DecodeNode::MysteryString(s) => s.len() + 2,
            DecodeNode::UnicodeChar(_) => 5,
            DecodeNode::Utf32String(s) => s.byte_len() + 5,
            DecodeNode::IndirectRef(_) => 5,
            DecodeNode::DoubleIndirectRef(_) => 5,
            DecodeNode::IndirectRefWithArgs(_, args) => 4 * args.len() + 9,
            DecodeNode::DoubleIndirectRefWithArgs(_, args) => 4 * args.len() + 9,
        }
    }

    pub(crate) fn resolve<R>(
        &self,
        ramstart: u32,
        resolver: &R,
    ) -> Result<ResolvedDecodeNode, AssemblerError<L>>
    where
        R: Resolver<Label = L>,
    {
        Ok(match self {
            DecodeNode::Branch(left, right) => ResolvedDecodeNode::Branch(
                Box::new(left.resolve(ramstart, resolver)?),
                Box::new(right.resolve(ramstart, resolver)?),
            ),
            DecodeNode::StringTerminator => ResolvedDecodeNode::StringTerminator,
            DecodeNode::MysteryChar(c) => ResolvedDecodeNode::MysteryChar(*c),
            DecodeNode::MysteryString(s) => ResolvedDecodeNode::MysteryString(s.clone()),
            DecodeNode::UnicodeChar(c) => ResolvedDecodeNode::UnicodeChar(*c),
            DecodeNode::Utf32String(s) => ResolvedDecodeNode::Utf32String(s.clone()),
            DecodeNode::IndirectRef(r) => {
                ResolvedDecodeNode::IndirectRef(r.resolve_absolute(ramstart, resolver)?)
            }
            DecodeNode::DoubleIndirectRef(r) => {
                ResolvedDecodeNode::DoubleIndirectRef(r.resolve_absolute(ramstart, resolver)?)
            }
            DecodeNode::IndirectRefWithArgs(r, args) => {
                u32::try_from(args.len()).overflow()?; // We can't serialize this. Serialization is infallible, so check here instead.
                let mut newargs = Vec::with_capacity(args.len());
                for arg in args {
                    newargs.push(arg.resolve(ramstart, resolver)?);
                }

                ResolvedDecodeNode::IndirectRefWithArgs(
                    r.resolve_absolute(ramstart, resolver)?,
                    newargs,
                )
            }
            DecodeNode::DoubleIndirectRefWithArgs(r, args) => {
                u32::try_from(args.len()).overflow()?;
                let mut newargs = Vec::with_capacity(args.len());
                for arg in args {
                    newargs.push(arg.resolve(ramstart, resolver)?);
                }

                ResolvedDecodeNode::DoubleIndirectRefWithArgs(
                    r.resolve_absolute(ramstart, resolver)?,
                    newargs,
                )
            }
        })
    }
}

impl ResolvedDecodeNode {
    pub(crate) fn count_nodes(&self) -> usize {
        match self {
            ResolvedDecodeNode::Branch(left, right) => 1 + left.count_nodes() + right.count_nodes(),
            _ => 1,
        }
    }

    pub(crate) fn len(&self) -> usize {
        match self {
            ResolvedDecodeNode::Branch(left, right) => left.len() + right.len() + 9,
            ResolvedDecodeNode::StringTerminator => 1,
            ResolvedDecodeNode::MysteryChar(_) => 2,
            ResolvedDecodeNode::MysteryString(s) => s.len() + 2,
            ResolvedDecodeNode::UnicodeChar(_) => 5,
            ResolvedDecodeNode::Utf32String(s) => s.byte_len() + 5,
            ResolvedDecodeNode::IndirectRef(_) => 5,
            ResolvedDecodeNode::DoubleIndirectRef(_) => 5,
            ResolvedDecodeNode::IndirectRefWithArgs(_, args) => 4 * args.len() + 9,
            ResolvedDecodeNode::DoubleIndirectRefWithArgs(_, args) => 4 * args.len() + 9,
        }
    }

    pub(crate) fn serialize<B>(&self, num: u32, mut buf: B)
    where
        B: BufMut,
    {
        self.serialize_inner(num, &mut buf)
    }

    fn serialize_inner<B>(&self, num: u32, buf: &mut B)
    where
        B: BufMut,
    {
        match self {
            ResolvedDecodeNode::Branch(left, right) => {
                let panic_msg = "decode tables with >= 2**32 nodes should have been rejected before serialization";
                let left_num = num.checked_add(1).expect(panic_msg);
                let right_num = left_num
                    .checked_add(left.count_nodes().try_into().expect(panic_msg))
                    .expect(panic_msg);
                buf.put_u8(0);
                left.serialize_inner(left_num, &mut *buf);
                right.serialize_inner(right_num, &mut *buf);
            }
            ResolvedDecodeNode::StringTerminator => {
                buf.put_u8(1);
            }
            ResolvedDecodeNode::MysteryChar(x) => {
                buf.put_u8(2);
                buf.put_u8(*x);
            }
            ResolvedDecodeNode::MysteryString(s) => {
                buf.put_u8(3);
                buf.put(s.to_bytes());
                buf.put_u8(0);
            }
            ResolvedDecodeNode::UnicodeChar(c) => {
                buf.put_u8(4);
                buf.put_u32((*c).into());
            }
            ResolvedDecodeNode::Utf32String(s) => {
                buf.put_u8(5);
                buf.put(s.to_bytes());
                buf.put_u32(0);
            }
            ResolvedDecodeNode::IndirectRef(r) => {
                buf.put_u8(8);
                buf.put_u32(*r);
            }
            ResolvedDecodeNode::DoubleIndirectRef(r) => {
                buf.put_u8(9);
                buf.put_u32(*r);
            }
            ResolvedDecodeNode::IndirectRefWithArgs(r, args) => {
                buf.put_u8(0xa);
                buf.put_u32(*r);
                buf.put_u32(
                    args.len().try_into().expect(
                        "refs with >= 2**32 args should have been rejected during resolution",
                    ),
                );
                for arg in args {
                    buf.put_i32(*arg)
                }
            }
            ResolvedDecodeNode::DoubleIndirectRefWithArgs(r, args) => {
                buf.put_u8(0xb);
                buf.put_u32(*r);
                buf.put_u32(
                    args.len().try_into().expect(
                        "refs with >= 2**32 args should have been rejected during resolution",
                    ),
                );
                for arg in args {
                    buf.put_i32(*arg)
                }
            }
        }
    }
}
