#![no_std]
//! This crate provides low-level bindings to the API available to games when
//! they are compiled for the Glulx virtual machine by way of Wasm2Glulx. This
//! crate is part of the [Bedquilt project](https://bedquilt.io).

pub mod glk;
pub mod glulx;