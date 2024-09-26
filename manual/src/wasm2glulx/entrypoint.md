# Your Program's Entrypoint

There are two ways for a WebAssembly module to define an entrypoint for
Wasm2Glulx. The module can either define a [start
function](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-start),
or it can export a function named `glulx_main`. In either case, the function
must take no parameters and return no result.  If the module defines a start
function *and* a `glulx_main` function, and the two are distinct from each
other, then the start function will be called first and `glulx_main` will be
called after the start function returns.

No matter how you define your entrypoint, Wasm2Glulx will always generate some
initialization code that runs prior to the entrypoint being called. This code
takes care of initializing memory from any [active data
segments](https://webassembly.github.io/spec/core/syntax/modules.html#data-segments)
that your module defines, and initializing tables from [active element
segments](https://webassembly.github.io/spec/core/syntax/modules.html#element-segments).
It will also execute a `setiosys 2 0` instruction to set Glk as the output
system.