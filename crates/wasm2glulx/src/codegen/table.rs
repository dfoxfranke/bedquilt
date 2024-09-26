// SPDX-License-Identifier: Apache-2.0 WITH LLVM-Exception
// Copyright 2024 Daniel Fox Franke.

use glulx_asm::concise::*;
use glulx_asm::LoadOperand;
use walrus::ir;

use crate::common::Context;

use super::{
    loadstore::{Credits, Debts},
    toplevel::Frame,
};

pub fn gen_table_get(
    ctx: &mut Context,
    _frame: &mut Frame,
    table_get: &ir::TableGet,
    mut credits: Credits,
    mut debts: Debts,
) {
    let index = credits.pop();
    let out = debts.pop();
    let table = ctx.layout.table(table_get.table);
    credits.gen(ctx);

    if matches!(index, LoadOperand::Pop) {
        ctx.rom_items.push(stkpeek(imm(0), push()));
    }
    ctx.rom_items.push(jgeu(
        index,
        derefl(table.cur_count),
        ctx.rt.trap_out_of_bounds_table_access,
    ));
    ctx.rom_items.push(aload(imml(table.addr), index, out));
    debts.gen(ctx);
}

pub fn gen_table_set(
    ctx: &mut Context,
    _frame: &mut Frame,
    table_set: &ir::TableSet,
    mut credits: Credits,
    mut debts: Debts,
) {
    let table = ctx.layout.table(table_set.table);

    credits.gen(ctx);
    ctx.rom_items.push(stkswap());
    ctx.rom_items.push(stkpeek(imm(0), push()));
    ctx.rom_items.push(jgeu(
        pop(),
        derefl(table.cur_count),
        ctx.rt.trap_out_of_bounds_table_access,
    ));
    ctx.rom_items.push(astore(imml(table.addr), pop(), pop()));
    debts.gen(ctx);
}

pub fn gen_table_init(
    ctx: &mut Context,
    _frame: &mut Frame,
    table_init: &ir::TableInit,
    mut credits: Credits,
    mut debts: Debts,
) {
    let table = ctx.layout.table(table_init.table);
    let elem = ctx.layout.element(table_init.elem);

    credits.gen(ctx);
    ctx.rom_items.push(copy(imml(table.addr), push()));
    ctx.rom_items.push(copy(derefl(table.cur_count), push()));
    ctx.rom_items.push(copy(imml(elem.addr), push()));
    ctx.rom_items.push(copy(derefl(elem.cur_count), push()));
    ctx.rom_items
        .push(call(imml(ctx.rt.table_init_or_copy), imm(7), discard()));
    debts.gen(ctx);
}

pub fn gen_table_copy(
    ctx: &mut Context,
    _frame: &mut Frame,
    table_copy: &ir::TableCopy,
    mut credits: Credits,
    mut debts: Debts,
) {
    let dtable = ctx.layout.table(table_copy.dst);
    let stable = ctx.layout.table(table_copy.src);

    credits.gen(ctx);
    ctx.rom_items.push(copy(imml(dtable.addr), push()));
    ctx.rom_items.push(copy(derefl(dtable.cur_count), push()));
    ctx.rom_items.push(copy(imml(stable.addr), push()));
    ctx.rom_items.push(copy(derefl(stable.cur_count), push()));
    ctx.rom_items
        .push(call(imml(ctx.rt.table_init_or_copy), imm(7), discard()));
    debts.gen(ctx);
}

pub fn gen_table_grow(
    ctx: &mut Context,
    _frame: &mut Frame,
    table_grow: &ir::TableGrow,
    mut credits: Credits,
    mut debts: Debts,
) {
    let table = ctx.layout.table(table_grow.table);
    let n = credits.pop();
    let out = debts.pop();

    credits.gen(ctx);
    ctx.rom_items.push(callfiii(
        imml(ctx.rt.table_grow),
        uimm(table.max_count),
        imml(table.cur_count),
        n,
        out,
    ));
    debts.gen(ctx);
}

pub fn gen_table_fill(
    ctx: &mut Context,
    _frame: &mut Frame,
    table_fill: &ir::TableFill,
    mut credits: Credits,
    mut debts: Debts,
) {
    let table = ctx.layout.table(table_fill.table);
    credits.gen(ctx);
    ctx.rom_items.push(copy(imml(table.addr), push()));
    ctx.rom_items.push(copy(derefl(table.cur_count), push()));
    ctx.rom_items
        .push(call(imml(ctx.rt.table_fill), imm(5), discard()));
    debts.gen(ctx);
}

pub fn gen_elem_drop(
    ctx: &mut Context,
    _frame: &mut Frame,
    elem_drop: &ir::ElemDrop,
    mut credits: Credits,
    mut debts: Debts,
) {
    let elem = ctx.layout.element(elem_drop.elem);
    credits.gen(ctx);
    ctx.rom_items.push(copy(imm(0), storel(elem.cur_count)));
    debts.gen(ctx);
}
