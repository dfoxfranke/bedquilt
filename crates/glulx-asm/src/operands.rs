// SPDX-License-Identifier: Apache-2.0 WITH LLVM-Exception
// Copyright 2024 Daniel Fox Franke.

//! Types related to instruction operands.

use bytes::BufMut;

use crate::{
    cast::CastSign,
    error::AssemblerError,
    resolver::{ResolvedAddr, Resolver},
    LabelRef,
};

/// An operand indicating where to get a value from.
///
/// For variants that accept a label+offset, it is unsupported to provide a
/// label in RAM with an offset that points backward into ROM. This will return
/// an overflow error when you try to assemble it.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum LoadOperand<L> {
    /// Pop the value from the stack.
    Pop,
    /// Use the immediate value of the operand.
    Imm(i32),
    /// Load the value from the stack at the given offset from the frame
    /// pointer.
    FrameAddr(u32),
    /// Use the address corresponding to the given label+offset and right-shift
    /// as an immediate value.
    ///
    /// Generating an operand with a right-shift of 1 or 2 is useful with the
    /// array load/store instructions, allowing an unaligned access to an
    /// aligned array, as opposed to the usual pattern of an aligned access to
    /// an unaligned array. The shift is computed *after* the offset, *i.e.*,
    /// the offset is still given in bytes. Shifting a label by more than its
    /// alignment will produce an error at assembly time.
    ImmLabel(LabelRef<L>, u8),
    /// Load the value from the address at the given label+offset.
    DerefLabel(LabelRef<L>),
    /// Compute an offset in order for a branch instruction to jump to the given
    /// label.
    ///
    /// When this label is resolved, the computed offset will be relative to the
    /// end of the *operand*. Jumps are computed relative to the end of the
    /// *instruction*. Fortunately, these are one-in-the-same, because every
    /// operand that Glulx interprets as an offset is the last operand of the
    /// instruction in which it occurs. The assembler won't stop you from using
    /// this variant in other locations. If you do, you'll get a nonsensical
    /// result, but you were already doing something nonsensical so GIGO.
    Branch(L),
}

/// An operand indicating where to put a value.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum StoreOperand<L> {
    /// Push the value to the stack.
    Push,
    /// Discard the value.
    Discard,
    /// Store the value to the stack address at the given offset from the frame
    /// pointer.
    FrameAddr(u32),
    /// Store the value to the address given by the label+offset.
    DerefLabel(LabelRef<L>),
}

/// An encoded operand ready to be serialized.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) enum RawOperand {
    /// 0: Load zero, or discard store
    Null,
    /// 1: Constant, -80 to 7F
    Imm8(i8),
    /// 2: Constant, -8000 to 7FFFF
    Imm16(i16),
    /// 3: Constant, any value
    Imm32(i32),
    /// 5: Contents of address 00 to FF
    Addr8(u8),
    /// 6: Contents of address 0000 to FFFF
    Addr16(u16),
    /// 7: Contents of any address
    Addr32(u32),
    /// 8: Value pushed/popped off stack
    Stack,
    /// 9: Call frame local at address 00 to FF
    Frame8(u8),
    /// A: Call frame local at address 0000 to FFFF
    Frame16(u16),
    /// B: Call frame local at any address
    Frame32(u32),
    /// D: Contents of RAM address 00 to FF
    Ram8(u8),
    /// E: Contents of RAM address 0000 to FFFF
    Ram16(u16),
    /// F: Contents of RAM, any address
    Ram32(u32),
}

impl<L> LoadOperand<L> {
    /// Applies the given mapping function to the label (if any) within the operand.
    pub fn map<F, M>(self, mut f: F) -> LoadOperand<M>
    where
        F: FnMut(L) -> M,
    {
        match self {
            LoadOperand::Pop => LoadOperand::Pop,
            LoadOperand::Imm(x) => LoadOperand::Imm(x),
            LoadOperand::FrameAddr(p) => LoadOperand::FrameAddr(p),
            LoadOperand::ImmLabel(l, shift) => LoadOperand::ImmLabel(l.map(f), shift),
            LoadOperand::DerefLabel(l) => LoadOperand::DerefLabel(l.map(f)),
            LoadOperand::Branch(l) => LoadOperand::Branch(f(l)),
        }
    }
}

