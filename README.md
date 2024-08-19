# Project Bedquilt

This repository hosts a collection of tools developed with the aim of enabling
the use of Rust as a development language for interactive fiction which can run
seamlessly on existing interpreters that support
[Glulx](https://www.eblong.com/zarf/glulx/). The approach will be to compile
Rust into WebAssembly and then translate the WebAssembly into Glulx.

Currently, the only thing here is a [Glulx assembler](crates/glulx-asm). The
WASM-to-Glulx translator will come next, followed by safe and idiomatic Rust
bindings to the [Glk](https://www.eblong.com/zarf/glk/index.html) API and other
Glulx intrinsics which wasm2glulx will expose as WASM imports.