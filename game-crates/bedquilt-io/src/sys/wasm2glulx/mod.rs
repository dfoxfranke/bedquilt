#![allow(missing_docs)]

use cfg_if::cfg_if;
pub mod glk;
pub mod mutex;
pub mod random;

cfg_if! {
    if #[cfg(feature = "global_allocator")] {
        #[global_allocator]
        static ALLOCATOR : dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;
    }
}

/// Exits the program.
pub fn exit() -> ! {
    unsafe {
        wasm2glulx_ffi::glk::exit()
    }
}