impl<L> LoadOperand<L>
where
    L: Clone,
{
    /// Resolve labels in the operand, provided that the operand occurs at the
    /// given position and RAM begins at the given address.
    pub(crate) fn resolve<R>(
        &self,
        position: u32,
        resolver: &R,
    ) -> Result<RawOperand, AssemblerError<L>>
    where
        R: Resolver<Label = L>,
    {
        Ok(match self {
            LoadOperand::Pop => RawOperand::Stack,
            LoadOperand::Imm(x) => {
                if *x == 0 {
                    RawOperand::Null
                } else if let Ok(x) = i8::try_from(*x) {
                    RawOperand::Imm8(x)
                } else if let Ok(x) = i16::try_from(*x) {
                    RawOperand::Imm16(x)
                } else {
                    RawOperand::Imm32(*x)
                }
            }
            LoadOperand::FrameAddr(x) => {
                if let Ok(x) = u8::try_from(*x) {
                    RawOperand::Frame8(x)
                } else if let Ok(x) = u16::try_from(*x) {
                    RawOperand::Frame16(x)
                } else {
                    RawOperand::Frame32(*x)
                }
            }
            LoadOperand::ImmLabel(l, shift) => {
                let unshifted_addr = l.resolve_absolute(resolver)?;
                if unshifted_addr.trailing_zeros() < (*shift).into() {
                    return Err(AssemblerError::InsufficientAlignment {
                        label: l.0.clone(),
                        offset: l.1,
                        shift: *shift,
                    });
                }

                let addr = (unshifted_addr >> *shift).cast_sign();

                if addr == 0 {
                    RawOperand::Null
                } else if let Ok(x) = i8::try_from(addr) {
                    RawOperand::Imm8(x)
                } else if let Ok(x) = i16::try_from(addr) {
                    RawOperand::Imm16(x)
                } else {
                    RawOperand::Imm32(addr)
                }
            }
            LoadOperand::DerefLabel(l) => match l.resolve(resolver)? {
                ResolvedAddr::Rom(addr) => {
                    if let Ok(x) = u8::try_from(addr) {
                        RawOperand::Addr8(x)
                    } else if let Ok(x) = u16::try_from(addr) {
                        RawOperand::Addr16(x)
                    } else {
                        RawOperand::Addr32(addr)
                    }
                }
                ResolvedAddr::Ram(ramaddr) => {
                    // An offset in RAM which points backward into ROM is
                    // intentionally unsupported here.
                    if let Ok(x) = u8::try_from(ramaddr) {
                        RawOperand::Ram8(x)
                    } else if let Ok(x) = u16::try_from(ramaddr) {
                        RawOperand::Ram16(x)
                    } else {
                        RawOperand::Ram32(ramaddr)
                    }
                }
            },
            LoadOperand::Branch(l) => {
                let target = resolver.resolve_absolute(l)?;

                // We have to be careful here not to shrink an operand in such a
                // way as it has to grow again because shrinking it increased
                // the offset. Also not to accidentally generate 0 or 1 as an
                // offset, which are interpreted specially by Glulx. So, first
                // pick an operand size starting with 1, compute the resulting
                // offset based on that size, and see if it would fit and not be
                // 0/1.  If not, move on to the next larger size.

                let null_offset = (target.cast_sign())
                    .wrapping_sub(position.cast_sign())
                    .wrapping_add(2);

                let i8_offset = null_offset.wrapping_sub(1);
                if let Ok(x) = i8::try_from(i8_offset) {
                    if x != 0 && x != 1 {
                        return Ok(RawOperand::Imm8(x));
                    }
                }

                let i16_offset = null_offset.wrapping_sub(2);
                if let Ok(x) = i16::try_from(i16_offset) {
                    if x != 0 && x != 1 {
                        return Ok(RawOperand::Imm16(x));
                    }
                }

                let i32_offset = null_offset.wrapping_sub(4);
                // Shouldn't be possible with a 4-byte operand because we'd be
                // jumping into the middle of the operand.
                assert!(i32_offset != 0 && i32_offset != 1);
                return Ok(RawOperand::Imm32(i32_offset));
            }
        })
    }

    /// Returns an upper bound on how long this operand can end up being,
    /// regardless of where it's placed.
    pub(crate) fn worst_len(&self) -> usize {
        match self {
            LoadOperand::Pop => 0,
            LoadOperand::Imm(x) => {
                if *x == 0 {
                    0
                } else if i8::try_from(*x).is_ok() {
                    1
                } else if i16::try_from(*x).is_ok() {
                    2
                } else {
                    4
                }
            }
            LoadOperand::FrameAddr(x) => {
                if u8::try_from(*x).is_ok() {
                    1
                } else if u16::try_from(*x).is_ok() {
                    2
                } else {
                    4
                }
            }
            LoadOperand::ImmLabel(_, _) => 4,
            LoadOperand::DerefLabel(_) => 4,
            LoadOperand::Branch(_) => 4,
        }
    }
}

