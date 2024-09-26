// SPDX-License-Identifier: Apache-2.0 WITH LLVM-Exception
// Copyright 2024 Daniel Fox Franke.

use super::{
    classify::Test,
    loadstore::{gen_copies, Credits, Debts},
    toplevel::{Frame, JumpTarget},
};

use crate::common::{Context, Label, WordCount};
use glulx_asm::{concise::*, LoadOperand};
use walrus::{ir, ValType};

pub fn gen_test(ctx: &mut Context, test: Test, label: Label, mut credits: Credits) {
    match test {
        Test::I32Nez => {
            let operand = credits.pop();
            credits.gen(ctx);
            ctx.rom_items.push(jnz(operand, label));
        }
        Test::I32Eqz => {
            let operand = credits.pop();
            credits.gen(ctx);
            ctx.rom_items.push(jz(operand, label));
        }
        Test::I32Eq => {
            let y = credits.pop();
            let x = credits.pop();
            credits.gen(ctx);
            ctx.rom_items.push(jeq(x, y, label));
        }
        Test::I32Ne => {
            let y = credits.pop();
            let x = credits.pop();
            credits.gen(ctx);
            ctx.rom_items.push(jne(x, y, label));
        }
        Test::I32LtS => {
            let y = credits.pop();
            let x = credits.pop();
            credits.gen(ctx);
            ctx.rom_items.push(jgt(y, x, label));
        }
        Test::I32LtU => {
            let y = credits.pop();
            let x = credits.pop();
            credits.gen(ctx);
            ctx.rom_items.push(jgtu(y, x, label));
        }
        Test::I32GtS => {
            let y = credits.pop();
            let x = credits.pop();
            credits.gen(ctx);
            ctx.rom_items.push(jlt(y, x, label));
        }
        Test::I32GtU => {
            let y = credits.pop();
            let x = credits.pop();
            credits.gen(ctx);
            ctx.rom_items.push(jltu(y, x, label));
        }
        Test::I32LeS => {
            let y = credits.pop();
            let x = credits.pop();
            credits.gen(ctx);
            ctx.rom_items.push(jge(y, x, label));
        }
        Test::I32LeU => {
            let y = credits.pop();
            let x = credits.pop();
            credits.gen(ctx);
            ctx.rom_items.push(jgeu(y, x, label));
        }
        Test::I32GeS => {
            let y = credits.pop();
            let x = credits.pop();
            credits.gen(ctx);
            ctx.rom_items.push(jle(y, x, label));
        }
        Test::I32GeU => {
            let y = credits.pop();
            let x = credits.pop();
            credits.gen(ctx);
            ctx.rom_items.push(jleu(y, x, label));
        }
        Test::F32Eq => {
            let y = credits.pop();
            let x = credits.pop();
            credits.gen(ctx);
            ctx.rom_items.push(jfeq(y, x, imm(0), label));
        }
        Test::F32Ne => {
            let y = credits.pop();
            let x = credits.pop();
            credits.gen(ctx);
            ctx.rom_items.push(jfne(y, x, imm(0), label));
        }
        Test::F32Lt => {
            let y = credits.pop();
            let x = credits.pop();
            credits.gen(ctx);
            ctx.rom_items.push(jfgt(y, x, label));
        }
        Test::F32Gt => {
            let y = credits.pop();
            let x = credits.pop();
            credits.gen(ctx);
            ctx.rom_items.push(jflt(y, x, label));
        }
        Test::F32Le => {
            let y = credits.pop();
            let x = credits.pop();
            credits.gen(ctx);
            ctx.rom_items.push(jfge(y, x, label));
        }
        Test::F32Ge => {
            let y = credits.pop();
            let x = credits.pop();
            credits.gen(ctx);
            ctx.rom_items.push(jfle(y, x, label));
        }
        Test::F64Eq => {
            let y_hi = credits.pop();
            let y_lo = credits.pop();
            let x_hi = credits.pop();
            let x_lo = credits.pop();
            credits.gen(ctx);
            ctx.rom_items
                .push(jdeq(y_hi, y_lo, x_hi, x_lo, imm(0), imm(0), label));
        }
        Test::F64Ne => {
            let y_hi = credits.pop();
            let y_lo = credits.pop();
            let x_hi = credits.pop();
            let x_lo = credits.pop();
            credits.gen(ctx);
            ctx.rom_items
                .push(jdne(y_hi, y_lo, x_hi, x_lo, imm(0), imm(0), label));
        }
        Test::F64Lt => {
            let y_hi = credits.pop();
            let y_lo = credits.pop();
            let x_hi = credits.pop();
            let x_lo = credits.pop();
            credits.gen(ctx);
            ctx.rom_items.push(jdgt(y_hi, y_lo, x_hi, x_lo, label));
        }
        Test::F64Gt => {
            let y_hi = credits.pop();
            let y_lo = credits.pop();
            let x_hi = credits.pop();
            let x_lo = credits.pop();
            credits.gen(ctx);
            ctx.rom_items.push(jdlt(y_hi, y_lo, x_hi, x_lo, label));
        }
        Test::F64Le => {
            let y_hi = credits.pop();
            let y_lo = credits.pop();
            let x_hi = credits.pop();
            let x_lo = credits.pop();
            credits.gen(ctx);
            ctx.rom_items.push(jdge(y_hi, y_lo, x_hi, x_lo, label));
        }
        Test::F64Ge => {
            let y_hi = credits.pop();
            let y_lo = credits.pop();
            let x_hi = credits.pop();
            let x_lo = credits.pop();
            credits.gen(ctx);
            ctx.rom_items.push(jdle(y_hi, y_lo, x_hi, x_lo, label));
        }
    }
}

