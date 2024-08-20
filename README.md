# Project Bedquilt

This repository hosts a collection of tools developed with the aim of enabling
the use of Rust as a development language for interactive fiction which can run
seamlessly on existing interpreters that support
[Glulx](https://www.eblong.com/zarf/glulx/). The approach will be to compile
Rust into WebAssembly and then translate the WebAssembly into Glulx.

Currently, there is a [Glulx assembler](crates/glulx-asm), which is currently
alpha but approaching release-readiness. The [WASM-to-Glulx
translator](crates/wasm2glulx) is under development; currently it can compile a
"Hello, sailor!" program but not much else. Once wasm2glulx is complete enough
to handle real programs, development will begin on safe and idiomatic Rust
bindings to the [Glk](https://www.eblong.com/zarf/glk/index.html) API and other
Glulx intrinsics, which wasm2glulx exposes as WASM imports.