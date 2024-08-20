// SPDX-License-Identifier: Apache-2.0 WITH LLVM-Exception
// Copyright 2024 Daniel Fox Franke.

//! Implementations for [`Instr`].

use crate::error::AssemblerError;
use crate::instr_def::Instr;
use crate::operands::RawOperand;
use crate::resolver::Resolver;
use arrayvec::ArrayVec;
use bytes::BufMut;

/// The largest number of operands taken by any instruction.
///
/// `linearsearch` and `binarysearch` set the high-water mark.
pub(crate) const MAX_OPERANDS: usize = 8;

/// An encoded instruction ready to be serialized.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct RawInstr {
    /// The instruction's opcode.
    pub opcode: u32,
    /// List of operands to the instruction.
    pub operands: ArrayVec<RawOperand, MAX_OPERANDS>,
}

/// Return the number of arguments the macro was called with.
macro_rules! count {
    ($(,)*) => (0u32);
    ($current:expr $(,)*) => (1u32);
    ( $current:expr, $($rest:expr),* $(,)*) => (const { 1 + count!($($rest),*) });
}

/// Call arg.resolve(position, ramstart, resolver) for each argument after the
/// third, correctly updating position for each call. Position at the start of
/// the invocation should the end of the opcode.
macro_rules! resolve {
    ($position:expr, $ramstart:expr, $resolver:expr, $($x:expr),* $(,)*) => {
        {
            let _ramstart = $ramstart;
            #[allow(unused_mut)]
            let mut v = ArrayVec::<RawOperand, MAX_OPERANDS>::new();
            let n_args = count!($($x),*);

            let mut _position = $position.checked_add(n_args.div_ceil(2))
                .ok_or(AssemblerError::Overflow)?;

            $(
                let operand = $x.resolve(_position, _ramstart, $resolver)?;
                _position = _position.checked_add(u32::try_from(operand.len())
                    .or(Err(AssemblerError::Overflow))?)
                    .ok_or(AssemblerError::Overflow)?;
                v.push(operand);
            )*

            v
        }
    };
}

/// Call arg.worst_len() on each argument and return the sum of the results plus
/// the space occupied by the addressing-mode nibbles.
macro_rules! worst_len {
    ($($x:expr),* $(,)*) => {
        {
            let oplens = [$($x.worst_len()),*];
            let oplens_slice = oplens.as_slice();
            let modelen  = oplens_slice.len().div_ceil(2);
            let oplen_sum = oplens_slice.iter().copied().sum::<usize>();
            modelen + oplen_sum
        }
    };
}

impl<L> Instr<L> {
    /// Returrns the instruction's opcode.
    pub fn opcode(&self) -> u32 {
        match self {
            Instr::Nop => 0x00,
            Instr::Add(_, _, _) => 0x10,
            Instr::Sub(_, _, _) => 0x11,
            Instr::Mul(_, _, _) => 0x12,
            Instr::Div(_, _, _) => 0x13,
            Instr::Mod(_, _, _) => 0x14,
            Instr::Neg(_, _) => 0x15,
            Instr::Bitand(_, _, _) => 0x18,
            Instr::Bitor(_, _, _) => 0x19,
            Instr::Bitxor(_, _, _) => 0x1A,
            Instr::Bitnot(_, _) => 0x1B,
            Instr::Shiftl(_, _, _) => 0x1C,
            Instr::Sshiftr(_, _, _) => 0x1D,
            Instr::Ushiftr(_, _, _) => 0x1E,
            Instr::Jump(_) => 0x20,
            Instr::Jz(_, _) => 0x22,
            Instr::Jnz(_, _) => 0x23,
            Instr::Jeq(_, _, _) => 0x24,
            Instr::Jne(_, _, _) => 0x25,
            Instr::Jlt(_, _, _) => 0x26,
            Instr::Jge(_, _, _) => 0x27,
            Instr::Jgt(_, _, _) => 0x28,
            Instr::Jle(_, _, _) => 0x29,
            Instr::Jltu(_, _, _) => 0x2A,
            Instr::Jgeu(_, _, _) => 0x2B,
            Instr::Jgtu(_, _, _) => 0x2C,
            Instr::Jleu(_, _, _) => 0x2D,
            Instr::Call(_, _, _) => 0x30,
            Instr::Return(_) => 0x31,
            Instr::Catch(_, _) => 0x32,
            Instr::Throw(_, _) => 0x33,
            Instr::Tailcall(_, _) => 0x34,
            Instr::Copy(_, _) => 0x40,
            Instr::Copys(_, _) => 0x41,
            Instr::Copyb(_, _) => 0x42,
            Instr::Sexs(_, _) => 0x44,
            Instr::Sexb(_, _) => 0x45,
            Instr::Aload(_, _, _) => 0x48,
            Instr::Aloads(_, _, _) => 0x49,
            Instr::Aloadb(_, _, _) => 0x4A,
            Instr::Aloadbit(_, _, _) => 0x4B,
            Instr::Astore(_, _, _) => 0x4C,
            Instr::Astores(_, _, _) => 0x4D,
            Instr::Astoreb(_, _, _) => 0x4E,
            Instr::Astorebit(_, _, _) => 0x4F,
            Instr::Stkcount(_) => 0x50,
            Instr::Stkpeek(_, _) => 0x51,
            Instr::Stkswap => 0x52,
            Instr::Stkroll(_, _) => 0x53,
            Instr::Stkcopy(_) => 0x54,
            Instr::Streamchar(_) => 0x70,
            Instr::Streamnum(_) => 0x71,
            Instr::Streamstr(_) => 0x72,
            Instr::Streamunichar(_) => 0x73,
            Instr::Gestalt(_, _, _) => 0x100,
            Instr::Debugtrap(_) => 0x101,
            Instr::Getmemsize(_) => 0x102,
            Instr::Setmemsize(_, _) => 0x103,
            Instr::Jumpabs(_) => 0x104,
            Instr::Random(_, _) => 0x110,
            Instr::Setrandom(_) => 0x111,
            Instr::Quit => 0x120,
            Instr::Verify(_) => 0x121,
            Instr::Restart => 0x122,
            Instr::Save(_, _) => 0x123,
            Instr::Restore(_, _) => 0x124,
            Instr::Saveundo(_) => 0x125,
            Instr::Restoreundo(_) => 0x126,
            Instr::Protect(_, _) => 0x127,
            Instr::Hasundo(_) => 0x128,
            Instr::Discardundo => 0x129,
            Instr::Glk(_, _, _) => 0x130,
            Instr::Getstringtbl(_) => 0x140,
            Instr::Setstringtbl(_) => 0x141,
            Instr::Getiosys(_, _) => 0x148,
            Instr::Setiosys(_, _) => 0x149,
            Instr::Linearsearch(_, _, _, _, _, _, _, _) => 0x150,
            Instr::Binarysearch(_, _, _, _, _, _, _, _) => 0x151,
            Instr::Linkedsearch(_, _, _, _, _, _, _) => 0x152,
            Instr::Callf(_, _) => 0x160,
            Instr::Callfi(_, _, _) => 0x161,
            Instr::Callfii(_, _, _, _) => 0x162,
            Instr::Callfiii(_, _, _, _, _) => 0x163,
            Instr::Mzero(_, _) => 0x170,
            Instr::Mcopy(_, _, _) => 0x171,
            Instr::Malloc(_, _) => 0x178,
            Instr::Mfree(_) => 0x179,
            Instr::Accelfunc(_, _) => 0x180,
            Instr::Accelparam(_, _) => 0x181,
            Instr::Numtof(_, _) => 0x190,
            Instr::Ftonumz(_, _) => 0x191,
            Instr::Ftonumn(_, _) => 0x192,
            Instr::Ceil(_, _) => 0x198,
            Instr::Floor(_, _) => 0x199,
            Instr::Fadd(_, _, _) => 0x1A0,
            Instr::Fsub(_, _, _) => 0x1A1,
            Instr::Fmul(_, _, _) => 0x1A2,
            Instr::Fdiv(_, _, _) => 0x1A3,
            Instr::Fmod(_, _, _) => 0x1A4,
            Instr::Sqrt(_, _) => 0x1A8,
            Instr::Exp(_, _) => 0x1A9,
            Instr::Log(_, _) => 0x1AA,
            Instr::Pow(_, _, _) => 0x1AB,
            Instr::Sin(_, _) => 0x1B0,
            Instr::Cos(_, _) => 0x1B1,
            Instr::Tan(_, _) => 0x1B2,
            Instr::Asin(_, _) => 0x1B3,
            Instr::Acos(_, _) => 0x1B4,
            Instr::Atan(_, _) => 0x1B5,
            Instr::Atan2(_, _) => 0x1B6,
            Instr::Jfeq(_, _, _, _) => 0x1C0,
            Instr::Jfne(_, _, _, _) => 0x1C1,
            Instr::Jflt(_, _, _) => 0x1C2,
            Instr::Jfle(_, _, _) => 0x1C3,
            Instr::Jfgt(_, _, _) => 0x1C4,
            Instr::Jfge(_, _, _) => 0x1C5,
            Instr::Jisnan(_, _) => 0x1C8,
            Instr::Jisinf(_, _) => 0x1C9,
            Instr::Numtod(_, _, _) => 0x200,
            Instr::Dtonumz(_, _, _) => 0x201,
            Instr::Dtonumn(_, _, _) => 0x202,
            Instr::Ftod(_, _, _) => 0x203,
            Instr::Dtof(_, _, _) => 0x204,
            Instr::Dceil(_, _, _, _) => 0x208,
            Instr::Dfloor(_, _, _, _) => 0x209,
            Instr::Dadd(_, _, _, _, _, _) => 0x210,
            Instr::Dsub(_, _, _, _, _, _) => 0x211,
            Instr::Dmul(_, _, _, _, _, _) => 0x212,
            Instr::Ddiv(_, _, _, _, _, _) => 0x213,
            Instr::Dmodr(_, _, _, _, _, _) => 0x214,
            Instr::Dmodq(_, _, _, _, _, _) => 0x215,
            Instr::Dsqrt(_, _, _, _) => 0x218,
            Instr::Dexp(_, _, _, _) => 0x219,
            Instr::Dlog(_, _, _, _) => 0x21A,
            Instr::Dpow(_, _, _, _, _, _) => 0x21B,
            Instr::Dsin(_, _, _, _) => 0x220,
            Instr::Dcos(_, _, _, _) => 0x221,
            Instr::Dtan(_, _, _, _) => 0x222,
            Instr::Dasin(_, _, _, _) => 0x223,
            Instr::Dacos(_, _, _, _) => 0x224,
            Instr::Datan(_, _, _, _) => 0x225,
            Instr::Datan2(_, _, _, _, _, _) => 0x226,
            Instr::Jdeq(_, _, _, _, _, _, _) => 0x230,
            Instr::Jdne(_, _, _, _, _, _, _) => 0x231,
            Instr::Jdlt(_, _, _, _, _) => 0x232,
            Instr::Jdle(_, _, _, _, _) => 0x233,
            Instr::Jdgt(_, _, _, _, _) => 0x234,
            Instr::Jdge(_, _, _, _, _) => 0x235,
            Instr::Jdisnan(_, _, _) => 0x238,
            Instr::Jdisinf(_, _, _) => 0x239,
        }
    }

