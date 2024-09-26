# Supported WASM Feature Extensions

Wasm2Glulx supports 100% of the [WebAssembly 1.0 Core
Specification](https://www.w3.org/TR/wasm-core-1/), and most of the [current 2.0
draft](https://webassembly.github.io/spec/core/): as of the 2024-09-21 draft,
everything except the SIMD instructions. In addition, a number of feature
extensions that are not (or not yet) part of the core spec are either supported
or planned to be supported. Here is the support status of every feature extension
defined by the WebAssembly working group:

The following features are **fully supported**:

* [Bulk Memory Operations](https://github.com/WebAssembly/bulk-memory-operations/blob/master/proposals/bulk-memory-operations/Overview.md)
* [Multi-value](https://github.com/WebAssembly/spec/blob/master/proposals/multi-value/Overview.md)
* [Reference Types](https://github.com/WebAssembly/reference-types/blob/master/proposals/reference-types/Overview.md)
* [Non-trapping float-to-int Conversions](https://github.com/WebAssembly/spec/blob/master/proposals/nontrapping-float-to-int-conversion/Overview.md)
* [Sign-extension Operators](https://github.com/WebAssembly/spec/blob/master/proposals/sign-extension-ops/Overview.md)

The following features are **not yet supported, but planned**:

* [Fixed-width SIMD](https://github.com/WebAssembly/simd/blob/master/proposals/simd/SIMD.md)
  - Glulx does not natively support SIMD. It's straightforward to emulate,
    albeit very tedious because there are a huge number of instructions to
    implement. Adding this will be a low priority unless a compelling use
    case comes up, which is doubtful.
* [Relaxed SIMD](https://github.com/WebAssembly/relaxed-simd/tree/main/proposals/relaxed-simd)
* [Tail Call](https://github.com/WebAssembly/tail-call/blob/master/proposals/tail-call/Overview.md)
  - Coming soon.
* [Typed Function References](https://github.com/WebAssembly/function-references/blob/main/proposals/function-references/Overview.md)
  - Awaiting upstream support from Walrus (the library that Wasm2Glulx uses for parsing WebAssembly).
* [Exception Handling with exnref](https://github.com/WebAssembly/exception-handling/blob/master/proposals/exception-handling/Exceptions.md)
  - Awaiting upstream support.
* [Threads](https://github.com/WebAssembly/threads/blob/master/proposals/threads/Overview.md)
  - Yes, Wasm2Glulx will be able to support this even though Glulx is strictly
  single-threaded. The Threads proposal really just defines atomics and
  synchronization primitives, and doesn't define any way to spawn a thread,
  leaving that up to the embedder. So, the atomics can be implemented as
  ordinary instructions and the synchronization primitives can be no-ops.
* [Custom Page Sizes](https://github.com/WebAssembly/custom-page-sizes/blob/main/proposals/custom-page-sizes/Overview.md)
  - Too early a draft right now, but should be easy to support once fleshed out.


The following features are **not planned**:
* [Branch Hinting](https://github.com/WebAssembly/branch-hinting/blob/master/proposals/branch-hinting/Overview.md)
  - Wasm2Glulx accepts input which includes branch hints, but it ignores them.
* [Multiple Memories](https://github.com/WebAssembly/multi-memory/blob/master/proposals/multi-memory/Overview.md)
  - This is planned *for* in the sense that Wasm2Glulx is architected in a way
    that will make it possible to add in the future if necessary. But, doing so
    would significantly complicate things and add some runtime overhead, so it
    will be avoided unless a compelling use case comes up.
* [Garbage Collection](https://github.com/WebAssembly/gc)
  - This would be a massive amount of work to implement, on top Multiple
    Memories support which would be a prerequisite. Glulx doesn't have a garbage
    collector, and any polyfill that Wasm2Glulx could provide would not be any
    more (and probably less) efficient than the polyfills that GCed languages
    targeting WebAssembly already usually provide.
* [Memory64](https://github.com/WebAssembly/memory64/blob/master/proposals/memory64/Overview.md)
  - Impossible, since Glulx is a 32-bit machine.
* [Instrumentation and Tracing Technology](https://github.com/WebAssembly/instrument-tracing/blob/main/proposals/instrument-tracing/Overview.md)
  - This proposal doesn't seem to be fleshed out and seems to be dead.
* [Stack Switching](https://github.com/WebAssembly/stack-switching)
  - Infeasible without first extending Glulx.
* [Legacy Exception Handling](https://github.com/WebAssembly/exception-handling/blob/master/proposals/exception-handling/Exceptions.md)
  - This feature is deprecated. Its replacement, Exception Handling with exnref, is
    planned.

The following features are **N/A**. They relate to aspects of WebAssembly that
aren't applicable to Wasm2Glulx, such as its JavaScript embedding.

* [JS BigInt to Wasm i64 Integration](https://github.com/WebAssembly/JS-BigInt-integration)
* [Custom Text Format Annotations](https://github.com/WebAssembly/annotations/blob/main/proposals/annotations/Overview.md)
* [Extended Constant Expressions](https://github.com/WebAssembly/extended-const/blob/master/proposals/extended-const/Overview.md)
  - Constant expressions are functions of imported constant globals. Since
    Wasm2Glulx doesn't make any constant globals available for import, it's
    impossible to write a well-typed extended constant expression.
* [Import/Export of Mutable Globals](https://github.com/WebAssembly/mutable-global/blob/master/proposals/mutable-global/Overview.md)
  - Exported mutable globals are accepted but ignored.
* [JS String Builtins](https://github.com/WebAssembly/js-string-builtins/blob/main/proposals/js-string-builtins/Overview.md)
* [ESM Integration](https://github.com/WebAssembly/esm-integration)
* [JS Promise Integration](https://github.com/WebAssembly/js-promise-integration)
* [Type Reflection for JS API](https://github.com/WebAssembly/js-types/blob/main/proposals/js-types/Overview.md)
* [Web Content Security Policy](https://github.com/WebAssembly/content-security-policy/blob/main/proposals/CSP.md)