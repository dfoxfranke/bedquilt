# Known Bugs

Running Wasm2Glulx's test suite gave Glulx interpreters some exercise they've
never gotten before, and this turned up a handful of bugs in them. There are two
such bugs for which Wasm2Glulx does not currently implement any workaround.

1. The Glulx specification did not state what the result of dividing
   `-0x80000000/-1` should be, while WebAssembly specifies that this should
   trap. Glulx interpreters had divergent behavior: Glulxe and Quixe return
   `-0x80000000`, while Git had undefined behavior; on most compilers and
   architectures, it would crash. It was decided that Glulx should consider this
   case to be an error, thus matching WASM semantics. Compiling a WASM program
   which computes `-0x80000000/-1` and running it on an interpreter which has
   not been patched to implement this spec change will yield a result which does
   not comply with the WASM specification. This is only a problem for 32-bit
   integers; 64-bit integer division is implemented in Wasm2Glulx's runtime
   library and does not use Glulx's `div` instruction.

2. Glulxe and Quixe do not correctly propagate NaN payloads, and return
   non-canonical payloads for non-propagated double precision NaNs. C and Rust
   programmers will never care about this, but it's a big problem for dynamic
   language runtimes which implement [NaN
   boxing](https://craftinginterpreters.com/optimization.html#nan-boxing). In
   particular, [AssemblyScript](https://www.assemblyscript.org/) is probably
   going to break. If you are interested in developing for Glulx in
   AssemblyScript, please file a ticket about it and I can add a command-line
   flag to enable generating workaround code.