pub fn gen_call(
    ctx: &mut Context,
    _frame: &mut Frame,
    call_instr: &ir::Call,
    mut credits: Credits,
    mut debts: Debts,
) {
    let function = ctx.module.funcs.get(call_instr.func);
    let ty = ctx.module.types.get(function.ty());
    let addr = imml(ctx.layout.func(call_instr.func).addr);

    let param_words: u32 = ty.params().word_count();
    let result_words: u32 = ty.results().word_count();

    let return_operand = if result_words > 0 {
        if result_words == 1 && debts.len() == 1 {
            debts.pop()
        } else {
            push()
        }
    } else {
        discard()
    };

    match param_words {
        0 => {
            credits.gen(ctx);
            ctx.rom_items.push(callf(addr, return_operand));
        }
        1 => {
            let arg_a = credits.pop();
            credits.gen(ctx);
            ctx.rom_items.push(callfi(addr, arg_a, return_operand));
        }
        2 => {
            let arg_a = credits.pop();
            let arg_b = credits.pop();
            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(addr, arg_a, arg_b, return_operand));
        }
        3 => {
            let arg_a = credits.pop();
            let arg_b = credits.pop();
            let arg_c = credits.pop();
            credits.gen(ctx);
            ctx.rom_items
                .push(callfiii(addr, arg_a, arg_b, arg_c, return_operand));
        }
        _ => {
            credits.gen(ctx);
            ctx.rom_items
                .push(call(addr, uimm(param_words), return_operand));
        }
    }

    let return_credits = Credits::from_returns(ctx, ty.results());
    gen_copies(ctx, return_credits, debts);
}

pub fn gen_call_indirect(
    ctx: &mut Context,
    _frame: &mut Frame,
    call_indirect: &ir::CallIndirect,
    mut credits: Credits,
    mut debts: Debts,
) {
    let ty = ctx.module.types.get(call_indirect.ty);
    let typenum = ctx.layout.ty(call_indirect.ty).typenum;
    let table_addr = ctx.layout.table(call_indirect.table).addr;
    let table_count = ctx.layout.table(call_indirect.table).cur_count;
    let param_words = ty.params().word_count();
    let result_words: u32 = ty.results().word_count();

    let return_operand = if result_words > 0 {
        if result_words == 1 && debts.len() == 1 {
            debts.pop()
        } else {
            push()
        }
    } else {
        discard()
    };

    let table_index = credits.pop();
    credits.gen(ctx);

    // Steal hi_return as a scratch register
    let fnptr = ctx.layout.hi_return().addr;

    if matches!(table_index, LoadOperand::Pop) {
        ctx.rom_items.push(stkpeek(imm(0), push()));
        ctx.rom_items.push(jgeu(
            pop(),
            derefl(table_count),
            ctx.rt.trap_undefined_element,
        ));
    } else {
        ctx.rom_items.push(jgeu(
            table_index,
            derefl(table_count),
            ctx.rt.trap_undefined_element,
        ));
    }
    ctx.rom_items
        .push(aload(imml(table_addr), table_index, storel(fnptr)));
    ctx.rom_items
        .push(jz(derefl(fnptr), ctx.rt.trap_uninitialized_element));
    ctx.rom_items.push(aload(derefl(fnptr), imm(-1), push()));
    ctx.rom_items.push(jne(
        pop(),
        uimm(typenum),
        ctx.rt.trap_indirect_call_type_mismatch,
    ));
    ctx.rom_items
        .push(call(derefl(fnptr), uimm(param_words), return_operand));

    let return_credits = Credits::from_returns(ctx, ty.results());
    gen_copies(ctx, return_credits, debts);
}

fn gen_br_inner(ctx: &mut Context, frame: &Frame, target: &JumpTarget, height: usize) {
    if target.base + target.arity != height {
        assert!(height > target.base + target.arity);
        let total = height - target.base;
        let drop = total - target.arity;

        let total_i32: i32 = if let Ok(total_i32) = total.try_into() {
            total_i32
        } else {
            ctx.errors.push(crate::CompilationError::Overflow(
                crate::OverflowLocation::Stack(frame.function_name.map(|s| s.to_owned())),
            ));
            return;
        };

        let drop_i32: i32 = drop
            .try_into()
            .expect("If total fits in an i32 then the smaller drop amount should too");

        if drop_i32 != total_i32 {
            ctx.rom_items
                .push(stkroll(imm(total_i32), imm(total_i32 - drop_i32)));
        }
        for _ in 0..drop {
            ctx.rom_items.push(copy(pop(), discard()));
        }
    }
    ctx.rom_items.push(jump(target.target));
}

