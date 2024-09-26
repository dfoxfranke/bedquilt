# Wasm2Glulx

Wasm2Glulx translates [WebAssembly](https://webassembly.org) into
[Glulx](https://www.eblong.com/zarf/glulx/Glulx-Spec.html). It is mainly a
command line tool but also has a Rust [library
interface](https://docs.rs/wasm2glulx). Its raison d'être is to make it possible
to use general-purpose programming languages to develop interactive fiction,
while producing portable game files that run seamlessly on existing Glulx
interpreters.

Wasm2Glulx is plumbing; it is not intended to provide a direct, friendly
interface for game developers. It deals only in WebAssembly and in Glulx, and is
agnostic to whatever high-level language its input may have been compiled from.
As such, the audience for this manual is people developing game engines which
target Glulx via WebAssembly, and not so much the users of those engines. The
reader is assumed to be familiar with Glulx and with at least the [high-level
structure](https://webassembly.github.io/spec/core/syntax/index.html) of a
WebAssembly module.