use glulx_asm::{concise::*, LoadOperand, StoreOperand};
use walrus::{
    ir::{self, ExtendedLoad},
    ValType,
};

use crate::common::Context;

use super::{
    loadstore::{gen_copies, Credits, Debts},
    toplevel::Frame,
};

pub fn gen_memory_init(
    ctx: &mut Context,
    _frame: &mut Frame,
    init_instr: &ir::MemoryInit,
    mut credits: Credits,
    mut debts: Debts,
) {
    credits.gen(ctx);
    let data_layout = ctx.layout.data(init_instr.data);
    ctx.rom_items.push(copy(imml(data_layout.addr), push()));
    ctx.rom_items
        .push(copy(derefl(data_layout.cur_size), push()));
    ctx.rom_items
        .push(call(imml(ctx.rt.memory_init), imm(5), discard()));
    debts.gen(ctx);
}

pub fn gen_memory_grow(
    ctx: &mut Context,
    _frame: &mut Frame,
    _grow_instr: &ir::MemoryGrow,
    mut credits: Credits,
    mut debts: Debts,
) {
    let arg = credits.pop();
    let out = debts.pop();
    credits.gen(ctx);
    ctx.rom_items
        .push(callfi(imml(ctx.rt.memory_grow), arg, out));
    debts.gen(ctx);
}

pub fn gen_memory_copy(
    ctx: &mut Context,
    _frame: &mut Frame,
    _copy_instr: &ir::MemoryCopy,
    mut credits: Credits,
    mut debts: Debts,
) {
    let n = credits.pop();
    let s = credits.pop();
    let d = credits.pop();
    credits.gen(ctx);
    ctx.rom_items
        .push(callfiii(imml(ctx.rt.memory_copy), n, s, d, discard()));
    debts.gen(ctx);
}

pub fn gen_memory_fill(
    ctx: &mut Context,
    _frame: &mut Frame,
    _fill_instr: &ir::MemoryFill,
    mut credits: Credits,
    mut debts: Debts,
) {
    let n = credits.pop();
    let val = credits.pop();
    let d = credits.pop();
    credits.gen(ctx);
    ctx.rom_items
        .push(callfiii(imml(ctx.rt.memory_fill), n, val, d, discard()));
    debts.gen(ctx);
}

pub fn gen_memory_size(
    ctx: &mut Context,
    _frame: &mut Frame,
    _size_instr: &ir::MemorySize,
    mut credits: Credits,
    mut debts: Debts,
) {
    let out = debts.pop();
    credits.gen(ctx);
    ctx.rom_items
        .push(ushiftr(derefl(ctx.layout.memory().cur_size), imm(16), out));
    debts.gen(ctx);
}

pub fn gen_data_drop(
    ctx: &mut Context,
    _frame: &mut Frame,
    drop_instr: &ir::DataDrop,
    credits: Credits,
    debts: Debts,
) {
    ctx.rom_items.push(copy(
        imm(0),
        storel(ctx.layout.data(drop_instr.data).cur_size),
    ));
    gen_copies(ctx, credits, debts);
}

pub fn gen_load(
    ctx: &mut Context,
    frame: &mut Frame,
    load_instr: &ir::Load,
    mut credits: Credits,
    mut debts: Debts,
) {
    let offset = load_instr.arg.offset;

    match load_instr.kind {
        ir::LoadKind::F32 | ir::LoadKind::I32 { atomic: _ } => {
            let addr = credits.pop();
            let out = debts.pop();
            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.memload32), uimm(offset), addr, out));
            debts.gen(ctx);
        }
        ir::LoadKind::F64 | ir::LoadKind::I64 { atomic: _ } => {
            let addr = credits.pop();
            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.memload64), uimm(offset), addr, push()));
            gen_copies(ctx, Credits::from_returns(ctx, &[ValType::I64]), debts);
        }
        ir::LoadKind::V128 => {
            credits.gen(ctx);
            ctx.errors
                .push(crate::CompilationError::UnsupportedInstruction {
                    function: frame.function_name.map(|s| s.to_owned()),
                    instr: "v128.load",
                });
            debts.gen(ctx);
        }
        ir::LoadKind::I32_8 { kind } => {
            let addr = credits.pop();
            let out = debts.pop();
            credits.gen(ctx);
            match kind {
                ExtendedLoad::SignExtend => {
                    ctx.rom_items
                        .push(callfii(imml(ctx.rt.memload8), uimm(offset), addr, push()));
                    ctx.rom_items.push(sexb(pop(), out));
                }
                ExtendedLoad::ZeroExtend | ExtendedLoad::ZeroExtendAtomic => {
                    ctx.rom_items
                        .push(callfii(imml(ctx.rt.memload8), uimm(offset), addr, out));
                }
            }
            debts.gen(ctx);
        }
        ir::LoadKind::I32_16 { kind } => {
            let addr = credits.pop();
            let out = debts.pop();
            credits.gen(ctx);
            match kind {
                ExtendedLoad::SignExtend => {
                    ctx.rom_items
                        .push(callfii(imml(ctx.rt.memload16), uimm(offset), addr, push()));
                    ctx.rom_items.push(sexs(pop(), out));
                }
                ExtendedLoad::ZeroExtend | ExtendedLoad::ZeroExtendAtomic => {
                    ctx.rom_items
                        .push(callfii(imml(ctx.rt.memload16), uimm(offset), addr, out));
                }
            }
            debts.gen(ctx);
        }
        ir::LoadKind::I64_8 { kind } => {
            let addr = credits.pop();
            let out_hi = debts.pop();
            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.memload8), uimm(offset), addr, push()));

            match kind {
                ExtendedLoad::SignExtend => {
                    ctx.rom_items.push(sexb(pop(), push()));
                    ctx.rom_items.push(stkpeek(imm(0), push()));
                    ctx.rom_items.push(sshiftr(pop(), imm(31), out_hi));
                }
                ExtendedLoad::ZeroExtend | ExtendedLoad::ZeroExtendAtomic => {
                    if !matches!(out_hi, StoreOperand::Discard) {
                        ctx.rom_items.push(copy(imm(0), out_hi));
                    }
                }
            }
            debts.gen(ctx);
        }
        ir::LoadKind::I64_16 { kind } => {
            let addr = credits.pop();
            let out_hi = debts.pop();
            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.memload16), uimm(offset), addr, push()));

            match kind {
                ExtendedLoad::SignExtend => {
                    ctx.rom_items.push(sexs(pop(), push()));
                    ctx.rom_items.push(stkpeek(imm(0), push()));
                    ctx.rom_items.push(sshiftr(pop(), imm(31), out_hi));
                }
                ExtendedLoad::ZeroExtend | ExtendedLoad::ZeroExtendAtomic => {
                    if !matches!(out_hi, StoreOperand::Discard) {
                        ctx.rom_items.push(copy(imm(0), out_hi));
                    }
                }
            }
            debts.gen(ctx);
        }
        ir::LoadKind::I64_32 { kind } => {
            let addr = credits.pop();
            let out_hi = debts.pop();
            credits.gen(ctx);
            ctx.rom_items
                .push(callfii(imml(ctx.rt.memload32), uimm(offset), addr, push()));

            match kind {
                ExtendedLoad::SignExtend => {
                    ctx.rom_items.push(stkpeek(imm(0), push()));
                    ctx.rom_items.push(sshiftr(pop(), imm(31), out_hi));
                }
                ExtendedLoad::ZeroExtend | ExtendedLoad::ZeroExtendAtomic => {
                    if !matches!(out_hi, StoreOperand::Discard) {
                        ctx.rom_items.push(copy(imm(0), out_hi));
                    }
                }
            }
            debts.gen(ctx);
        }
    }
}

