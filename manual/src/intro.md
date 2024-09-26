# Bedquilt

The Bedquilt project is an effort to build a set of tools for developing
interactive fiction using general-purpose programming languages — particularly
but not exclusively Rust — and producing portable game files that are playable
on any interpreter that supports the
[Glk/Glulx/Blorb](https://github.com/iftechfoundation/ifarchive-if-specs) tech
stack. Eventually, Bedquilt will become a full-fledged text adventure engine
competing with the likes of [Inform](https://ganelson.github.io/inform-website/)
and [TADS](https://www.tads.org/). It isn't there yet, but a major foundational
piece is complete: Wasm2Glulx, which translates
[WebAssembly](https://webassembly.org/) into Glulx. Wasm2Glulx makes it possible
develop for Glulx using [any high-level-language compiler that has a WebAssembly
backend](https://webassembly.org/getting-started/developers-guide/). Wasm2Glulx has
already been used to produce one complete game: a [new
port](https://github.com/dfoxfranke/bedquilt/tree/master/advent430) of
[Adventure](https://en.wikipedia.org/wiki/Colossal_Cave_Adventure).