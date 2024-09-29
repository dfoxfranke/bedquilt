# bogoglulx

This is a stripped-down fork of [glulxe](https://github.com/erkyrath/glulxe)
designed for running the WebAssembly test suite.  Most IO capabilities — and all
dependencies on GLK — have been removed. So have various other unneeded features
such as save/restore/undo, string decoding tables, and random number generation.
However, a modified `streamnum` instruction is still present: it will print the
number to stdout as eight hexadecimal characters (ignoring whether any IO system
has been set). The `debugtrap` instruction will print an exclamation point
followed by a error message determined by its argument; the error messages
correspond to those expected by the test suite. If the interpreter itself
encounters an error — which in a successful test should never happen — the error
message is prefixed with a question mark.
