// SPDX-License-Identifier: Apache-2.0 WITH LLVM-Exception
// Copyright 2024 Daniel Fox Franke.

//! This crate implements an assembler for the Glulx virtual machine for
//! interactive fiction. It supports version 3.1.3 of the [Glulx
//! specification](https://www.eblong.com/zarf/glulx/Glulx-Spec.html#moving-data).
//!
//! Currently, the functionality of this crate is limited to generating binary
//! Glulx files from the in-memory data structures defined herein. It is
//! designed and suitable as a library for use by translation tools that
//! generate Glulx, but cannot be used as a standalone assembler. `Display`
//! impls are provided for generating human-readable assembly listings, but the
//! syntax is subject to change and there is no tool which parses what these
//! impls emit. This crate may be extended with such functionality in the
//! future.
//!
//! This crate's main entry point is the [`Assembly`] struct and its
//! [`assemble`](Assembly::assemble) method, which outputs a
//! [`BytesMut`](bytes::BytesMut) (see the [`bytes`] crate) from the public
//! fields you create the `Assembly` from.
//!
//! The bulk of what you provide to the `Assembly` is a list of [`Item`]s, each
//! of which may be tagged with a label. The label parameter is generic; you can
//! use any type you like provided only that it implements `Clone + Eq + Hash`.
//! Do note that the assembler is a bit indiscriminant about cloning, so if
//! you're considering using [`String`]s as labels you may want to use something
//! like [`Rc<str>`](alloc::rc::Rc) instead. Every type in this crate which
//! takes a label parameter is, in the Haskell/loosely-category-theoretical
//! sense, a functor over its labels. That is, it provides a `map` method which
//! lets you replace every label within it with the output of a callback,
//! possibly changing the label's type.
//!
//! See `examples/hello.rs` for an illustration of using this crate to assemble
//! a story file that prints "Hello, Sailor!" and exits.

#![warn(
    clippy::as_conversions,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs
)]
#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

mod assemble;
mod cast;
pub mod concise;
mod decoding_table;
mod error;
mod instr_def;
mod instr_impls;
mod items;
mod operands;
mod resolver;
mod strings;

pub use assemble::Assembly;
pub use decoding_table::{DecodeArg, DecodeNode};
pub use error::AssemblerError;
pub use instr_def::Instr;
pub use items::{CallingConvention, Item, LabelRef, ZeroItem};
pub use operands::{
    f32_to_imm, f64_to_imm, LoadOperand, StoreOperand,
};
pub use strings::{MysteryString, StringConversionError, Utf32String};
