# Bindings to Glulx Intrinsics

The functions specified in this section can be imported, using a module name of
`glulx`, to access various special Glulx instructions which don't have already
have a WASM equivalent.

## Math functions

All of these functions are equivalent to the similarly-named functions in C's
`<math.h>`. Math functions which already have WASM instructions (such as `trunc`
and `sqrt`) do not have bindings, nor do functions which have neither WASM
instructions nor any special Glulx instructions which accelerate them (such as
`expm1`).

Single-precision:

```wasm
(import "glulx" "fmodf" (func (param $x f32) (param $y f32) (result f32)))
(import "glulx" "floorf" (func (param $x f32) (result f32)))
(import "glulx" "ceilf" (func (param $x f32) (result f32)))
(import "glulx" "expf" (func (param $x f32) (result f32)))
(import "glulx" "logf" (func (param $x f32) (result f32)))
(import "glulx" "powf" (func (param $x f32) (param $y f32) (result f32)))
(import "glulx" "sinf" (func (param $x f32) (result f32)))
(import "glulx" "cosf" (func (param $x f32) (result f32)))
(import "glulx" "tanf" (func (param $x f32) (result f32)))
(import "glulx" "asinf" (func (param $x f32) (result f32)))
(import "glulx" "acosf" (func (param $x f32) (result f32)))
(import "glulx" "atanf" (func (param $x f32) (result f32)))
(import "glulx" "atan2f" (func (param $y f32) (param $x f32) (result f32)))
```

Double-precision:

```wasm
(import "glulx" "fmod" (func (param $x f64) (param $y f64) (result f64)))
(import "glulx" "floor" (func (param $x f64) (result f64)))
(import "glulx" "ceil" (func (param $x f64) (result f64)))
(import "glulx" "exp" (func (param $x f64) (result f64)))
(import "glulx" "log" (func (param $x f64) (result f64)))
(import "glulx" "pow" (func (param $x f64) (param $y f64) (result f64)))
(import "glulx" "sin" (func (param $x f64) (result f64)))
(import "glulx" "cos" (func (param $x f64) (result f64)))
(import "glulx" "tan" (func (param $x f64) (result f64)))
(import "glulx" "asin" (func (param $x f64) (result f64)))
(import "glulx" "acos" (func (param $x f64) (result f64)))
(import "glulx" "atan" (func (param $x f64) (result f64)))
(import "glulx" "atan2" (func (param $y f64) (param $x f64) (result f64)))
```

## Game state functions

Each of these functions performs the same task as its equivalently-named Glulx
instruction. There is no binding for `quit` because it is redundant with
`glk_exit`.

```wasm
(import "glulx" "restart" (func))
(import "glulx" "save" (func (param $strid i32) (result i32)))
(import "glulx" "restore" (func (param $strid i32) (result i32)))
(import "glulx" "saveundo" (func (result i32)))
(import "glulx" "restoreundo" (func (result i32)))
(import "glulx" "hasundo" (func (result i32)))
(import "glulx" "discardundo" (func))
(import "glulx" "protect" (fun (param $addr i32) (param $len i32)))
```

The `$addr` argument to `protect` is a memory index. Protecting other parts of a
WASM instance's state, such tables and globals, is not supported.

## Miscellaneous functions

Each of these functions performs the same task as its equivalently-named Glulx
instruction.

```wasm
(import "glulx" "gestalt"
        (func (param $selector i32) (param $extra i32) (result i32)))
(import "glulx" "random" (func (param $range i32) (result i32)))
(import "glulx" "setrandom" (func (param $seed i32)))
```

## Bindings intentionally omitted

The search instructions `linearsearch`, `binarysearch` and `linkedsearch` do not
have bindings because they rely on assumptions that are incompatible with
Wasm2Glulx's internal ABI.

The heap instructions `malloc` and `mfree` are not bound because they are
reserved for future internal use by Wasm2Glulx runtime. You can still allocate
memory using WASM's `memory.grow` instruction, and bring your own heap
implementation.

There are no bindings for `getstringtbl` or `setstringtbl`, and string-decoding
tables are unsupported in general.

There are currently no bindings for `throw` and `catch`. The functionality
provided by these instructions will be supported in the future by way of the
`exnref` WASM feature extension.