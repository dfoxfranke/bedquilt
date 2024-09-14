use super::{
    classify::Test,
    toplevel::{gen_copies, Credits, Debts, Frame, JumpTarget},
};

use crate::common::{vt_words, Context, LabelGenerator};
use glulx_asm::{concise::*, LoadOperand};
use walrus::{ir, Type, ValType};

pub fn gen_test<G>(
    ctx: &mut Context<G>,
    test: Test,
    label: G::Label,
    mut credits: Credits<G::Label>,
) where
    G: LabelGenerator,
{
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

pub fn gen_call<G>(
    ctx: &mut Context<G>,
    _frame: &mut Frame<G::Label>,
    call: &ir::Call,
    credits: Credits<G::Label>,
    debts: Debts<G::Label>,
) where
    G: LabelGenerator,
{
    let function = ctx.module.funcs.get(call.func);
    let ty = ctx.module.types.get(function.ty());
    let addr = ctx.layout.func(call.func).addr.clone();

    gen_call_inner(ctx, ty, addr, credits, debts);
}

fn gen_call_inner<G>(
    ctx: &mut Context<G>,
    ty: &Type,
    addr: G::Label,
    mut credits: Credits<G::Label>,
    mut debts: Debts<G::Label>,
) where
    G: LabelGenerator,
{
    let param_words: u32 = ty.params().iter().map(|vt| vt_words(*vt)).sum();
    let result_words: u32 = ty.results().iter().map(|vt| vt_words(*vt)).sum();

    let return_operand = if result_words > 0 {
        if result_words == 1 && debts.0.len() == 1 {
            debts.pop()
        } else {
            push()
        }
    } else {
        discard()
    };

    match param_words {
        0 => {
            std::mem::take(&mut credits).gen(ctx);
            ctx.rom_items.push(callf(imml(addr), return_operand));
        }
        1 => {
            let arg_a = credits.pop();
            std::mem::take(&mut credits).gen(ctx);
            ctx.rom_items
                .push(callfi(imml(addr), arg_a, return_operand));
        }
        2 => {
            let arg_a = credits.pop();
            let arg_b = credits.pop();
            std::mem::take(&mut credits).gen(ctx);
            ctx.rom_items
                .push(callfii(imml(addr), arg_a, arg_b, return_operand));
        }
        3 => {
            let arg_a = credits.pop();
            let arg_b = credits.pop();
            let arg_c = credits.pop();
            std::mem::take(&mut credits).gen(ctx);
            ctx.rom_items
                .push(callfiii(imml(addr), arg_a, arg_b, arg_c, return_operand));
        }
        _ => {
            std::mem::take(&mut credits).gen(ctx);
            ctx.rom_items
                .push(call(imml(addr), uimm(param_words), return_operand));
        }
    }

    if result_words > 0 {
        for i in (0..result_words - 1).rev() {
            credits.0.push(derefl_off(
                ctx.layout.hi_return().addr.clone(),
                4 * i as i32,
            ));
        }
    }

    gen_copies(ctx, credits, debts);
}

fn gen_br_inner<G>(
    ctx: &mut Context<G>,
    frame: &Frame<G::Label>,
    target: &JumpTarget<G::Label>,
    height: usize,
) where
    G: LabelGenerator,
{
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
            ctx.rom_items.push(stkroll(imm(total_i32), imm(total_i32 - drop_i32)));
        }
        for _ in 0..drop {
            ctx.rom_items.push(copy(pop(), discard()));
        }
        ctx.rom_items.push(jump(target.target.clone()));
    }
}

pub fn gen_br<G>(
    ctx: &mut Context<G>,
    frame: &mut Frame<G::Label>,
    br: &ir::Br,
    height: usize,
    credits: Credits<G::Label>,
    debts: Debts<G::Label>,
) where
    G: LabelGenerator,
{
    let ir::Br { block: id } = br;
    let target = frame
        .jump_targets
        .get(id)
        .expect("Branch target should be present on stack");
    credits.gen(ctx);
    gen_br_inner(ctx, frame, target, height);
    debts.gen(ctx);
}

pub fn gen_br_if<G>(
    ctx: &mut Context<G>,
    frame: &mut Frame<G::Label>,
    test: Test,
    br_if: &ir::BrIf,
    height: usize,
    credits: Credits<G::Label>,
    debts: Debts<G::Label>,
) where
    G: LabelGenerator,
{
    let height = height - test.popped_words();
    let ir::BrIf { block: id } = br_if;
    let target = frame
        .jump_targets
        .get(id)
        .expect("Branch target should be present on stack");

    if height == target.base + target.arity {
        gen_test(ctx, test, target.target.clone(), credits);
    } else {
        let branch_prep = ctx.gen.gen("branch_prep");
        let no_branch = ctx.gen.gen("no_branch");
        gen_test(ctx, test, branch_prep.clone(), credits);
        ctx.rom_items.push(jump(no_branch.clone()));
        ctx.rom_items.push(label(branch_prep));
        gen_br_inner(ctx, frame, target, height);
        ctx.rom_items.push(label(no_branch));
    }
    debts.gen(ctx);
}

pub fn gen_br_table<G>(
    ctx: &mut Context<G>,
    frame: &mut Frame<G::Label>,
    br_table: &ir::BrTable,
    height: usize,
    mut credits: Credits<G::Label>,
    debts: Debts<G::Label>,
) where
    G: LabelGenerator,
{
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
        default_target.target.clone()
    } else {
        ctx.gen.gen("brtable_default")
    };

    ctx.rom_items.push(jgeu(
        test_value.clone(),
        uimm(jump_table_len),
        default_label.clone(),
    ));

    let jump_table_label = ctx.gen.gen("jump_table");
    ctx.rom_items.push(aload(
        imml(jump_table_label.clone()),
        test_value.clone(),
        push(),
    ));
    ctx.rom_items.push(jumpabs(pop()));

    let mut jump_table = Vec::with_capacity(br_table.blocks.len());
    for block in &br_table.blocks {
        let target = frame
            .jump_targets
            .get(block)
            .expect("Branch target should be present on stack");
        if height - 1 == target.base + target.arity {
            jump_table.push(target.target.clone());
        } else {
            let prepare = ctx.gen.gen("brtable_prepare");
            jump_table.push(prepare.clone());
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

    debts.gen(ctx)
}

pub fn gen_select<G>(
    ctx: &mut Context<G>,
    _frame: &mut Frame<G::Label>,
    test: Test,
    _select: &ir::Select,
    post_stack: &[ValType],
    credits: Credits<G::Label>,
    debts: Debts<G::Label>,
) where
    G: LabelGenerator,
{
    let noroll = ctx.gen.gen("noroll");
    let words = vt_words(
        *post_stack
            .last()
            .expect("Stack should not be empty after a select"),
    ) as i32;

    gen_test(ctx, test, noroll.clone(), credits);
    ctx.rom_items.push(stkroll(imm(2 * words), imm(words)));
    ctx.rom_items.push(label(noroll));
    for _ in 0..words {
        ctx.rom_items.push(copy(pop(), discard()));
    }
    debts.gen(ctx);
}