impl<L> StoreOperand<L> {
    /// Applies the given mapping function to the label (if any) within the operand.
    pub fn map<F, M>(self, f: F) -> StoreOperand<M>
    where
        F: FnMut(L) -> M,
    {
        match self {
            StoreOperand::Push => StoreOperand::Push,
            StoreOperand::Discard => StoreOperand::Discard,
            StoreOperand::FrameAddr(x) => StoreOperand::FrameAddr(x),
            StoreOperand::DerefLabel(l) => StoreOperand::DerefLabel(l.map(f)),
        }
    }
}

impl<L> StoreOperand<L>
where
    L: Clone,
{
    /// Resolve labels in the operand, provided that the operand occurs at the
    /// given position and RAM begins at the given address. These arguments are
    /// in fact ignored, but we need this type signature to be the same as the
    /// one for [`LoadOperand::resolve`] in order for our macros to work.
    pub(crate) fn resolve<R>(
        &self,
        _position: u32,
        resolver: &R,
    ) -> Result<RawOperand, AssemblerError<L>>
    where
        R: Resolver<Label = L>,
    {
        Ok(match self {
            StoreOperand::Push => RawOperand::Stack,
            StoreOperand::Discard => RawOperand::Null,
            StoreOperand::FrameAddr(x) => {
                if let Ok(x) = u8::try_from(*x) {
                    RawOperand::Frame8(x)
                } else if let Ok(x) = u16::try_from(*x) {
                    RawOperand::Frame16(x)
                } else {
                    RawOperand::Frame32(*x)
                }
            }
            StoreOperand::DerefLabel(l) => match l.resolve(resolver)? {
                ResolvedAddr::Rom(addr) => {
                    if let Ok(x) = u8::try_from(addr) {
                        RawOperand::Addr8(x)
                    } else if let Ok(x) = u16::try_from(addr) {
                        RawOperand::Addr16(x)
                    } else {
                        RawOperand::Addr32(addr)
                    }
                }
                ResolvedAddr::Ram(addr) => {
                    if let Ok(x) = u8::try_from(addr) {
                        RawOperand::Ram8(x)
                    } else if let Ok(x) = u16::try_from(addr) {
                        RawOperand::Ram16(x)
                    } else {
                        RawOperand::Ram32(addr)
                    }
                }
            },
        })
    }

    /// Returns an upper bound on how long this operand can end up being,
    /// regardless of where it's placed.
    pub(crate) fn worst_len(&self) -> usize {
        match self {
            StoreOperand::Push => 0,
            StoreOperand::Discard => 0,
            StoreOperand::FrameAddr(x) => {
                if u8::try_from(*x).is_ok() {
                    1
                } else if u16::try_from(*x).is_ok() {
                    2
                } else {
                    4
                }
            }
            StoreOperand::DerefLabel(_) => 4,
        }
    }
}

