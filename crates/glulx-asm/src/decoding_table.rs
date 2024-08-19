// SPDX-License-Identifier: Apache-2.0 WITH LLVM-Exception
// Copyright 2024 Daniel Fox Franke.

//! Definition and impls for [`DecodeNode`] and related types.

use crate::cast::{CastSign, Overflow};
use crate::error::AssemblerError;
use crate::items::ItemRef;
use crate::resolver::{ResolvedAddr, Resolver};
use crate::strings::{MysteryString, Utf32String};
use bytes::BufMut;
use never::Never;

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
    IndirectRef(ItemRef<L>),
    /// Emit the string or call the function found by doubly dereferencing the
    /// given address.
    DoubleIndirectRef(ItemRef<L>),
    /// Call the function found by derefencing the given address, passing it the
    /// given arguments.
    IndirectRefWithArgs(ItemRef<L>, Vec<DecodeArg<L>>),
    /// Call the function found by doubly derefencing the given address, passing
    /// it the given arguments.
    DoubleIndirectRefWithArgs(ItemRef<L>, Vec<DecodeArg<L>>),
}

/// Argument to a function invoked from a decoding table.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DecodeArg<L> {
    /// Argument is the absolute address of the given label.
    Label(L),
    /// Argument is the given literal.
    Literal(i32),
}

impl<L> DecodeArg<L> {
    /// Applies the given mapping function to the label within the argument, if any.
    pub fn map<F, M>(self, mut f: F) -> DecodeArg<M>
    where
        F: FnMut(L) -> M,
    {
        match self {
            DecodeArg::Label(l) => DecodeArg::Label(f(l)),
            DecodeArg::Literal(x) => DecodeArg::Literal(x),
        }
    }

    pub(crate) fn resolve<R>(
        &self,
        ramstart: u32,
        resolver: &R,
    ) -> Result<DecodeArg<Never>, AssemblerError<L>>
    where
        R: Resolver<Label = L>,
    {
        Ok(match self {
            DecodeArg::Label(l) => match resolver.resolve(l)? {
                ResolvedAddr::Rom(addr) => DecodeArg::Literal(addr.cast_sign()),
                ResolvedAddr::Ram(addr) => {
                    DecodeArg::Literal(addr.checked_add(ramstart).overflow()?.cast_sign())
                }
            },
            DecodeArg::Literal(x) => DecodeArg::Literal(*x),
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

    pub(crate) fn count_nodes(&self) -> usize {
        match self {
            DecodeNode::Branch(left, right) => 1 + left.count_nodes() + right.count_nodes(),
            _ => 1,
        }
    }

    pub(crate) fn resolve<R>(
        &self,
        ramstart: u32,
        resolver: &R,
    ) -> Result<DecodeNode<Never>, AssemblerError<L>>
    where
        R: Resolver<Label = L>,
    {
        Ok(match self {
            DecodeNode::Branch(left, right) => DecodeNode::Branch(
                Box::new(left.resolve(ramstart, resolver)?),
                Box::new(right.resolve(ramstart, resolver)?),
            ),
            DecodeNode::StringTerminator => DecodeNode::StringTerminator,
            DecodeNode::MysteryChar(c) => DecodeNode::MysteryChar(*c),
            DecodeNode::MysteryString(s) => DecodeNode::MysteryString(s.clone()),
            DecodeNode::UnicodeChar(c) => DecodeNode::UnicodeChar(*c),
            DecodeNode::Utf32String(s) => DecodeNode::Utf32String(s.clone()),
            DecodeNode::IndirectRef(r) => DecodeNode::IndirectRef(r.resolve(ramstart, resolver)?),
            DecodeNode::DoubleIndirectRef(r) => {
                DecodeNode::DoubleIndirectRef(r.resolve(ramstart, resolver)?)
            }
            DecodeNode::IndirectRefWithArgs(r, args) => {
                u32::try_from(args.len()).overflow()?; // We can't serialize this. Serialization is infallible, so check here instead.
                let mut newargs = Vec::with_capacity(args.len());
                for arg in args {
                    newargs.push(arg.resolve(ramstart, resolver)?);
                }

                DecodeNode::IndirectRefWithArgs(r.resolve(ramstart, resolver)?, newargs)
            }
            DecodeNode::DoubleIndirectRefWithArgs(r, args) => {
                u32::try_from(args.len()).overflow()?;
                let mut newargs = Vec::with_capacity(args.len());
                for arg in args {
                    newargs.push(arg.resolve(ramstart, resolver)?);
                }

                DecodeNode::DoubleIndirectRefWithArgs(r.resolve(ramstart, resolver)?, newargs)
            }
        })
    }
}

impl From<DecodeArg<Never>> for i32 {
    fn from(value: DecodeArg<Never>) -> Self {
        match value {
            DecodeArg::Label(l) => l.into_any(),
            DecodeArg::Literal(x) => x,
        }
    }
}

impl DecodeNode<Never> {
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
            DecodeNode::Branch(left, right) => {
                let panic_msg = "decode tables with >= 2**32 nodes should have been rejected before serialization";
                let left_num = num.checked_add(1).expect(panic_msg);
                let right_num = left_num
                    .checked_add(left.count_nodes().try_into().expect(panic_msg))
                    .expect(panic_msg);
                buf.put_u8(0);
                left.serialize_inner(left_num, &mut *buf);
                right.serialize_inner(right_num, &mut *buf);
            }
            DecodeNode::StringTerminator => {
                buf.put_u8(1);
            }
            DecodeNode::MysteryChar(x) => {
                buf.put_u8(2);
                buf.put_u8(*x);
            }
            DecodeNode::MysteryString(s) => {
                buf.put_u8(3);
                buf.put(s.to_bytes());
                buf.put_u8(0);
            }
            DecodeNode::UnicodeChar(c) => {
                buf.put_u8(4);
                buf.put_u32((*c).into());
            }
            DecodeNode::Utf32String(s) => {
                buf.put_u8(5);
                buf.put(s.to_bytes());
                buf.put_u32(0);
            }
            DecodeNode::IndirectRef(r) => {
                buf.put_u8(8);
                buf.put_u32((*r).into());
            }
            DecodeNode::DoubleIndirectRef(r) => {
                buf.put_u8(9);
                buf.put_u32((*r).into());
            }
            DecodeNode::IndirectRefWithArgs(r, args) => {
                buf.put_u8(0xa);
                buf.put_u32((*r).into());
                buf.put_u32(
                    args.len().try_into().expect(
                        "refs with >= 2**32 args should have been rejected during resolution",
                    ),
                );
                for arg in args {
                    buf.put_i32((*arg).into())
                }
            }
            DecodeNode::DoubleIndirectRefWithArgs(r, args) => {
                buf.put_u8(0xb);
                buf.put_u32((*r).into());
                buf.put_u32(
                    args.len().try_into().expect(
                        "refs with >= 2**32 args should have been rejected during resolution",
                    ),
                );
                for arg in args {
                    buf.put_i32((*arg).into())
                }
            }
        }
    }
}

#[test]
fn foo() {
    use crate::Item;
    let x: Item<()> = Item::Align(1.try_into().unwrap());
    x.map(&mut |_| ());
}
