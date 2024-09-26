# Installation & Command Line Interface

## Installation

Wasm2Glulx does not currently distribute binary packages. The supported way to
install it is via the Rust package manager, `cargo`. If you haven't already done
so, first [install Rust](https://www.rust-lang.org/tools/install). Then, to build
and install Wasm2Glulx, simply run

```sh
cargo install wasm2glulx
```

## Command Line Interface

A typical invocation of Wasm2Glulx is simply

```sh
wasm2glulx mygame.wasm
```

where `mygame.wasm` is a WASM module; this will output a Glulx story file as
`mygame.ulx`. If no file is provided as an argument, it will default to reading
from stdin and writing to stdout. Additionally, the following command-line options
are supported:

* `-o, --output <FILE>`
    
  Name of output file, or "-" for stdout.

  The default is stdout if the input comes from stdin.
  Otherwise, the default is to strip any .wasm suffix from the
  input file name, add a .ulx suffix, and output it to the
  current directory.

* `--glk-area-size <SIZE>`

  Size (in bytes) of the Glk area. See section [Bindings to Glk](glk.md) on the
  role of this. The default is 4096 (4KiB).

* `--stack-size <SIZE>`

  Size (in bytes) of the program stack. This goes into the `stacksize` field of
  the Glulx header. The default is an extremely generous 1048576 (1MiB), chosen
  because this matches what Rust allocates by default on other platforms. Users
  of modern systems will never miss 1 MiB of memory, but consider reducing this
  if you want to keep your games friendly to retrocomputing hobbyists.

* `--table-growth-limit <N>`

  Growth limit (in entries) for WASM tables.

  If the input module specifies a smaller maximum, the smaller value will be
  used. Most programs don't use growable tables and will specify a maximum size
  the same as the initial one, so this option is usually ignored.

* `--text`

  Output human-readable assembly rather than a story file.

  The format of the assembly is not fully defined and is subject to change in
  future versions; there is no tool which will re-parse what whis outputs.
  Unless overridden by `-o`, the output file will have a suffix of `.glulxasm`.

* `-h, --help`

  Print a summary of command line options, similar to this manual section.

* `-V, --version`

  Print version information.