pub fn gen_br(
    ctx: &mut Context,
    frame: &mut Frame,
    br: &ir::Br,
    height: usize,
    mut credits: Credits,
) {
    let ir::Br { block: id } = br;
    let target = frame
        .jump_targets
        .get(id)
        .expect("Branch target should be present on stack");
    credits.gen(ctx);
    gen_br_inner(ctx, frame, target, height);
}

pub fn gen_br_if(
    ctx: &mut Context,
    frame: &mut Frame,
    test: Test,
    br_if: &ir::BrIf,
    height: usize,
    credits: Credits,
    mut debts: Debts,
) {
    let height = height - test.popped_words();
    let ir::BrIf { block: id } = br_if;
    let target = frame
        .jump_targets
        .get(id)
        .expect("Branch target should be present on stack");

    if height == target.base + target.arity {
        gen_test(ctx, test, target.target, credits);
    } else {
        let branch_prep = ctx.gen.gen("branch_prep");
        let no_branch = ctx.gen.gen("no_branch");
        gen_test(ctx, test, branch_prep, credits);
        ctx.rom_items.push(jump(no_branch));
        ctx.rom_items.push(label(branch_prep));
        gen_br_inner(ctx, frame, target, height);
        ctx.rom_items.push(label(no_branch));
    }
    debts.gen(ctx);
}

pub fn gen_br_table(
    ctx: &mut Context,
    frame: &mut Frame,
    br_table: &ir::BrTable,
    height: usize,
    mut credits: Credits,
) {
    let default_target = frame
        .jump_targets
        .get(&br_table.default)
        .expect("Branch target should be present on stack");

    let jump_table_len: u32 = if let Ok(len) = br_table.blocks.len().try_into() {
        len
    } else {
        ctx.errors.push(crate::CompilationError::Overflow(
            crate::OverflowLocation::Table,
        ));
        return;
    };

    let test_value = credits.pop();
    credits.gen(ctx);

    let simple_default = if test_value == LoadOperand::Pop {
        ctx.rom_items.push(stkpeek(imm(0), push()));
        false
    } else {
        height - 1 == default_target.base + default_target.arity
    };

    let default_label = if simple_default {
        default_target.target
    } else {
        ctx.gen.gen("brtable_default")
    };

    ctx.rom_items
        .push(jgeu(test_value, uimm(jump_table_len), default_label));

    let jump_table_label = ctx.gen.gen("jump_table");
    ctx.rom_items
        .push(aload(imml(jump_table_label), test_value, push()));
    ctx.rom_items.push(jumpabs(pop()));

    let mut jump_table = Vec::with_capacity(br_table.blocks.len());
    for block in &br_table.blocks {
        let target = frame
            .jump_targets
            .get(block)
            .expect("Branch target should be present on stack");
        if height - 1 == target.base + target.arity {
            jump_table.push(target.target);
        } else {
            let prepare = ctx.gen.gen("brtable_prepare");
            jump_table.push(prepare);
            ctx.rom_items.push(label(prepare));
            gen_br_inner(ctx, frame, target, height - 1);
        }
    }

    frame.jump_tables.insert(jump_table_label, jump_table);

    if !simple_default {
        ctx.rom_items.push(label(default_label));
        if test_value == LoadOperand::Pop {
            // Corresponds to the stkpeek from earlier
            ctx.rom_items.push(copy(pop(), discard()));
        }
        gen_br_inner(ctx, frame, default_target, height - 1);
    }
}

pub fn gen_select(
    ctx: &mut Context,
    _frame: &mut Frame,
    test: Test,
    _select: &ir::Select,
    post_stack: &[ValType],
    credits: Credits,
    mut debts: Debts,
) {
    let noroll = ctx.gen.gen("noroll");
    let words: u8 = post_stack
        .last()
        .expect("Stack should not be empty after a select")
        .word_count();

    gen_test(ctx, test, noroll, credits);

    ctx.rom_items
        .push(stkroll(uimm(u32::from(words) * 2), imm(words.into())));
    ctx.rom_items.push(label(noroll));
    for _ in 0..words {
        ctx.rom_items.push(copy(pop(), discard()));
    }

    debts.gen(ctx);
}

pub fn gen_unreachable(
    ctx: &mut Context,
    _frame: &mut Frame,
    _unreachable: &ir::Unreachable,
    mut credits: Credits,
) {
    credits.gen(ctx);
    ctx.rom_items.push(jump(ctx.rt.trap_unreachable));
}
