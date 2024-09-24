use super::classify::{ClassifiedInstr, Other};
use super::loadstore::{copy_if_sensible, gen_copies, Credits, Debts};
use super::toplevel::Frame;
use crate::common::*;
use glulx_asm::{concise::*, LoadOperand, StoreOperand};
use walrus::ir;

pub fn gen_unop(
    ctx: &mut Context,
    frame: &Frame,
    unop: &ir::Unop,
    mut credits: Credits,
    mut debts: Debts,
) {
    match unop.op {
        ir::UnaryOp::I32Eqz => {
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(callfi(imml(ctx.rt.i32_eqz), x, out));
            debts.gen(ctx);
        }
        ir::UnaryOp::I32Clz => {
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(callfi(imml(ctx.rt.i32_clz), x, out));
            debts.gen(ctx);
        }
        ir::UnaryOp::I32Ctz => {
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(callfi(imml(ctx.rt.i32_ctz), x, out));
            debts.gen(ctx);
        }
        ir::UnaryOp::I32Popcnt => {
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(callfi(imml(ctx.rt.i32_popcnt), x, out));
            debts.gen(ctx);
        }
        ir::UnaryOp::I64Eqz => {
            let (x_hi, x_lo) = credits.pop_hi_lo();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.i64_eqz), x_hi, x_lo, out));
            debts.gen(ctx);
        }
        ir::UnaryOp::I64Clz => {
            let (x_hi, x_lo) = credits.pop_hi_lo();
            let (out_lo, out_hi) = debts.pop_lo_hi();

            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.i64_clz), x_hi, x_lo, out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::UnaryOp::I64Ctz => {
            let (x_hi, x_lo) = credits.pop_hi_lo();
            let (out_lo, out_hi) = debts.pop_lo_hi();

            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.i64_ctz), x_hi, x_lo, out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::UnaryOp::I64Popcnt => {
            let (x_hi, x_lo) = credits.pop_hi_lo();
            let (out_lo, out_hi) = debts.pop_lo_hi();

            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.i64_popcnt), x_hi, x_lo, out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::UnaryOp::I32WrapI64 => {
            let x_hi = credits.pop();
            credits.gen(ctx);
            copy_if_sensible(ctx, x_hi, discard());
            debts.gen(ctx);
        }
        ir::UnaryOp::I64ExtendSI32 => {
            let x = credits.pop();
            let (out_lo, out_hi) = debts.pop_lo_hi();

            credits.gen(ctx);
            if matches!(x, LoadOperand::Pop) {
                ctx.rom_items.push(stkpeek(imm(0), push()));
                copy_if_sensible(ctx, pop(), out_lo);
                ctx.rom_items.push(sshiftr(pop(), imm(63), out_hi));
            } else {
                copy_if_sensible(ctx, x, out_lo);
                if !matches!(out_hi, StoreOperand::Discard) {
                    ctx.rom_items.push(sshiftr(x, imm(63), out_hi));
                }
            }
            debts.gen(ctx);
        }
        ir::UnaryOp::I64ExtendUI32 => {
            let x = credits.pop();
            let (out_lo, out_hi) = debts.pop_lo_hi();

            credits.gen(ctx);
            copy_if_sensible(ctx, x, out_lo);
            ctx.rom_items.push(copy(imm(0), out_hi));
            debts.gen(ctx);
        }
        ir::UnaryOp::I32Extend8S => {
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(sexb(x, out));
            debts.gen(ctx);
        }
        ir::UnaryOp::I32Extend16S => {
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(sexs(x, out));
            debts.gen(ctx);
        }
        ir::UnaryOp::I64Extend8S => {
            let (x_hi, x_lo) = credits.pop_hi_lo();
            let (out_lo, out_hi) = debts.pop_lo_hi();

            credits.gen(ctx);
            copy_if_sensible(ctx, x_hi, discard());
            ctx.rom_items.push(sexb(x_lo, push()));
            ctx.rom_items.push(stkpeek(imm(0), out_lo));
            ctx.rom_items.push(sshiftr(pop(), imm(63), out_hi));
            debts.gen(ctx);
        }
        ir::UnaryOp::I64Extend16S => {
            let (x_hi, x_lo) = credits.pop_hi_lo();
            let (out_lo, out_hi) = debts.pop_lo_hi();

            credits.gen(ctx);
            copy_if_sensible(ctx, x_hi, discard());
            ctx.rom_items.push(sexs(x_lo, push()));
            ctx.rom_items.push(stkpeek(imm(0), out_lo));
            ctx.rom_items.push(sshiftr(pop(), imm(63), out_hi));
            debts.gen(ctx);
        }
        ir::UnaryOp::I64Extend32S => {
            let (x_hi, x_lo) = credits.pop_hi_lo();
            let (out_lo, out_hi) = debts.pop_lo_hi();

            credits.gen(ctx);
            copy_if_sensible(ctx, x_hi, discard());
            copy_if_sensible(ctx, x_lo, push());
            ctx.rom_items.push(stkpeek(imm(0), out_lo));
            ctx.rom_items.push(sshiftr(pop(), imm(63), out_hi));
            debts.gen(ctx);
        }

        ir::UnaryOp::F32Abs | ir::UnaryOp::F64Abs => {
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(bitand(x, imm(0x7fffffff), out));
            debts.gen(ctx);
        }
        ir::UnaryOp::F32Neg | ir::UnaryOp::F64Neg => {
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(bitxor(x, uimm(0x80000000), out));
            debts.gen(ctx);
        }
        ir::UnaryOp::F32Ceil => {
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(ceil(x, out));
            debts.gen(ctx);
        }

        ir::UnaryOp::F32Floor => {
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(floor(x, out));
            debts.gen(ctx);
        }
        ir::UnaryOp::F32Sqrt => {
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(sqrt(x, out));
            debts.gen(ctx);
        }
        ir::UnaryOp::F32Trunc => {
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(callfi(imml(ctx.rt.f32_trunc), x, out));
            debts.gen(ctx);
        }
        ir::UnaryOp::F32Nearest => {
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(callfi(imml(ctx.rt.f32_nearest), x, out));
            debts.gen(ctx);
        }
        ir::UnaryOp::I32TruncSF32 => {
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items
                .push(callfi(imml(ctx.rt.i32_trunc_s_f32), x, out));
            debts.gen(ctx);
        }
        ir::UnaryOp::I32TruncUF32 => {
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items
                .push(callfi(imml(ctx.rt.i32_trunc_u_f32), x, out));
            debts.gen(ctx);
        }
        ir::UnaryOp::I64TruncSF32 => {
            let x = credits.pop();
            let (out_lo, out_hi) = debts.pop_lo_hi();

            credits.gen(ctx);
            ctx.rom_items
                .push(callfi(imml(ctx.rt.i64_trunc_s_f32), x, out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::UnaryOp::I64TruncUF32 => {
            let x = credits.pop();
            let (out_lo, out_hi) = debts.pop_lo_hi();

            credits.gen(ctx);
            ctx.rom_items
                .push(callfi(imml(ctx.rt.i64_trunc_u_f32), x, out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::UnaryOp::F32ConvertSI32 => {
            let x = credits.pop();
            let out = debts.pop();
            credits.gen(ctx);
            ctx.rom_items.push(numtof(x, out));
            debts.gen(ctx);
        }
        ir::UnaryOp::F32ConvertUI32 => {
            let x = credits.pop();
            let out = debts.pop();
            credits.gen(ctx);
            ctx.rom_items
                .push(callfi(imml(ctx.rt.f32_convert_i32_u), x, out));
            debts.gen(ctx);
        }
        ir::UnaryOp::F32ConvertUI64 => {
            let (hi, lo) = credits.pop_hi_lo();
            let out = debts.pop();
            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.f32_convert_i64_u), hi, lo, out));
            debts.gen(ctx);
        }
        ir::UnaryOp::F32ConvertSI64 => {
            let (hi, lo) = credits.pop_hi_lo();
            let out = debts.pop();
            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.f32_convert_i64_s), hi, lo, out));
            debts.gen(ctx);
        }
        ir::UnaryOp::F64Ceil => {
            let (x_hi, x_lo) = credits.pop_hi_lo();
            let (out_lo, out_hi) = debts.pop_lo_hi();

            credits.gen(ctx);
            ctx.rom_items.push(dceil(x_hi, x_lo, out_lo, out_hi));
            debts.gen(ctx);
        }
        ir::UnaryOp::F64Floor => {
            let (x_hi, x_lo) = credits.pop_hi_lo();
            let (out_lo, out_hi) = debts.pop_lo_hi();

            credits.gen(ctx);
            ctx.rom_items.push(dfloor(x_hi, x_lo, out_lo, out_hi));
            debts.gen(ctx);
        }
        ir::UnaryOp::F64Sqrt => {
            let (x_hi, x_lo) = credits.pop_hi_lo();
            let (out_lo, out_hi) = debts.pop_lo_hi();

            credits.gen(ctx);
            ctx.rom_items.push(dsqrt(x_hi, x_lo, out_lo, out_hi));
            debts.gen(ctx);
        }
        ir::UnaryOp::F64Trunc => {
            let (x_hi, x_lo) = credits.pop_hi_lo();
            let (out_lo, out_hi) = debts.pop_lo_hi();

            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.f64_trunc), x_hi, x_lo, out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::UnaryOp::F64Nearest => {
            let (x_hi, x_lo) = credits.pop_hi_lo();
            let (out_lo, out_hi) = debts.pop_lo_hi();

            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.f64_nearest), x_hi, x_lo, out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::UnaryOp::F32DemoteF64 => {
            let (x_hi, x_lo) = credits.pop_hi_lo();
            let out = debts.pop();
            credits.gen(ctx);
            ctx.rom_items.push(dtof(x_hi, x_lo, out));
            debts.gen(ctx);
        }
        ir::UnaryOp::F64PromoteF32 => {
            let x = credits.pop();
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items.push(ftod(x, out_lo, out_hi));
            debts.gen(ctx);
        }
        ir::UnaryOp::I32TruncSF64 => {
            let (x_hi, x_lo) = credits.pop_hi_lo();
            let out = debts.pop();
            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.i32_trunc_s_f64), x_hi, x_lo, out));
            debts.gen(ctx);
        }
        ir::UnaryOp::I32TruncUF64 => {
            let (x_hi, x_lo) = credits.pop_hi_lo();
            let out = debts.pop();
            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.i32_trunc_u_f64), x_hi, x_lo, out));
            debts.gen(ctx);
        }
        ir::UnaryOp::I64TruncUF64 => {
            let (x_hi, x_lo) = credits.pop_hi_lo();
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.i64_trunc_u_f64), x_hi, x_lo, out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::UnaryOp::I64TruncSF64 => {
            let (x_hi, x_lo) = credits.pop_hi_lo();
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.i64_trunc_s_f64), x_hi, x_lo, out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::UnaryOp::F64ConvertSI32 => {
            let x = credits.pop();
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items.push(numtod(x, out_lo, out_hi));
            debts.gen(ctx);
        }
        ir::UnaryOp::F64ConvertUI32 => {
            let x = credits.pop();
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(callfi(imml(ctx.rt.f64_convert_i32_u), x, out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::UnaryOp::F64ConvertUI64 => {
            let (x_hi, x_lo) = credits.pop_hi_lo();
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.f64_convert_i64_u), x_hi, x_lo, out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::UnaryOp::F64ConvertSI64 => {
            let (x_hi, x_lo) = credits.pop_hi_lo();
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.f64_convert_i64_s), x_hi, x_lo, out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::UnaryOp::I32ReinterpretF32
        | ir::UnaryOp::F32ReinterpretI32
        | ir::UnaryOp::I64ReinterpretF64
        | ir::UnaryOp::F64ReinterpretI64 => {
            gen_copies(ctx, credits, debts);
        }
        ir::UnaryOp::I32TruncSSatF32 => {
            let x = credits.pop();
            let out = debts.pop();
            credits.gen(ctx);
            ctx.rom_items
                .push(callfi(imml(ctx.rt.i32_trunc_sat_s_f32), x, out));
            debts.gen(ctx);
        }
        ir::UnaryOp::I32TruncUSatF32 => {
            let x = credits.pop();
            let out = debts.pop();
            credits.gen(ctx);
            ctx.rom_items
                .push(callfi(imml(ctx.rt.i32_trunc_sat_u_f32), x, out));
            debts.gen(ctx);
        }
        ir::UnaryOp::I64TruncSSatF32 => {
            let x = credits.pop();
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(callfi(imml(ctx.rt.i64_trunc_sat_s_f32), x, out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::UnaryOp::I64TruncUSatF32 => {
            let x = credits.pop();
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(callfi(imml(ctx.rt.i64_trunc_sat_u_f32), x, out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::UnaryOp::I32TruncSSatF64 => {
            let (x_hi, x_lo) = credits.pop_hi_lo();
            let out = debts.pop();
            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.i32_trunc_sat_s_f64), x_hi, x_lo, out));
            debts.gen(ctx);
        }
        ir::UnaryOp::I32TruncUSatF64 => {
            let (x_hi, x_lo) = credits.pop_hi_lo();
            let out = debts.pop();
            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.i32_trunc_sat_u_f64), x_hi, x_lo, out));
            debts.gen(ctx);
        }
        ir::UnaryOp::I64TruncSSatF64 => {
            let (x_hi, x_lo) = credits.pop_hi_lo();
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items.push(callfii(
                imml(ctx.rt.i64_trunc_sat_s_f64),
                x_hi,
                x_lo,
                out_lo,
            ));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::UnaryOp::I64TruncUSatF64 => {
            let (x_hi, x_lo) = credits.pop_hi_lo();
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items.push(callfii(
                imml(ctx.rt.i64_trunc_sat_u_f64),
                x_hi,
                x_lo,
                out_lo,
            ));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        _ => {
            credits.gen(ctx);
            let mnemonic = Other::Unop(unop.clone()).mnemonic();
            ctx.errors
                .push(crate::CompilationError::UnsupportedInstruction {
                    function: frame.function_name.map(|s| s.to_owned()),
                    instr: mnemonic,
                });
            debts.gen(ctx);
        }
    }
}

pub fn gen_binop(
    ctx: &mut Context,
    frame: &Frame,
    binop: &ir::Binop,
    mut credits: Credits,
    mut debts: Debts,
) {
    match binop.op {
        ir::BinaryOp::I32Eq => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(callfii(imml(ctx.rt.i32_eq), y, x, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I32Ne => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(callfii(imml(ctx.rt.i32_ne), y, x, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I32LtS => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.i32_lt_s), y, x, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I32LtU => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.i32_lt_u), y, x, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I32GtS => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.i32_gt_s), y, x, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I32GtU => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.i32_gt_u), y, x, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I32LeS => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.i32_le_s), y, x, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I32LeU => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.i32_le_u), y, x, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I32GeS => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.i32_ge_s), y, x, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I32GeU => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.i32_ge_u), y, x, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I32Add => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(add(x, y, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I32Sub => {
            let (x, y) = credits.pop_swapped_pair(ctx);
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(sub(x, y, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I32Mul => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(mul(x, y, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I32DivS => {
            let (x, y) = credits.pop_swapped_pair(ctx);
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(div(x, y, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I32DivU => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.i32_div_u), y, x, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I32RemS => {
            let (x, y) = credits.pop_swapped_pair(ctx);
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(modulo(x, y, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I32RemU => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.i32_rem_u), y, x, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I32And => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(bitand(x, y, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I32Or => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(bitor(x, y, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I32Xor => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(bitxor(x, y, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I32Shl => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(callfii(imml(ctx.rt.i32_shl), y, x, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I32ShrS => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.i32_shr_s), y, x, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I32ShrU => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.i32_shr_u), y, x, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I32Rotl => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.i32_rotl), y, x, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I32Rotr => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.i32_rotr), y, x, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I64Eq => {
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(call(imml(ctx.rt.i64_eq), imm(4), out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I64Ne => {
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(call(imml(ctx.rt.i64_ne), imm(4), out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I64LtS => {
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(call(imml(ctx.rt.i64_lt_s), imm(4), out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I64LtU => {
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(call(imml(ctx.rt.i64_lt_u), imm(4), out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I64GtS => {
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(call(imml(ctx.rt.i64_gt_s), imm(4), out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I64GtU => {
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(call(imml(ctx.rt.i64_gt_u), imm(4), out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I64LeS => {
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(call(imml(ctx.rt.i64_le_s), imm(4), out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I64LeU => {
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(call(imml(ctx.rt.i64_le_u), imm(4), out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I64GeS => {
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(call(imml(ctx.rt.i64_ge_s), imm(4), out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I64GeU => {
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(call(imml(ctx.rt.i64_ge_u), imm(4), out));
            debts.gen(ctx);
        }
        ir::BinaryOp::I64Add => {
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_add), imm(4), out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::BinaryOp::I64Sub => {
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_sub), imm(4), out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::BinaryOp::I64Mul => {
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_mul), imm(4), out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::BinaryOp::I64DivU => {
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_div_u), imm(4), out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::BinaryOp::I64DivS => {
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_div_s), imm(4), out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::BinaryOp::I64RemU => {
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_rem_u), imm(4), out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::BinaryOp::I64RemS => {
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_rem_s), imm(4), out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::BinaryOp::I64And => {
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_and), imm(4), out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::BinaryOp::I64Or => {
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_or), imm(4), out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::BinaryOp::I64Xor => {
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_xor), imm(4), out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::BinaryOp::I64Shl => {
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_shl), imm(4), out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::BinaryOp::I64ShrS => {
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_shr_s), imm(4), out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::BinaryOp::I64ShrU => {
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_shr_u), imm(4), out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::BinaryOp::I64Rotl => {
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_rotl), imm(4), out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::BinaryOp::I64Rotr => {
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_rotr), imm(4), out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::BinaryOp::F32Eq => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(callfii(imml(ctx.rt.f32_eq), y, x, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::F32Ne => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(callfii(imml(ctx.rt.f32_ne), y, x, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::F32Lt => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(callfii(imml(ctx.rt.f32_lt), y, x, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::F32Gt => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(callfii(imml(ctx.rt.f32_gt), y, x, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::F32Le => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(callfii(imml(ctx.rt.f32_le), y, x, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::F32Ge => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(callfii(imml(ctx.rt.f32_ge), y, x, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::F32Min => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(callfii(imml(ctx.rt.f32_min), y, x, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::F32Max => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(callfii(imml(ctx.rt.f32_max), y, x, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::F32Add => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(fadd(y, x, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::F32Sub => {
            let (x, y) = credits.pop_swapped_pair(ctx);
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(fsub(x, y, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::F32Mul => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(fmul(y, x, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::F32Div => {
            let (x, y) = credits.pop_swapped_pair(ctx);
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items.push(fdiv(x, y, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::F32Copysign => {
            let y = credits.pop();
            let x = credits.pop();
            let out = debts.pop();

            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.f32_copysign), y, x, out));
            debts.gen(ctx);
        }
        ir::BinaryOp::F64Eq => {
            let out = debts.pop();
            credits.gen(ctx);
            ctx.rom_items.push(call(imml(ctx.rt.f64_eq), imm(4), out));
            debts.gen(ctx);
        }
        ir::BinaryOp::F64Ne => {
            let out = debts.pop();
            credits.gen(ctx);
            ctx.rom_items.push(call(imml(ctx.rt.f64_ne), imm(4), out));
            debts.gen(ctx);
        }
        ir::BinaryOp::F64Lt => {
            let out = debts.pop();
            credits.gen(ctx);
            ctx.rom_items.push(call(imml(ctx.rt.f64_lt), imm(4), out));
            debts.gen(ctx);
        }
        ir::BinaryOp::F64Gt => {
            let out = debts.pop();
            credits.gen(ctx);
            ctx.rom_items.push(call(imml(ctx.rt.f64_gt), imm(4), out));
            debts.gen(ctx);
        }
        ir::BinaryOp::F64Le => {
            let out = debts.pop();
            credits.gen(ctx);
            ctx.rom_items.push(call(imml(ctx.rt.f64_le), imm(4), out));
            debts.gen(ctx);
        }
        ir::BinaryOp::F64Ge => {
            let out = debts.pop();
            credits.gen(ctx);
            ctx.rom_items.push(call(imml(ctx.rt.f64_ge), imm(4), out));
            debts.gen(ctx);
        }
        ir::BinaryOp::F64Add => {
            let (y_hi, y_lo) = credits.pop_hi_lo();
            let (x_hi, x_lo) = credits.pop_hi_lo();
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(dadd(y_hi, y_lo, x_hi, x_lo, out_lo, out_hi));
            debts.gen(ctx);
        }
        ir::BinaryOp::F64Sub => {
            let (x_hi, x_lo, y_hi, y_lo) = credits.pop_swapped_quad(ctx);
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(dsub(x_hi, x_lo, y_hi, y_lo, out_lo, out_hi));
            debts.gen(ctx);
        }
        ir::BinaryOp::F64Mul => {
            let (y_hi, y_lo) = credits.pop_hi_lo();
            let (x_hi, x_lo) = credits.pop_hi_lo();
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(dmul(y_hi, y_lo, x_hi, x_lo, out_lo, out_hi));
            debts.gen(ctx);
        }
        ir::BinaryOp::F64Div => {
            let (x_hi, x_lo, y_hi, y_lo) = credits.pop_swapped_quad(ctx);
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(ddiv(x_hi, x_lo, y_hi, y_lo, out_lo, out_hi));
            debts.gen(ctx);
        }
        ir::BinaryOp::F64Min => {
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.f64_min), imm(4), out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::BinaryOp::F64Max => {
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.f64_max), imm(4), out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        ir::BinaryOp::F64Copysign => {
            let (out_lo, out_hi) = debts.pop_lo_hi();
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.f64_copysign), imm(4), out_lo));
            copy_if_sensible(ctx, derefl(ctx.layout.hi_return().addr), out_hi);
            debts.gen(ctx);
        }
        _ => {
            credits.gen(ctx);
            let mnemonic = Other::Binop(binop.clone()).mnemonic();
            ctx.errors
                .push(crate::CompilationError::UnsupportedInstruction {
                    function: frame.function_name.map(|s| s.to_owned()),
                    instr: mnemonic,
                });
            debts.gen(ctx);
        }
    }
}