    /// Applies the given mapping function to all labels within the instruction.
    pub fn map<F, M>(self, mut f: F) -> Instr<M>
    where
        F: FnMut(L) -> M,
    {
        match self {
            Instr::Nop => Instr::Nop,
            Instr::Add(l1, l2, s1) => Instr::Add(l1.map(&mut f), l2.map(&mut f), s1.map(&mut f)),
            Instr::Sub(l1, l2, s1) => Instr::Sub(l1.map(&mut f), l2.map(&mut f), s1.map(&mut f)),
            Instr::Mul(l1, l2, s1) => Instr::Mul(l1.map(&mut f), l2.map(&mut f), s1.map(&mut f)),
            Instr::Div(l1, l2, s1) => Instr::Div(l1.map(&mut f), l2.map(&mut f), s1.map(&mut f)),
            Instr::Mod(l1, l2, s1) => Instr::Mod(l1.map(&mut f), l2.map(&mut f), s1.map(&mut f)),
            Instr::Neg(l1, s1) => Instr::Neg(l1.map(&mut f), s1.map(&mut f)),
            Instr::Bitand(l1, l2, s1) => {
                Instr::Bitand(l1.map(&mut f), l2.map(&mut f), s1.map(&mut f))
            }
            Instr::Bitor(l1, l2, s1) => {
                Instr::Bitor(l1.map(&mut f), l2.map(&mut f), s1.map(&mut f))
            }
            Instr::Bitxor(l1, l2, s1) => {
                Instr::Bitxor(l1.map(&mut f), l2.map(&mut f), s1.map(&mut f))
            }
            Instr::Bitnot(l1, s1) => Instr::Bitnot(l1.map(&mut f), s1.map(&mut f)),
            Instr::Shiftl(l1, l2, s1) => {
                Instr::Shiftl(l1.map(&mut f), l2.map(&mut f), s1.map(&mut f))
            }
            Instr::Ushiftr(l1, l2, s1) => {
                Instr::Ushiftr(l1.map(&mut f), l2.map(&mut f), s1.map(&mut f))
            }
            Instr::Sshiftr(l1, l2, s1) => {
                Instr::Sshiftr(l1.map(&mut f), l2.map(&mut f), s1.map(&mut f))
            }
            Instr::Jump(l1) => Instr::Jump(l1.map(&mut f)),
            Instr::Jz(l1, l2) => Instr::Jz(l1.map(&mut f), l2.map(&mut f)),
            Instr::Jnz(l1, l2) => Instr::Jnz(l1.map(&mut f), l2.map(&mut f)),
            Instr::Jeq(l1, l2, l3) => Instr::Jeq(l1.map(&mut f), l2.map(&mut f), l3.map(&mut f)),
            Instr::Jne(l1, l2, l3) => Instr::Jne(l1.map(&mut f), l2.map(&mut f), l3.map(&mut f)),
            Instr::Jlt(l1, l2, l3) => Instr::Jlt(l1.map(&mut f), l2.map(&mut f), l3.map(&mut f)),
            Instr::Jle(l1, l2, l3) => Instr::Jle(l1.map(&mut f), l2.map(&mut f), l3.map(&mut f)),
            Instr::Jgt(l1, l2, l3) => Instr::Jgt(l1.map(&mut f), l2.map(&mut f), l3.map(&mut f)),
            Instr::Jge(l1, l2, l3) => Instr::Jge(l1.map(&mut f), l2.map(&mut f), l3.map(&mut f)),
            Instr::Jltu(l1, l2, l3) => Instr::Jltu(l1.map(&mut f), l2.map(&mut f), l3.map(&mut f)),
            Instr::Jleu(l1, l2, l3) => Instr::Jleu(l1.map(&mut f), l2.map(&mut f), l3.map(&mut f)),
            Instr::Jgtu(l1, l2, l3) => Instr::Jgtu(l1.map(&mut f), l2.map(&mut f), l3.map(&mut f)),
            Instr::Jgeu(l1, l2, l3) => Instr::Jgeu(l1.map(&mut f), l2.map(&mut f), l3.map(&mut f)),
            Instr::Jumpabs(l1) => Instr::Jumpabs(l1.map(&mut f)),
            Instr::Copy(l1, s1) => Instr::Copy(l1.map(&mut f), s1.map(&mut f)),
            Instr::Copys(l1, s1) => Instr::Copys(l1.map(&mut f), s1.map(&mut f)),
            Instr::Copyb(l1, s1) => Instr::Copyb(l1.map(&mut f), s1.map(&mut f)),
            Instr::Sexs(l1, s1) => Instr::Sexs(l1.map(&mut f), s1.map(&mut f)),
            Instr::Sexb(l1, s1) => Instr::Sexb(l1.map(&mut f), s1.map(&mut f)),
            Instr::Astore(l1, l2, l3) => {
                Instr::Astore(l1.map(&mut f), l2.map(&mut f), l3.map(&mut f))
            }
            Instr::Aload(l1, l2, s1) => {
                Instr::Aload(l1.map(&mut f), l2.map(&mut f), s1.map(&mut f))
            }
            Instr::Astores(l1, l2, l3) => {
                Instr::Astores(l1.map(&mut f), l2.map(&mut f), l3.map(&mut f))
            }
            Instr::Aloads(l1, l2, s1) => {
                Instr::Aloads(l1.map(&mut f), l2.map(&mut f), s1.map(&mut f))
            }
            Instr::Astoreb(l1, l2, l3) => {
                Instr::Astoreb(l1.map(&mut f), l2.map(&mut f), l3.map(&mut f))
            }
            Instr::Aloadb(l1, l2, s1) => {
                Instr::Aloadb(l1.map(&mut f), l2.map(&mut f), s1.map(&mut f))
            }
            Instr::Astorebit(l1, l2, l3) => {
                Instr::Astorebit(l1.map(&mut f), l2.map(&mut f), l3.map(&mut f))
            }
            Instr::Aloadbit(l1, l2, s1) => {
                Instr::Aloadbit(l1.map(&mut f), l2.map(&mut f), s1.map(&mut f))
            }
            Instr::Stkcount(s1) => Instr::Stkcount(s1.map(&mut f)),
            Instr::Stkpeek(l1, s1) => Instr::Stkpeek(l1.map(&mut f), s1.map(&mut f)),
            Instr::Stkswap => Instr::Stkswap,
            Instr::Stkcopy(l1) => Instr::Stkcopy(l1.map(&mut f)),
            Instr::Stkroll(l1, l2) => Instr::Stkroll(l1.map(&mut f), l2.map(&mut f)),
            Instr::Call(l1, l2, s1) => Instr::Call(l1.map(&mut f), l2.map(&mut f), s1.map(&mut f)),
            Instr::Callf(l1, s1) => Instr::Callf(l1.map(&mut f), s1.map(&mut f)),
            Instr::Callfi(l1, l2, s1) => {
                Instr::Callfi(l1.map(&mut f), l2.map(&mut f), s1.map(&mut f))
            }
            Instr::Callfii(l1, l2, l3, s1) => Instr::Callfii(
                l1.map(&mut f),
                l2.map(&mut f),
                l3.map(&mut f),
                s1.map(&mut f),
            ),
            Instr::Callfiii(l1, l2, l3, l4, s1) => Instr::Callfiii(
                l1.map(&mut f),
                l2.map(&mut f),
                l3.map(&mut f),
                l4.map(&mut f),
                s1.map(&mut f),
            ),
            Instr::Return(l1) => Instr::Return(l1.map(&mut f)),
            Instr::Tailcall(l1, l2) => Instr::Tailcall(l1.map(&mut f), l2.map(&mut f)),
            Instr::Catch(s1, l1) => Instr::Catch(s1.map(&mut f), l1.map(&mut f)),
            Instr::Throw(l1, l2) => Instr::Throw(l1.map(&mut f), l2.map(&mut f)),
            Instr::Getmemsize(s1) => Instr::Getmemsize(s1.map(&mut f)),
            Instr::Setmemsize(l1, s1) => Instr::Setmemsize(l1.map(&mut f), s1.map(&mut f)),
            Instr::Malloc(l1, s1) => Instr::Malloc(l1.map(&mut f), s1.map(&mut f)),
            Instr::Mfree(l1) => Instr::Mfree(l1.map(&mut f)),
            Instr::Quit => Instr::Quit,
            Instr::Restart => Instr::Restart,
            Instr::Save(l1, s1) => Instr::Save(l1.map(&mut f), s1.map(&mut f)),
            Instr::Restore(l1, s1) => Instr::Restore(l1.map(&mut f), s1.map(&mut f)),
            Instr::Saveundo(s1) => Instr::Saveundo(s1.map(&mut f)),
            Instr::Restoreundo(s1) => Instr::Restoreundo(s1.map(&mut f)),
            Instr::Hasundo(s1) => Instr::Hasundo(s1.map(&mut f)),
            Instr::Discardundo => Instr::Discardundo,
            Instr::Protect(l1, l2) => Instr::Protect(l1.map(&mut f), l2.map(&mut f)),
            Instr::Verify(s1) => Instr::Verify(s1.map(&mut f)),
            Instr::Getiosys(s1, s2) => Instr::Getiosys(s1.map(&mut f), s2.map(&mut f)),
            Instr::Setiosys(l1, l2) => Instr::Setiosys(l1.map(&mut f), l2.map(&mut f)),
            Instr::Streamchar(l1) => Instr::Streamchar(l1.map(&mut f)),
            Instr::Streamunichar(l1) => Instr::Streamunichar(l1.map(&mut f)),
            Instr::Streamnum(l1) => Instr::Streamnum(l1.map(&mut f)),
            Instr::Streamstr(l1) => Instr::Streamstr(l1.map(&mut f)),
            Instr::Getstringtbl(s1) => Instr::Getstringtbl(s1.map(&mut f)),
            Instr::Setstringtbl(l1) => Instr::Setstringtbl(l1.map(&mut f)),
            Instr::Numtof(l1, s1) => Instr::Numtof(l1.map(&mut f), s1.map(&mut f)),
            Instr::Ftonumz(l1, s1) => Instr::Ftonumz(l1.map(&mut f), s1.map(&mut f)),
            Instr::Ftonumn(l1, s1) => Instr::Ftonumn(l1.map(&mut f), s1.map(&mut f)),
            Instr::Fadd(l1, l2, s1) => Instr::Fadd(l1.map(&mut f), l2.map(&mut f), s1.map(&mut f)),
            Instr::Fsub(l1, l2, s1) => Instr::Fsub(l1.map(&mut f), l2.map(&mut f), s1.map(&mut f)),
            Instr::Fmul(l1, l2, s1) => Instr::Fmul(l1.map(&mut f), l2.map(&mut f), s1.map(&mut f)),
            Instr::Fdiv(l1, l2, s1) => Instr::Fdiv(l1.map(&mut f), l2.map(&mut f), s1.map(&mut f)),
            Instr::Fmod(l1, l2, s1) => Instr::Fmod(l1.map(&mut f), l2.map(&mut f), s1.map(&mut f)),
            Instr::Ceil(l1, s1) => Instr::Ceil(l1.map(&mut f), s1.map(&mut f)),
            Instr::Floor(l1, s1) => Instr::Floor(l1.map(&mut f), s1.map(&mut f)),
            Instr::Sqrt(l1, s1) => Instr::Sqrt(l1.map(&mut f), s1.map(&mut f)),
            Instr::Exp(l1, s1) => Instr::Exp(l1.map(&mut f), s1.map(&mut f)),
            Instr::Log(l1, s1) => Instr::Log(l1.map(&mut f), s1.map(&mut f)),
            Instr::Pow(l1, l2, s1) => Instr::Pow(l1.map(&mut f), l2.map(&mut f), s1.map(&mut f)),
            Instr::Sin(l1, s1) => Instr::Sin(l1.map(&mut f), s1.map(&mut f)),
            Instr::Cos(l1, s1) => Instr::Cos(l1.map(&mut f), s1.map(&mut f)),
            Instr::Tan(l1, s1) => Instr::Tan(l1.map(&mut f), s1.map(&mut f)),
            Instr::Asin(l1, s1) => Instr::Asin(l1.map(&mut f), s1.map(&mut f)),
            Instr::Acos(l1, s1) => Instr::Acos(l1.map(&mut f), s1.map(&mut f)),
            Instr::Atan(l1, s1) => Instr::Atan(l1.map(&mut f), s1.map(&mut f)),
            Instr::Atan2(l1, s1) => Instr::Atan2(l1.map(&mut f), s1.map(&mut f)),
            Instr::Numtod(l1, s1, s2) => {
                Instr::Numtod(l1.map(&mut f), s1.map(&mut f), s2.map(&mut f))
            }
            Instr::Dtonumz(l1, l2, s1) => {
                Instr::Dtonumz(l1.map(&mut f), l2.map(&mut f), s1.map(&mut f))
            }
            Instr::Dtonumn(l1, l2, s1) => {
                Instr::Dtonumn(l1.map(&mut f), l2.map(&mut f), s1.map(&mut f))
            }
            Instr::Ftod(l1, s1, s2) => Instr::Ftod(l1.map(&mut f), s1.map(&mut f), s2.map(&mut f)),
            Instr::Dtof(l1, l2, s1) => Instr::Dtof(l1.map(&mut f), l2.map(&mut f), s1.map(&mut f)),
            Instr::Dadd(l1, l2, l3, l4, s1, s2) => Instr::Dadd(
                l1.map(&mut f),
                l2.map(&mut f),
                l3.map(&mut f),
                l4.map(&mut f),
                s1.map(&mut f),
                s2.map(&mut f),
            ),
            Instr::Dsub(l1, l2, l3, l4, s1, s2) => Instr::Dsub(
                l1.map(&mut f),
                l2.map(&mut f),
                l3.map(&mut f),
                l4.map(&mut f),
                s1.map(&mut f),
                s2.map(&mut f),
            ),
            Instr::Dmul(l1, l2, l3, l4, s1, s2) => Instr::Dmul(
                l1.map(&mut f),
                l2.map(&mut f),
                l3.map(&mut f),
                l4.map(&mut f),
                s1.map(&mut f),
                s2.map(&mut f),
            ),
            Instr::Ddiv(l1, l2, l3, l4, s1, s2) => Instr::Ddiv(
                l1.map(&mut f),
                l2.map(&mut f),
                l3.map(&mut f),
                l4.map(&mut f),
                s1.map(&mut f),
                s2.map(&mut f),
            ),
            Instr::Dmodr(l1, l2, l3, l4, s1, s2) => Instr::Dmodr(
                l1.map(&mut f),
                l2.map(&mut f),
                l3.map(&mut f),
                l4.map(&mut f),
                s1.map(&mut f),
                s2.map(&mut f),
            ),
            Instr::Dmodq(l1, l2, l3, l4, s1, s2) => Instr::Dmodq(
                l1.map(&mut f),
                l2.map(&mut f),
                l3.map(&mut f),
                l4.map(&mut f),
                s1.map(&mut f),
                s2.map(&mut f),
            ),
            Instr::Dceil(l1, l2, s1, s2) => Instr::Dceil(
                l1.map(&mut f),
                l2.map(&mut f),
                s1.map(&mut f),
                s2.map(&mut f),
            ),
            Instr::Dfloor(l1, l2, s1, s2) => Instr::Dfloor(
                l1.map(&mut f),
                l2.map(&mut f),
                s1.map(&mut f),
                s2.map(&mut f),
            ),
            Instr::Dsqrt(l1, l2, s1, s2) => Instr::Dsqrt(
                l1.map(&mut f),
                l2.map(&mut f),
                s1.map(&mut f),
                s2.map(&mut f),
            ),
            Instr::Dexp(l1, l2, s1, s2) => Instr::Dexp(
                l1.map(&mut f),
                l2.map(&mut f),
                s1.map(&mut f),
                s2.map(&mut f),
            ),
            Instr::Dlog(l1, l2, s1, s2) => Instr::Dlog(
                l1.map(&mut f),
                l2.map(&mut f),
                s1.map(&mut f),
                s2.map(&mut f),
            ),
            Instr::Dpow(l1, l2, l3, l4, s1, s2) => Instr::Dpow(
                l1.map(&mut f),
                l2.map(&mut f),
                l3.map(&mut f),
                l4.map(&mut f),
                s1.map(&mut f),
                s2.map(&mut f),
            ),
            Instr::Dsin(l1, l2, s1, s2) => Instr::Dsin(
                l1.map(&mut f),
                l2.map(&mut f),
                s1.map(&mut f),
                s2.map(&mut f),
            ),
            Instr::Dcos(l1, l2, s1, s2) => Instr::Dcos(
                l1.map(&mut f),
                l2.map(&mut f),
                s1.map(&mut f),
                s2.map(&mut f),
            ),
            Instr::Dtan(l1, l2, s1, s2) => Instr::Dtan(
                l1.map(&mut f),
                l2.map(&mut f),
                s1.map(&mut f),
                s2.map(&mut f),
            ),
            Instr::Dasin(l1, l2, s1, s2) => Instr::Dasin(
                l1.map(&mut f),
                l2.map(&mut f),
                s1.map(&mut f),
                s2.map(&mut f),
            ),
            Instr::Dacos(l1, l2, s1, s2) => Instr::Dacos(
                l1.map(&mut f),
                l2.map(&mut f),
                s1.map(&mut f),
                s2.map(&mut f),
            ),
            Instr::Datan(l1, l2, s1, s2) => Instr::Datan(
                l1.map(&mut f),
                l2.map(&mut f),
                s1.map(&mut f),
                s2.map(&mut f),
            ),
            Instr::Datan2(l1, l2, l3, l4, s1, s2) => Instr::Datan2(
                l1.map(&mut f),
                l2.map(&mut f),
                l3.map(&mut f),
                l4.map(&mut f),
                s1.map(&mut f),
                s2.map(&mut f),
            ),
            Instr::Jisnan(l1, l2) => Instr::Jisnan(l1.map(&mut f), l2.map(&mut f)),
            Instr::Jisinf(l1, l2) => Instr::Jisinf(l1.map(&mut f), l2.map(&mut f)),
            Instr::Jfeq(l1, l2, l3, l4) => Instr::Jfeq(
                l1.map(&mut f),
                l2.map(&mut f),
                l3.map(&mut f),
                l4.map(&mut f),
            ),
            Instr::Jfne(l1, l2, l3, l4) => Instr::Jfne(
                l1.map(&mut f),
                l2.map(&mut f),
                l3.map(&mut f),
                l4.map(&mut f),
            ),
            Instr::Jflt(l1, l2, l3) => Instr::Jflt(l1.map(&mut f), l2.map(&mut f), l3.map(&mut f)),
            Instr::Jfle(l1, l2, l3) => Instr::Jfle(l1.map(&mut f), l2.map(&mut f), l3.map(&mut f)),
            Instr::Jfgt(l1, l2, l3) => Instr::Jfgt(l1.map(&mut f), l2.map(&mut f), l3.map(&mut f)),
            Instr::Jfge(l1, l2, l3) => Instr::Jfge(l1.map(&mut f), l2.map(&mut f), l3.map(&mut f)),
            Instr::Jdisnan(l1, l2, l3) => {
                Instr::Jdisnan(l1.map(&mut f), l2.map(&mut f), l3.map(&mut f))
            }
            Instr::Jdisinf(l1, l2, l3) => {
                Instr::Jdisinf(l1.map(&mut f), l2.map(&mut f), l3.map(&mut f))
            }
            Instr::Jdeq(l1, l2, l3, l4, l5, l6, l7) => Instr::Jdeq(
                l1.map(&mut f),
                l2.map(&mut f),
                l3.map(&mut f),
                l4.map(&mut f),
                l5.map(&mut f),
                l6.map(&mut f),
                l7.map(&mut f),
            ),
            Instr::Jdne(l1, l2, l3, l4, l5, l6, l7) => Instr::Jdne(
                l1.map(&mut f),
                l2.map(&mut f),
                l3.map(&mut f),
                l4.map(&mut f),
                l5.map(&mut f),
                l6.map(&mut f),
                l7.map(&mut f),
            ),
            Instr::Jdlt(l1, l2, l3, l4, l5) => Instr::Jdlt(
                l1.map(&mut f),
                l2.map(&mut f),
                l3.map(&mut f),
                l4.map(&mut f),
                l5.map(&mut f),
            ),
            Instr::Jdle(l1, l2, l3, l4, l5) => Instr::Jdle(
                l1.map(&mut f),
                l2.map(&mut f),
                l3.map(&mut f),
                l4.map(&mut f),
                l5.map(&mut f),
            ),
            Instr::Jdgt(l1, l2, l3, l4, l5) => Instr::Jdgt(
                l1.map(&mut f),
                l2.map(&mut f),
                l3.map(&mut f),
                l4.map(&mut f),
                l5.map(&mut f),
            ),
            Instr::Jdge(l1, l2, l3, l4, l5) => Instr::Jdge(
                l1.map(&mut f),
                l2.map(&mut f),
                l3.map(&mut f),
                l4.map(&mut f),
                l5.map(&mut f),
            ),
            Instr::Random(l1, s1) => Instr::Random(l1.map(&mut f), s1.map(&mut f)),
            Instr::Setrandom(l1) => Instr::Setrandom(l1.map(&mut f)),
            Instr::Mzero(l1, l2) => Instr::Mzero(l1.map(&mut f), l2.map(&mut f)),
            Instr::Mcopy(l1, l2, l3) => {
                Instr::Mcopy(l1.map(&mut f), l2.map(&mut f), l3.map(&mut f))
            }
            Instr::Linearsearch(l1, l2, l3, l4, l5, l6, l7, s1) => Instr::Linearsearch(
                l1.map(&mut f),
                l2.map(&mut f),
                l3.map(&mut f),
                l4.map(&mut f),
                l5.map(&mut f),
                l6.map(&mut f),
                l7.map(&mut f),
                s1.map(&mut f),
            ),
            Instr::Binarysearch(l1, l2, l3, l4, l5, l6, l7, s1) => Instr::Binarysearch(
                l1.map(&mut f),
                l2.map(&mut f),
                l3.map(&mut f),
                l4.map(&mut f),
                l5.map(&mut f),
                l6.map(&mut f),
                l7.map(&mut f),
                s1.map(&mut f),
            ),
            Instr::Linkedsearch(l1, l2, l3, l4, l5, l6, s1) => Instr::Linkedsearch(
                l1.map(&mut f),
                l2.map(&mut f),
                l3.map(&mut f),
                l4.map(&mut f),
                l5.map(&mut f),
                l6.map(&mut f),
                s1.map(&mut f),
            ),
            Instr::Accelfunc(l1, l2) => Instr::Accelfunc(l1.map(&mut f), l2.map(&mut f)),
            Instr::Accelparam(l1, l2) => Instr::Accelparam(l1.map(&mut f), l2.map(&mut f)),
            Instr::Gestalt(l1, l2, s1) => {
                Instr::Gestalt(l1.map(&mut f), l2.map(&mut f), s1.map(&mut f))
            }
            Instr::Debugtrap(l1) => Instr::Debugtrap(l1.map(&mut f)),
            Instr::Glk(l1, l2, s1) => Instr::Glk(l1.map(&mut f), l2.map(&mut f), s1.map(&mut f)),
        }
    }

