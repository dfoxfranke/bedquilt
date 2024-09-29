# `glulx-asm`

This crate implements an assembler for the Glulx virtual machine for
interactive fiction. It supports version 3.1.3 of the [Glulx
specification](https://www.eblong.com/zarf/glulx/Glulx-Spec.html#moving-data).

Currently, the functionality of this crate is limited to generating binary
Glulx files from the in-memory data structures defined herein. It is
designed and suitable as a library for use by translation tools that
generate Glulx, but cannot be used as a standalone assembler. `Display`
impls are provided for generating human-readable assembly listings, but the
syntax is subject to change and there is no tool which parses what these
impls emit. This crate may be extended with such functionality in the
future.

## License

In general, `glulx-asm` is licensed under the [Apache License 2.0 with LLVM
Exception](LICENSE.apache). If you take this crate as a dependency without
modifying it, this is the only license you need to worry about.

Portions of this crate's documentation are derived from the [Glulx
specification](https://www.eblong.com/zarf/glulx/Glulx-Spec.html), which is
licensed under [Creative Commons Attribution-NonCommercial-ShareAlike 4.0
International License](https://creativecommons.org/licenses/by-nc-sa/4.0/). If
you redestribute this crate's source code in unmodified form, or in modified
form without removing this documentation, you must comply with that license in
addition to complying with the Apache license.