pub fn gen_store(
    ctx: &mut Context,
    frame: &mut Frame,
    store_instr: &ir::Store,
    mut credits: Credits,
    mut debts: Debts,
) {
    let offset = store_instr.arg.offset;
    match store_instr.kind {
        ir::StoreKind::F32 | ir::StoreKind::I32 { atomic: _ } => {
            let val = credits.pop();
            let addr = credits.pop();
            credits.gen(ctx);
            ctx.rom_items.push(callfiii(
                imml(ctx.rt.memstore32),
                uimm(offset),
                val,
                addr,
                discard(),
            ));
            debts.gen(ctx);
        }
        ir::StoreKind::F64 | ir::StoreKind::I64 { atomic: _ } => {
            credits.gen(ctx);
            ctx.rom_items.push(copy(uimm(offset), push()));
            ctx.rom_items
                .push(call(imml(ctx.rt.memstore64), imm(4), discard()));
            debts.gen(ctx);
        }
        ir::StoreKind::V128 => {
            credits.gen(ctx);
            ctx.errors
                .push(crate::CompilationError::UnsupportedInstruction {
                    function: frame.function_name.map(|s| s.to_owned()),
                    instr: "v128.store",
                });
            debts.gen(ctx);
        }
        ir::StoreKind::I32_8 { atomic: _ } => {
            let val = credits.pop();
            let addr = credits.pop();
            credits.gen(ctx);
            ctx.rom_items.push(callfiii(
                imml(ctx.rt.memstore8),
                uimm(offset),
                val,
                addr,
                discard(),
            ));
            debts.gen(ctx);
        }
        ir::StoreKind::I32_16 { atomic: _ } => {
            let val = credits.pop();
            let addr = credits.pop();
            credits.gen(ctx);
            ctx.rom_items.push(callfiii(
                imml(ctx.rt.memstore16),
                uimm(offset),
                val,
                addr,
                discard(),
            ));
            debts.gen(ctx);
        }
        ir::StoreKind::I64_8 { atomic: _ } => {
            let val_hi = credits.pop();
            let val_lo = credits.pop();
            let addr = credits.pop();
            credits.gen(ctx);

            if matches!(val_hi, LoadOperand::Pop) {
                ctx.rom_items.push(copy(pop(), discard()));
            }
            ctx.rom_items.push(callfiii(
                imml(ctx.rt.memstore8),
                uimm(offset),
                val_lo,
                addr,
                discard(),
            ));
            debts.gen(ctx);
        }
        ir::StoreKind::I64_16 { atomic: _ } => {
            let val_hi = credits.pop();
            let val_lo = credits.pop();
            let addr = credits.pop();
            credits.gen(ctx);

            if matches!(val_hi, LoadOperand::Pop) {
                ctx.rom_items.push(copy(pop(), discard()));
            }
            ctx.rom_items.push(callfiii(
                imml(ctx.rt.memstore16),
                uimm(offset),
                val_lo,
                addr,
                discard(),
            ));
            debts.gen(ctx);
        }
        ir::StoreKind::I64_32 { atomic: _ } => {
            let val_hi = credits.pop();
            let val_lo = credits.pop();
            let addr = credits.pop();
            credits.gen(ctx);

            if matches!(val_hi, LoadOperand::Pop) {
                ctx.rom_items.push(copy(pop(), discard()));
            }
            ctx.rom_items.push(callfiii(
                imml(ctx.rt.memstore32),
                uimm(offset),
                val_lo,
                addr,
                discard(),
            ));
            debts.gen(ctx);
        }
    }
}
