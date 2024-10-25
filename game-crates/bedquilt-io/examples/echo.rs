#![no_std]
#![no_main]

use bedquilt_io::win::{LineInput, TextBufferWindow, Window};
use core::fmt::Write;

#[no_mangle]
extern "C" fn glulx_main() {
    bedquilt_io::task::spawn(main());
    bedquilt_io::task::run();   
}

async fn main() {
    let mut root = TextBufferWindow::create_as_root().unwrap();
    loop {
        let input = root.request_line("").unwrap().await;
        writeln!(root, "{}", input.input).unwrap();
    }
}