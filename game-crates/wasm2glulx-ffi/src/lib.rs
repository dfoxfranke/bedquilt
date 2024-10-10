// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
// Copyright 2024 Daniel Fox Franke.
#![no_std]
//! This crate provides low-level bindings to the system API available to games
//! when they are compiled for the Glulx virtual machine by way of
//! [Wasm2Glulx](https://docs.rs/wasm2glulx). This crate is part of the
//! [Bedquilt project](https://bedquilt.io).

pub mod glk;
pub mod glulx;
