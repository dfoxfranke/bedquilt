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
            (
                Some(hello_sailor_label),
                Item::MysteryString(MysteryString::from_chars_lossy(
                    "\n\n\nHello, sailor!\n".chars(),
                )),
            ),
            // Header for our main function, which takes no arguments.
            (
                Some(main_label),
                Item::FnHeader(CallingConvention::ArgsOnStack, 0),
            ),
            // Set Glk as our IO system.
            (
                None,
                Item::Instr(Instr::Setiosys(LoadOperand::Imm(2), LoadOperand::Imm(0))),
            ),
            // Push arguments to create our main window
            // rock = 0
            (
                None,
                Item::Instr(Instr::Copy(LoadOperand::Imm(0), StoreOperand::Push)),
            ),
            // wintype = Textbuffer
            (
                None,
                Item::Instr(Instr::Copy(LoadOperand::Imm(3), StoreOperand::Push)),
            ),
            // size = 0 (ignored for root window)
            (
                None,
                Item::Instr(Instr::Copy(LoadOperand::Imm(0), StoreOperand::Push)),
            ),
            // method = 0 (ignored for root window)
            (
                None,
                Item::Instr(Instr::Copy(LoadOperand::Imm(0), StoreOperand::Push)),
            ),
            // split = 0
            (
                None,
                Item::Instr(Instr::Copy(LoadOperand::Imm(0), StoreOperand::Push)),
            ),
            // call glk_window_open (0x23) with these five arguments. Push the
            // return value to the stack.
            (
                None,
                Item::Instr(Instr::Glk(
                    LoadOperand::Imm(0x23),
                    LoadOperand::Imm(5),
                    StoreOperand::Push,
                )),
            ),
            // Duplicate the return value on the stack.
            (None, Item::Instr(Instr::Stkcopy(LoadOperand::Imm(1)))),
            // Jump to fail_label if we got a null return.
            (
                None,
                Item::Instr(Instr::Jz(
                    LoadOperand::Pop,
                    LoadOperand::OffsetLabel(fail_label),
                )),
            ),
            // Call glk_set_window (0x21) with one argument (the winid that's on
            // the stack) to set our new window as current.
            (
                None,
                Item::Instr(Instr::Glk(
                    LoadOperand::Imm(0x2f),
                    LoadOperand::Imm(1),
                    StoreOperand::Discard,
                )),
            ),
            // Print our message.
            (
                None,
                Item::Instr(Instr::Streamstr(LoadOperand::ImmLabel(hello_sailor_label))),
            ),
            // Return from main.
            (
                Some(fail_label),
                Item::Instr(Instr::Return(LoadOperand::Imm(0))),
            ),
        ]),
        ram_items: Cow::Owned(vec![]),
        zero_items: Cow::Owned(vec![]),
        stack_size: 256,
        start_func: ItemRef::Label(main_label),
        decoding_table: None,
    };

    let bytes = assembly.assemble().unwrap();
    let mut stdout = std::io::stdout();
    stdout.write_all(&bytes).unwrap();
}