impl RawOperand {
    /// Returns the encoded length of the operand.
    pub(crate) fn len(&self) -> usize {
        match self {
            RawOperand::Null => 0,
            RawOperand::Imm8(_) => 1,
            RawOperand::Imm16(_) => 2,
            RawOperand::Imm32(_) => 4,
            RawOperand::Addr8(_) => 1,
            RawOperand::Addr16(_) => 2,
            RawOperand::Addr32(_) => 4,
            RawOperand::Stack => 0,
            RawOperand::Frame8(_) => 1,
            RawOperand::Frame16(_) => 2,
            RawOperand::Frame32(_) => 4,
            RawOperand::Ram8(_) => 1,
            RawOperand::Ram16(_) => 2,
            RawOperand::Ram32(_) => 4,
        }
    }

    /// Returns the addressing-mode nibble.
    pub(crate) fn mode(&self) -> u8 {
        match self {
            RawOperand::Null => 0,
            RawOperand::Imm8(_) => 1,
            RawOperand::Imm16(_) => 2,
            RawOperand::Imm32(_) => 3,
            RawOperand::Addr8(_) => 5,
            RawOperand::Addr16(_) => 6,
            RawOperand::Addr32(_) => 7,
            RawOperand::Stack => 8,
            RawOperand::Frame8(_) => 9,
            RawOperand::Frame16(_) => 0xa,
            RawOperand::Frame32(_) => 0xb,
            RawOperand::Ram8(_) => 0xd,
            RawOperand::Ram16(_) => 0xe,
            RawOperand::Ram32(_) => 0xf,
        }
    }

    /// Serializes the operand.
    pub(crate) fn serialize<B: BufMut>(&self, mut buf: B) {
        match self {
            RawOperand::Null => {}
            RawOperand::Imm8(x) => buf.put_i8(*x),
            RawOperand::Imm16(x) => buf.put_i16(*x),
            RawOperand::Imm32(x) => buf.put_i32(*x),
            RawOperand::Addr8(x) => buf.put_u8(*x),
            RawOperand::Addr16(x) => buf.put_u16(*x),
            RawOperand::Addr32(x) => buf.put_u32(*x),
            RawOperand::Stack => {}
            RawOperand::Frame8(x) => buf.put_u8(*x),
            RawOperand::Frame16(x) => buf.put_u16(*x),
            RawOperand::Frame32(x) => buf.put_u32(*x),
            RawOperand::Ram8(x) => buf.put_u8(*x),
            RawOperand::Ram16(x) => buf.put_u16(*x),
            RawOperand::Ram32(x) => buf.put_u32(*x),
        }
    }
}

/// Creates an immediate operand out of the given `f32`.
#[inline]
pub fn f32_to_imm<L>(x: f32) -> LoadOperand<L> {
    LoadOperand::Imm(x.to_bits().cast_sign())
}

/// Creates a pair of immediate operands out of the given `f64`, returned as (hi,lo).
#[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
#[inline]
pub fn f64_to_imm<L>(x: f64) -> (LoadOperand<L>, LoadOperand<L>) {
    let n = x.to_bits();
    let high = (n >> 32) as u32;
    let low = n as u32;
    (
        LoadOperand::Imm(high.cast_sign()),
        LoadOperand::Imm(low.cast_sign()),
    )
}

/// Computes the FramePtr-relative address for th `m`th of `n` locals.
fn local_pos(m: u32, n: u32) -> u32 {
    8 // Frame Len and Locals Pos 
    + (2 * n.div_ceil(255) + 2) // Format of Locals
    .next_multiple_of(4) // Padding
    + 4 * m // Offset of local
}

/// Returns a load operand which addresses the (zero-indexed) `m`th of `n` local
/// variables in a call frame.
pub fn load_local<L>(m: u32, n: u32) -> LoadOperand<L> {
    LoadOperand::FrameAddr(local_pos(m, n))
}

/// Returns a store operand which addresses the (zero-indexed) `m`th of `n`
/// local variables in a call frame.
pub fn store_local<L>(m: u32, n: u32) -> StoreOperand<L> {
    StoreOperand::FrameAddr(local_pos(m, n))
}
