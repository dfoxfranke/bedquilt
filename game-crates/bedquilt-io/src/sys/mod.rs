cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        mod wasm2glulx;
        pub use wasm2glulx::io;
        pub use wasm2glulx::mutex;
        pub use wasm2glulx::random;
    }
}
