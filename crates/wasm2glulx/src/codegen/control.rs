use super::toplevel::{Frame,Credits,Debts, gen_copies};

use crate::common::{vt_words, Context, LabelGenerator};
use glulx_asm::concise::*;
use walrus::{ir::Call, Type};

pub fn gen_call<G>(ctx: &mut Context<G>, _frame: &mut Frame<G::Label>, call: &Call, credits: Credits<G::Label>, debts: Debts<G::Label>)
where
    G: LabelGenerator,
{
    let function = ctx.module.funcs.get(call.func);
    let ty = ctx.module.types.get(function.ty());
    let addr = ctx.layout.func(call.func).addr.clone();

    gen_call_inner(ctx, ty, addr, credits, debts);
}

fn gen_call_inner<G>(ctx: &mut Context<G>, ty: &Type, addr: G::Label, mut credits: Credits<G::Label>, mut debts: Debts<G::Label>)
where
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
