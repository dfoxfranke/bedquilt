//! This crate provides a safe, idiomatic, mid-level abstraction over
//! [Glk](https://www.eblong.com/zarf/glk/Glk-Spec-075.html) and other system
//! facilities that are available from within the [Glulx virtual
//! machine](https://www.eblong.com/zarf/glulx/Glulx-Spec.html). You can write
//! Rust code which targets Glulx by depending on this crate, compiling for
//! `wasm32-unknown-unknown`, and then using
//! [Wasm2Glulx](https://bedquilt.io/manual/wasm2glulx/cli.html) to generate a
//! Glulx story file.
//! 
//! You can think of `bedquilt-io` (alongside [`core`], [`alloc`], and
//! [`hashbrown`]) as your substitute for `std` when developing for Glulx. Rust
//! does already have a `std` crate for `wasm32-unknown-unknown`, but it is
//! mostly useless: almost everything in it just panics or returns an
//! `Unimplemented` error. If you want to use it anyway, you will need to
//! disable this crate's `panic_handler` and `global_allocator` features to
//! prevent conflict with the ones that `std` provides. It is recommended,
//! however, to make your program `no_std`, as `bedquilt-io` should already
//! provide everything you need.
//! 
//! In the future, `bedquilt-io` will also support native compilation, using a
//! pure-Rust implementation of the
//! [RemGlk](https://www.eblong.com/zarf/glk/remglk/docs.html) protocol.
//! However, this is not yet implemented, and this crate will fail to compile on
//! for target other than `wasm32-unknown-unknown`. References in this
//! documentation to behavior on "other platforms" should be understood in the
//! future tense.
//! 
//! `bedquilt-io` is implemented around an included async executor. Your
//! program's `glulx_main` function should use [`task::spawn`] to start one or
//! more asynchronous tasks, and then call [`task::run`] to run them all to
//! completion. One of your tasks can then create a root window (typically a
//! [`TextBufferWindow`](win::TextBufferWindow)) using
//! [`win::Window::create_as_root`] and use the traits it implements to perform
//! IO asynchronously. A trivial program which reads keyboard input in a loop
//! and echoes it back looks like this:
//! 
//! ```ignore
//! #![no_std]
//! #![no_main]
//! 
//! use bedquilt_io::win::{LineInput, TextBufferWindow, Window};
//! use core::fmt::Write;
//!
//! #[no_mangle]
//! extern "C" fn glulx_main() {
//!     bedquilt_io::task::spawn(main());
//!     bedquilt_io::task::run();   
//! }
//!
//! async fn main() {
//!     let mut root = TextBufferWindow::create_as_root().unwrap();
//!     loop {
//!         let input = root.request_line("").unwrap().await;
//!         writeln!(root, "{}", input.input).unwrap();
//!     }
//! }
//! ```
//! 
//! Glk requires programs to query for a "gestalt" before attempting to use
//! certain optional features. In Bedquilt, this is not necessary. If you want
//! to do something possibly-unsupported, just try it, and if the necessary
//! gestalt is missing you'll get back
//! [`Error::Unsupported`](error::Error::Unsupported).

#![warn(missing_debug_implementations, missing_docs)]
#![cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), no_std)]
extern crate alloc;

pub mod error;
pub mod fs;
pub mod random;
mod reactor;
pub mod sync;
mod sys;
pub mod sound;
pub mod task;
pub mod time;
pub mod win;

pub use sys::exit;