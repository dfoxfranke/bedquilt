use super::{
    classify::Test,
    toplevel::{gen_copies, Credits, Debts, Frame},
};

use crate::common::{vt_words, Context, LabelGenerator};
use glulx_asm::concise::*;
use walrus::{ir::Call, Type};

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
    call: &Call,
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
            ctx.rom_items.push(callf(imml(addr), return_operand));
        }
        1 => {
            let arg_a = credits.pop();
            ctx.rom_items
                .push(callfi(imml(addr), arg_a, return_operand));
        }
        2 => {
            let arg_a = credits.pop();
            let arg_b = credits.pop();
            ctx.rom_items
                .push(callfii(imml(addr), arg_a, arg_b, return_operand));
        }
        3 => {
            let arg_a = credits.pop();
            let arg_b = credits.pop();
            let arg_c = credits.pop();
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
