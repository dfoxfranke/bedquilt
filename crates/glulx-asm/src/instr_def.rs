// SPDX-License-Identifier: CC-BY-NC-SA-4.0 AND Apache-2.0 WITH LLVM-Exception
// Copyright 2024 Daniel Fox Franke.
// Glulx specification excerpts Copyright 2020-2022 by the Interactive Fiction
// Technology Foundation.

//! Definition of [`Instr`].
//!
//! Keep this definition in its own file separate from any `impl`s, so that it's
//! easy to strip the CC-BY-NC-SA-4.0 stuff if needed.

use crate::operands::{LoadOperand, StoreOperand};

/// Representation of a Glulx instruction.
///
/// **License note**: the rustdoc comments on this enumeration incorporate
/// material from the Glulx VM Specification version 3.1.3, Copyright 2020-2022
/// by the Interactive Fiction Technology Foundation. The specification is
/// licensed under a Creative Commons Attribution-NonCommercial-ShareAlike 4.0
/// International License (SPDX-License-Identifier: CC-BY-NC-SA-4.0). Any
/// redistribution of this crate with this documentation intact must abide by
/// the terms of that license. Although the license is
/// "sharealike/"infectious"/"copyleft", linking against this crate or
/// distributing this crate in binary-only form generally will not bind you to
/// it, since such products generally will not incorporate the documentation and
/// so will not constitute works derived from it. Such activities are permitted
/// subject only to the Apache-2.0 WITH LLVM-Exception license under which this
/// crate's source code is provided.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Instr<L> {
    /// No-op
    Nop,

    // INTEGER MATH
    /// Add L1 and L2, using standard 32-bit addition. Truncate the result to 32
    /// bits if necessary. Store the result in S1.
    Add(LoadOperand<L>, LoadOperand<L>, StoreOperand<L>),
    /// Compute `(L1 - L2)`, and store the result in S1.
    Sub(LoadOperand<L>, LoadOperand<L>, StoreOperand<L>),
    /// Compute `(L1 * L2)`, and store the result in S1. Truncate the result to 32
    /// bits if necessary.
    Mul(LoadOperand<L>, LoadOperand<L>, StoreOperand<L>),
    /// Compute `(L1 / L2)`, and store the result in S1. This is signed integer
    /// division.
    Div(LoadOperand<L>, LoadOperand<L>, StoreOperand<L>),
    /// Compute `(L1 % L2)`, and store the result in S1. This is the remainder
    /// from signed integer division.
    ///
    /// In division and remainer, signs are annoying. Rounding is towards zero.
    /// The sign of a remainder equals the sign of the dividend. It is always
    /// true that `(A / B) * B + (A % B) == A`. Some examples (in decimal):
    ///
    /// ```
    /// 11 /  2 =  5
    /// -11 /  2 = -5
    /// 11 / -2 = -5
    /// -11 / -2 =  5
    /// 13 %  5 =  3
    /// -13 %  5 = -3
    /// 13 % -5 =  3
    /// -13 % -5 = -3
    /// ```
    Mod(LoadOperand<L>, LoadOperand<L>, StoreOperand<L>),
    /// Compute the negative of L1.
    Neg(LoadOperand<L>, StoreOperand<L>),
    /// Compute the bitwise AND of L1 and L2.
    Bitand(LoadOperand<L>, LoadOperand<L>, StoreOperand<L>),
    /// Compute the bitwise OR of L1 and L2.
    Bitor(LoadOperand<L>, LoadOperand<L>, StoreOperand<L>),
    /// Compute the bitwise XOR of L1 and L2.
    Bitxor(LoadOperand<L>, LoadOperand<L>, StoreOperand<L>),
    /// Compute the bitwise negation of L1.
    Bitnot(LoadOperand<L>, StoreOperand<L>),
    /// Shift the bits of L1 to the left (towards more significant bits) by L2
    /// places. The bottom L2 bits are filled in with zeroes. If L2 is 32 or
    /// more, the result is always zero.
    Shiftl(LoadOperand<L>, LoadOperand<L>, StoreOperand<L>),
    /// Shift the bits of L1 to the right by L2 places. The top L2 bits are
    /// filled in with zeroes. If L2 is 32 or more, the result is always zero.
    Ushiftr(LoadOperand<L>, LoadOperand<L>, StoreOperand<L>),
    /// Shift the bits of L1 to the right by L2 places. The top L2 bits are
    /// filled in with copies of the top bit of L1. If L2 is 32 or more, the
    /// result is always zero or `FFFFFFFF`, depending on the top bit of L1.
    Sshiftr(LoadOperand<L>, LoadOperand<L>, StoreOperand<L>),

    // BRANCHING
    /// Branch unconditionally to offset L1.
    Jump(LoadOperand<L>),
    /// If L1 is equal to zero, branch to L2.
    Jz(LoadOperand<L>, LoadOperand<L>),
    /// If L1 is not equal to zero, branch to L2.
    Jnz(LoadOperand<L>, LoadOperand<L>),
    /// If L1 is equal to L2, branch to L3.
    Jeq(LoadOperand<L>, LoadOperand<L>, LoadOperand<L>),
    /// If L1 is not equal to L2, branch to L3.
    Jne(LoadOperand<L>, LoadOperand<L>, LoadOperand<L>),
    /// If L1 is less than L2, branch to L3. The values are compared as signed
    /// 32-bit values.
    Jlt(LoadOperand<L>, LoadOperand<L>, LoadOperand<L>),
    /// If L1 is less than or equal to L2, branch to L3. The values are compared
    /// as signed 32-bit values.
    Jle(LoadOperand<L>, LoadOperand<L>, LoadOperand<L>),
    /// If L1 is greater than L2, branch to L3. The values are compared as
    /// signed 32-bit values.
    Jgt(LoadOperand<L>, LoadOperand<L>, LoadOperand<L>),
    /// If L1 is greater than or equal to L2, branch to L3. The values are
    /// compared as signed 32-bit values.
    Jge(LoadOperand<L>, LoadOperand<L>, LoadOperand<L>),
    /// If L1 is less than L2, branch to L3. The values are compared as unsigned
    /// 32-bit values.
    Jltu(LoadOperand<L>, LoadOperand<L>, LoadOperand<L>),
    /// If L1 is less than or equal to L2, branch to L3. The values are compared
    /// as unsigned 32-bit values.
    Jleu(LoadOperand<L>, LoadOperand<L>, LoadOperand<L>),
    /// If L1 is greater than L2, branch to L3. The values are compared as
    /// unsigned 32-bit values.
    Jgtu(LoadOperand<L>, LoadOperand<L>, LoadOperand<L>),
    /// If L1 is greater than or equal to L2, branch to L3. The values are
    /// compared as unsigned 32-bit values.
    Jgeu(LoadOperand<L>, LoadOperand<L>, LoadOperand<L>),
    /// Branch unconditionally to address L1. Unlike the other branch opcodes,
    /// this takes an absolute address, not an offset. The special cases 0 and 1
    /// (for returning) do not apply; `jumpabs 0` would branch to memory address
    /// 0, if that were ever a good idea, which it isn't.
    Jumpabs(LoadOperand<L>),

    // MOVING DATA
    /// Read L1 and store it at S1, without change.
    Copy(LoadOperand<L>, StoreOperand<L>),
    /// Read a 16-bit value from L1 and store it at S1.
    Copys(LoadOperand<L>, StoreOperand<L>),
    /// Read an 8-bit value from L1 and store it at S1.
    Copyb(LoadOperand<L>, StoreOperand<L>),
    /// Sign-extend a value, considered as a 16-bit value. If the value's `8000`
    /// bit is set, the upper 16 bits are all set; otherwise, the upper 16 bits
    /// are all cleared.
    Sexs(LoadOperand<L>, StoreOperand<L>),
    /// Sign-extend a value, considered as an 8-bit value. If the value's 80 bit
    /// is set, the upper 24 bits are all set; otherwise, the upper 24 bits are
    /// all cleared.
    Sexb(LoadOperand<L>, StoreOperand<L>),

    // ARRAY DATA
    /// Store L3 into the 32-bit field at main memory address `(L1+4*L2)`.
    Astore(LoadOperand<L>, LoadOperand<L>, LoadOperand<L>),
    /// Load a 32-bit value from main memory address `(L1+4*L2)`, and store it in S1.
    Aload(LoadOperand<L>, LoadOperand<L>, StoreOperand<L>),
    /// Store L3 into the 16-bit field at main memory address `(L1+4*L2)`.
    Astores(LoadOperand<L>, LoadOperand<L>, LoadOperand<L>),
    /// Load a 16-bit value from main memory address `(L1+4*L2)`, and store it in S1.
    Aloads(LoadOperand<L>, LoadOperand<L>, StoreOperand<L>),
    /// Store L3 into the 8-bit field at main memory address `(L1+4*L2)`.
    Astoreb(LoadOperand<L>, LoadOperand<L>, LoadOperand<L>),
    /// Load a 8-bit value from main memory address `(L1+4*L2)`, and store it in S1.
    Aloadb(LoadOperand<L>, LoadOperand<L>, StoreOperand<L>),
    /// Set or clear a single bit. This is bit number `(L2 mod 8)` of memory
    /// address `(L1+L2/8)`. It is cleared if L3 is zero, set if nonzero.
    Astorebit(LoadOperand<L>, LoadOperand<L>, LoadOperand<L>),
    /// Test a single bit, similarly. If it is set, 1 is stored at S1; if clear, 0 is stored.
    Aloadbit(LoadOperand<L>, LoadOperand<L>, StoreOperand<L>),

    // THE STACK
    /// Store a count of the number of values on the stack. This counts only
    /// values above the current call-frame. In other words, it is always zero
    /// when a C1 function starts executing, and `(numargs+1)` when a C0
    /// function starts executing. It then increases and decreases thereafter as
    /// values are pushed and popped; it is always the number of values that can
    /// be popped legally. (If S1 uses the stack push mode, the count is done
    /// before the result is pushed.)
    Stkcount(StoreOperand<L>),

    /// Peek at the L1'th value on the stack, without actually popping anything.
    /// If L1 is zero, this is the top value; if one, it's the value below that;
    /// etc. L1 must be less than the current stack-count. (If L1 or S1 use the
    /// stack pop/push modes, the peek is counted after L1 is popped, but before
    /// the result is pushed.)
    Stkpeek(LoadOperand<L>, StoreOperand<L>),

    /// Swap the top two values on the stack. The current stack-count must be at
    /// least two.
    Stkswap,

    /// Peek at the top L1 values in the stack, and push duplicates onto the
    /// stack in the same order. If L1 is zero, nothing happens. L1 must not be
    /// greater than the current stack-count. (If L1 uses the stack pop mode,
    /// the stkcopy is counted after L1 is popped.)
    ///
    /// An example of stkcopy, starting with six values on the stack:
    ///
    /// ```
    /// 5 4 3 2 1 0 <top>
    /// stkcopy 3
    /// 5 4 3 2 1 0 2 1 0 <top>
    /// ```
    Stkcopy(LoadOperand<L>),

    /// Rotate the top L1 values on the stack. They are rotated up or down L2
    /// places, with positive values meaning up and negative meaning down. The
    /// current stack-count must be at least L1. If either L1 or L2 is zero,
    /// nothing happens. (If L1 and/or L2 use the stack pop mode, the roll
    /// occurs after they are popped.)
    ///
    /// An example of two stkrolls, starting with nine values on the stack:
    ///
    /// ```
    /// 8 7 6 5 4 3 2 1 0 <top>
    /// stkroll 5 1
    /// 8 7 6 5 0 4 3 2 1 <top>
    /// stkroll 9 -3
    /// 5 0 4 3 2 1 8 7 6 <top>
    /// ```
    ///
    /// Note that stkswap is equivalent to stkroll 2 1, or for that matter
    /// stkroll 2 -1. Also, stkcopy 1 is equivalent to stkpeek 0 sp.
    ///
    /// These opcodes can only access the values pushed on the stack above the
    /// current call-frame. It is illegal to stkswap, stkpeek, stkcopy, or
    /// stkroll values below that – i.e, the locals segment or any previous
    /// function call frames.
    Stkroll(LoadOperand<L>, LoadOperand<L>),

    // FUNCTIONS
    /// Call function whose address is L1, passing in L2 arguments, and store
    /// the return result at S1.
    ///
    /// The arguments are taken from the stack. Before you execute the call
    /// opcode, you must push the arguments on, in backward order (last argument
    /// pushed first, first argument topmost on the stack.) The L2 arguments are
    /// removed before the new function's call frame is constructed. (If L1, L2,
    /// or S1 use the stack pop/push modes, the arguments are taken after L1 or
    /// L2 is popped, but before the result is pushed.)
    ///
    /// Recall that all functions in Glulx have a single 32-bit return value. If
    /// you do not care about the return value, you can use operand mode 0
    /// ("discard value") for operand S1.
    Call(LoadOperand<L>, LoadOperand<L>, StoreOperand<L>),
    /// Call function whose address is L1, passing no arguments. Store the
    /// return result at S1.
    Callf(LoadOperand<L>, StoreOperand<L>),
    /// Call function whose address is L1, passing one argument as L2. Store the
    /// return result at S1.
    Callfi(LoadOperand<L>, LoadOperand<L>, StoreOperand<L>),
    /// Call function whose address is L1, passing two argument as L2/L3. Store
    /// the return result at S1.
    Callfii(
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        StoreOperand<L>,
    ),
    /// Call function whose address is L1, passing three argument as L2/L3/L4.
    /// Store the return result at S1.
    Callfiii(
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        StoreOperand<L>,
    ),
    /// Return from the current function, with the given return value. If this
    /// is the top-level function, Glulx execution is over.
    Return(LoadOperand<L>),

    ///Call function whose address is L1, passing in L2 arguments, and pass the
    ///return result out to whoever called the current function.
    ///
    /// This destroys the current call-frame, as if a return had been executed,
    /// but does not touch the call stub below that. It then immediately calls
    /// L1, creating a new call-frame. The effect is the same as a call
    /// immediately followed by a return, but takes less stack space.
    ///
    /// It is legal to use tailcall from the top-level function. L1 becomes the
    /// top-level function.
    Tailcall(LoadOperand<L>, LoadOperand<L>),

    // CONTINUATIONS
    /// Generates a "catch token", which can be used to jump back to this
    /// execution point from a throw opcode. The token is stored in S1, and then
    /// execution branches to offset L1. If execution is proceeding from this
    /// point because of a throw, the thrown value is stored instead, and the
    /// branch is ignored.
    ///
    /// Remember if the branch value is not 0 or 1, the branch is to to (Addr +
    /// L1 - 2), where Addr is the address of the instruction after the catch.
    /// If the value is 0 or 1, the function returns immediately, invalidating
    /// the catch token.
    ///
    ///If S1 or L1 uses the stack push/pop modes, note that the precise order of
    ///execution is: evaluate L1 (popping if appropriate); generate a call stub
    ///and compute the token; store S1 (pushing if appropriate).
    Catch(StoreOperand<L>, LoadOperand<L>),

    /// Jump back to a previously-executed catch opcode, and store the value L1.
    /// L2 must be a valid catch token.
    ///
    /// The exact catch/throw procedure is as follows:
    ///
    /// When catch is executed, a four-value call stub is pushed on the stack –
    /// result destination, PC, and FramePtr. (See section 1.3.2, "Call Stubs".
    /// The PC is the address of the next instruction after the catch.) The
    /// catch token is the value of the stack pointer after these are pushed.
    /// The token value is stored in the result destination, and execution
    /// proceeds, branching to L1.
    ///
    /// When throw is executed, the stack is popped down until the stack pointer
    /// equals the given token. Then the four values are read back off the
    /// stack, the thrown value is stored in the destination, and execution
    /// proceeds with the instruction after the catch.
    ///
    /// If the call stub (or any part of it) is removed from the stack, the
    /// catch token becomes invalid, and must not be used. This will certainly
    /// occur when you return from the function containing the catch opcode. It
    /// will also occur if you pop too many values from the stack after
    /// executing the catch. (You may wish to do this to "cancel" the catch; if
    /// you pop and discard those four values, the token is invalidated, and it
    /// is as if you had never executed the catch at all.) The catch token is
    /// also invalidated if any part of the call stub is overwritten (e.g. with
    /// stkswap or stkroll).
    Throw(LoadOperand<L>, LoadOperand<L>),

    // MEMORY MAP
    /// Store the current size of the memory map. This is originally the ENDMEM
    /// value from the header, but you can change it with the setmemsize opcode.
    /// (The malloc and mfree opcodes may also cause this value to change; see
    /// section 2.9, "Memory Allocation Heap".) It will always be greater than
    /// or equal to ENDMEM, and will always be a multiple of 256.
    Getmemsize(StoreOperand<L>),

    /// Set the current size of the memory map. The new value must be a multiple
    /// of 256, like all memory boundaries in Glulx. It must be greater than or
    /// equal to ENDMEM (the initial memory-size value which is stored in the
    /// header.) It does not have to be greater than the previous memory size.
    /// The memory size may grow and shrink over time, as long as it never gets
    /// smaller than the initial size.
    ///
    /// When the memory size grows, the new space is filled with zeroes. When it
    /// shrinks, the contents of the old space are lost.
    ///
    /// If the allocation heap is active (see section 2.9, "Memory Allocation
    /// Heap") you may not use setmemsize – the memory map is under the control
    /// of the heap system. If you free all heap objects, the heap will then no
    /// longer be active, and you can use setmemsize.
    ///
    /// Since memory allocation is never guaranteed, you must be prepared for
    /// the possibility that setmemsize will fail. The opcode stores the value
    /// zero if it succeeded, and 1 if it failed. If it failed, the memory size
    /// is unchanged.
    ///
    /// Some interpreters do not have the capability to resize memory at all. On
    /// such interpreters, setmemsize will always fail. You can check this in
    /// advance with the ResizeMem gestalt selector.
    ///
    /// Note that the memory size is considered part of the game state. If you
    /// restore a saved game, the current memory size is changed to the size
    /// that was in effect when the game was saved. If you restart, the current
    /// memory size is reset to its initial value.
    Setmemsize(LoadOperand<L>, StoreOperand<L>),

    /// Manage the memory allocation heap.
    ///
    /// Allocate a memory block of L1 bytes. (L1 must be positive.) This stores
    /// the address of the new memory block, which will be within the heap and
    /// will not overlap any other extant block. The interpreter may have to
    /// extend the memory map (see section 2.8, "Memory Map") to accomodate the
    /// new block.
    ///
    /// This operation does not change the contents of the memory block (or,
    /// indeed, the contents of the memory map at all). If you want the memory
    /// block to be initialized, you must do it yourself.
    ///
    /// If the allocation fails, this stores zero.
    ///
    /// Glulx is able to maintain a list of dynamically-allocated memory
    /// objects. These objects exist in the memory map, above ENDMEM. The malloc
    /// and mfree opcodes allow the game to request the allocation and
    /// destruction of these objects.
    ///
    /// Some interpreters do not have the capability to manage an allocation
    /// heap. On such interpreters, malloc will always fail. You can check this
    /// in advance with the MAlloc gestalt selector.
    ///
    /// When you first allocate a block of memory, the heap becomes active. The
    /// current end of memory – that is, the current getmemsize value – becomes
    /// the beginning address of the heap. The memory map is then extended to
    /// accomodate the memory block.
    ///
    /// Subsequent memory allocations and deallocations are done within the
    /// heap. The interpreter may extend or reduce the memory map, as needed,
    /// when allocations and deallocations occur. While the heap is active, you
    /// may not manually resize the memory map with setmemsize; the heap system
    /// is responsible for doing that.
    ///
    /// When you free the last extant memory block, the heap becomes inactive.
    /// The interpreter will reduce the memory map size down to the heap-start
    /// address. (That is, the getmemsize value returns to what it was before
    /// you allocated the first block.) Thereafter, it is legal to call
    /// setmemsize again.
    ///
    /// It is legitimate to read or write any memory address in the heap range
    /// (from ENDMEM to the end of the memory map). You are not restricted to
    /// extant blocks. [The VM's heap state is not stored in its own memory map.
    /// So, unlike the familiar C heap, you cannot damage it by writing outside
    /// valid blocks.]
    ///
    ///The heap state (whether it is active, its starting address, and the
    ///addresses and sizes of all extant blocks) is part of the saved game
    ///state.
    Malloc(LoadOperand<L>, StoreOperand<L>),

    /// Free the memory block at address L1. This must be the address of an
    /// extant block – that is, a value returned by malloc and not previously
    /// freed.
    ///
    /// This operation does not change the contents of the memory block (or,
    /// indeed, the contents of the memory map at all).
    Mfree(LoadOperand<L>),

    // GAME STATE
    /// Shut down the terp and exit. This is equivalent to returning from the
    /// top-level function, or for that matter calling glk_exit().
    ///
    /// Note that (in the Glk I/O system) Glk is responsible for any "hit any
    /// key to exit" prompt. It is safe for you to print a bunch of final text
    /// and then exit immediately.
    Quit,

    /// Restore the VM to its initial state (memory, stack, and registers). Note
    /// that the current memory size is reset, as well as the contents of
    /// memory.
    Restart,

    /// Save the VM state to the output stream L1. It is your responsibility to
    /// prompt the player for a filespec, open the stream, and then destroy
    /// these objects afterward. S1 is set to zero if the operation succeeded, 1
    /// if it failed, and -1 if the VM has just been restored and is continuing
    /// from this instruction.

    /// (In the Glk I/O system, L1 should be the ID of a writable Glk stream. In
    /// other I/O systems, it will mean something different. In the "filter" and
    /// "null" I/O systems, the save opcode is illegal, as the interpreter has
    /// nowhere to write the state.)
    Save(LoadOperand<L>, StoreOperand<L>),

    /// Restore the VM state from the input stream L1. S1 is set to 1 if the
    /// operation failed. If it succeeded, of course, this instruction never
    /// returns a value.
    Restore(LoadOperand<L>, StoreOperand<L>),

    /// Save the VM state in a temporary location. The terp will choose a
    /// location appropriate for rapid access, so this may be called once per
    /// turn. S1 is set to zero if the operation succeeded, 1 if it failed, and
    /// -1 if the VM state has just been restored.
    Saveundo(StoreOperand<L>),

    /// Restore the VM state from temporary storage. S1 is set to 1 if the
    /// operation failed.
    Restoreundo(StoreOperand<L>),

    /// Test whether a VM state is available in temporary storage. S1 is set to
    /// 0 if a state is available, 1 if not. If this returns 0, then restoreundo
    /// is expected to succeed.
    Hasundo(StoreOperand<L>),

    /// Discard a VM state (the most recently saved) from temporary storage. If
    /// none is available, this does nothing.
    ///
    /// The hasundo and discardundo opcodes were added in Glulx 3.1.3. You can
    /// check for their existence with the ExtUndo gestalt selector.
    Discardundo,

    /// Protect a range of memory from restart, restore, restoreundo. The
    /// protected range starts at address L1 and has a length of L2 bytes. This
    /// memory is silently unaffected by the state-restoring operations.
    /// (However, if the result-storage S1 is directed into the protected range,
    /// that is not blocked.)
    ///
    /// When the VM starts up, there is no protection range. Only one range can
    /// be protected at a time. Calling protect cancels any previous range. To
    /// turn off protection, call protect with L1 and L2 set to zero.
    ///
    /// It is important to note that the protection range itself (its existence,
    /// location, and length) is not part of the saved game state! If you save a
    /// game, move the protection range to a new location, and then restore that
    /// game, it is the new range that will be protected, and the range will
    /// remain there afterwards.
    Protect(LoadOperand<L>, LoadOperand<L>),

    /// Perform sanity checks on the game file, using its length and checksum.
    /// S1 is set to zero if everything looks good, 1 if there seems to be a
    /// problem. (Many interpreters will do this automatically, before the game
    /// starts executing. This opcode is provided mostly for slower
    /// interpreters, where auto-verify might cause an unacceptable delay.)
    Verify(StoreOperand<L>),

    // OUTPUT
    /// Return the current I/O system mode and rock.
    ///
    /// Due to a long-standing bug in the reference interpreter, the two store
    /// operands must be of the same general type: both main-memory/global
    /// stores, both local variable stores, or both stack pushes.
    Getiosys(StoreOperand<L>, StoreOperand<L>),

    /// Set the I/O system mode and rock. If the system L1 is not supported by
    /// the interpreter, it will default to the "null" system (0).

    /// These systems are currently defined:

    /// * 0: The null system. All output is discarded. (When the Glulx machine
    ///    starts up, this is the current system.)
    ///
    /// * 1: The filtering system. The rock (L2) value should be the address of a
    ///    Glulx function. This function will be called for every character output
    ///    (with the character value as its sole argument). The function's return
    ///    value is ignored.
    ///
    /// * 2: The Glk system. All output will be handled through Glk function
    ///    calls, sent to the current Glk stream.
    ///
    /// * 20: The FyreVM channel system. See section 0.2, "Glulx and Other IF
    ///   Systems".
    ///
    /// The values 140-14F are reserved for extension projects by ZZO38. These
    /// are not documented here.
    ///
    /// It is important to recall that when Glulx starts up, the Glk I/O system
    /// is not set. And when Glk starts up, there are no windows and no current
    /// output stream. To make anything appear to the user, you must first do
    /// three things: select the Glk I/O system, open a Glk window, and set its
    /// stream as the current one. (It is illegal in Glk to send output when
    /// there is no stream set. Sending output to Glulx's "null" I/O system is
    /// legal, but pointless.)
    Setiosys(LoadOperand<L>, LoadOperand<L>),

    /// Send L1 to the current stream. This sends a single character; the value L1 is truncated to eight bits.
    Streamchar(LoadOperand<L>),

    /// Send L1 to the current stream. This sends a single (32-bit) character.
    ///
    /// This opcode was added in Glulx version 3.0.
    Streamunichar(LoadOperand<L>),

    /// Send L1 to the current stream, represented as a signed decimal number in ASCII.
    Streamnum(LoadOperand<L>),

    /// Send a string object to the current stream. L1 must be the address of a
    /// Glulx string object (type E0, E1, or E2.) The string is decoded and sent
    /// as a sequence of characters.
    ///
    /// When the Glk I/O system is set, these opcodes are implemented using the
    /// Glk API. You can bypass them and directly call glk_put_char(),
    /// glk_put_buffer(), and so on. Remember, however, that glk_put_string()
    /// only accepts unencoded string (E0) objects; glk_put_string_uni() only
    /// accepts unencoded Unicode (E2) objects.
    ///
    /// Note that it is illegal to decode a compressed string (E1) if there is
    /// no string-decoding table set.
    Streamstr(LoadOperand<L>),

    /// Return the address the terp is currently using for its string-decoding
    /// table. If there is no table, set, this returns zero.
    Getstringtbl(StoreOperand<L>),

    /// Change the address the terp is using for its string-decoding table. This
    /// may be zero, indicating that there is no table (in which case it is
    /// illegal to print any compressed string). Otherwise, it must be the
    /// address of a valid string-decoding table.
    Setstringtbl(LoadOperand<L>),

    /// Convert an integer value to the closest equivalent float. (That is, if
    /// L1 is 1, then 3F800000 – the float encoding of 1.0 – will be stored in
    /// S1.) Integer zero is converted to (positive) float zero.
    ///
    /// If the value is less than -1000000 or greater than 1000000 (hex), the
    /// conversion may not be exact. (More specifically, it may round to a
    /// nearby multiple of a power of 2.)
    Numtof(LoadOperand<L>, StoreOperand<L>),

    /// Convert a float value to an integer, rounding towards zero (i.e.,
    /// truncating the fractional part). If the value is outside the 32-bit
    /// integer range, or is NaN or infinity, the result will be 7FFFFFFF (for
    /// positive values) or 80000000 (for negative values).
    Ftonumz(LoadOperand<L>, StoreOperand<L>),

    /// Convert a float value to an integer, rounding towards the nearest
    /// integer. Again, overflows become 7FFFFFFF or 80000000.\
    Ftonumn(LoadOperand<L>, StoreOperand<L>),

    /// Floating point addition. Overflows produce infinite values (with the
    /// appropriate sign); underflows produce zero values (ditto). 0/0 is NaN.
    /// Inf/Inf, or Inf-Inf, is NaN. Any finite number added to infinity is
    /// infinity. Any nonzero number divided by an infinity, or multiplied by
    /// zero, is a zero. Any nonzero number multiplied by an infinity, or
    /// divided by zero, is an infinity.
    Fadd(LoadOperand<L>, LoadOperand<L>, StoreOperand<L>),

    ///  Floating pointing subtraction.
    Fsub(LoadOperand<L>, LoadOperand<L>, StoreOperand<L>),

    ///  Floating pointing multiplication.
    Fmul(LoadOperand<L>, LoadOperand<L>, StoreOperand<L>),

    ///  Floating pointing division.
    Fdiv(LoadOperand<L>, LoadOperand<L>, StoreOperand<L>),

    ///  Perform a floating-point modulo operation. S1 is the remainder (or
    ///  modulus); S2 is the quotient.
    ///
    /// S2 is L1/L2, rounded (towards zero) to an integral value. S1 is
    /// L1-(S2*L2). Note that S1 always has the same sign as L1; S2 has the
    /// appropriate sign for L1/L2.
    ///
    /// If L2 is 1, this gives you the fractional and integer parts of L1. If L1
    /// is zero, both results are zero. If L2 is infinite, S1 is L1 and S2 is
    /// zero. If L1 is infinite or L2 is zero, both results are NaN.
    Fmod(LoadOperand<L>, LoadOperand<L>, StoreOperand<L>),

    /// Round L1 up (towards +Inf) to the nearest integral value. (The result is
    /// still in float format, however.) These opcodes are idempotent.
    ///
    /// The result keeps the sign of L1; in particular, floor(0.5) is 0 and
    /// ceil(−0.5) is −0. Rounding −0 up or down gives −0. Rounding an infinite
    /// value gives infinity.
    Ceil(LoadOperand<L>, StoreOperand<L>),

    /// Round L1 down (toward -Inf) to the nearest integral value.
    Floor(LoadOperand<L>, StoreOperand<L>),

    /// Compute the square root of L1.
    ///
    /// sqrt(−0) is −0. sqrt returns NaN for all other negative values.
    Sqrt(LoadOperand<L>, StoreOperand<L>),
    /// Compute exp(L1).
    ///
    /// exp(+0) and exp(−0) are 1; exp(−Inf) is +0.
    Exp(LoadOperand<L>, StoreOperand<L>),
    /// Compute ln(L1).
    ///
    /// log(+0) and log(−0) are −Inf. log returns NaN for all other negative values.
    Log(LoadOperand<L>, StoreOperand<L>),

    /// Compute L1 raised to the L2 power.
    Pow(LoadOperand<L>, LoadOperand<L>, StoreOperand<L>),

    /// Standard trigonometric sine function.
    Sin(LoadOperand<L>, StoreOperand<L>),
    /// Standard trigonometric cosine function.
    Cos(LoadOperand<L>, StoreOperand<L>),
    /// Standard trigonometric tangent function.
    Tan(LoadOperand<L>, StoreOperand<L>),
    /// Standard trigonometric arcsine function.
    Asin(LoadOperand<L>, StoreOperand<L>),
    /// Standard trigonometric arccosine function.
    Acos(LoadOperand<L>, StoreOperand<L>),
    /// Standard trigonometric arctangent function.
    Atan(LoadOperand<L>, StoreOperand<L>),
    /// Computes the arctangent of L1/L2, using the signs of both arguments to
    /// determine the quadrant of the return value. (Note that the Y argument is
    /// first and the X argument is second.)
    Atan2(LoadOperand<L>, StoreOperand<L>),

    /// Convert an integer value to the closest equivalent double. Integer zero
    /// is converted to (positive) double zero. The result is stored as S2:S1.
    Numtod(LoadOperand<L>, StoreOperand<L>, StoreOperand<L>),

    /// Convert a double value L1:L2 to an integer, rounding towards zero (i.e.,
    /// truncating the fractional part). If the value is outside the 32-bit
    /// integer range, or is NaN or infinity, the result will be 7FFFFFFF (for
    /// positive values) or 80000000 (for negative values).
    Dtonumz(LoadOperand<L>, LoadOperand<L>, StoreOperand<L>),

    /// Convert a double value L1:L2 to an integer, rounding towards the nearest
    /// integer. Again, overflows become 7FFFFFFF or 80000000.
    Dtonumn(LoadOperand<L>, LoadOperand<L>, StoreOperand<L>),

    /// Convert a float value L1 to a double value, stored as S2:S1.
    Ftod(LoadOperand<L>, StoreOperand<L>, StoreOperand<L>),

    /// Convert a double value L1:L2 to a float value, stored as S1.
    Dtof(LoadOperand<L>, LoadOperand<L>, StoreOperand<L>),

    /// Add doubles. The arguments are L1:L2 and L3:L4; the result is stored as S2:S1.
    Dadd(
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        StoreOperand<L>,
        StoreOperand<L>,
    ),

    /// Subtract doubles. The arguments are L1:L2 and L3:L4; the result is stored as S2:S1.
    Dsub(
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        StoreOperand<L>,
        StoreOperand<L>,
    ),

    /// Multiply doubles. The arguments are L1:L2 and L3:L4; the result is stored as S2:S1.
    Dmul(
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        StoreOperand<L>,
        StoreOperand<L>,
    ),

    /// Divide doubles. The arguments are L1:L2 and L3:L4; the result is stored as S2:S1.
    Ddiv(
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        StoreOperand<L>,
        StoreOperand<L>,
    ),

    /// Get the remainder of a floating point modulo operation. The arguments
    /// are L1:L2 and L3:L4; the result is stored as S2:S1.
    ///
    /// Unlike fmod, there are separate opcodes to compute the remainder and
    /// modulus.
    Dmodr(
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        StoreOperand<L>,
        StoreOperand<L>,
    ),

    /// Get the quotient of a float point modulo operation. The arguments are
    /// L1:L2 and L3:L4; the result is stored as S2:S1.
    ///
    /// Unlike fmod, there are separate opcodes to compute the remainder and
    /// modulus.
    Dmodq(
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        StoreOperand<L>,
        StoreOperand<L>,
    ),

    /// Round L1:L2 up (towards +Inf) to the nearest integral value. (The result
    /// is still in double format, however.) The result is stored as S2:S1.
    /// These opcodes are idempotent.
    Dceil(
        LoadOperand<L>,
        LoadOperand<L>,
        StoreOperand<L>,
        StoreOperand<L>,
    ),

    /// Round L1:L2 down (towards −Inf) to the nearest integral value. (The
    /// result is still in double format, however.) The result is stored as
    /// S2:S1. These opcodes are idempotent.
    Dfloor(
        LoadOperand<L>,
        LoadOperand<L>,
        StoreOperand<L>,
        StoreOperand<L>,
    ),

    /// Compute the square root of L1:L2.
    Dsqrt(
        LoadOperand<L>,
        LoadOperand<L>,
        StoreOperand<L>,
        StoreOperand<L>,
    ),

    /// Compute exp(L1:L2).
    Dexp(
        LoadOperand<L>,
        LoadOperand<L>,
        StoreOperand<L>,
        StoreOperand<L>,
    ),

    /// Compute ln(L1:L2).
    Dlog(
        LoadOperand<L>,
        LoadOperand<L>,
        StoreOperand<L>,
        StoreOperand<L>,
    ),

    /// Compute L1:L2 raised to the L3:L4 power. The result is stored as S2:S1.
    Dpow(
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        StoreOperand<L>,
        StoreOperand<L>,
    ),

    /// Compute the standard trigonometric sine of (L1:L2).
    Dsin(
        LoadOperand<L>,
        LoadOperand<L>,
        StoreOperand<L>,
        StoreOperand<L>,
    ),
    /// Compute the standard trigonometric cosine of (L1:L2).
    Dcos(
        LoadOperand<L>,
        LoadOperand<L>,
        StoreOperand<L>,
        StoreOperand<L>,
    ),
    /// Compute the standard trigonometric tangent of (L1:L2).
    Dtan(
        LoadOperand<L>,
        LoadOperand<L>,
        StoreOperand<L>,
        StoreOperand<L>,
    ),
    /// Compute the standard trigonometric arcsine of (L1:L2).
    Dasin(
        LoadOperand<L>,
        LoadOperand<L>,
        StoreOperand<L>,
        StoreOperand<L>,
    ),
    /// Compute the standard trigonometric arccosine of (L1:L2).
    Dacos(
        LoadOperand<L>,
        LoadOperand<L>,
        StoreOperand<L>,
        StoreOperand<L>,
    ),
    /// Compute the standard trigonometric arctangent of (L1:L2).
    Datan(
        LoadOperand<L>,
        LoadOperand<L>,
        StoreOperand<L>,
        StoreOperand<L>,
    ),

    /// Computes the arctangent of L1:L2/L3:L4, using the signs of both
    /// arguments to determine the quadrant of the return value. (Note that the
    /// Y argument is first and the X argument is second.) The result is stored
    /// as S2:S1.
    Datan2(
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        StoreOperand<L>,
        StoreOperand<L>,
    ),

    // FLOATING POINT COMPARISONS
    /// Branch to L2 if the floating-point value L1 is a NaN value.
    Jisnan(LoadOperand<L>, LoadOperand<L>),

    /// Branch to L2 if the floating-point value L1 is an infinity (7F800000 or FF800000).
    Jisinf(LoadOperand<L>, LoadOperand<L>),

    /// Branch to L4 if the difference between L1 and L2 is less than or equal
    /// to (plus or minus) L3. The sign of L3 is ignored.
    ///
    /// If any of the arguments are NaN, this will not branch. If L3 is
    /// infinite, this will always branch – unless L1 and L2 are opposite
    /// infinities. (Opposite infinities are never equal, regardless of L3.
    /// Infinities of the same sign are always equal.)
    ///
    /// If L3 is (plus or minus) zero, this tests for exact equality. Note that
    /// +0 is considered exactly equal to −0.
    Jfeq(
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
    ),

    /// The reverse of jfeq. This will branch if any of the arguments is NaN.
    Jfne(
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
    ),

    /// Branch to L3 if L1 is less than L2.
    Jflt(LoadOperand<L>, LoadOperand<L>, LoadOperand<L>),
    /// Branch to L3 if L1 is less than or equal to L2.
    Jfle(LoadOperand<L>, LoadOperand<L>, LoadOperand<L>),
    /// Branch to L3 if L1 is greater than L2.
    Jfgt(LoadOperand<L>, LoadOperand<L>, LoadOperand<L>),
    /// Branch to L3 if L1 is greater than or equal to L2.
    Jfge(LoadOperand<L>, LoadOperand<L>, LoadOperand<L>),

    // DOUBLE PRECISION COMPARISONS
    /// Branch to L3 if the double value L1:L2 is a NaN value.
    Jdisnan(LoadOperand<L>, LoadOperand<L>, LoadOperand<L>),

    /// Branch to L3 if the double value L1:L2 is an infinity.
    Jdisinf(LoadOperand<L>, LoadOperand<L>, LoadOperand<L>),

    /// Branch to L7 if the difference between L1:L2 and L3:L4 is less than or
    /// equal to (plus or minus) L5:L6. The sign of L5:L6 is ignored.
    Jdeq(
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
    ),

    /// The reverse of jdeq
    Jdne(
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
    ),

    /// Branch to L5 if L1:L2 is less than L3:L4.
    Jdlt(
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
    ),
    /// Branch to L5 if L1:L2 is less than or equal to L3:L4.
    Jdle(
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
    ),
    /// Branch to L5 if L1:L2 is greater than L3:L4.
    Jdgt(
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
    ),
    /// Branch to L5 if L1:L2 is greater than or equal to L3:L4.
    Jdge(
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
    ),

    // RANDOM NUMBER GENERATOR
    /// Return a random number in the range 0 to (L1-1); or, if L1 is negative,
    /// the range (L1+1) to 0. If L1 is zero, return a random number in the full
    /// 32-bit integer range. (Remember that this may be either positive or
    /// negative.)
    Random(LoadOperand<L>, StoreOperand<L>),

    /// Seed the random-number generator with the value L1. If L1 is zero,
    /// subsequent random numbers will be as genuinely unpredictable as the terp
    /// can provide; it may include timing data or other random sources in its
    /// generation. If L1 is nonzero, subsequent random numbers will follow a
    /// deterministic sequence, always the same for a given nonzero seed.
    ///
    /// The terp starts up in the "nondeterministic" mode (as if setrandom 0 had
    /// been invoked.)
    ///
    /// The random-number generator is not part of the saved-game state.
    Setrandom(LoadOperand<L>),

    // BLOCK COPY AND CLEAR
    /// Write L1 zero bytes, starting at address L2.
    Mzero(LoadOperand<L>, LoadOperand<L>),

    /// Copy L1 bytes from address L2 to address L3. It is safe to copy a block to an overlapping block.
    Mcopy(LoadOperand<L>, LoadOperand<L>, LoadOperand<L>),

    // SEARCHING
    /// Accelerated linear search.
    ///
    /// * L1: Key
    /// * L2: KeySize
    /// * L3: Start
    /// * L4: StructSize
    /// * L5: NumStructs
    /// * L6: KeyOffset
    /// * L7: Options
    /// * S1: Result
    ///
    /// An array of data structures is stored in memory, beginning at `Start`,
    /// each structure being `StructSize` bytes. Within each struct, there is a
    /// key value `KeySize` bytes long, starting at position `KeyOffset` (from
    /// the start of the structure.) Search through these in order. If one is
    /// found whose key matches, return it. If `NumStructs` are searched with no
    /// result, the search fails.
    ///
    /// NumStructs may be -1 (`0xFFFFFFFF``) to indicate no upper limit to the
    /// number of structures to search. The search will continue until a match
    /// is found, or (if `ZeroKeyTerminates`` is used) a zero key.  
    ///
    /// The following options may be set in `L7`:
    /// * KeyIndirect (`0x01`): This flag indicates that the `Key`` argument passed
    ///   to the opcode is the address of the actual key. If this flag is not
    ///   used, the Key argument is the key value itself. (In this case, the
    ///   KeySize must be 1, 2, or 4 – the native sizes of Glulx values. If the
    ///   KeySize is 1 or 2, the lower bytes of the Key are used and the upper
    ///   bytes ignored.)
    /// * ZeroKeyTerminates (`0x02`): This flag indicates that the search should
    ///   stop (and return failure) if it encounters a structure whose key is
    ///   all zeroes. If the searched-for key happens to also be all zeroes, the
    ///   success takes precedence.
    /// * ReturnIndex (`0x04`): This flag indicates that search should return the
    ///   array index of the structure that it finds, or -1 (`0xFFFFFFFF`) for
    ///   failure. If this flag is not used, the search returns the address of
    ///   the structure that it finds, or 0 for failure.
    Linearsearch(
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        StoreOperand<L>,
    ),
    /// Accelerated binary search.
    ///
    /// * L1: Key
    /// * L2: KeySize
    /// * L3: Start
    /// * L4: StructSize
    /// * L5: NumStructs
    /// * L6: KeyOffset
    /// * L7: Options
    /// * S1: Result
    ///
    /// An array of data structures is in memory, as above. However, the structs
    /// must be stored in forward order of their keys (taking each key to be a
    /// big-endian unsigned integer.) There can be no duplicate keys. `NumStructs``
    /// must indicate the exact length of the array; it cannot be -1.
    ///
    /// The `KeyIndirect`` and `ReturnIndex`` options may be used.
    Binarysearch(
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        StoreOperand<L>,
    ),
    /// Accelerated linked-list search.
    ///
    /// * L1: Key
    /// * L2: KeySize
    /// * L3: Start
    /// * L4: KeyOffset
    /// * L5: NextOffset
    /// * L6: Options
    /// * S1: Result
    ///
    /// The structures need not be consecutive; they may be anywhere in memory,
    /// in any order. They are linked by a four-byte address field, which is
    /// found in each struct at position NextOffset. If this field contains
    /// zero, it indicates the end of the linked list.
    ///
    /// The KeyIndirect and ZeroKeyTerminates options may be used.
    Linkedsearch(
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        LoadOperand<L>,
        StoreOperand<L>,
    ),

    // ACCELERATED FUNCTIONS
    /// Request that the VM function with address L2 be replaced by the
    /// accelerated function whose number is L1. If L1 is zero, the acceleration
    /// for address L2 is cancelled.
    ///
    /// If the terp does not offer accelerated function L1, this does nothing.
    ///
    /// If you request acceleration at an address which is already accelerated,
    /// the previous request is cancelled before the new one is considered. If
    /// you cancel at an unaccelerated address, nothing happens.
    ///
    /// A given accelerated function L1 may replace several VM functions (at
    /// different addresses) at the same time. Each request is considered
    /// separate, and must be cancelled separately.
    Accelfunc(LoadOperand<L>, LoadOperand<L>),
    /// Store the value L2 in the parameter table at position L1. If the terp
    /// does not know about parameter L1, this does nothing.
    Accelparam(LoadOperand<L>, LoadOperand<L>),

    // MISCELLANEOUS
    /// Test the Gestalt selector number L1, with optional extra argument L2,
    /// and store the result in S1. If the selector is not known, store zero.

    /// The list of L1 selectors is as follows. Note that if a selector does not
    /// mention L2, you should always set that argument to zero.

    /// * GlulxVersion (0): Returns the version of the Glulx spec which the
    ///   interpreter implements. The upper 16 bits of the value contain a major
    ///   version number; the next 8 bits contain a minor version number; and
    ///   the lowest 8 bits contain an even more minor version number, if any.
    ///   This specification is version 3.1.3, so a terp implementing it would
    ///   return 0x00030103. Future Glulx specs will try to maintain the
    ///   convention that minor version changes are backwards compatible, and
    ///   subminor version changes are backwards and forwards compatible.
    /// * TerpVersion (1): Returns the version of the interpreter. The format is
    ///   the same as the GlulxVersion. [Each interpreter has its own version
    ///   numbering system, defined by its author, so this information is not
    ///   terribly useful. But it is convenient for the game to be able to
    ///   display it, in case the player is capturing version information for a
    ///   bug report.]
    /// * ResizeMem (2): Returns 1 if the terp has the potential to resize the
    ///   memory map, with the setmemsize opcode. If this returns 0, setmemsize
    ///   will always fail. [But remember that setmemsize might fail in any
    ///   case.]
    /// * Undo (3): Returns 1 if the terp has the potential to undo. If this
    ///   returns 0, saveundo, restoreundo, and hasundo will always fail.
    /// * IOSystem (4): Returns 1 if the terp supports the I/O system given in
    ///   L2. (The constants are the same as for the setiosys opcode: 0 for
    ///   null, 1 for filter, 2 for Glk, 20 for FyreVM. 0 and 1 will always
    ///   succeed.)
    /// * Unicode (5): Returns 1 if the terp supports Unicode operations. These
    ///   are: the E2 Unicode string type; the 04 and 05 string node types (in
    ///   compressed strings); the streamunichar opcode; the type-14 call stub.
    ///   If the Unicode selector returns 0, encountering any of these will
    ///   cause a fatal interpreter error.
    /// * MemCopy (6): Returns 1 if the interpreter supports the mzero and mcopy
    ///   opcodes. (This must true for any terp supporting Glulx 3.1.)
    /// * MAlloc (7): Returns 1 if the interpreter supports the malloc and mfree
    ///   opcodes. (If this is true, MemCopy and ResizeMem must also both be
    ///   true, so there is no need to check all three.)
    /// * MAllocHeap (8): Returns the start address of the heap. This is the
    ///   value that getmemsize had when the first memory block was allocated.
    ///   If the heap is not active (no blocks are extant), this returns zero.
    /// * Acceleration (9): Returns 1 if the interpreter supports the accelfunc
    ///   and accelparam opcodes. (This must true for any terp supporting Glulx
    ///   3.1.1.)
    /// * AccelFunc (10): Returns 1 if the terp implements the accelerated
    ///   function given in L2.
    /// * Float (11): Returns 1 if the interpreter supports the floating-point
    ///   arithmetic opcodes.
    /// * ExtUndo (12): Returns 1 if the interpreter supports the hasundo and
    ///   discardundo opcodes.
    /// * Double (13): Returns 1 if the interpreter supports the
    ///   double-precision floating-point arithmetic opcodes.
    ///
    /// Selectors 0x1000 to 0x10FF are reserved for use by FyreVM. Selectors
    /// 0x1100 to 0x11FF are reserved for extension projects by Dannii Willis.
    /// Selectors 0x1200 to 0x12FF are reserved for iOS extension features by
    /// Andrew Plotkin. Selectors 0x1400 to 0x14FF are reserved for iOS
    /// extension features by ZZO38.
    Gestalt(LoadOperand<L>, LoadOperand<L>, StoreOperand<L>),

    /// Interrupt execution to do something interpreter-specific with L1. If the
    /// interpreter has nothing in mind, it should halt with a visible error
    /// message.
    Debugtrap(LoadOperand<L>),

    /// Call the Glk API function whose identifier is L1, passing in L2
    /// arguments. The return value is stored at S1. (If the Glk function has no
    /// return value, zero is stored at S1.)

    /// The arguments are passed on the stack, last argument pushed first, just
    /// as for the call opcode.

    /// Arguments should be represented in the obvious way. Integers and
    /// character are passed as integers. Glk opaque objects are passed as
    /// integer identifiers, with zero representing NULL. Strings and Unicode
    /// strings are passed as the addresses of Glulx string objects (see section
    /// 1.6.1, "Strings".) References to values are passed by their addresses.
    /// Arrays are passed by their addresses; note that an array argument,
    /// unlike a string argument, is always followed by an array length
    /// argument.

    /// Reference arguments require more explanation. A reference to an integer
    /// or opaque object is the address of a 32-bit value (which, being in main
    /// memory, does not have to be aligned, but must be big-endian.)
    /// Alternatively, the value -1 (FFFFFFFF) may be passed; this is a special
    /// case, which means that the value is read from or written to the stack.
    /// Arguments are always evaluated left to right, which means that input
    /// arguments are popped from the stack first-topmost, but output arguments
    /// are pushed on last-topmost.

    /// A reference to a Glk structure is the address of an array of 32-bit
    /// values in main memory. Again, -1 means that all the values are written
    /// to the stack. Also again, an input structure is popped off
    /// first-topmost, and an output structure is pushed on last-topmost.

    /// All stack input references (-1 addresses) are popped after the Glk
    /// argument list is popped. [This should be obvious, since the -1 occurs in
    /// the Glk argument list.] Stack output references are pushed after the Glk
    /// call, but before the S1 result value is stored.
    Glk(LoadOperand<L>, LoadOperand<L>, StoreOperand<L>),
}
