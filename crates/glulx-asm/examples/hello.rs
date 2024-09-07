use glulx_asm::concise::*;
use glulx_asm::*;
use std::{borrow::Cow, io::Write};

fn main() {
    let main_label = 0;
    let hello_sailor_label = 1;
    let fail_label = 2;

    let assembly: Assembly<i32> = Assembly {
        rom_items: Cow::Owned(vec![
            // The string we'll print. Add some newlines at the beginning so
            // that the "game session has ended" message doesn't cover it.
            label(hello_sailor_label),
            mystery_string(&"\n\n\nHello, sailor!\n"),
            // Header for our main function, which uses no locals.
            label(main_label),
            fnhead_stack(0),
            // Set Glk as our IO system.
            setiosys(imm(2), imm(0)),
            // Push arguments to create our main window
            // rock = 0
            copy(imm(0), push()),
            // wintype = Textbuffer
            copy(imm(3), push()),
            // size = 0 (ignored for root window)
            copy(imm(0), push()),
            // method = 0 (ignored for root window)
            copy(imm(0), push()),
            // split = 0
            copy(imm(0), push()),
            // call glk_window_open (0x23) with these five arguments. Push the
            // return value to the stack.
            glk(imm(0x23), imm(5), push()),
            // Duplicate the return value on the stack.
            stkcopy(imm(1)),
            // Jump to fail_label if we got a null return.
            jz(pop(), fail_label),
            // Call glk_set_window (0x21) with one argument (the winid that's on
            // the stack) to set our new window as current.
            glk(imm(0x2f), imm(1), discard()),
            // Print our message.
            streamstr(imml(hello_sailor_label)),
            // Return from main.
            label(fail_label),
            ret(imm(0)),
        ]),
        ram_items: Cow::Owned(vec![]),
        zero_items: Cow::Owned(vec![]),
        stack_size: 256,
        start_func: LabelRef(main_label, 0),
        decoding_table: None,
    };

    let bytes = assembly.assemble().unwrap();
    let mut stdout = std::io::stdout();
    stdout.write_all(&bytes).unwrap();
}