    /// Returns an upper bound on how long the serialized instruction might be,
    /// regardless of its position.
    pub(crate) fn worst_len(&self) -> usize {
        let opcode = self.opcode();
        let opcode_length = opcode_len(opcode);

        let operands_length: usize = match self {
            Instr::Nop => 0,
            Instr::Add(l1, l2, s1) => worst_len!(l1, l2, s1),
            Instr::Sub(l1, l2, s1) => worst_len!(l1, l2, s1),
            Instr::Mul(l1, l2, s1) => worst_len!(l1, l2, s1),
            Instr::Div(l1, l2, s1) => worst_len!(l1, l2, s1),
            Instr::Mod(l1, l2, s1) => worst_len!(l1, l2, s1),
            Instr::Neg(l1, s1) => worst_len!(l1, s1),
            Instr::Bitand(l1, l2, s1) => worst_len!(l1, l2, s1),
            Instr::Bitor(l1, l2, s1) => worst_len!(l1, l2, s1),
            Instr::Bitxor(l1, l2, s1) => worst_len!(l1, l2, s1),
            Instr::Bitnot(l1, s1) => worst_len!(l1, s1),
            Instr::Shiftl(l1, l2, s1) => worst_len!(l1, l2, s1),
            Instr::Ushiftr(l1, l2, s1) => worst_len!(l1, l2, s1),
            Instr::Sshiftr(l1, l2, s1) => worst_len!(l1, l2, s1),
            Instr::Jump(l1) => worst_len!(l1),
            Instr::Jz(l1, l2) => worst_len!(l1, l2),
            Instr::Jnz(l1, l2) => worst_len!(l1, l2),
            Instr::Jeq(l1, l2, l3) => worst_len!(l1, l2, l3),
            Instr::Jne(l1, l2, l3) => worst_len!(l1, l2, l3),
            Instr::Jlt(l1, l2, l3) => worst_len!(l1, l2, l3),
            Instr::Jle(l1, l2, l3) => worst_len!(l1, l2, l3),
            Instr::Jgt(l1, l2, l3) => worst_len!(l1, l2, l3),
            Instr::Jge(l1, l2, l3) => worst_len!(l1, l2, l3),
            Instr::Jltu(l1, l2, l3) => worst_len!(l1, l2, l3),
            Instr::Jleu(l1, l2, l3) => worst_len!(l1, l2, l3),
            Instr::Jgtu(l1, l2, l3) => worst_len!(l1, l2, l3),
            Instr::Jgeu(l1, l2, l3) => worst_len!(l1, l2, l3),
            Instr::Jumpabs(l1) => worst_len!(l1),
            Instr::Copy(l1, s1) => worst_len!(l1, s1),
            Instr::Copys(l1, s1) => worst_len!(l1, s1),
            Instr::Copyb(l1, s1) => worst_len!(l1, s1),
            Instr::Sexs(l1, s1) => worst_len!(l1, s1),
            Instr::Sexb(l1, s1) => worst_len!(l1, s1),
            Instr::Astore(l1, l2, l3) => worst_len!(l1, l2, l3),
            Instr::Aload(l1, l2, s1) => worst_len!(l1, l2, s1),
            Instr::Astores(l1, l2, l3) => worst_len!(l1, l2, l3),
            Instr::Aloads(l1, l2, s1) => worst_len!(l1, l2, s1),
            Instr::Astoreb(l1, l2, l3) => worst_len!(l1, l2, l3),
            Instr::Aloadb(l1, l2, s1) => worst_len!(l1, l2, s1),
            Instr::Astorebit(l1, l2, l3) => worst_len!(l1, l2, l3),
            Instr::Aloadbit(l1, l2, s1) => worst_len!(l1, l2, s1),
            Instr::Stkcount(s1) => worst_len!(s1),
            Instr::Stkpeek(l1, s1) => worst_len!(l1, s1),
            Instr::Stkswap => 0,
            Instr::Stkcopy(l1) => worst_len!(l1),
            Instr::Stkroll(l1, l2) => worst_len!(l1, l2),
            Instr::Call(l1, l2, s1) => worst_len!(l1, l2, s1),
            Instr::Callf(l1, s1) => worst_len!(l1, s1),
            Instr::Callfi(l1, l2, s1) => worst_len!(l1, l2, s1),
            Instr::Callfii(l1, l2, l3, s1) => worst_len!(l1, l2, l3, s1),
            Instr::Callfiii(l1, l2, l3, l4, s1) => worst_len!(l1, l2, l3, l4, s1),
            Instr::Return(l1) => worst_len!(l1),
            Instr::Tailcall(l1, l2) => worst_len!(l1, l2),
            Instr::Catch(s1, l1) => worst_len!(s1, l1),
            Instr::Throw(l1, l2) => worst_len!(l1, l2),
            Instr::Getmemsize(s1) => worst_len!(s1),
            Instr::Setmemsize(l1, s1) => worst_len!(l1, s1),
            Instr::Malloc(l1, s1) => worst_len!(l1, s1),
            Instr::Mfree(l1) => worst_len!(l1),
            Instr::Quit => 0,
            Instr::Restart => 0,
            Instr::Save(l1, s1) => worst_len!(l1, s1),
            Instr::Restore(l1, s1) => worst_len!(l1, s1),
            Instr::Saveundo(s1) => worst_len!(s1),
            Instr::Restoreundo(s1) => worst_len!(s1),
            Instr::Hasundo(s1) => worst_len!(s1),
            Instr::Discardundo => 0,
            Instr::Protect(l1, l2) => worst_len!(l1, l2),
            Instr::Verify(s1) => worst_len!(s1),
            Instr::Getiosys(s1, s2) => worst_len!(s1, s2),
            Instr::Setiosys(l1, l2) => worst_len!(l1, l2),
            Instr::Streamchar(l1) => worst_len!(l1),
            Instr::Streamunichar(l1) => worst_len!(l1),
            Instr::Streamnum(l1) => worst_len!(l1),
            Instr::Streamstr(l1) => worst_len!(l1),
            Instr::Getstringtbl(s1) => worst_len!(s1),
            Instr::Setstringtbl(l1) => worst_len!(l1),
            Instr::Numtof(l1, s1) => worst_len!(l1, s1),
            Instr::Ftonumz(l1, s1) => worst_len!(l1, s1),
            Instr::Ftonumn(l1, s1) => worst_len!(l1, s1),
            Instr::Fadd(l1, l2, s1) => worst_len!(l1, l2, s1),
            Instr::Fsub(l1, l2, s1) => worst_len!(l1, l2, s1),
            Instr::Fmul(l1, l2, s1) => worst_len!(l1, l2, s1),
            Instr::Fdiv(l1, l2, s1) => worst_len!(l1, l2, s1),
            Instr::Fmod(l1, l2, s1) => worst_len!(l1, l2, s1),
            Instr::Ceil(l1, s1) => worst_len!(l1, s1),
            Instr::Floor(l1, s1) => worst_len!(l1, s1),
            Instr::Sqrt(l1, s1) => worst_len!(l1, s1),
            Instr::Exp(l1, s1) => worst_len!(l1, s1),
            Instr::Log(l1, s1) => worst_len!(l1, s1),
            Instr::Pow(l1, l2, s1) => worst_len!(l1, l2, s1),
            Instr::Sin(l1, s1) => worst_len!(l1, s1),
            Instr::Cos(l1, s1) => worst_len!(l1, s1),
            Instr::Tan(l1, s1) => worst_len!(l1, s1),
            Instr::Asin(l1, s1) => worst_len!(l1, s1),
            Instr::Acos(l1, s1) => worst_len!(l1, s1),
            Instr::Atan(l1, s1) => worst_len!(l1, s1),
            Instr::Atan2(l1, s1) => worst_len!(l1, s1),
            Instr::Numtod(l1, s1, s2) => worst_len!(l1, s1, s2),
            Instr::Dtonumz(l1, l2, s1) => worst_len!(l1, l2, s1),
            Instr::Dtonumn(l1, l2, s1) => worst_len!(l1, l2, s1),
            Instr::Ftod(l1, s1, s2) => worst_len!(l1, s1, s2),
            Instr::Dtof(l1, l2, s1) => worst_len!(l1, l2, s1),
            Instr::Dadd(l1, l2, l3, l4, s1, s2) => worst_len!(l1, l2, l3, l4, s1, s2),
            Instr::Dsub(l1, l2, l3, l4, s1, s2) => worst_len!(l1, l2, l3, l4, s1, s2),
            Instr::Dmul(l1, l2, l3, l4, s1, s2) => worst_len!(l1, l2, l3, l4, s1, s2),
            Instr::Ddiv(l1, l2, l3, l4, s1, s2) => worst_len!(l1, l2, l3, l4, s1, s2),
            Instr::Dmodr(l1, l2, l3, l4, s1, s2) => worst_len!(l1, l2, l3, l4, s1, s2),
            Instr::Dmodq(l1, l2, l3, l4, s1, s2) => worst_len!(l1, l2, l3, l4, s1, s2),
            Instr::Dceil(l1, l2, s1, s2) => worst_len!(l1, l2, s1, s2),
            Instr::Dfloor(l1, l2, s1, s2) => worst_len!(l1, l2, s1, s2),
            Instr::Dsqrt(l1, l2, s1, s2) => worst_len!(l1, l2, s1, s2),
            Instr::Dexp(l1, l2, s1, s2) => worst_len!(l1, l2, s1, s2),
            Instr::Dlog(l1, l2, s1, s2) => worst_len!(l1, l2, s1, s2),
            Instr::Dpow(l1, l2, l3, l4, s1, s2) => worst_len!(l1, l2, l3, l4, s1, s2),
            Instr::Dsin(l1, l2, s1, s2) => worst_len!(l1, l2, s1, s2),
            Instr::Dcos(l1, l2, s1, s2) => worst_len!(l1, l2, s1, s2),
            Instr::Dtan(l1, l2, s1, s2) => worst_len!(l1, l2, s1, s2),
            Instr::Dasin(l1, l2, s1, s2) => worst_len!(l1, l2, s1, s2),
            Instr::Dacos(l1, l2, s1, s2) => worst_len!(l1, l2, s1, s2),
            Instr::Datan(l1, l2, s1, s2) => worst_len!(l1, l2, s1, s2),
            Instr::Datan2(l1, l2, l3, l4, s1, s2) => worst_len!(l1, l2, l3, l4, s1, s2),
            Instr::Jisnan(l1, l2) => worst_len!(l1, l2),
            Instr::Jisinf(l1, l2) => worst_len!(l1, l2),
            Instr::Jfeq(l1, l2, l3, l4) => worst_len!(l1, l2, l3, l4),
            Instr::Jfne(l1, l2, l3, l4) => worst_len!(l1, l2, l3, l4),
            Instr::Jflt(l1, l2, l3) => worst_len!(l1, l2, l3),
            Instr::Jfle(l1, l2, l3) => worst_len!(l1, l2, l3),
            Instr::Jfgt(l1, l2, l3) => worst_len!(l1, l2, l3),
            Instr::Jfge(l1, l2, l3) => worst_len!(l1, l2, l3),
            Instr::Jdisnan(l1, l2, l3) => worst_len!(l1, l2, l3),
            Instr::Jdisinf(l1, l2, l3) => worst_len!(l1, l2, l3),
            Instr::Jdeq(l1, l2, l3, l4, l5, l6, l7) => worst_len!(l1, l2, l3, l4, l5, l6, l7),
            Instr::Jdne(l1, l2, l3, l4, l5, l6, l7) => worst_len!(l1, l2, l3, l4, l5, l6, l7),
            Instr::Jdlt(l1, l2, l3, l4, l5) => worst_len!(l1, l2, l3, l4, l5),
            Instr::Jdle(l1, l2, l3, l4, l5) => worst_len!(l1, l2, l3, l4, l5),
            Instr::Jdgt(l1, l2, l3, l4, l5) => worst_len!(l1, l2, l3, l4, l5),
            Instr::Jdge(l1, l2, l3, l4, l5) => worst_len!(l1, l2, l3, l4, l5),
            Instr::Random(l1, s1) => worst_len!(l1, s1),
            Instr::Setrandom(l1) => worst_len!(l1),
            Instr::Mzero(l1, l2) => worst_len!(l1, l2),
            Instr::Mcopy(l1, l2, l3) => worst_len!(l1, l2, l3),
            Instr::Linearsearch(l1, l2, l3, l4, l5, l6, l7, s1) => {
                worst_len!(l1, l2, l3, l4, l5, l6, l7, s1)
            }
            Instr::Binarysearch(l1, l2, l3, l4, l5, l6, l7, s1) => {
                worst_len!(l1, l2, l3, l4, l5, l6, l7, s1)
            }
            Instr::Linkedsearch(l1, l2, l3, l4, l5, l6, s1) => {
                worst_len!(l1, l2, l3, l4, l5, l6, s1)
            }
            Instr::Accelfunc(l1, l2) => worst_len!(l1, l2),
            Instr::Accelparam(l1, l2) => worst_len!(l1, l2),
            Instr::Gestalt(l1, l2, s1) => worst_len!(l1, l2, s1),
            Instr::Debugtrap(l1) => worst_len!(l1),
            Instr::Glk(l1, l2, s1) => worst_len!(l1, l2, s1),
        };

        opcode_length + operands_length
    }

