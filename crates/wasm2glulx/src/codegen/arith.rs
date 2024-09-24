use super::classify::{ClassifiedInstr, Other};
use super::loadstore::{copy_if_sensible, gen_copies, Credits, Debts};
use super::toplevel::Frame;
use crate::common::*;
use glulx_asm::{concise::*, LoadOperand, StoreOperand};
use walrus::{ir, ValType};

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

            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.i64_clz), x_hi, x_lo, push()));
            gen_copies(ctx, Credits::from_returns(ctx, &[ValType::I64]), debts);
        }
        ir::UnaryOp::I64Ctz => {
            let (x_hi, x_lo) = credits.pop_hi_lo();

            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.i64_ctz), x_hi, x_lo, push()));
            gen_copies(ctx, Credits::from_returns(ctx, &[ValType::I64]), debts);
        }
        ir::UnaryOp::I64Popcnt => {
            let (x_hi, x_lo) = credits.pop_hi_lo();

            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.i64_popcnt), x_hi, x_lo, push()));
            gen_copies(ctx, Credits::from_returns(ctx, &[ValType::I64]), debts);
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

            credits.gen(ctx);
            ctx.rom_items
                .push(callfi(imml(ctx.rt.i64_trunc_s_f32), x, push()));
            gen_copies(ctx, Credits::from_returns(ctx, &[ValType::I64]), debts);
        }
        ir::UnaryOp::I64TruncUF32 => {
            let x = credits.pop();

            credits.gen(ctx);
            ctx.rom_items
                .push(callfi(imml(ctx.rt.i64_trunc_u_f32), x, push()));
            gen_copies(ctx, Credits::from_returns(ctx, &[ValType::I64]), debts);
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
            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.f64_trunc), x_hi, x_lo, push()));
            gen_copies(ctx, Credits::from_returns(ctx, &[ValType::F64]), debts);
        }
        ir::UnaryOp::F64Nearest => {
            let (x_hi, x_lo) = credits.pop_hi_lo();
            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.f64_nearest), x_hi, x_lo, push()));
            gen_copies(ctx, Credits::from_returns(ctx, &[ValType::F64]), debts);
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
        } /*
          ir::UnaryOp::I8x16Splat => todo!(),
          ir::UnaryOp::I8x16ExtractLaneS { idx } => todo!(),
          ir::UnaryOp::I8x16ExtractLaneU { idx } => todo!(),
          ir::UnaryOp::I16x8Splat => todo!(),
          ir::UnaryOp::I16x8ExtractLaneS { idx } => todo!(),
          ir::UnaryOp::I16x8ExtractLaneU { idx } => todo!(),
          ir::UnaryOp::I32x4Splat => todo!(),
          ir::UnaryOp::I32x4ExtractLane { idx } => todo!(),
          ir::UnaryOp::I64x2Splat => todo!(),
          ir::UnaryOp::I64x2ExtractLane { idx } => todo!(),
          ir::UnaryOp::F32x4Splat => todo!(),
          ir::UnaryOp::F32x4ExtractLane { idx } => todo!(),
          ir::UnaryOp::F64x2Splat => todo!(),
          ir::UnaryOp::F64x2ExtractLane { idx } => todo!(),
          ir::UnaryOp::V128Not => todo!(),
          ir::UnaryOp::V128AnyTrue => todo!(),
          ir::UnaryOp::I8x16Abs => todo!(),
          ir::UnaryOp::I8x16Popcnt => todo!(),
          ir::UnaryOp::I8x16Neg => todo!(),
          ir::UnaryOp::I8x16AllTrue => todo!(),
          ir::UnaryOp::I8x16Bitmask => todo!(),
          ir::UnaryOp::I16x8Abs => todo!(),
          ir::UnaryOp::I16x8Neg => todo!(),
          ir::UnaryOp::I16x8AllTrue => todo!(),
          ir::UnaryOp::I16x8Bitmask => todo!(),
          ir::UnaryOp::I32x4Abs => todo!(),
          ir::UnaryOp::I32x4Neg => todo!(),
          ir::UnaryOp::I32x4AllTrue => todo!(),
          ir::UnaryOp::I32x4Bitmask => todo!(),
          ir::UnaryOp::I64x2Abs => todo!(),
          ir::UnaryOp::I64x2Neg => todo!(),
          ir::UnaryOp::I64x2AllTrue => todo!(),
          ir::UnaryOp::I64x2Bitmask => todo!(),
          ir::UnaryOp::F32x4Abs => todo!(),
          ir::UnaryOp::F32x4Neg => todo!(),
          ir::UnaryOp::F32x4Sqrt => todo!(),
          ir::UnaryOp::F32x4Ceil => todo!(),
          ir::UnaryOp::F32x4Floor => todo!(),
          ir::UnaryOp::F32x4Trunc => todo!(),
          ir::UnaryOp::F32x4Nearest => todo!(),
          ir::UnaryOp::F64x2Abs => todo!(),
          ir::UnaryOp::F64x2Neg => todo!(),
          ir::UnaryOp::F64x2Sqrt => todo!(),
          ir::UnaryOp::F64x2Ceil => todo!(),
          ir::UnaryOp::F64x2Floor => todo!(),
          ir::UnaryOp::F64x2Trunc => todo!(),
          ir::UnaryOp::F64x2Nearest => todo!(),
          ir::UnaryOp::I16x8ExtAddPairwiseI8x16S => todo!(),
          ir::UnaryOp::I16x8ExtAddPairwiseI8x16U => todo!(),
          ir::UnaryOp::I32x4ExtAddPairwiseI16x8S => todo!(),
          ir::UnaryOp::I32x4ExtAddPairwiseI16x8U => todo!(),
          ir::UnaryOp::I64x2ExtendLowI32x4S => todo!(),
          ir::UnaryOp::I64x2ExtendHighI32x4S => todo!(),
          ir::UnaryOp::I64x2ExtendLowI32x4U => todo!(),
          ir::UnaryOp::I64x2ExtendHighI32x4U => todo!(),
          ir::UnaryOp::I32x4TruncSatF64x2SZero => todo!(),
          ir::UnaryOp::I32x4TruncSatF64x2UZero => todo!(),
          ir::UnaryOp::F64x2ConvertLowI32x4S => todo!(),
          ir::UnaryOp::F64x2ConvertLowI32x4U => todo!(),
          ir::UnaryOp::F32x4DemoteF64x2Zero => todo!(),
          ir::UnaryOp::F64x2PromoteLowF32x4 => todo!(),
          ir::UnaryOp::I32x4TruncSatF32x4S => todo!(),
          ir::UnaryOp::I32x4TruncSatF32x4U => todo!(),
          ir::UnaryOp::F32x4ConvertI32x4S => todo!(),
          ir::UnaryOp::F32x4ConvertI32x4U => todo!(),
          ir::UnaryOp::I16x8WidenLowI8x16S => todo!(),
          ir::UnaryOp::I16x8WidenLowI8x16U => todo!(),
          ir::UnaryOp::I16x8WidenHighI8x16S => todo!(),
          ir::UnaryOp::I16x8WidenHighI8x16U => todo!(),
          ir::UnaryOp::I32x4WidenLowI16x8S => todo!(),
          ir::UnaryOp::I32x4WidenLowI16x8U => todo!(),
          ir::UnaryOp::I32x4WidenHighI16x8S => todo!(),
          ir::UnaryOp::I32x4WidenHighI16x8U => todo!(),
          */
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
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_add), imm(4), push()));
            gen_copies(ctx, Credits::from_returns(ctx, &[ValType::I64]), debts);
        }
        ir::BinaryOp::I64Sub => {
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_sub), imm(4), push()));
            gen_copies(ctx, Credits::from_returns(ctx, &[ValType::I64]), debts);
        }
        ir::BinaryOp::I64Mul => {
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_mul), imm(4), push()));
            gen_copies(ctx, Credits::from_returns(ctx, &[ValType::I64]), debts);
        }
        ir::BinaryOp::I64DivU => {
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_div_u), imm(4), push()));
            gen_copies(ctx, Credits::from_returns(ctx, &[ValType::I64]), debts);
        }
        ir::BinaryOp::I64DivS => {
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_div_s), imm(4), push()));
            gen_copies(ctx, Credits::from_returns(ctx, &[ValType::I64]), debts);
        }
        ir::BinaryOp::I64RemU => {
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_rem_u), imm(4), push()));
            gen_copies(ctx, Credits::from_returns(ctx, &[ValType::I64]), debts);
        }
        ir::BinaryOp::I64RemS => {
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_rem_s), imm(4), push()));
            gen_copies(ctx, Credits::from_returns(ctx, &[ValType::I64]), debts);
        }
        ir::BinaryOp::I64And => {
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_and), imm(4), push()));
            gen_copies(ctx, Credits::from_returns(ctx, &[ValType::I64]), debts);
        }
        ir::BinaryOp::I64Or => {
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_or), imm(4), push()));
            gen_copies(ctx, Credits::from_returns(ctx, &[ValType::I64]), debts);
        }
        ir::BinaryOp::I64Xor => {
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_xor), imm(4), push()));
            gen_copies(ctx, Credits::from_returns(ctx, &[ValType::I64]), debts);
        }
        ir::BinaryOp::I64Shl => {
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_shl), imm(4), push()));
            gen_copies(ctx, Credits::from_returns(ctx, &[ValType::I64]), debts);
        }
        ir::BinaryOp::I64ShrS => {
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_shr_s), imm(4), push()));
            gen_copies(ctx, Credits::from_returns(ctx, &[ValType::I64]), debts);
        }
        ir::BinaryOp::I64ShrU => {
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_shr_u), imm(4), push()));
            gen_copies(ctx, Credits::from_returns(ctx, &[ValType::I64]), debts);
        }
        ir::BinaryOp::I64Rotl => {
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_rotl), imm(4), push()));
            gen_copies(ctx, Credits::from_returns(ctx, &[ValType::I64]), debts);
        }
        ir::BinaryOp::I64Rotr => {
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.i64_rotr), imm(4), push()));
            gen_copies(ctx, Credits::from_returns(ctx, &[ValType::I64]), debts);
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
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.f64_min), imm(4), push()));
            gen_copies(ctx, Credits::from_returns(ctx, &[ValType::F64]), debts);
        }
        ir::BinaryOp::F64Max => {
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.f64_max), imm(4), push()));
            gen_copies(ctx, Credits::from_returns(ctx, &[ValType::F64]), debts);
        }
        ir::BinaryOp::F64Copysign => {
            credits.gen(ctx);
            ctx.rom_items
                .push(call(imml(ctx.rt.f64_copysign), imm(4), push()));
            gen_copies(ctx, Credits::from_returns(ctx, &[ValType::F64]), debts);
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
        } /*
          ir::BinaryOp::I8x16ReplaceLane { idx } => todo!(),
          ir::BinaryOp::I16x8ReplaceLane { idx } => todo!(),
          ir::BinaryOp::I32x4ReplaceLane { idx } => todo!(),
          ir::BinaryOp::I64x2ReplaceLane { idx } => todo!(),
          ir::BinaryOp::F32x4ReplaceLane { idx } => todo!(),
          ir::BinaryOp::F64x2ReplaceLane { idx } => todo!(),
          ir::BinaryOp::I8x16Eq => todo!(),
          ir::BinaryOp::I8x16Ne => todo!(),
          ir::BinaryOp::I8x16LtS => todo!(),
          ir::BinaryOp::I8x16LtU => todo!(),
          ir::BinaryOp::I8x16GtS => todo!(),
          ir::BinaryOp::I8x16GtU => todo!(),
          ir::BinaryOp::I8x16LeS => todo!(),
          ir::BinaryOp::I8x16LeU => todo!(),
          ir::BinaryOp::I8x16GeS => todo!(),
          ir::BinaryOp::I8x16GeU => todo!(),
          ir::BinaryOp::I16x8Eq => todo!(),
          ir::BinaryOp::I16x8Ne => todo!(),
          ir::BinaryOp::I16x8LtS => todo!(),
          ir::BinaryOp::I16x8LtU => todo!(),
          ir::BinaryOp::I16x8GtS => todo!(),
          ir::BinaryOp::I16x8GtU => todo!(),
          ir::BinaryOp::I16x8LeS => todo!(),
          ir::BinaryOp::I16x8LeU => todo!(),
          ir::BinaryOp::I16x8GeS => todo!(),
          ir::BinaryOp::I16x8GeU => todo!(),
          ir::BinaryOp::I32x4Eq => todo!(),
          ir::BinaryOp::I32x4Ne => todo!(),
          ir::BinaryOp::I32x4LtS => todo!(),
          ir::BinaryOp::I32x4LtU => todo!(),
          ir::BinaryOp::I32x4GtS => todo!(),
          ir::BinaryOp::I32x4GtU => todo!(),
          ir::BinaryOp::I32x4LeS => todo!(),
          ir::BinaryOp::I32x4LeU => todo!(),
          ir::BinaryOp::I32x4GeS => todo!(),
          ir::BinaryOp::I32x4GeU => todo!(),
          ir::BinaryOp::I64x2Eq => todo!(),
          ir::BinaryOp::I64x2Ne => todo!(),
          ir::BinaryOp::I64x2LtS => todo!(),
          ir::BinaryOp::I64x2GtS => todo!(),
          ir::BinaryOp::I64x2LeS => todo!(),
          ir::BinaryOp::I64x2GeS => todo!(),
          ir::BinaryOp::F32x4Eq => todo!(),
          ir::BinaryOp::F32x4Ne => todo!(),
          ir::BinaryOp::F32x4Lt => todo!(),
          ir::BinaryOp::F32x4Gt => todo!(),
          ir::BinaryOp::F32x4Le => todo!(),
          ir::BinaryOp::F32x4Ge => todo!(),
          ir::BinaryOp::F64x2Eq => todo!(),
          ir::BinaryOp::F64x2Ne => todo!(),
          ir::BinaryOp::F64x2Lt => todo!(),
          ir::BinaryOp::F64x2Gt => todo!(),
          ir::BinaryOp::F64x2Le => todo!(),
          ir::BinaryOp::F64x2Ge => todo!(),
          ir::BinaryOp::V128And => todo!(),
          ir::BinaryOp::V128Or => todo!(),
          ir::BinaryOp::V128Xor => todo!(),
          ir::BinaryOp::V128AndNot => todo!(),
          ir::BinaryOp::I8x16Shl => todo!(),
          ir::BinaryOp::I8x16ShrS => todo!(),
          ir::BinaryOp::I8x16ShrU => todo!(),
          ir::BinaryOp::I8x16Add => todo!(),
          ir::BinaryOp::I8x16AddSatS => todo!(),
          ir::BinaryOp::I8x16AddSatU => todo!(),
          ir::BinaryOp::I8x16Sub => todo!(),
          ir::BinaryOp::I8x16SubSatS => todo!(),
          ir::BinaryOp::I8x16SubSatU => todo!(),
          ir::BinaryOp::I16x8Shl => todo!(),
          ir::BinaryOp::I16x8ShrS => todo!(),
          ir::BinaryOp::I16x8ShrU => todo!(),
          ir::BinaryOp::I16x8Add => todo!(),
          ir::BinaryOp::I16x8AddSatS => todo!(),
          ir::BinaryOp::I16x8AddSatU => todo!(),
          ir::BinaryOp::I16x8Sub => todo!(),
          ir::BinaryOp::I16x8SubSatS => todo!(),
          ir::BinaryOp::I16x8SubSatU => todo!(),
          ir::BinaryOp::I16x8Mul => todo!(),
          ir::BinaryOp::I32x4Shl => todo!(),
          ir::BinaryOp::I32x4ShrS => todo!(),
          ir::BinaryOp::I32x4ShrU => todo!(),
          ir::BinaryOp::I32x4Add => todo!(),
          ir::BinaryOp::I32x4Sub => todo!(),
          ir::BinaryOp::I32x4Mul => todo!(),
          ir::BinaryOp::I64x2Shl => todo!(),
          ir::BinaryOp::I64x2ShrS => todo!(),
          ir::BinaryOp::I64x2ShrU => todo!(),
          ir::BinaryOp::I64x2Add => todo!(),
          ir::BinaryOp::I64x2Sub => todo!(),
          ir::BinaryOp::I64x2Mul => todo!(),
          ir::BinaryOp::F32x4Add => todo!(),
          ir::BinaryOp::F32x4Sub => todo!(),
          ir::BinaryOp::F32x4Mul => todo!(),
          ir::BinaryOp::F32x4Div => todo!(),
          ir::BinaryOp::F32x4Min => todo!(),
          ir::BinaryOp::F32x4Max => todo!(),
          ir::BinaryOp::F32x4PMin => todo!(),
          ir::BinaryOp::F32x4PMax => todo!(),
          ir::BinaryOp::F64x2Add => todo!(),
          ir::BinaryOp::F64x2Sub => todo!(),
          ir::BinaryOp::F64x2Mul => todo!(),
          ir::BinaryOp::F64x2Div => todo!(),
          ir::BinaryOp::F64x2Min => todo!(),
          ir::BinaryOp::F64x2Max => todo!(),
          ir::BinaryOp::F64x2PMin => todo!(),
          ir::BinaryOp::F64x2PMax => todo!(),
          ir::BinaryOp::I8x16NarrowI16x8S => todo!(),
          ir::BinaryOp::I8x16NarrowI16x8U => todo!(),
          ir::BinaryOp::I16x8NarrowI32x4S => todo!(),
          ir::BinaryOp::I16x8NarrowI32x4U => todo!(),
          ir::BinaryOp::I8x16AvgrU => todo!(),
          ir::BinaryOp::I16x8AvgrU => todo!(),
          ir::BinaryOp::I8x16MinS => todo!(),
          ir::BinaryOp::I8x16MinU => todo!(),
          ir::BinaryOp::I8x16MaxS => todo!(),
          ir::BinaryOp::I8x16MaxU => todo!(),
          ir::BinaryOp::I16x8MinS => todo!(),
          ir::BinaryOp::I16x8MinU => todo!(),
          ir::BinaryOp::I16x8MaxS => todo!(),
          ir::BinaryOp::I16x8MaxU => todo!(),
          ir::BinaryOp::I32x4MinS => todo!(),
          ir::BinaryOp::I32x4MinU => todo!(),
          ir::BinaryOp::I32x4MaxS => todo!(),
          ir::BinaryOp::I32x4MaxU => todo!(),
          ir::BinaryOp::I32x4DotI16x8S => todo!(),
          ir::BinaryOp::I16x8Q15MulrSatS => todo!(),
          ir::BinaryOp::I16x8ExtMulLowI8x16S => todo!(),
          ir::BinaryOp::I16x8ExtMulHighI8x16S => todo!(),
          ir::BinaryOp::I16x8ExtMulLowI8x16U => todo!(),
          ir::BinaryOp::I16x8ExtMulHighI8x16U => todo!(),
          ir::BinaryOp::I32x4ExtMulLowI16x8S => todo!(),
          ir::BinaryOp::I32x4ExtMulHighI16x8S => todo!(),
          ir::BinaryOp::I32x4ExtMulLowI16x8U => todo!(),
          ir::BinaryOp::I32x4ExtMulHighI16x8U => todo!(),
          ir::BinaryOp::I64x2ExtMulLowI32x4S => todo!(),
          ir::BinaryOp::I64x2ExtMulHighI32x4S => todo!(),
          ir::BinaryOp::I64x2ExtMulLowI32x4U => todo!(),
          ir::BinaryOp::I64x2ExtMulHighI32x4U => todo!(),
          */
    }
}
