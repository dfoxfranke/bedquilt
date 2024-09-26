# High-Level Language Examples

## C

Here is an example of a simple freestanding C program which can be compiled into
Glulx using clang and Wasm2Glulx, which will print "Hello, sailor!" and then
exit.

```c
#define NULL ((void*)0)

typedef unsigned int glui32;
typedef struct glk_window_struct *winid_t;
typedef struct glk_stream_struct *strid_t;

extern winid_t glk_window_open(winid_t split, glui32 method, glui32 size,
                               glui32 wintype, glui32 rock)
    __attribute__((import_module("glk"), import_name("window_open")));
extern void glk_stream_set_current(strid_t str)
    __attribute__((import_module("glk"), import_name("stream_set_current")));
extern void glk_put_string(const char *s)
    __attribute__((import_module("glk"), import_name("put_string")));
extern strid_t glk_window_get_stream(winid_t win)
    __attribute__((import_module("glk"), import_name("window_get_stream")));

#define wintype_TextBuffer 3

void glulx_main() {
    winid_t root_window = glk_window_open(NULL, 0, 0, wintype_TextBuffer, 0);
    glk_stream_set_current(glk_window_get_stream(root_window));
    glk_put_string("Hello, sailor!\n");
}
```

This can be compiled by running

```
clang --target=wasm32-unknown-unknown -ffreestanding -nostdinc -nostdlib \
        -Wl,--no-entry -Wl,--import-undefined -Wl,--export,glulx_main \
        -o hello_sailor.wasm hello_sailor.c
wasm2glulx hello_sailor.wasm
```

and the resulting `hello_sailor.ulx` will run in any Glulx interpreter.

## Rust

A similar program in Rust:

```rust
#![no_std]
#![no_main]

use core::ffi::{c_char, c_void, CStr};
use core::panic::PanicInfo;

#[derive(Copy, Clone)]
#[repr(transparent)]
struct Strid(*const c_void);

#[derive(Copy, Clone)]
#[repr(transparent)]
struct Winid(*const c_void);

const WINTYPE_TEXT_BUFFER: u32 = 3;

#[link(wasm_import_module = "glk")]
extern "C" {
    #[link_name = "exit"]
    fn glk_exit() -> !;

    #[link_name = "window_open"]
    fn glk_window_open(
        split: Winid,
        method: u32,
        size: u32,
        wintype: u32,
        rock: u32
    ) -> Winid;

    #[link_name = "stream_set_current"]
    fn glk_stream_set_current(stream: Strid);

    #[link_name = "put_string"]
    fn glk_put_string(s: *const c_char);

    #[link_name = "window_get_stream"]
    fn glk_window_get_stream(window: Winid) -> Strid;
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    unsafe {
        glk_exit();
    }
}

#[no_mangle]
extern "C" fn glulx_main() {
    unsafe {
        let root_window = glk_window_open(
            Winid(core::ptr::null()), 
            0, 
            0, 
            WINTYPE_TEXT_BUFFER,
        0);
        glk_stream_set_current(glk_window_get_stream(root_window));
        glk_put_string(
            CStr::from_bytes_with_nul(b"Hello, sailor!\n\0")
                .unwrap()
                .as_ptr(),
        );
    }
}
```

This can be compiled by running

```
rustc --target=wasm32-unknown-unknown -o hello_sailor.wasm hello_sailor.rs
wasm2glulx hello_sailor.wasm
```