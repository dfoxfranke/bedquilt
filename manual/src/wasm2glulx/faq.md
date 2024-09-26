# Architectural FAQ

## Why WebAssembly? Why not write an LLVM backend instead?

[That has been done](https://github.com/dfremont/glulx-llvm).

The answer comes down to stability and maintainability. LLVM-IR evolves rapidly,
as do LLVM's internal APIs. If you maintain a frontend which targets LLVM,
that's great: you're getting a constant stream of new features and new
optimizations. Highly-active compiler projects like Rust love this; every
sesquimonthly Rust release depends on the latest bleeding edge LLVM. On the
other hand, maintaining a *backend* and keeping up with bitrot becomes a
never-ending red queen's race. Glulx-LLVM was last updated in November 2021 as
of September 2024. It only ever worked as a forked LLVM with a forked clang, and
now that fork is three years out of date. C is pretty static, so maybe working
with a three-year-old compiler is acceptable, but for Rust it certainly wouldn't
be.

WebAssembly is different. Being a web standard, it has to support many
interoperable implementations. While the W3C has become remarkably efficient
compared to other standards bodies, changes to WebAssembly are still a much
slower and deliberate process than changes to LLVM internals, and when the
standard moves, it leaves fixed and durable milestones along its path.
WebAssembly 1.0 was finalized in 2019, and although it has been extended
considerably since then, the 1.0 standard is still what LLVM and most LLVM-based
compilers target by default unless you supply flags to enable newer features.
Support for targeting 1.0 is not likely to go away any time soon, just as
support for old CPUs is seldom dropped without good reason. Wasm2Glulx already
supports 1.0 and much more. Consequently, the version of Wasm2Glulx that exists
today will probably work just fine with compilers released a decade from now.

## How efficient is Wasm2Glulx's output? Does it have an optimizer?

It does have a bit of an optimizer, yes. The optimizer doesn't have to be
complicated, because it's assumed that Wasm2Glulx's input already came from an
optimizing compiler. The primary concern is to optimize cases where sequences of
multiple WASM instructions can be replaced by a single Glulx instruction. This
happens most often with respect to pushing and popping local variables. WASM is
strictly load/store, so adding two local variables and assigning the result to a
third will look something like

```wasm
local.get 0
local.get 1
i32.add
local.get 2
```

A naïve translation into Glulx might look like

```
copy $0 push
copy $1 push
add pop pop push
copy pop $2
```

But since Glulx has a richer set of address operands, we'd rather just
say

```
add $0 $1 $2
```

and Wasm2Glulx does optimize this.

As to overall efficiency, there is one pain point that simple optimization
techniques can't address, which relates to the mismatch of endianness between
WebAssembly (little endian) and Glulx (big endian). This requires every memory
load/store operation to be accompanied by several instructions for byteswapping.
(Thankfully, this isn't required for local variable operations, because just
like in Glulx, WASM locals don't have any particular endianness.) This is a
pretty big performance hit, probably 2x overall to a typical game. In the near
future, this will be addressed using Glulx's `accelfunc` facility. Accelfunc has
only ever been used previously to accelerate Inform's veneer functions, but it
is actually very flexible and extensible. New accelfuncs will be defined for the
load/store functions in Wasm2Glulx's runtime library, and support for these will
be upstreamed into interpreters.
