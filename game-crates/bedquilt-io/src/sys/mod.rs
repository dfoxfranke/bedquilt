cfg_if::cfg_if! {
    if #[cfg(all(target_arch = "wasm32", target_os="unknown"))] {
        mod wasm2glulx;
        pub use wasm2glulx::glk;
        pub use wasm2glulx::mutex;
        pub use wasm2glulx::random;
        pub use wasm2glulx::exit;
    }
}
