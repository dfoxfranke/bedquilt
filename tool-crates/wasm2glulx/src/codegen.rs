// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
// Copyright 2024 Daniel Fox Franke.

mod arith;
mod classify;
mod control;
mod loadstore;
mod memory;
mod table;
mod toplevel;

pub use toplevel::gen_function;