    /// Resolves all labels in the instruction to produce a [`RawInstr`].
    pub(crate) fn resolve<R>(
        &self,
        mut position: u32,
        ramstart: u32,
        resolver: &R,
    ) -> Result<RawInstr, AssemblerError<L>>
    where
        R: Resolver<Label = L>,
    {
        let opcode = self.opcode();
        let opcode_length = u32::try_from(opcode_len(opcode)).unwrap();

        position = position
            .checked_add(opcode_length)
            .ok_or(AssemblerError::Overflow)?;

        let operands = match self {
            Instr::Nop => resolve!(position, ramstart, resolver,),
            Instr::Add(l1, l2, s1) => resolve!(position, ramstart, resolver, l1, l2, s1),
            Instr::Sub(l1, l2, s1) => resolve!(position, ramstart, resolver, l1, l2, s1),
            Instr::Mul(l1, l2, s1) => resolve!(position, ramstart, resolver, l1, l2, s1),
            Instr::Div(l1, l2, s1) => resolve!(position, ramstart, resolver, l1, l2, s1),
            Instr::Mod(l1, l2, s1) => resolve!(position, ramstart, resolver, l1, l2, s1),
            Instr::Neg(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Bitand(l1, l2, s1) => resolve!(position, ramstart, resolver, l1, l2, s1),
            Instr::Bitor(l1, l2, s1) => resolve!(position, ramstart, resolver, l1, l2, s1),
            Instr::Bitxor(l1, l2, s1) => resolve!(position, ramstart, resolver, l1, l2, s1),
            Instr::Bitnot(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Shiftl(l1, l2, s1) => resolve!(position, ramstart, resolver, l1, l2, s1),
            Instr::Ushiftr(l1, l2, s1) => resolve!(position, ramstart, resolver, l1, l2, s1),
            Instr::Sshiftr(l1, l2, s1) => resolve!(position, ramstart, resolver, l1, l2, s1),
            Instr::Jump(l1) => resolve!(position, ramstart, resolver, l1),
            Instr::Jz(l1, l2) => resolve!(position, ramstart, resolver, l1, l2),
            Instr::Jnz(l1, l2) => resolve!(position, ramstart, resolver, l1, l2),
            Instr::Jeq(l1, l2, l3) => resolve!(position, ramstart, resolver, l1, l2, l3),
            Instr::Jne(l1, l2, l3) => resolve!(position, ramstart, resolver, l1, l2, l3),
            Instr::Jlt(l1, l2, l3) => resolve!(position, ramstart, resolver, l1, l2, l3),
            Instr::Jle(l1, l2, l3) => resolve!(position, ramstart, resolver, l1, l2, l3),
            Instr::Jgt(l1, l2, l3) => resolve!(position, ramstart, resolver, l1, l2, l3),
            Instr::Jge(l1, l2, l3) => resolve!(position, ramstart, resolver, l1, l2, l3),
            Instr::Jltu(l1, l2, l3) => resolve!(position, ramstart, resolver, l1, l2, l3),
            Instr::Jleu(l1, l2, l3) => resolve!(position, ramstart, resolver, l1, l2, l3),
            Instr::Jgtu(l1, l2, l3) => resolve!(position, ramstart, resolver, l1, l2, l3),
            Instr::Jgeu(l1, l2, l3) => resolve!(position, ramstart, resolver, l1, l2, l3),
            Instr::Jumpabs(l1) => resolve!(position, ramstart, resolver, l1),
            Instr::Copy(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Copys(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Copyb(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Sexs(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Sexb(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Astore(l1, l2, l3) => resolve!(position, ramstart, resolver, l1, l2, l3),
            Instr::Aload(l1, l2, s1) => resolve!(position, ramstart, resolver, l1, l2, s1),
            Instr::Astores(l1, l2, l3) => resolve!(position, ramstart, resolver, l1, l2, l3),
            Instr::Aloads(l1, l2, s1) => resolve!(position, ramstart, resolver, l1, l2, s1),
            Instr::Astoreb(l1, l2, l3) => resolve!(position, ramstart, resolver, l1, l2, l3),
            Instr::Aloadb(l1, l2, s1) => resolve!(position, ramstart, resolver, l1, l2, s1),
            Instr::Astorebit(l1, l2, l3) => resolve!(position, ramstart, resolver, l1, l2, l3),
            Instr::Aloadbit(l1, l2, s1) => resolve!(position, ramstart, resolver, l1, l2, s1),
            Instr::Stkcount(s1) => resolve!(position, ramstart, resolver, s1),
            Instr::Stkpeek(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Stkswap => resolve!(position, ramstart, resolver,),
            Instr::Stkcopy(l1) => resolve!(position, ramstart, resolver, l1),
            Instr::Stkroll(l1, l2) => resolve!(position, ramstart, resolver, l1, l2),
            Instr::Call(l1, l2, s1) => resolve!(position, ramstart, resolver, l1, l2, s1),
            Instr::Callf(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Callfi(l1, l2, s1) => resolve!(position, ramstart, resolver, l1, l2, s1),
            Instr::Callfii(l1, l2, l3, s1) => {
                resolve!(position, ramstart, resolver, l1, l2, l3, s1)
            }
            Instr::Callfiii(l1, l2, l3, l4, s1) => {
                resolve!(position, ramstart, resolver, l1, l2, l3, l4, s1)
            }
            Instr::Return(l1) => resolve!(position, ramstart, resolver, l1),
            Instr::Tailcall(l1, l2) => resolve!(position, ramstart, resolver, l1, l2),
            Instr::Catch(s1, l1) => resolve!(position, ramstart, resolver, s1, l1),
            Instr::Throw(l1, l2) => resolve!(position, ramstart, resolver, l1, l2),
            Instr::Getmemsize(s1) => resolve!(position, ramstart, resolver, s1),
            Instr::Setmemsize(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Malloc(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Mfree(l1) => resolve!(position, ramstart, resolver, l1),
            Instr::Quit => resolve!(position, ramstart, resolver,),
            Instr::Restart => resolve!(position, ramstart, resolver,),
            Instr::Save(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Restore(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Saveundo(s1) => resolve!(position, ramstart, resolver, s1),
            Instr::Restoreundo(s1) => resolve!(position, ramstart, resolver, s1),
            Instr::Hasundo(s1) => resolve!(position, ramstart, resolver, s1),
            Instr::Discardundo => resolve!(position, ramstart, resolver,),
            Instr::Protect(l1, l2) => resolve!(position, ramstart, resolver, l1, l2),
            Instr::Verify(s1) => resolve!(position, ramstart, resolver, s1),
            Instr::Getiosys(s1, s2) => resolve!(position, ramstart, resolver, s1, s2),
            Instr::Setiosys(l1, l2) => resolve!(position, ramstart, resolver, l1, l2),
            Instr::Streamchar(l1) => resolve!(position, ramstart, resolver, l1),
            Instr::Streamunichar(l1) => resolve!(position, ramstart, resolver, l1),
            Instr::Streamnum(l1) => resolve!(position, ramstart, resolver, l1),
            Instr::Streamstr(l1) => resolve!(position, ramstart, resolver, l1),
            Instr::Getstringtbl(s1) => resolve!(position, ramstart, resolver, s1),
            Instr::Setstringtbl(l1) => resolve!(position, ramstart, resolver, l1),
            Instr::Numtof(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Ftonumz(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Ftonumn(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Fadd(l1, l2, s1) => resolve!(position, ramstart, resolver, l1, l2, s1),
            Instr::Fsub(l1, l2, s1) => resolve!(position, ramstart, resolver, l1, l2, s1),
            Instr::Fmul(l1, l2, s1) => resolve!(position, ramstart, resolver, l1, l2, s1),
            Instr::Fdiv(l1, l2, s1) => resolve!(position, ramstart, resolver, l1, l2, s1),
            Instr::Fmod(l1, l2, s1) => resolve!(position, ramstart, resolver, l1, l2, s1),
            Instr::Ceil(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Floor(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Sqrt(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Exp(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Log(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Pow(l1, l2, s1) => resolve!(position, ramstart, resolver, l1, l2, s1),
            Instr::Sin(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Cos(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Tan(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Asin(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Acos(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Atan(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Atan2(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Numtod(l1, s1, s2) => resolve!(position, ramstart, resolver, l1, s1, s2),
            Instr::Dtonumz(l1, l2, s1) => resolve!(position, ramstart, resolver, l1, l2, s1),
            Instr::Dtonumn(l1, l2, s1) => resolve!(position, ramstart, resolver, l1, l2, s1),
            Instr::Ftod(l1, s1, s2) => resolve!(position, ramstart, resolver, l1, s1, s2),
            Instr::Dtof(l1, l2, s1) => resolve!(position, ramstart, resolver, l1, l2, s1),
            Instr::Dadd(l1, l2, l3, l4, s1, s2) => {
                resolve!(position, ramstart, resolver, l1, l2, l3, l4, s1, s2)
            }
            Instr::Dsub(l1, l2, l3, l4, s1, s2) => {
                resolve!(position, ramstart, resolver, l1, l2, l3, l4, s1, s2)
            }
            Instr::Dmul(l1, l2, l3, l4, s1, s2) => {
                resolve!(position, ramstart, resolver, l1, l2, l3, l4, s1, s2)
            }
            Instr::Ddiv(l1, l2, l3, l4, s1, s2) => {
                resolve!(position, ramstart, resolver, l1, l2, l3, l4, s1, s2)
            }
            Instr::Dmodr(l1, l2, l3, l4, s1, s2) => {
                resolve!(position, ramstart, resolver, l1, l2, l3, l4, s1, s2)
            }
            Instr::Dmodq(l1, l2, l3, l4, s1, s2) => {
                resolve!(position, ramstart, resolver, l1, l2, l3, l4, s1, s2)
            }
            Instr::Dceil(l1, l2, s1, s2) => resolve!(position, ramstart, resolver, l1, l2, s1, s2),
            Instr::Dfloor(l1, l2, s1, s2) => resolve!(position, ramstart, resolver, l1, l2, s1, s2),
            Instr::Dsqrt(l1, l2, s1, s2) => resolve!(position, ramstart, resolver, l1, l2, s1, s2),
            Instr::Dexp(l1, l2, s1, s2) => resolve!(position, ramstart, resolver, l1, l2, s1, s2),
            Instr::Dlog(l1, l2, s1, s2) => resolve!(position, ramstart, resolver, l1, l2, s1, s2),
            Instr::Dpow(l1, l2, l3, l4, s1, s2) => {
                resolve!(position, ramstart, resolver, l1, l2, l3, l4, s1, s2)
            }
            Instr::Dsin(l1, l2, s1, s2) => resolve!(position, ramstart, resolver, l1, l2, s1, s2),
            Instr::Dcos(l1, l2, s1, s2) => resolve!(position, ramstart, resolver, l1, l2, s1, s2),
            Instr::Dtan(l1, l2, s1, s2) => resolve!(position, ramstart, resolver, l1, l2, s1, s2),
            Instr::Dasin(l1, l2, s1, s2) => resolve!(position, ramstart, resolver, l1, l2, s1, s2),
            Instr::Dacos(l1, l2, s1, s2) => resolve!(position, ramstart, resolver, l1, l2, s1, s2),
            Instr::Datan(l1, l2, s1, s2) => resolve!(position, ramstart, resolver, l1, l2, s1, s2),
            Instr::Datan2(l1, l2, l3, l4, s1, s2) => {
                resolve!(position, ramstart, resolver, l1, l2, l3, l4, s1, s2)
            }
            Instr::Jisnan(l1, l2) => resolve!(position, ramstart, resolver, l1, l2),
            Instr::Jisinf(l1, l2) => resolve!(position, ramstart, resolver, l1, l2),
            Instr::Jfeq(l1, l2, l3, l4) => resolve!(position, ramstart, resolver, l1, l2, l3, l4),
            Instr::Jfne(l1, l2, l3, l4) => resolve!(position, ramstart, resolver, l1, l2, l3, l4),
            Instr::Jflt(l1, l2, l3) => resolve!(position, ramstart, resolver, l1, l2, l3),
            Instr::Jfle(l1, l2, l3) => resolve!(position, ramstart, resolver, l1, l2, l3),
            Instr::Jfgt(l1, l2, l3) => resolve!(position, ramstart, resolver, l1, l2, l3),
            Instr::Jfge(l1, l2, l3) => resolve!(position, ramstart, resolver, l1, l2, l3),
            Instr::Jdisnan(l1, l2, l3) => resolve!(position, ramstart, resolver, l1, l2, l3),
            Instr::Jdisinf(l1, l2, l3) => resolve!(position, ramstart, resolver, l1, l2, l3),
            Instr::Jdeq(l1, l2, l3, l4, l5, l6, l7) => {
                resolve!(position, ramstart, resolver, l1, l2, l3, l4, l5, l6, l7)
            }
            Instr::Jdne(l1, l2, l3, l4, l5, l6, l7) => {
                resolve!(position, ramstart, resolver, l1, l2, l3, l4, l5, l6, l7)
            }
            Instr::Jdlt(l1, l2, l3, l4, l5) => {
                resolve!(position, ramstart, resolver, l1, l2, l3, l4, l5)
            }
            Instr::Jdle(l1, l2, l3, l4, l5) => {
                resolve!(position, ramstart, resolver, l1, l2, l3, l4, l5)
            }
            Instr::Jdgt(l1, l2, l3, l4, l5) => {
                resolve!(position, ramstart, resolver, l1, l2, l3, l4, l5)
            }
            Instr::Jdge(l1, l2, l3, l4, l5) => {
                resolve!(position, ramstart, resolver, l1, l2, l3, l4, l5)
            }
            Instr::Random(l1, s1) => resolve!(position, ramstart, resolver, l1, s1),
            Instr::Setrandom(l1) => resolve!(position, ramstart, resolver, l1),
            Instr::Mzero(l1, l2) => resolve!(position, ramstart, resolver, l1, l2),
            Instr::Mcopy(l1, l2, l3) => resolve!(position, ramstart, resolver, l1, l2, l3),
            Instr::Linearsearch(l1, l2, l3, l4, l5, l6, l7, s1) => {
                resolve!(position, ramstart, resolver, l1, l2, l3, l4, l5, l6, l7, s1)
            }
            Instr::Binarysearch(l1, l2, l3, l4, l5, l6, l7, s1) => {
                resolve!(position, ramstart, resolver, l1, l2, l3, l4, l5, l6, l7, s1)
            }
            Instr::Linkedsearch(l1, l2, l3, l4, l5, l6, s1) => {
                resolve!(position, ramstart, resolver, l1, l2, l3, l4, l5, l6, s1)
            }
            Instr::Accelfunc(l1, l2) => resolve!(position, ramstart, resolver, l1, l2),
            Instr::Accelparam(l1, l2) => resolve!(position, ramstart, resolver, l1, l2),
            Instr::Gestalt(l1, l2, s1) => resolve!(position, ramstart, resolver, l1, l2, s1),
            Instr::Debugtrap(l1) => resolve!(position, ramstart, resolver, l1),
            Instr::Glk(l1, l2, s1) => resolve!(position, ramstart, resolver, l1, l2, s1),
        };

        Ok(RawInstr { opcode, operands })
    }
}

impl RawInstr {
    /// Returns the serialized length of the instruction.
    pub(crate) fn len(&self) -> usize {
        opcode_len(self.opcode)
            + self.operands.len().div_ceil(2)
            + self.operands.iter().map(|op| op.len()).sum::<usize>()
    }

    /// Serializes the instruction.
    pub(crate) fn serialize<B: BufMut>(&self, mut buf: B) {
        if self.opcode < 0x80 {
            buf.put_u8(
                self.opcode
                    .try_into()
                    .expect("opcode range should have already been checked"),
            )
        } else if self.opcode < 0x4000 {
            buf.put_u16(
                (self.opcode + 0x8000)
                    .try_into()
                    .expect("opcode range should have already been checked"),
            )
        } else {
            buf.put_u32(
                self.opcode
                    .checked_add(0xC0000000)
                    .expect("opcode should not exceed 0x0FFFFFFF"),
            )
        }

        let mut odd = false;
        let mut modebyte: u8 = 0;
        for operand in &self.operands {
            if odd {
                modebyte += operand.mode() << 4;
                buf.put_u8(modebyte);
                odd = false;
            } else {
                modebyte = operand.mode();
                odd = true;
            }
        }

        if odd {
            buf.put_u8(modebyte);
        }

        for operand in &self.operands {
            operand.serialize(&mut buf)
        }
    }
}

/// Returns the serialized length of the opcode.
fn opcode_len(opcode: u32) -> usize {
    if opcode < 0x80 {
        1
    } else if opcode < 0x4000 {
        2
    } else {
        4
    }
}
