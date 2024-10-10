# Bindings to Glk

Wasm2Glulx provides a complete set of bindings to the [Glk 0.7.5
API](https://www.eblong.com/zarf/glk/Glk-Spec-075.html). To access a Glk
function, your module should declare a function import with a module name of
`"glk"` and a function name which matches the one defined in the Glk spec, but
with the `glk_` prefix removed.

All parameters to Glk functions are of type `i32`, and the result type is either
`i32` or empty.  Wasm2Glulx will typecheck your imports and give a compile-time
error if they are incorrectly declared. Wherever a Glk function takes a pointer
argument, the argument passed to the corresponding WASM function should be an
index into the module's memory. An index of zero is interpreted as a null
pointer.

Pointer arguments must not alias each other; treat all pointers as though they
were declared `restrict` in Glk's C header. This is not something you have to
actually worry about, because the only Glk function which could possibly take
aliased pointers without UB according to C's aliasing rules is
`glk_image_get_info`, and since those are pointers to output parameters it would
be nonsense for them to alias.

WASM Glk functions take their arguments in the same order as described in the
Glk spec. This is despite WASM and Glulx having opposite calling conventions (in
Glulx, the first function argument is on top of the stack; in WASM, the last
argument is on top). The necessary swapping happens behind the scenes.

When an argument to a Glk function is a null-terminated string, Glulx expects
the string to be prefixed with `0xE0` (for Latin-1 strings) or `0xE2000000` (for
Unicode strings). Wasm2Glulx does **not** require this prefix to be included.
The generated bindings will automatically patch the prefix into memory and then
replace the original memory before returning. This works cleanly because none of
these functions take any other pointer arguments, so there's no need to worry
that the prefix patch will overwrite another argument.

There is no binding for `glk_set_interrupt_handler`. However, if your module
exports a function named `glulx_interrupt_handler`, it will be configured as
your interrupt handler at startup. (This was cleaner than trying to pass
function pointers in and out of WASM. Although Wasm2Glulx fully supports
`funcref`, LLVM's support for it seems to be very buggy and my first experiment
with it segfaulted Clang.)

There are no bindings for Glulx's `setiosys` and `getiosys` instructions. Glk is
automatically set as the IO system at startup, and it cannot be changed. Your
own code is still responsible for the rest of Glk initialization, such as
creating a root window.

# The Glk area

Certain Glk functions pass it ownership of memory buffers that you provide to
it. These include:

* `glk_request_line_event`
* `glk_request_line_event_uni`
* `glk_stream_open_memory`
* `glk_stream_open_memory_uni`

Working with these functions is a bit more complicated. Wasm2Glulx creates a
special region of your program image, called the Glk area, which lives outside
the address space of your module's memory. The size of this region is fixed at
compile time but controllable by the `--glk-area-size` command line argument.
When you call one of the above four functions, the `buf` argument is an index
into the Glk area, rather than an index into memory. Unlike pointers to main
memory, `0` is an ordinary and valid Glk area offset and will not be interpreted
as a null pointer.

This extra bit of ceremony is necessary for two reasons. The first reason arises
from the fact that WebAssembly is a little-endian architecture, but Glulx is
big-endian and Glk expects to see big-endian data. When passing *borrowed*
pointers to Glk, this difference is kept transparent: the generated bindings
automatically swap memory into big-endian before calling Glk, and swap it back
before returning. But for owned buffers, this doesn't work, because the Glk API
makes it too complex for Wasm2Glulx to infer when buffer ownership has been
returned to the caller and the buffer should be swapped back to little-endian.
Having a separate Glk area solves this problem: the Glk area is always
big-endian, memory is always little-endian, and swapping happens whenever you
transfer data from one to the other. The second reason pertains to
future-proofing. Currently, conversion between Glulx memory addresses and
indexes into WASM memory is just a matter of adding or subtracting a fixed
offset determined at compile time. However, this may change, because supporting
future WASM features may make it necessary for WASM memory to move around in
Glulx's address space. If some of that memory were potentially owned by Glk,
then this movement would wreak havoc. Keeping the Glk area separate solves this
too, by ensuring that it can always remain at a fixed address even when main
memory moves around.

The following intrinsics are provided for moving data in and out of the Glk area:

```wasm
(import "glulx" "glkarea_get_byte" (func (param $glkaddr i32) (result i32)))
(import "glulx" "glkarea_get_word" (func (param $glkaddr i32) (result i32)))
(import "glulx" "glkarea_put_byte" (func (param $glkaddr i32) (param $byte i32)))
(import "glulx" "glkarea_put_word" (func (param $glkaddr i32) (param $word i32)))

(import "glulx" "glkarea_get_bytes"
        (func (param $addr) (param $glkaddr i32) (param $n i32)))
(import "glulx" "glkarea_get_words"
        (func (param $addr) (param $glkaddr i32) (param $n i32)))
(import "glulx" "glkarea_put_bytes"
        (func (param $glkaddr) (param $addr i32) (param $n i32)))
(import "glulx" "glkarea_put_words"
        (func (param $glkaddr) (param $addr i32) (param $n i32)))

(import "glulx "glkarea_size" (func (result i32)))
```

The first four functions read or write an individual byte or word to or from the
Glk area at offset `$glkaddr`, while the second four move `$n` bytes or words
between the Glk area at offset `$glkaddr` and main memory at offset `$addr`.
Note that the destination argument always comes first. The word functions will
perform endianness swaps as required, while the byte functions will not swap
anything. `glkarea_size` returns the size of the Glk area in bytes.