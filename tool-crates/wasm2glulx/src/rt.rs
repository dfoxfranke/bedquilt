// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
// Copyright 2024 Daniel Fox Franke.
// Some bits adapted from the LLVM/CompilerRT project.
use core::{f32, f64};

use crate::common::*;
use glulx_asm::concise::*;

use bytes::{BufMut, BytesMut};
pub struct RuntimeLabels {
    pub swap: Label,
    pub swaps: Label,
    pub checkaddr: Label,
    pub checkglkaddr: Label,
    pub checkstr: Label,
    pub checkunistr: Label,
    pub memload64: Label,
    pub memload32: Label,
    pub memload16: Label,
    pub memload8: Label,
    pub memstore64: Label,
    pub memstore32: Label,
    pub memstore16: Label,
    pub memstore8: Label,
    pub swaparray: Label,
    pub swapglkarray: Label,
    pub swapunistr: Label,
    pub i32_div_u: Label,
    pub i32_rem_u: Label,
    pub i32_shl: Label,
    pub i32_shr_s: Label,
    pub i32_shr_u: Label,
    pub i32_rotl: Label,
    pub i32_rotr: Label,
    pub i32_clz: Label,
    pub i32_ctz: Label,
    pub i32_popcnt: Label,
    pub i32_eqz: Label,
    pub i32_eq: Label,
    pub i32_ne: Label,
    pub i32_lt_s: Label,
    pub i32_lt_u: Label,
    pub i32_gt_s: Label,
    pub i32_gt_u: Label,
    pub i32_le_s: Label,
    pub i32_le_u: Label,
    pub i32_ge_s: Label,
    pub i32_ge_u: Label,
    pub i64_add: Label,
    pub i64_sub: Label,
    pub i64_mul: Label,
    pub i64_div_u: Label,
    pub i64_div_s: Label,
    pub i64_rem_u: Label,
    pub i64_rem_s: Label,
    pub i64_and: Label,
    pub i64_or: Label,
    pub i64_xor: Label,
    pub i64_shl: Label,
    pub i64_shr_s: Label,
    pub i64_shr_u: Label,
    pub i64_rotl: Label,
    pub i64_rotr: Label,
    pub i64_eqz: Label,
    pub i64_eq: Label,
    pub i64_ne: Label,
    pub i64_lt_s: Label,
    pub i64_lt_u: Label,
    pub i64_gt_s: Label,
    pub i64_gt_u: Label,
    pub i64_le_s: Label,
    pub i64_le_u: Label,
    pub i64_ge_s: Label,
    pub i64_ge_u: Label,
    pub i64_clz: Label,
    pub i64_ctz: Label,
    pub i64_popcnt: Label,
    pub f32_trunc: Label,
    pub f32_nearest: Label,
    pub f32_eq: Label,
    pub f32_ne: Label,
    pub f32_lt: Label,
    pub f32_gt: Label,
    pub f32_le: Label,
    pub f32_ge: Label,
    pub f32_min: Label,
    pub f32_max: Label,
    pub f32_copysign: Label,
    pub i32_trunc_s_f32: Label,
    pub i32_trunc_u_f32: Label,
    pub i64_trunc_s_f32: Label,
    pub i64_trunc_u_f32: Label,
    pub i32_trunc_sat_s_f32: Label,
    pub i32_trunc_sat_u_f32: Label,
    pub i64_trunc_sat_s_f32: Label,
    pub i64_trunc_sat_u_f32: Label,
    pub f32_convert_i32_u: Label,
    pub f32_convert_i64_s: Label,
    pub f32_convert_i64_u: Label,
    pub f64_trunc: Label,
    pub f64_nearest: Label,
    pub f64_eq: Label,
    pub f64_ne: Label,
    pub f64_lt: Label,
    pub f64_gt: Label,
    pub f64_le: Label,
    pub f64_ge: Label,
    pub f64_min: Label,
    pub f64_max: Label,
    pub f64_copysign: Label,
    pub i32_trunc_s_f64: Label,
    pub i32_trunc_u_f64: Label,
    pub i64_trunc_s_f64: Label,
    pub i64_trunc_u_f64: Label,
    pub i32_trunc_sat_s_f64: Label,
    pub i32_trunc_sat_u_f64: Label,
    pub i64_trunc_sat_s_f64: Label,
    pub i64_trunc_sat_u_f64: Label,
    pub f64_convert_i32_u: Label,
    pub f64_convert_i64_s: Label,
    pub f64_convert_i64_u: Label,
    pub table_init_or_copy: Label,
    pub table_grow: Label,
    pub table_fill: Label,
    pub memory_init: Label,
    pub memory_copy: Label,
    pub memory_fill: Label,
    pub memory_grow: Label,
    pub trap_unreachable: Label,
    pub trap_integer_overflow: Label,
    pub trap_integer_divide_by_zero: Label,
    pub trap_invalid_conversion_to_integer: Label,
    pub trap_out_of_bounds_memory_access: Label,
    pub trap_indirect_call_type_mismatch: Label,
    pub trap_out_of_bounds_table_access: Label,
    pub trap_undefined_element: Label,
    pub trap_uninitialized_element: Label,
    pub trap_call_stack_exhausted: Label,
}

impl RuntimeLabels {
    pub fn new(gen: &mut LabelGenerator) -> Self {
        RuntimeLabels {
            swap: gen.gen("rt_swap"),
            swaps: gen.gen("rt_swaps"),
            checkaddr: gen.gen("rt_checkaddr"),
            checkglkaddr: gen.gen("rt_checkglkaddr"),
            checkstr: gen.gen("rt_checkstr"),
            checkunistr: gen.gen("rt_checkunistr"),
            memload64: gen.gen("rt_memload64"),
            memload32: gen.gen("rt_memload32"),
            memload16: gen.gen("rt_memload16"),
            memload8: gen.gen("rt_memload8"),
            memstore64: gen.gen("rt_memstore64"),
            memstore32: gen.gen("rt_memstore32"),
            memstore16: gen.gen("rt_memstore16"),
            memstore8: gen.gen("rt_memstore8"),
            swaparray: gen.gen("rt_swaparray"),
            swapglkarray: gen.gen("rt_swapglkarray"),
            swapunistr: gen.gen("rt_swapunistr"),
            i32_div_u: gen.gen("rt_i32_div_u"),
            i32_rem_u: gen.gen("rt_i32_rem_u"),
            i32_shl: gen.gen("rt_i32_shl"),
            i32_shr_s: gen.gen("rt_i32_shr_s"),
            i32_shr_u: gen.gen("rt_i32_shr_u"),
            i32_rotl: gen.gen("rt_i32_rotl"),
            i32_rotr: gen.gen("rt_i32_rotr"),
            i32_clz: gen.gen("rt_i32_clz"),
            i32_ctz: gen.gen("rt_i32_ctz"),
            i32_popcnt: gen.gen("rt_i32_popcnt"),
            i32_eqz: gen.gen("rt_i32_eqz"),
            i32_eq: gen.gen("rt_i32_eq"),
            i32_ne: gen.gen("rt_i32_ne"),
            i32_lt_s: gen.gen("rt_i32_lt_s"),
            i32_lt_u: gen.gen("rt_i32_lt_u"),
            i32_gt_s: gen.gen("rt_i32_gt_s"),
            i32_gt_u: gen.gen("rt_i32_gt_u"),
            i32_le_s: gen.gen("rt_i32_le_s"),
            i32_le_u: gen.gen("rt_i32_le_u"),
            i32_ge_s: gen.gen("rt_i32_ge_s"),
            i32_ge_u: gen.gen("rt_i32_ge_u"),
            i64_add: gen.gen("rt_i64_add"),
            i64_sub: gen.gen("rt_i64_sub"),
            i64_mul: gen.gen("rt_i64_mul"),
            i64_div_u: gen.gen("rt_i64_div_u"),
            i64_div_s: gen.gen("rt_i64_div_s"),
            i64_rem_u: gen.gen("rt_i64_rem_u"),
            i64_rem_s: gen.gen("rt_i64_rem_s"),
            i64_and: gen.gen("rt_i64_and"),
            i64_or: gen.gen("rt_i64_or"),
            i64_xor: gen.gen("rt_i64_xor"),
            i64_shl: gen.gen("rt_i64_shl"),
            i64_shr_s: gen.gen("rt_i64_shr_s"),
            i64_shr_u: gen.gen("rt_i64_shr_u"),
            i64_rotl: gen.gen("rt_i64_rotl"),
            i64_rotr: gen.gen("rt_i64_rotr"),
            i64_eqz: gen.gen("rt_i64_eqz"),
            i64_eq: gen.gen("rt_i64_eq"),
            i64_ne: gen.gen("rt_i64_ne"),
            i64_lt_s: gen.gen("rt_i64_lt_s"),
            i64_lt_u: gen.gen("rt_i64_lt_u"),
            i64_gt_s: gen.gen("rt_i64_gt_s"),
            i64_gt_u: gen.gen("rt_i64_gt_u"),
            i64_le_s: gen.gen("rt_i64_le_s"),
            i64_le_u: gen.gen("rt_i64_le_u"),
            i64_ge_s: gen.gen("rt_i64_ge_s"),
            i64_ge_u: gen.gen("rt_i64_ge_u"),
            i64_clz: gen.gen("rt_i64_clz"),
            i64_ctz: gen.gen("rt_i64_ctz"),
            i64_popcnt: gen.gen("rt_i64_popcnt"),
            f32_trunc: gen.gen("rt_f32_trunc"),
            f32_nearest: gen.gen("rt_f32_nearest"),
            f32_eq: gen.gen("rt_f32_eq"),
            f32_ne: gen.gen("rt_f32_ne"),
            f32_lt: gen.gen("rt_f32_lt"),
            f32_gt: gen.gen("rt_f32_gt"),
            f32_le: gen.gen("rt_f32_le"),
            f32_ge: gen.gen("rt_f32_ge"),
            f32_min: gen.gen("rt_f32_min"),
            f32_max: gen.gen("rt_f32_max"),
            f32_copysign: gen.gen("rt_f32_copysign"),
            i32_trunc_s_f32: gen.gen("rt_i32_trunc_s_f32"),
            i32_trunc_u_f32: gen.gen("rt_i32_trunc_u_f32"),
            i64_trunc_s_f32: gen.gen("rt_i64_trunc_s_f32"),
            i64_trunc_u_f32: gen.gen("rt_i64_trunc_u_f32"),
            i32_trunc_sat_s_f32: gen.gen("rt_i32_trunc_sat_s_f32"),
            i32_trunc_sat_u_f32: gen.gen("rt_i32_trunc_sat_u_f32"),
            i64_trunc_sat_s_f32: gen.gen("rt_i64_trunc_sat_s_f32"),
            i64_trunc_sat_u_f32: gen.gen("rt_i64_trunc_sat_u_f32"),
            f32_convert_i32_u: gen.gen("rt_i32_convert_i32_u"),
            f32_convert_i64_s: gen.gen("rt_i32_convert_i64_s"),
            f32_convert_i64_u: gen.gen("rt_i32_convert_i64_u"),
            f64_trunc: gen.gen("rt_f64_trunc"),
            f64_nearest: gen.gen("rt_f64_nearest"),
            f64_eq: gen.gen("rt_f64_eq"),
            f64_ne: gen.gen("rt_f64_ne"),
            f64_lt: gen.gen("rt_f64_lt"),
            f64_gt: gen.gen("rt_f64_gt"),
            f64_le: gen.gen("rt_f64_le"),
            f64_ge: gen.gen("rt_f64_ge"),
            f64_min: gen.gen("rt_f64_min"),
            f64_max: gen.gen("rt_f64_max"),
            f64_copysign: gen.gen("rt_f64_max"),
            i32_trunc_s_f64: gen.gen("rt_i32_trunc_s_f64"),
            i32_trunc_u_f64: gen.gen("rt_i32_trunc_u_f64"),
            i64_trunc_s_f64: gen.gen("rt_i64_trunc_s_f64"),
            i64_trunc_u_f64: gen.gen("rt_i64_trunc_u_f64"),
            i32_trunc_sat_s_f64: gen.gen("rt_i32_trunc_sat_s_f64"),
            i32_trunc_sat_u_f64: gen.gen("rt_i32_trunc_sat_u_f64"),
            i64_trunc_sat_s_f64: gen.gen("rt_i64_trunc_sat_s_f64"),
            i64_trunc_sat_u_f64: gen.gen("rt_i64_trunc_sat_u_f64"),
            f64_convert_i32_u: gen.gen("rt_i64_convert_i32_u"),
            f64_convert_i64_s: gen.gen("rt_i64_convert_i64_s"),
            f64_convert_i64_u: gen.gen("rt_i64_convert_i64_u"),
            table_init_or_copy: gen.gen("rt_table_init"),
            table_grow: gen.gen("rt_table_grow"),
            table_fill: gen.gen("rt_table_fill"),
            memory_init: gen.gen("rt_memory_init"),
            memory_copy: gen.gen("rt_memory_copy"),
            memory_fill: gen.gen("rt_memory_fill"),
            memory_grow: gen.gen("rt_memory_grow"),
            trap_unreachable: gen.gen("trap_unreachable"),
            trap_integer_overflow: gen.gen("trap_integer_overflow"),
            trap_integer_divide_by_zero: gen.gen("trap_integer_divide_by_zero"),
            trap_invalid_conversion_to_integer: gen.gen("trap_invalid_conversion_to_integer"),
            trap_out_of_bounds_memory_access: gen.gen("trap_out_of_bounds_memory_access"),
            trap_indirect_call_type_mismatch: gen.gen("trap_indirect_call_type_mismatch"),
            trap_out_of_bounds_table_access: gen.gen("trap_out_of_bounds_table_access"),
            trap_undefined_element: gen.gen("trap_undefined_element"),
            trap_uninitialized_element: gen.gen("trap_uninitialized_element"),
            trap_call_stack_exhausted: gen.gen("trap_call_stack_exhausted"),
        }
    }
}

fn gen_swap(ctx: &mut Context) {
    push_all!(
        ctx.rom_items,
        label(ctx.rt.swap),
        fnhead_local(1),
        shiftl(lloc(0), imm(16), push()),
        ushiftr(lloc(0), imm(16), push()),
        bitor(pop(), pop(), sloc(0)),
        bitand(lloc(0), uimm(0xff00ff00), push()),
        ushiftr(pop(), imm(8), push()),
        bitand(lloc(0), uimm(0x00ff00ff), push()),
        shiftl(pop(), imm(8), push()),
        bitor(pop(), pop(), push()),
        ret(pop())
    );
}

fn gen_swaps(ctx: &mut Context) {
    push_all!(
        ctx.rom_items,
        label(ctx.rt.swaps),
        fnhead_local(1),
        bitand(lloc(0), uimm(0xff00ff00), push()),
        ushiftr(pop(), imm(8), push()),
        bitand(lloc(0), uimm(0x00ff00ff), push()),
        shiftl(pop(), imm(8), push()),
        bitor(pop(), pop(), push()),
        ret(pop())
    );
}

fn gen_checkaddr(ctx: &mut Context) {
    let addr = 0;
    let offset = 1;
    let size = 2;

    let end_minus_size = 3;
    let addr_plus_offset = 4;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.checkaddr),
        fnhead_local(5),
        jgtu(
            lloc(size),
            derefl(ctx.layout.memory().cur_size),
            ctx.rt.trap_out_of_bounds_memory_access
        ),
        sub(
            derefl(ctx.layout.memory().cur_size),
            lloc(size),
            sloc(end_minus_size)
        ),
        add(lloc(addr), lloc(offset), sloc(addr_plus_offset)),
        jltu(
            lloc(addr_plus_offset),
            lloc(addr),
            ctx.rt.trap_out_of_bounds_memory_access
        ),
        jgtu(
            lloc(addr_plus_offset),
            lloc(end_minus_size),
            ctx.rt.trap_out_of_bounds_memory_access
        ),
        ret(lloc(addr_plus_offset))
    );
}

fn gen_checkglkaddr(ctx: &mut Context) {
    let addr = 0;
    let size = 1;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.checkglkaddr),
        fnhead_local(2),
        jgtu(
            lloc(size),
            uimm(ctx.layout.glk_area().size),
            ctx.rt.trap_out_of_bounds_memory_access
        ),
        sub(uimm(ctx.layout.glk_area().size), lloc(size), push()),
        jgtu(lloc(addr), pop(), ctx.rt.trap_out_of_bounds_memory_access),
        ret(imm(0)),
    );
}

fn gen_checkstr(ctx: &mut Context) {
    let addr = 0;

    let limit = 1;
    let len = 2;

    let loop_label = ctx.gen.gen("checkstr_loop");
    let loop_done = ctx.gen.gen("checkstr_loop_done");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.checkstr),
        fnhead_local(3),
        jgeu(
            lloc(addr),
            derefl(ctx.layout.memory().cur_size),
            ctx.rt.trap_out_of_bounds_memory_access
        ),
        sub(
            derefl(ctx.layout.memory().cur_size),
            lloc(addr),
            sloc(limit)
        ),
        copy(imm(0), sloc(len)),
        label(loop_label),
        jgeu(
            lloc(len),
            lloc(limit),
            ctx.rt.trap_out_of_bounds_memory_access
        ),
        aloadb(imml(ctx.layout.memory().addr), lloc(len), push()),
        jz(pop(), loop_done),
        add(lloc(len), imm(1), sloc(len)),
        jump(loop_label),
        label(loop_done),
        ret(lloc(len))
    );
}

fn gen_checkunistr(ctx: &mut Context) {
    let addr = 0;

    let limit = 1;
    let len = 2;

    let loop_label = ctx.gen.gen("checkunistr_loop");
    let loop_done = ctx.gen.gen("checkunistr_loop_done");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.checkunistr),
        fnhead_local(3),
        jgeu(
            lloc(addr),
            derefl(ctx.layout.memory().cur_size),
            ctx.rt.trap_out_of_bounds_memory_access
        ),
        sub(
            derefl(ctx.layout.memory().cur_size),
            lloc(addr),
            sloc(limit)
        ),
        ushiftr(lloc(limit), imm(2), sloc(limit)),
        copy(imm(0), sloc(len)),
        label(loop_label),
        jgeu(
            lloc(len),
            lloc(limit),
            ctx.rt.trap_out_of_bounds_memory_access
        ),
        aload(
            lloc(len),
            imml_off_shift(ctx.layout.memory().addr, 0, 2),
            push()
        ),
        jz(pop(), loop_done),
        add(lloc(len), imm(1), sloc(len)),
        jump(loop_label),
        label(loop_done),
        ret(lloc(len))
    );
}

fn gen_memload64(ctx: &mut Context) {
    let addr = 1;
    let offset = 0;

    let addr_plus_offset = 2;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.memload64),
        fnhead_local(3),
        callfiii(
            imml(ctx.rt.checkaddr),
            lloc(addr),
            lloc(offset),
            imm(8),
            sloc(addr_plus_offset)
        ),
        aload(
            lloc(addr_plus_offset),
            imml_off_shift(ctx.layout.memory().addr, 4, 2),
            push()
        ),
        callfi(
            imml(ctx.rt.swap),
            pop(),
            storel(ctx.layout.hi_return().addr)
        ),
        aload(
            lloc(addr_plus_offset),
            imml_off_shift(ctx.layout.memory().addr, 0, 2),
            push()
        ),
        tailcall(imml(ctx.rt.swap), imm(1)),
    )
}

fn gen_memload32(ctx: &mut Context) {
    let addr = 1;
    let offset = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.memload32),
        fnhead_local(2),
        callfiii(
            imml(ctx.rt.checkaddr),
            lloc(addr),
            lloc(offset),
            imm(4),
            push()
        ),
        aload(
            pop(),
            imml_off_shift(ctx.layout.memory().addr, 0, 2),
            push()
        ),
        tailcall(imml(ctx.rt.swap), imm(1)),
    );
}

fn gen_memload16(ctx: &mut Context) {
    let addr = 1;
    let offset = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.memload16),
        fnhead_local(2),
        callfiii(
            imml(ctx.rt.checkaddr),
            lloc(addr),
            lloc(offset),
            imm(2),
            push()
        ),
        aloads(
            pop(),
            imml_off_shift(ctx.layout.memory().addr, 0, 1),
            push()
        ),
        tailcall(imml(ctx.rt.swaps), imm(1)),
    );
}

fn gen_memload8(ctx: &mut Context) {
    let addr = 1;
    let offset = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.memload8),
        fnhead_local(2),
        callfiii(
            imml(ctx.rt.checkaddr),
            lloc(addr),
            lloc(offset),
            imm(1),
            push()
        ),
        aloadb(pop(), imml(ctx.layout.memory().addr), push()),
        ret(pop()),
    );
}

fn gen_memstore64(ctx: &mut Context) {
    let addr = 3;
    let val_lo = 2;
    let val_hi = 1;
    let offset = 0;

    let addr_plus_offset = 4;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.memstore64),
        fnhead_local(5),
        callfiii(
            imml(ctx.rt.checkaddr),
            lloc(addr),
            lloc(offset),
            imm(8),
            sloc(addr_plus_offset)
        ),
        callfi(imml(ctx.rt.swap), lloc(val_lo), push()),
        astore(
            lloc(addr_plus_offset),
            imml_off_shift(ctx.layout.memory().addr, 0, 2),
            pop()
        ),
        callfi(imml(ctx.rt.swap), lloc(val_hi), push()),
        astore(
            lloc(addr_plus_offset),
            imml_off_shift(ctx.layout.memory().addr, 4, 2),
            pop()
        ),
        ret(imm(0)),
    );
}

fn gen_memstore32(ctx: &mut Context) {
    let addr = 2;
    let val = 1;
    let offset = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.memstore32),
        fnhead_local(3),
        callfi(imml(ctx.rt.swap), lloc(val), push()),
        callfiii(
            imml(ctx.rt.checkaddr),
            lloc(addr),
            lloc(offset),
            imm(4),
            push(),
        ),
        astore(pop(), imml_off_shift(ctx.layout.memory().addr, 0, 2), pop()),
        ret(imm(0)),
    );
}

fn gen_memstore16(ctx: &mut Context) {
    let addr = 2;
    let val = 1;
    let offset = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.memstore16),
        fnhead_local(3),
        callfi(imml(ctx.rt.swaps), lloc(val), push()),
        callfiii(
            imml(ctx.rt.checkaddr),
            lloc(addr),
            lloc(offset),
            imm(2),
            push(),
        ),
        astores(pop(), imml_off_shift(ctx.layout.memory().addr, 0, 1), pop()),
        ret(imm(0)),
    );
}

fn gen_memstore8(ctx: &mut Context) {
    let addr = 2;
    let val = 1;
    let offset = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.memstore8),
        fnhead_local(3),
        callfiii(
            imml(ctx.rt.checkaddr),
            lloc(addr),
            lloc(offset),
            imm(1),
            push(),
        ),
        astoreb(pop(), imml(ctx.layout.memory().addr), lloc(val)),
        ret(imm(0)),
    );
}

fn gen_swaparray(ctx: &mut Context) {
    let arraybase = 0;
    let arraylen = 1;

    let loop_head = ctx.gen.gen("swaparray_loop_head");
    let loop_end = ctx.gen.gen("swaparray_loop_end");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.swaparray),
        fnhead_local(3),
        label(loop_head),
        jz(lloc(arraylen), loop_end),
        aload(
            lloc(arraybase),
            imml_off_shift(ctx.layout.memory().addr, 0, 2),
            push()
        ),
        callfi(imml(ctx.rt.swap), pop(), push()),
        astore(
            lloc(arraybase),
            imml_off_shift(ctx.layout.memory().addr, 0, 2),
            pop()
        ),
        add(lloc(arraybase), imm(4), sloc(arraybase)),
        sub(lloc(arraylen), imm(1), sloc(arraylen)),
        jump(loop_head),
        label(loop_end),
        ret(imm(0)),
    );
}

fn gen_swapglkarray(ctx: &mut Context) {
    let arraybase = 0;
    let arraylen = 1;

    let loop_head = ctx.gen.gen("swapglkarray_loop_head");
    let loop_end = ctx.gen.gen("swapglkarray_loop_end");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.swapglkarray),
        fnhead_local(3),
        label(loop_head),
        jz(lloc(arraylen), loop_end),
        aload(
            lloc(arraybase),
            imml_off_shift(ctx.layout.glk_area().addr, 0, 2),
            push()
        ),
        callfi(imml(ctx.rt.swap), pop(), push()),
        astore(
            lloc(arraybase),
            imml_off_shift(ctx.layout.glk_area().addr, 0, 2),
            pop()
        ),
        add(lloc(arraybase), imm(4), sloc(arraybase)),
        sub(lloc(arraylen), imm(1), sloc(arraylen)),
        jump(loop_head),
        label(loop_end),
        ret(imm(0)),
    );
}

fn gen_swapunistr(ctx: &mut Context) {
    let arraybase = 0;
    let curword = 1;

    let loop_head = ctx.gen.gen("swapunistr_loop_head");
    let loop_end = ctx.gen.gen("swapunistr_loop_end");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.swapunistr),
        fnhead_local(2),
        label(loop_head),
        aload(
            lloc(arraybase),
            imml_off_shift(ctx.layout.memory().addr, 0, 2),
            sloc(curword)
        ),
        jz(lloc(curword), loop_end),
        callfi(imml(ctx.rt.swap), lloc(curword), push()),
        astore(
            lloc(arraybase),
            imml_off_shift(ctx.layout.memory().addr, 0, 2),
            pop()
        ),
        add(lloc(arraybase), imm(4), sloc(arraybase)),
        jump(loop_head),
        label(loop_end),
        ret(imm(0)),
    );
}

fn gen_i32_div_u(ctx: &mut Context) {
    let divs = ctx.gen.gen("divu_divs");
    let div1 = ctx.gen.gen("divu_div1");
    let dont_add1 = ctx.gen.gen("divu_dontadd1");

    let n = 1; // numerator
    let d = 0; // denominator

    let n_lo = 2; // n & 0x7fffffff
    let hi_quot = 3; // 0x7fffffff / d
    let hi_rem = 4; // 0x7fffffff % d
    let lo_quot = 5; // n_lo / d
    let lo_rem = 6; // n_lo % d

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i32_div_u),
        fnhead_local(7),
        // If d > n, quotient is 0
        jgtu_ret(lloc(d), lloc(n), false),
        // If d fills 32 bits, getting here from previous test means n does too.
        // So the quotient must be 1.
        jlt_ret(lloc(d), imm(0), true),
        // d is at most 31 bits. If n also fits in 31 bits, just do signed division.
        jge(lloc(n), imm(0), divs),
        // Treat division by 1 as a special case so that afterward we can assume
        // 1 / d = 0 and 1 % d = 1.
        jeq(lloc(d), imm(1), div1),
        // We have 32-bit n, sub-32-bit d. This is the hard case. Break up n =
        // (n & 0x7fffffff + 0x7fffffff + 1). Take the sum of the quotients,
        // then add 1 if the sum of the remainders > 1.
        bitand(lloc(n), imm(0x7fffffff), sloc(n_lo)),
        div(imm(0x7fffffff), lloc(d), sloc(hi_quot)),
        modulo(imm(0x7fffffff), lloc(d), sloc(hi_rem)),
        div(lloc(n_lo), lloc(d), sloc(lo_quot)),
        modulo(lloc(n_lo), lloc(d), sloc(lo_rem)),
        // Push the sum of the two quotients...
        add(lloc(hi_quot), lloc(lo_quot), push()),
        // ...then push the sum of the three remainders
        add(lloc(hi_rem), lloc(lo_rem), push()),
        add(pop(), imm(1), push()),
        // If the remainder sum >= d, add 1 to the quotient sum, otherwise
        // don't. Either way, that's our result.
        jltu(pop(), lloc(d), dont_add1),
        add(pop(), imm(1), push()),
        label(dont_add1),
        ret(pop()),
        label(divs),
        // Jump here for the 31-bit signed division case.
        div(lloc(n), lloc(d), push()),
        ret(pop()),
        // Jump here for the division-by-1 case.
        label(div1),
        ret(lloc(n))
    );
}

fn gen_i32_rem_u(ctx: &mut Context) {
    let n = 1;
    let d = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i32_rem_u),
        fnhead_local(2),
        callfii(imml(ctx.rt.i32_div_u), lloc(d), lloc(n), push()),
        mul(pop(), lloc(d), push()),
        sub(lloc(n), pop(), push()),
        ret(pop())
    )
}

fn gen_i32_shl(ctx: &mut Context) {
    let x = 1;
    let r = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i32_shl),
        fnhead_local(2),
        bitand(lloc(r), imm(0x1f), push()),
        shiftl(lloc(x), pop(), push()),
        ret(pop())
    );
}

fn gen_i32_shr_s(ctx: &mut Context) {
    let x = 1;
    let r = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i32_shr_s),
        fnhead_local(2),
        bitand(lloc(r), imm(0x1f), push()),
        sshiftr(lloc(x), pop(), push()),
        ret(pop())
    );
}

fn gen_i32_shr_u(ctx: &mut Context) {
    let x = 1;
    let r = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i32_shr_u),
        fnhead_local(2),
        bitand(lloc(r), imm(0x1f), push()),
        ushiftr(lloc(x), pop(), push()),
        ret(pop())
    );
}

fn gen_i32_rotl(ctx: &mut Context) {
    let x = 1;
    let r = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i32_rotl),
        fnhead_local(2),
        bitand(lloc(r), imm(0x1f), sloc(r)),
        shiftl(lloc(x), lloc(r), push()),
        sub(imm(32), lloc(r), push()),
        ushiftr(lloc(x), pop(), push()),
        bitor(pop(), pop(), push()),
        ret(pop()),
    )
}

fn gen_i32_rotr(ctx: &mut Context) {
    let x = 1;
    let r = 0;
    push_all!(
        ctx.rom_items,
        label(ctx.rt.i32_rotr),
        fnhead_local(2),
        bitand(lloc(r), imm(0x1f), sloc(r)),
        ushiftr(lloc(x), lloc(r), push()),
        sub(imm(32), lloc(r), push()),
        shiftl(lloc(x), pop(), push()),
        bitor(pop(), pop(), push()),
        ret(pop()),
    )
}

fn gen_i32_clz(ctx: &mut Context) {
    let lead8 = ctx.gen.gen("clz_lead8");
    let lead16 = ctx.gen.gen("clz_lead16");
    let lead24 = ctx.gen.gen("clz_lead24");

    let clz_table = ctx.gen.gen("clz_table");
    let mut table_bytes = BytesMut::with_capacity(256);

    for x in 0u8..=255 {
        table_bytes.put_u8(
            x.leading_zeros()
                .try_into()
                .expect("leading zero count of a u8 should fit in a u8"),
        );
    }

    let arg = 0;
    let tmp = 1;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i32_clz),
        fnhead_local(2),
        ushiftr(lloc(arg), imm(24), sloc(tmp)),
        jz(lloc(tmp), lead8),
        aloadb(imml(clz_table), lloc(tmp), push()),
        ret(pop()),
        label(lead8),
        ushiftr(lloc(0), imm(16), sloc(tmp)),
        jz(lloc(tmp), lead16),
        aloadb(imml(clz_table), lloc(tmp), push()),
        add(pop(), imm(8), push()),
        ret(pop()),
        label(lead16),
        ushiftr(lloc(arg), imm(8), sloc(tmp)),
        jz(lloc(tmp), lead24),
        aloadb(imml(clz_table), lloc(tmp), push()),
        add(pop(), imm(16), push()),
        ret(pop()),
        label(lead24),
        aloadb(imml(clz_table), lloc(arg), push()),
        add(pop(), imm(24), push()),
        ret(pop()),
        label(clz_table),
        blob(table_bytes.freeze()),
    )
}

fn gen_i32_ctz(ctx: &mut Context) {
    let trail8 = ctx.gen.gen("ctz_trail8");
    let trail16 = ctx.gen.gen("ctz_trail16");
    let trail24 = ctx.gen.gen("ctz_trail24");

    let ctz_table = ctx.gen.gen("ctz_table");
    let mut table_bytes = BytesMut::with_capacity(256);

    for x in 0u8..=255 {
        table_bytes.put_u8(
            x.trailing_zeros()
                .try_into()
                .expect("trailing zero count of a u8 should fit in a u8"),
        );
    }

    let arg = 0;
    let tmp = 1;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i32_ctz),
        fnhead_local(2),
        bitand(lloc(arg), imm(0xff), sloc(tmp)),
        jz(lloc(tmp), trail8),
        aloadb(imml(ctz_table), lloc(tmp), push()),
        ret(pop()),
        label(trail8),
        ushiftr(lloc(arg), imm(8), push()),
        bitand(pop(), imm(0xff), sloc(tmp)),
        jz(lloc(tmp), trail16),
        aloadb(imml(ctz_table), lloc(tmp), push()),
        add(pop(), imm(8), push()),
        ret(pop()),
        label(trail16),
        ushiftr(lloc(arg), imm(16), push()),
        bitand(pop(), imm(0xff), sloc(tmp)),
        jz(lloc(tmp), trail24),
        aloadb(imml(ctz_table), lloc(tmp), push()),
        add(pop(), imm(16), push()),
        ret(pop()),
        label(trail24),
        ushiftr(lloc(arg), imm(24), push()),
        aloadb(imml(ctz_table), pop(), push()),
        add(pop(), imm(24), push()),
        ret(pop()),
        label(ctz_table),
        blob(table_bytes.freeze()),
    );
}

fn gen_i32_popcnt(ctx: &mut Context) {
    let popcnt_table = ctx.gen.gen("popcnt_table");
    let mut table_bytes = BytesMut::with_capacity(256);

    for x in 0u8..=255 {
        table_bytes.put_u8(
            x.count_ones()
                .try_into()
                .expect("popcnt of a u8 should fit in a u8"),
        );
    }

    let arg = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i32_popcnt),
        fnhead_local(1),
        bitand(lloc(arg), imm(0xff), push()),
        aloadb(imml(popcnt_table), pop(), push()),
        ushiftr(lloc(arg), imm(8), push()),
        bitand(pop(), imm(0xff), push()),
        aloadb(imml(popcnt_table), pop(), push()),
        add(pop(), pop(), push()),
        ushiftr(lloc(arg), imm(16), push()),
        bitand(pop(), imm(0xff), push()),
        aloadb(imml(popcnt_table), pop(), push()),
        add(pop(), pop(), push()),
        ushiftr(lloc(arg), imm(24), push()),
        aloadb(imml(popcnt_table), pop(), push()),
        add(pop(), pop(), push()),
        ret(pop()),
        label(popcnt_table),
        blob(table_bytes.freeze()),
    );
}

fn gen_i32_eqz(ctx: &mut Context) {
    let x = 0;
    push_all!(
        ctx.rom_items,
        label(ctx.rt.i32_eqz),
        fnhead_local(1),
        jz_ret(lloc(x), true),
        ret(imm(0))
    )
}

fn gen_i32_eq(ctx: &mut Context) {
    let x = 1;
    let y = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i32_eq),
        fnhead_local(2),
        jeq_ret(lloc(x), lloc(y), true),
        ret(imm(0))
    )
}

fn gen_i32_ne(ctx: &mut Context) {
    let x = 1;
    let y = 0;
    push_all!(
        ctx.rom_items,
        label(ctx.rt.i32_ne),
        fnhead_local(2),
        jne_ret(lloc(x), lloc(y), true),
        ret(imm(0))
    )
}

fn gen_i32_lt_s(ctx: &mut Context) {
    let x = 1;
    let y = 0;
    push_all!(
        ctx.rom_items,
        label(ctx.rt.i32_lt_s),
        fnhead_local(2),
        jlt_ret(lloc(x), lloc(y), true),
        ret(imm(0))
    )
}

fn gen_i32_lt_u(ctx: &mut Context) {
    let x = 1;
    let y = 0;
    push_all!(
        ctx.rom_items,
        label(ctx.rt.i32_lt_u),
        fnhead_local(2),
        jltu_ret(lloc(x), lloc(y), true),
        ret(imm(0))
    )
}

fn gen_i32_le_s(ctx: &mut Context) {
    let x = 1;
    let y = 0;
    push_all!(
        ctx.rom_items,
        label(ctx.rt.i32_le_s),
        fnhead_local(2),
        jle_ret(lloc(x), lloc(y), true),
        ret(imm(0))
    )
}

fn gen_i32_le_u(ctx: &mut Context) {
    let x = 1;
    let y = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i32_le_u),
        fnhead_local(2),
        jleu_ret(lloc(x), lloc(y), true),
        ret(imm(0))
    )
}

fn gen_i32_gt_s(ctx: &mut Context) {
    let x = 1;
    let y = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i32_gt_s),
        fnhead_local(2),
        jgt_ret(lloc(x), lloc(y), true),
        ret(imm(0))
    )
}

fn gen_i32_gt_u(ctx: &mut Context) {
    let x = 1;
    let y = 0;
    push_all!(
        ctx.rom_items,
        label(ctx.rt.i32_gt_u),
        fnhead_local(2),
        jgtu_ret(lloc(x), lloc(y), true),
        ret(imm(0))
    )
}

fn gen_i32_ge_s(ctx: &mut Context) {
    let x = 1;
    let y = 0;
    push_all!(
        ctx.rom_items,
        label(ctx.rt.i32_ge_s),
        fnhead_local(2),
        jge_ret(lloc(x), lloc(y), true),
        ret(imm(0))
    )
}

fn gen_i32_ge_u(ctx: &mut Context) {
    let x = 1;
    let y = 0;
    push_all!(
        ctx.rom_items,
        label(ctx.rt.i32_ge_u),
        fnhead_local(2),
        jgeu_ret(lloc(x), lloc(y), true),
        ret(imm(0))
    )
}

fn gen_i64_add(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let y_lo = 1;
    let y_hi = 0;

    let sum_lo = 4;
    let sum_hi = 5;

    let nocarry = ctx.gen.gen("add64_nocarry");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_add),
        fnhead_local(6),
        add(lloc(x_lo), lloc(y_lo), sloc(sum_lo)),
        add(lloc(x_hi), lloc(y_hi), sloc(sum_hi)),
        jgeu(lloc(sum_lo), lloc(x_lo), nocarry),
        add(lloc(sum_hi), imm(1), sloc(sum_hi)),
        label(nocarry),
        copy(lloc(sum_hi), storel(ctx.layout.hi_return().addr)),
        ret(lloc(sum_lo)),
    );
}

fn gen_i64_sub(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let y_lo = 1;
    let y_hi = 0;

    let diff_lo = 4;
    let diff_hi = 5;

    let noborrow = ctx.gen.gen("sub64_noborrow");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_sub),
        fnhead_local(6),
        sub(lloc(x_lo), lloc(y_lo), sloc(diff_lo)),
        sub(lloc(x_hi), lloc(y_hi), sloc(diff_hi)),
        jleu(lloc(diff_lo), lloc(x_lo), noborrow),
        sub(lloc(diff_hi), imm(1), sloc(diff_hi)),
        label(noborrow),
        copy(lloc(diff_hi), storel(ctx.layout.hi_return().addr)),
        ret(lloc(diff_lo)),
    );
}

fn gen_i64_mul(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let y_lo = 1;
    let y_hi = 0;

    let x_lohi = 4;
    let x_lolo = 5;
    let y_lohi = 6;
    let y_lolo = 7;

    let out_hi = 8;
    let out_lo = 9;

    let tmp = 10;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_mul),
        fnhead_local(11),
        ushiftr(lloc(x_lo), imm(16), sloc(x_lohi)),
        bitand(lloc(x_lo), imm(0xffff), sloc(x_lolo)),
        ushiftr(lloc(y_lo), imm(16), sloc(y_lohi)),
        bitand(lloc(y_lo), imm(0xffff), sloc(y_lolo)),
        mul(lloc(x_lolo), lloc(y_lolo), sloc(out_lo)),
        ushiftr(lloc(out_lo), imm(16), sloc(tmp)),
        bitand(lloc(out_lo), imm(0xffff), sloc(out_lo)),
        mul(lloc(x_lohi), lloc(y_lolo), push()),
        add(lloc(tmp), pop(), sloc(tmp)),
        bitand(lloc(tmp), imm(0xffff), push()),
        shiftl(pop(), imm(16), push()),
        add(lloc(out_lo), pop(), sloc(out_lo)),
        ushiftr(lloc(tmp), imm(16), sloc(out_hi)),
        ushiftr(lloc(out_lo), imm(16), sloc(tmp)),
        bitand(lloc(out_lo), imm(0xffff), sloc(out_lo)),
        mul(lloc(y_lohi), lloc(x_lolo), push()),
        add(lloc(tmp), pop(), sloc(tmp)),
        bitand(lloc(tmp), imm(0xffff), push()),
        shiftl(pop(), imm(16), push()),
        add(lloc(out_lo), pop(), sloc(out_lo)),
        ushiftr(lloc(tmp), imm(16), push()),
        add(lloc(out_hi), pop(), sloc(out_hi)),
        mul(lloc(x_lohi), lloc(y_lohi), push()),
        add(lloc(out_hi), pop(), sloc(out_hi)),
        mul(lloc(x_hi), lloc(y_lo), push()),
        mul(lloc(x_lo), lloc(y_hi), push()),
        add(pop(), pop(), push()),
        add(lloc(out_hi), pop(), sloc(out_hi)),
        copy(lloc(out_hi), storel(ctx.layout.hi_return().addr)),
        ret(lloc(out_lo)),
    )
}

fn gen_i64_div_u(ctx: &mut Context) {
    let n_lo = 3;
    let n_hi = 2;
    let d_lo = 1;
    let d_hi = 0;

    let sr = 4;
    let q_hi = 5;
    let q_lo = 6;
    let r_hi = 7;
    let r_lo = 8;
    let carry = 9;
    let sr_from_32 = 10;
    let sr_minus_32 = 11;
    let sr_from_64 = 12;

    let kx_xx = ctx.gen.gen("rt_div64u_kx_xx");
    let kx_xk = ctx.gen.gen("rt_div64u_kx_xk");
    let kk_xz = ctx.gen.gen("rt_div64u_kx_xz");
    //let kk_kz_notpow2 = ctx.gen.gen("rt_div64u_kk_kz_notpow2");
    let kk_kz_srlt32 = ctx.gen.gen("rt_div64u_kk_kz_srlt32");
    let kx_kk = ctx.gen.gen("rt_divu64_kx_kk");
    let kx_zk_srne32 = ctx.gen.gen("rt_div64u_kx_zx_srne32");
    let kx_zk_srgt32 = ctx.gen.gen("rt_div64u_kx_zx_srgt32");
    let kx_kk_srle32 = ctx.gen.gen("rt_div64u_kx_kk_srle32");
    let kx_kk_srlt32 = ctx.gen.gen("rt_div64u_kx_kk_srlt32");

    let main_loop = ctx.gen.gen("rt_div64_mainloop");
    let main_loop_done = ctx.gen.gen("rt_div64_mainloop_done");
    let main_loop_nocarry = ctx.gen.gen("rt_div64_mainloop_nocarry");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_div_u),
        fnhead_local(13),
        jnz(lloc(n_hi), kx_xx),
        // 0X/XX
        copy(imm(0), storel(ctx.layout.hi_return().addr)),
        jnz_ret(lloc(d_hi), false),
        // 0X/0X
        callfii(imml(ctx.rt.i32_div_u), lloc(d_lo), lloc(n_lo), push()),
        ret(pop()),
        label(kx_xx),
        // KX/XX
        jnz(lloc(d_lo), kx_xk),
        // KX/X0
        jnz(lloc(n_lo), kk_xz),
        // K0/X0
        copy(imm(0), storel(ctx.layout.hi_return().addr)),
        callfii(imml(ctx.rt.i32_div_u), lloc(d_lo), lloc(n_lo), push()),
        ret(pop()),
        label(kk_xz),
        // KK/X0
        jz(lloc(d_hi), ctx.rt.trap_integer_divide_by_zero),
        // KK/K0
        callfi(imml(ctx.rt.i32_clz), lloc(n_hi), push()),
        callfi(imml(ctx.rt.i32_clz), lloc(d_hi), push()),
        sub(pop(), pop(), push()),
        add(pop(), imm(1), sloc(sr)),
        jltu(lloc(sr), imm(32), kk_kz_srlt32),
        copy(imm(0), storel(ctx.layout.hi_return().addr)),
        ret(imm(0)),
        label(kk_kz_srlt32),
        copy(imm(0), sloc(q_lo)),
        sub(imm(32), lloc(sr), sloc(sr_from_32)),
        shiftl(lloc(n_lo), lloc(sr_from_32), sloc(q_hi)),
        ushiftr(lloc(n_hi), lloc(sr), sloc(r_hi)),
        shiftl(lloc(n_hi), lloc(sr_from_32), push()),
        ushiftr(lloc(n_lo), lloc(sr), push()),
        bitor(pop(), pop(), sloc(r_lo)),
        copy(imm(0), sloc(carry)),
        jump(main_loop),
        label(kx_xk),
        // KX/XK
        jnz(lloc(d_hi), kx_kk),
        // KX/0K
        copy(imm(0), sloc(carry)),
        callfi(imml(ctx.rt.i32_clz), lloc(n_hi), push()),
        callfi(imml(ctx.rt.i32_clz), lloc(d_lo), push()),
        sub(pop(), pop(), push()),
        add(pop(), imm(33), sloc(sr)),
        jne(lloc(sr), imm(32), kx_zk_srne32),
        copy(imm(0), sloc(q_lo)),
        copy(lloc(n_lo), sloc(q_hi)),
        copy(imm(0), sloc(r_hi)),
        copy(lloc(n_hi), sloc(r_lo)),
        jump(main_loop),
        label(kx_zk_srne32),
        jgtu(lloc(sr), imm(32), kx_zk_srgt32),
        copy(imm(0), sloc(q_lo)),
        sub(imm(32), lloc(sr), sloc(sr_from_32)),
        shiftl(lloc(n_lo), lloc(sr_from_32), sloc(q_hi)),
        ushiftr(lloc(n_hi), lloc(sr), sloc(r_hi)),
        shiftl(lloc(n_hi), lloc(sr_from_32), push()),
        ushiftr(lloc(n_lo), lloc(sr), push()),
        bitor(pop(), pop(), sloc(r_lo)),
        jump(main_loop),
        label(kx_zk_srgt32),
        sub(imm(64), lloc(sr), sloc(sr_from_64)),
        sub(lloc(sr), imm(32), sloc(sr_minus_32)),
        shiftl(lloc(n_lo), lloc(sr_from_64), sloc(q_lo)),
        shiftl(lloc(n_hi), lloc(sr_from_64), push()),
        ushiftr(lloc(n_lo), lloc(sr_minus_32), push()),
        bitor(pop(), pop(), sloc(q_hi)),
        copy(imm(0), sloc(r_hi)),
        ushiftr(lloc(n_hi), lloc(sr_minus_32), sloc(r_lo)),
        jump(main_loop),
        label(kx_kk),
        // KX/KK
        callfi(imml(ctx.rt.i32_clz), lloc(n_hi), push()),
        callfi(imml(ctx.rt.i32_clz), lloc(d_hi), push()),
        sub(pop(), pop(), push()),
        add(imm(1), pop(), sloc(sr)),
        jleu(lloc(sr), imm(32), kx_kk_srle32),
        copy(imm(0), storel(ctx.layout.hi_return().addr)),
        ret(imm(0)),
        label(kx_kk_srle32),
        copy(imm(0), sloc(carry)),
        copy(imm(0), sloc(q_lo)),
        jltu(lloc(sr), imm(32), kx_kk_srlt32),
        copy(lloc(n_lo), sloc(q_hi)),
        copy(imm(0), sloc(r_hi)),
        copy(lloc(n_hi), sloc(r_lo)),
        jump(main_loop),
        label(kx_kk_srlt32),
        sub(imm(32), lloc(sr), sloc(sr_from_32)),
        shiftl(lloc(n_lo), lloc(sr_from_32), sloc(q_hi)),
        ushiftr(lloc(n_hi), lloc(sr), sloc(r_hi)),
        ushiftr(lloc(n_lo), lloc(sr), push()),
        shiftl(lloc(n_hi), lloc(sr_from_32), push()),
        bitor(pop(), pop(), sloc(r_lo)),
        label(main_loop),
        // MAIN LOOP
        jz(lloc(sr), main_loop_done),
        ushiftr(lloc(r_lo), imm(31), push()),
        shiftl(lloc(r_hi), imm(1), push()),
        bitor(pop(), pop(), sloc(r_hi)),
        ushiftr(lloc(q_hi), imm(31), push()),
        shiftl(lloc(r_lo), imm(1), push()),
        bitor(pop(), pop(), sloc(r_lo)),
        ushiftr(lloc(q_lo), imm(31), push()),
        shiftl(lloc(q_hi), imm(1), push()),
        bitor(pop(), pop(), sloc(q_hi)),
        shiftl(lloc(q_lo), imm(1), push()),
        bitor(pop(), lloc(carry), sloc(q_lo)),
        copy(imm(0), sloc(carry)),
        copy(lloc(r_lo), push()),
        copy(lloc(r_hi), push()),
        copy(lloc(d_lo), push()),
        copy(lloc(d_hi), push()),
        call(imml(ctx.rt.i64_lt_u), imm(4), push()),
        jnz(pop(), main_loop_nocarry),
        copy(lloc(r_lo), push()),
        copy(lloc(r_hi), push()),
        copy(lloc(d_lo), push()),
        copy(lloc(d_hi), push()),
        call(imml(ctx.rt.i64_sub), imm(4), sloc(r_lo)),
        copy(derefl(ctx.layout.hi_return().addr), sloc(r_hi)),
        copy(imm(1), sloc(carry)),
        label(main_loop_nocarry),
        sub(lloc(sr), imm(1), sloc(sr)),
        jump(main_loop),
        label(main_loop_done),
        shiftl(lloc(q_hi), imm(1), push()),
        ushiftr(lloc(q_lo), imm(31), push()),
        bitor(pop(), pop(), storel(ctx.layout.hi_return().addr)),
        shiftl(lloc(q_lo), imm(1), push()),
        bitor(pop(), lloc(carry), push()),
        ret(pop())
    );
}

fn gen_i64_div_s(ctx: &mut Context) {
    let n_lo = 3;
    let n_hi = 2;
    let d_lo = 1;
    let d_hi = 0;

    let sign = 4;
    let out_lo = 5;

    let main_case = ctx.gen.gen("rt_divs64_main_case");
    let n_pos = ctx.gen.gen("rt_divs64_n_pos");
    let d_pos = ctx.gen.gen("rt_divs64_d_pos");
    let out_pos = ctx.gen.gen("rt_divs64_out_pos");
    let out_nocarry = ctx.gen.gen("rt_divs64_out_nocarry");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_div_s),
        fnhead_local(6),
        jnz(lloc(n_lo), main_case),
        jne(lloc(n_hi), uimm(0x80000000), main_case),
        jne(lloc(d_lo), imm(-1), main_case),
        jeq(lloc(d_hi), imm(-1), ctx.rt.trap_integer_overflow),
        label(main_case),
        copy(imm(0), sloc(sign)),
        jge(lloc(n_hi), imm(0), n_pos),
        copy(imm(1), sloc(sign)),
        bitxor(lloc(n_hi), imm(-1), sloc(n_hi)),
        bitxor(lloc(n_lo), imm(-1), sloc(n_lo)),
        add(lloc(n_lo), imm(1), sloc(n_lo)),
        jnz(lloc(n_lo), n_pos),
        add(lloc(n_hi), imm(1), sloc(n_hi)),
        label(n_pos),
        jge(lloc(d_hi), imm(0), d_pos),
        bitxor(lloc(sign), imm(1), sloc(sign)),
        bitxor(lloc(d_hi), imm(-1), sloc(d_hi)),
        bitxor(lloc(d_lo), imm(-1), sloc(d_lo)),
        add(lloc(d_lo), imm(1), sloc(d_lo)),
        jnz(lloc(d_lo), d_pos),
        add(lloc(d_hi), imm(1), sloc(d_hi)),
        label(d_pos),
        copy(lloc(n_lo), push()),
        copy(lloc(n_hi), push()),
        copy(lloc(d_lo), push()),
        copy(lloc(d_hi), push()),
        jz(lloc(sign), out_pos),
        call(imml(ctx.rt.i64_div_u), imm(4), sloc(out_lo)),
        bitxor(
            derefl(ctx.layout.hi_return().addr),
            imm(-1),
            storel(ctx.layout.hi_return().addr)
        ),
        bitxor(lloc(out_lo), imm(-1), sloc(out_lo)),
        add(lloc(out_lo), imm(1), sloc(out_lo)),
        jnz(lloc(out_lo), out_nocarry),
        add(
            derefl(ctx.layout.hi_return().addr),
            imm(1),
            storel(ctx.layout.hi_return().addr)
        ),
        label(out_nocarry),
        ret(lloc(out_lo)),
        label(out_pos),
        call(imml(ctx.rt.i64_div_u), imm(4), push()),
        ret(pop())
    );
}

fn gen_i64_rem_u(ctx: &mut Context) {
    let n_lo = 3;
    let n_hi = 2;
    let d_lo = 1;
    let d_hi = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_rem_u),
        fnhead_local(4),
        copy(lloc(n_lo), push()),
        copy(lloc(n_hi), push()),
        copy(lloc(n_lo), push()),
        copy(lloc(n_hi), push()),
        copy(lloc(d_lo), push()),
        copy(lloc(d_hi), push()),
        call(imml(ctx.rt.i64_div_u), imm(4), push()),
        copy(derefl(ctx.layout.hi_return().addr), push()),
        copy(lloc(d_lo), push()),
        copy(lloc(d_hi), push()),
        call(imml(ctx.rt.i64_mul), imm(4), push()),
        copy(derefl(ctx.layout.hi_return().addr), push()),
        call(imml(ctx.rt.i64_sub), imm(4), push()),
        ret(pop())
    );
}

fn gen_i64_rem_s(ctx: &mut Context) {
    let n_lo = 3;
    let n_hi = 2;
    let d_lo = 1;
    let d_hi = 0;

    let main_case = ctx.gen.gen("rems64_main_case");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_rem_s),
        fnhead_local(4),
        jne(lloc(d_lo), imm(-1), main_case),
        jne(lloc(d_hi), imm(-1), main_case),
        copy(imm(0), storel(ctx.layout.hi_return().addr)),
        ret(imm(0)),
        label(main_case),
        copy(lloc(n_lo), push()),
        copy(lloc(n_hi), push()),
        copy(lloc(n_lo), push()),
        copy(lloc(n_hi), push()),
        copy(lloc(d_lo), push()),
        copy(lloc(d_hi), push()),
        call(imml(ctx.rt.i64_div_s), imm(4), push()),
        copy(derefl(ctx.layout.hi_return().addr), push()),
        copy(lloc(d_lo), push()),
        copy(lloc(d_hi), push()),
        call(imml(ctx.rt.i64_mul), imm(4), push()),
        copy(derefl(ctx.layout.hi_return().addr), push()),
        call(imml(ctx.rt.i64_sub), imm(4), push()),
        ret(pop())
    );
}

fn gen_i64_and(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let y_lo = 1;
    let y_hi = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_and),
        fnhead_local(4),
        bitand(lloc(x_hi), lloc(y_hi), storel(ctx.layout.hi_return().addr)),
        bitand(lloc(x_lo), lloc(y_lo), push()),
        ret(pop())
    );
}

fn gen_i64_or(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let y_lo = 1;
    let y_hi = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_or),
        fnhead_local(4),
        bitor(lloc(x_hi), lloc(y_hi), storel(ctx.layout.hi_return().addr)),
        bitor(lloc(x_lo), lloc(y_lo), push()),
        ret(pop())
    );
}

fn gen_i64_xor(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let y_lo = 1;
    let y_hi = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_xor),
        fnhead_local(4),
        bitxor(lloc(x_hi), lloc(y_hi), storel(ctx.layout.hi_return().addr)),
        bitxor(lloc(x_lo), lloc(y_lo), push()),
        ret(pop())
    );
}

fn gen_i64_shl(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let r = 1;
    //r_hi = 0

    let shift32 = ctx.gen.gen("shl64_shift32");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_shl),
        fnhead_local(4),
        bitand(lloc(r), imm(0x3f), sloc(r)),
        jgeu(lloc(r), imm(32), shift32),
        shiftl(lloc(x_hi), lloc(r), sloc(x_hi)),
        sub(imm(32), lloc(r), push()),
        ushiftr(lloc(x_lo), pop(), push()),
        bitor(lloc(x_hi), pop(), storel(ctx.layout.hi_return().addr)),
        shiftl(lloc(x_lo), lloc(r), push()),
        ret(pop()),
        label(shift32),
        sub(lloc(r), imm(32), push()),
        shiftl(lloc(x_lo), pop(), storel(ctx.layout.hi_return().addr)),
        ret(imm(0)),
    )
}

fn gen_i64_shr_s(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let r = 1;
    //r_hi = 0

    let shift32 = ctx.gen.gen("shr64_shift32");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_shr_s),
        fnhead_local(4),
        bitand(lloc(r), imm(0x3f), sloc(r)),
        jgeu(lloc(r), imm(32), shift32),
        sshiftr(lloc(x_hi), lloc(r), storel(ctx.layout.hi_return().addr)),
        ushiftr(lloc(x_lo), lloc(r), sloc(x_lo)),
        sub(imm(32), lloc(r), push()),
        shiftl(lloc(x_hi), pop(), push()),
        bitor(lloc(x_lo), pop(), push()),
        ret(pop()),
        label(shift32),
        sshiftr(lloc(x_hi), imm(31), push()),
        copy(pop(), storel(ctx.layout.hi_return().addr)),
        sub(lloc(r), imm(32), push()),
        sshiftr(lloc(x_hi), pop(), push()),
        ret(pop()),
    )
}

fn gen_i64_shr_u(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let r = 1;
    //r_hi = 0

    let shift32 = ctx.gen.gen("shru64_shift32");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_shr_u),
        fnhead_local(4),
        bitand(lloc(r), imm(0x3f), sloc(r)),
        jgeu(lloc(r), imm(32), shift32),
        ushiftr(lloc(x_hi), lloc(r), storel(ctx.layout.hi_return().addr)),
        ushiftr(lloc(x_lo), lloc(r), sloc(x_lo)),
        sub(imm(32), lloc(r), push()),
        shiftl(lloc(x_hi), pop(), push()),
        bitor(lloc(x_lo), pop(), push()),
        ret(pop()),
        label(shift32),
        copy(imm(0), storel(ctx.layout.hi_return().addr)),
        sub(lloc(r), imm(32), push()),
        ushiftr(lloc(x_hi), pop(), push()),
        ret(pop()),
    )
}

fn gen_i64_rotl(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let r = 1;
    //r_hi = 0

    let x_hi_shifted = 4;
    let x_lo_shifted = 5;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_rotl),
        fnhead_local(6),
        bitand(lloc(r), imm(0x3f), sloc(r)),
        copy(lloc(x_lo), push()),
        copy(lloc(x_hi), push()),
        copy(lloc(r), push()),
        copy(imm(0), push()),
        call(imml(ctx.rt.i64_shl), imm(4), sloc(x_lo_shifted)),
        copy(derefl(ctx.layout.hi_return().addr), sloc(x_hi_shifted)),
        copy(lloc(x_lo), push()),
        copy(lloc(x_hi), push()),
        sub(imm(64), lloc(r), push()),
        copy(imm(0), push()),
        call(imml(ctx.rt.i64_shr_u), imm(4), push()),
        bitor(
            lloc(x_hi_shifted),
            derefl(ctx.layout.hi_return().addr),
            storel(ctx.layout.hi_return().addr)
        ),
        bitor(lloc(x_lo_shifted), pop(), push()),
        ret(pop()),
    );
}

fn gen_i64_rotr(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let r = 1;
    //r_hi = 0

    let x_hi_shifted = 4;
    let x_lo_shifted = 5;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_rotr),
        fnhead_local(6),
        bitand(lloc(r), imm(0x3f), sloc(r)),
        copy(lloc(x_lo), push()),
        copy(lloc(x_hi), push()),
        copy(lloc(r), push()),
        copy(imm(0), push()),
        call(imml(ctx.rt.i64_shr_u), imm(4), sloc(x_lo_shifted)),
        copy(derefl(ctx.layout.hi_return().addr), sloc(x_hi_shifted)),
        copy(lloc(x_lo), push()),
        copy(lloc(x_hi), push()),
        sub(imm(64), lloc(r), push()),
        copy(imm(0), push()),
        call(imml(ctx.rt.i64_shl), imm(4), push()),
        bitor(
            lloc(x_hi_shifted),
            derefl(ctx.layout.hi_return().addr),
            storel(ctx.layout.hi_return().addr)
        ),
        bitor(lloc(x_lo_shifted), pop(), push()),
        ret(pop()),
    );
}

fn gen_i64_eqz(ctx: &mut Context) {
    let x_lo = 1;
    let x_hi = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_eqz),
        fnhead_local(2),
        jnz_ret(lloc(x_hi), false),
        jnz_ret(lloc(x_lo), false),
        ret(imm(1))
    );
}

fn gen_i64_eq(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let y_lo = 1;
    let y_hi = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_eq),
        fnhead_local(4),
        jne_ret(lloc(x_hi), lloc(y_hi), false),
        jne_ret(lloc(x_lo), lloc(y_lo), false),
        ret(imm(1))
    );
}

fn gen_i64_ne(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let y_lo = 1;
    let y_hi = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_ne),
        fnhead_local(4),
        jne_ret(lloc(x_hi), lloc(y_hi), true),
        jne_ret(lloc(x_lo), lloc(y_lo), true),
        ret(imm(0))
    );
}

fn gen_i64_lt_s(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let y_lo = 1;
    let y_hi = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_lt_s),
        fnhead_local(4),
        jlt_ret(lloc(x_hi), lloc(y_hi), true),
        jlt_ret(lloc(y_hi), lloc(x_hi), false),
        jlt_ret(lloc(x_lo), lloc(y_lo), true),
        ret(imm(0)),
    );
}

fn gen_i64_lt_u(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let y_lo = 1;
    let y_hi = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_lt_u),
        fnhead_local(4),
        jltu_ret(lloc(x_hi), lloc(y_hi), true),
        jltu_ret(lloc(y_hi), lloc(x_hi), false),
        jltu_ret(lloc(x_lo), lloc(y_lo), true),
        ret(imm(0)),
    );
}

fn gen_i64_gt_s(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let y_lo = 1;
    let y_hi = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_gt_s),
        fnhead_local(4),
        jgt_ret(lloc(x_hi), lloc(y_hi), true),
        jgt_ret(lloc(y_hi), lloc(x_hi), false),
        jgt_ret(lloc(x_lo), lloc(y_lo), true),
        ret(imm(0)),
    );
}

fn gen_i64_gt_u(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let y_lo = 1;
    let y_hi = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_gt_u),
        fnhead_local(4),
        jgtu_ret(lloc(x_hi), lloc(y_hi), true),
        jgtu_ret(lloc(y_hi), lloc(x_hi), false),
        jgtu_ret(lloc(x_lo), lloc(y_lo), true),
        ret(imm(0)),
    );
}

fn gen_i64_le_s(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let y_lo = 1;
    let y_hi = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_le_s),
        fnhead_local(4),
        jlt_ret(lloc(x_hi), lloc(y_hi), true),
        jlt_ret(lloc(y_hi), lloc(x_hi), false),
        jle_ret(lloc(x_lo), lloc(y_lo), true),
        ret(imm(0)),
    );
}

fn gen_i64_le_u(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let y_lo = 1;
    let y_hi = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_le_u),
        fnhead_local(4),
        jltu_ret(lloc(x_hi), lloc(y_hi), true),
        jltu_ret(lloc(y_hi), lloc(x_hi), false),
        jleu_ret(lloc(x_lo), lloc(y_lo), true),
        ret(imm(0)),
    );
}

fn gen_i64_ge_s(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let y_lo = 1;
    let y_hi = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_ge_s),
        fnhead_local(4),
        jgt_ret(lloc(x_hi), lloc(y_hi), true),
        jgt_ret(lloc(y_hi), lloc(x_hi), false),
        jge_ret(lloc(x_lo), lloc(y_lo), true),
        ret(imm(0)),
    );
}

fn gen_i64_ge_u(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let y_lo = 1;
    let y_hi = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_ge_u),
        fnhead_local(4),
        jgtu_ret(lloc(x_hi), lloc(y_hi), true),
        jgtu_ret(lloc(y_hi), lloc(x_hi), false),
        jgeu_ret(lloc(x_lo), lloc(y_lo), true),
        ret(imm(0)),
    );
}

fn gen_i64_clz(ctx: &mut Context) {
    let x_lo = 1;
    let x_hi = 0;

    let hi_clz = 2;

    let hi32 = ctx.gen.gen("clz64_hi32");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_clz),
        fnhead_local(3),
        copy(imm(0), storel(ctx.layout.hi_return().addr)),
        callfi(imml(ctx.rt.i32_clz), lloc(x_hi), sloc(hi_clz)),
        jeq(lloc(hi_clz), imm(32), hi32),
        ret(lloc(hi_clz)),
        label(hi32),
        callfi(imml(ctx.rt.i32_clz), lloc(x_lo), push()),
        add(imm(32), pop(), push()),
        ret(pop()),
    )
}

fn gen_i64_ctz(ctx: &mut Context) {
    let x_lo = 1;
    let x_hi = 0;

    let lo_ctz = 2;

    let lo32 = ctx.gen.gen("ctz64_lo32");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_ctz),
        fnhead_local(3),
        copy(imm(0), storel(ctx.layout.hi_return().addr)),
        callfi(imml(ctx.rt.i32_ctz), lloc(x_lo), sloc(lo_ctz)),
        jeq(lloc(lo_ctz), imm(32), lo32),
        ret(lloc(lo_ctz)),
        label(lo32),
        callfi(imml(ctx.rt.i32_ctz), lloc(x_hi), push()),
        add(imm(32), pop(), push()),
        ret(pop()),
    )
}

fn gen_i64_popcnt(ctx: &mut Context) {
    let x_lo = 1;
    let x_hi = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_popcnt),
        fnhead_local(2),
        copy(imm(0), storel(ctx.layout.hi_return().addr)),
        callfi(imml(ctx.rt.i32_popcnt), lloc(x_hi), push()),
        callfi(imml(ctx.rt.i32_popcnt), lloc(x_lo), push()),
        add(pop(), pop(), push()),
        ret(pop()),
    )
}

fn gen_f32_trunc(ctx: &mut Context) {
    let x = 0;
    let neg = ctx.gen.gen("f32_trunc_neg");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.f32_trunc),
        fnhead_local(1),
        // Intentionally an integer comparison
        jlt(lloc(x), imm(0), neg),
        floor(lloc(x), push()),
        ret(pop()),
        label(neg),
        ceil(lloc(x), push()),
        ret(pop())
    );
}

fn gen_f32_nearest(ctx: &mut Context) {
    let x = 0;

    let x_ceil = 1;
    let x_floor = 2;

    let ident = ctx.gen.gen("f32_nearest_ident");
    let nan = ctx.gen.gen("f32_nearest_nan");
    let neg = ctx.gen.gen("f32_nearest_neg");
    let lehalf = ctx.gen.gen("f32_nearest_lehalf");
    let geneghalf = ctx.gen.gen("f32_nearest_geneghalf");
    let choose_floor = ctx.gen.gen("f32_nearest_choose_floor");
    let choose_ceil = ctx.gen.gen("f32_nearest_choose_ceil");
    let main_case = ctx.gen.gen("f32_nearest_maincase");
    push_all!(
        ctx.rom_items,
        label(ctx.rt.f32_nearest),
        fnhead_local(3),
        jisnan(lloc(x), nan),
        jisinf(lloc(x), ident),
        jfeq(lloc(x), f32_to_imm(0.), f32_to_imm(0.), ident),
        jflt(lloc(x), f32_to_imm(0.), neg),
        jfle(lloc(x), f32_to_imm(0.5), lehalf),
        jump(main_case),
        label(neg),
        jfge(lloc(x), f32_to_imm(-0.5), geneghalf),
        label(main_case),
        ceil(lloc(x), sloc(x_ceil)),
        floor(lloc(x), sloc(x_floor)),
        fsub(lloc(x), lloc(x_floor), push()),
        jflt(pop(), f32_to_imm(0.5), choose_floor),
        fsub(lloc(x_ceil), lloc(x), push()),
        jflt(pop(), f32_to_imm(0.5), choose_ceil),
        callfi(imml(ctx.rt.i32_ctz), lloc(x_ceil), push()),
        callfi(imml(ctx.rt.i32_ctz), lloc(x_floor), push()),
        jgtu(pop(), pop(), choose_floor),
        label(choose_ceil),
        ret(lloc(x_ceil)),
        label(nan),
        bitor(lloc(x), uimm(0x00400000), sloc(x)),
        label(ident),
        ret(lloc(x)),
        label(lehalf),
        ret(f32_to_imm(0.)),
        label(geneghalf),
        ret(f32_to_imm(-0.)),
        label(choose_floor),
        ret(lloc(x_floor)),
    )
}

fn gen_f32_eq(ctx: &mut Context) {
    let x = 1;
    let y = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.f32_eq),
        fnhead_local(2),
        jfeq_ret(lloc(x), lloc(y), f32_to_imm(0.), true),
        ret(imm(0)),
    );
}

fn gen_f32_ne(ctx: &mut Context) {
    let x = 1;
    let y = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.f32_ne),
        fnhead_local(2),
        jfne_ret(lloc(x), lloc(y), f32_to_imm(0.), true),
        ret(imm(0)),
    );
}

fn gen_f32_lt(ctx: &mut Context) {
    let x = 1;
    let y = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.f32_lt),
        fnhead_local(2),
        jflt_ret(lloc(x), lloc(y), true),
        ret(imm(0)),
    );
}

fn gen_f32_gt(ctx: &mut Context) {
    let x = 1;
    let y = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.f32_gt),
        fnhead_local(2),
        jfgt_ret(lloc(x), lloc(y), true),
        ret(imm(0)),
    );
}

fn gen_f32_le(ctx: &mut Context) {
    let x = 1;
    let y = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.f32_le),
        fnhead_local(2),
        jfle_ret(lloc(x), lloc(y), true),
        ret(imm(0)),
    );
}

fn gen_f32_ge(ctx: &mut Context) {
    let x = 1;
    let y = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.f32_ge),
        fnhead_local(2),
        jfge_ret(lloc(x), lloc(y), true),
        ret(imm(0)),
    );
}

fn gen_f32_min(ctx: &mut Context) {
    let x = 1;
    let y = 0;

    let x_nan = ctx.gen.gen("f32_x_nan");
    let y_nan = ctx.gen.gen("f32_y_nan");
    let choose_x = ctx.gen.gen("f32_choose_x");
    let choose_y = ctx.gen.gen("f32_choose_y");
    let x_neg_zero = ctx.gen.gen("f32_y_neg_zero");
    let y_neg_zero = ctx.gen.gen("f32_y_neg_zero");
    let main_case = ctx.gen.gen("f32_main_case");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.f32_min),
        fnhead_local(2),
        jisnan(lloc(x), x_nan),
        jisnan(lloc(y), y_nan),
        jeq(lloc(x), f32_to_imm(f32::NEG_INFINITY), choose_x),
        jeq(lloc(y), f32_to_imm(f32::NEG_INFINITY), choose_y),
        jeq(lloc(x), f32_to_imm(f32::INFINITY), choose_y),
        jeq(lloc(y), f32_to_imm(f32::INFINITY), choose_x),
        jeq(lloc(x), f32_to_imm(-0.), x_neg_zero),
        jeq(lloc(y), f32_to_imm(-0.), y_neg_zero),
        label(main_case),
        jflt(lloc(x), lloc(y), choose_x),
        label(choose_y),
        ret(lloc(y)),
        label(x_nan),
        bitor(lloc(x), imm(0x00400000), sloc(x)),
        label(choose_x),
        ret(lloc(x)),
        label(y_nan),
        bitor(lloc(y), imm(0x00400000), sloc(y)),
        ret(lloc(y)),
        label(x_neg_zero),
        jeq(lloc(y), f32_to_imm(0.), choose_x),
        jump(main_case),
        label(y_neg_zero),
        jeq(lloc(x), f32_to_imm(0.), choose_y),
        jump(main_case),
    )
}

fn gen_f32_max(ctx: &mut Context) {
    let x = 1;
    let y = 0;

    let x_nan = ctx.gen.gen("f32_x_nan");
    let y_nan = ctx.gen.gen("f32_y_nan");
    let choose_x = ctx.gen.gen("f32_choose_x");
    let choose_y = ctx.gen.gen("f32_choose_y");
    let x_neg_zero = ctx.gen.gen("f32_x_neg_zero");
    let y_neg_zero = ctx.gen.gen("f32_y_neg_zero");
    let main_case = ctx.gen.gen("f32_main_case");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.f32_max),
        fnhead_local(2),
        jisnan(lloc(x), x_nan),
        jisnan(lloc(y), y_nan),
        jeq(lloc(x), f32_to_imm(f32::INFINITY), choose_x),
        jeq(lloc(y), f32_to_imm(f32::INFINITY), choose_y),
        jeq(lloc(x), f32_to_imm(f32::NEG_INFINITY), choose_y),
        jeq(lloc(y), f32_to_imm(f32::NEG_INFINITY), choose_x),
        jeq(lloc(x), f32_to_imm(-0.), x_neg_zero),
        jeq(lloc(y), f32_to_imm(-0.), y_neg_zero),
        label(main_case),
        jfgt(lloc(x), lloc(y), choose_x),
        label(choose_y),
        ret(lloc(y)),
        label(x_nan),
        bitor(lloc(x), imm(0x00400000), sloc(x)),
        label(choose_x),
        ret(lloc(x)),
        label(y_nan),
        bitor(lloc(y), imm(0x00400000), sloc(y)),
        ret(lloc(y)),
        label(x_neg_zero),
        jeq(lloc(y), f32_to_imm(0.), choose_y),
        jump(main_case),
        label(y_neg_zero),
        jeq(lloc(x), f32_to_imm(0.), choose_x),
        jump(main_case)
    )
}

fn gen_f32_copysign(ctx: &mut Context) {
    let x = 1;
    let y = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.f32_copysign),
        fnhead_local(2),
        bitand(lloc(y), uimm(0x80000000), push()),
        bitand(lloc(x), uimm(0x7fffffff), push()),
        bitor(pop(), pop(), push()),
        ret(pop())
    )
}

fn gen_i32_trunc_s_f32(ctx: &mut Context) {
    let x = 0;

    let common = ctx.gen.gen("i32_trunc_s_f32_common");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i32_trunc_sat_s_f32),
        fnhead_local(1),
        jisnan_ret(lloc(x), false),
        jump(common),
        label(ctx.rt.i32_trunc_s_f32),
        fnhead_local(1),
        jisnan(lloc(x), ctx.rt.trap_invalid_conversion_to_integer),
        jfge(lloc(x), uimm(0x4f000000), ctx.rt.trap_integer_overflow),
        jflt(lloc(x), uimm(0xcf000000), ctx.rt.trap_integer_overflow),
        label(common),
        ftonumz(lloc(x), push()),
        ret(pop())
    )
}

fn gen_i32_trunc_u_f32(ctx: &mut Context) {
    let x = 0;
    let sl = 1;

    let posshift = ctx.gen.gen("i32_trunc_u_f32_posshift");
    let common = ctx.gen.gen("i32_trunc_u_f32_common");
    let retmax = ctx.gen.gen("i32_trunc_u_f32_retmax");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i32_trunc_sat_u_f32),
        fnhead_local(2),
        jisnan_ret(lloc(x), false),
        jfge(lloc(x), uimm(0x4f800000), retmax),
        jfle_ret(lloc(x), f32_to_imm(-1.), false),
        jump(common),
        label(retmax),
        ret(uimm(0xffffffff)),
        label(ctx.rt.i32_trunc_u_f32),
        fnhead_local(2),
        jisnan(lloc(x), ctx.rt.trap_invalid_conversion_to_integer),
        jfge(lloc(x), uimm(0x4f800000), ctx.rt.trap_integer_overflow),
        jfle(lloc(x), f32_to_imm(-1.), ctx.rt.trap_integer_overflow),
        label(common),
        jflt_ret(lloc(x), f32_to_imm(1.), false),
        floor(lloc(x), sloc(x)),
        bitand(lloc(x), uimm(0x7f800000), push()),
        ushiftr(pop(), imm(23), push()),
        sub(pop(), imm(127 + 23), sloc(sl)),
        jgt(lloc(sl), imm(0), posshift),
        sub(imm(0), lloc(sl), push()),
        bitand(lloc(x), uimm(0x7fffff), push()),
        bitor(pop(), uimm(0x800000), push()),
        ushiftr(pop(), pop(), push()),
        ret(pop()),
        label(posshift),
        bitand(lloc(x), uimm(0x7fffff), push()),
        bitor(pop(), uimm(0x800000), push()),
        shiftl(pop(), lloc(sl), push()),
        ret(pop())
    )
}

fn gen_i64_trunc_u_f32(ctx: &mut Context) {
    let x = 0;
    let sl = 1;

    let posshift = ctx.gen.gen("i64_trunc_u_f32_posshift");
    let retzero = ctx.gen.gen("i64_trunc_u_f32_retzero");
    let retmax = ctx.gen.gen("i32_trunc_u_f32_retmax");
    let common = ctx.gen.gen("i32_trunc_u_f32_common");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_trunc_sat_u_f32),
        fnhead_local(2),
        jisnan(lloc(x), retzero),
        jfge(lloc(x), uimm(0x5f800000), retmax),
        jfle(lloc(x), f32_to_imm(-1.), retzero),
        jump(common),
        label(retmax),
        copy(uimm(0xffffffff), storel(ctx.layout.hi_return().addr)),
        ret(uimm(0xffffffff)),
        label(ctx.rt.i64_trunc_u_f32),
        fnhead_local(2),
        jisnan(lloc(x), ctx.rt.trap_invalid_conversion_to_integer),
        jfge(lloc(x), uimm(0x5f800000), ctx.rt.trap_integer_overflow),
        jfle(lloc(x), f32_to_imm(-1.), ctx.rt.trap_integer_overflow),
        label(common),
        jflt(lloc(x), f32_to_imm(1.), retzero),
        floor(lloc(x), sloc(x)),
        bitand(lloc(x), uimm(0x7f800000), push()),
        ushiftr(pop(), imm(23), push()),
        sub(pop(), imm(127 + 23), sloc(sl)),
        jgt(lloc(sl), imm(0), posshift),
        sub(imm(0), lloc(sl), push()),
        bitand(lloc(x), uimm(0x7fffff), push()),
        bitor(pop(), uimm(0x800000), push()),
        ushiftr(pop(), pop(), push()),
        copy(imm(0), storel(ctx.layout.hi_return().addr)),
        ret(pop()),
        label(posshift),
        bitand(lloc(x), uimm(0x7fffff), push()),
        bitor(pop(), uimm(0x800000), push()),
        copy(imm(0), push()),
        copy(lloc(sl), push()),
        copy(imm(0), push()),
        tailcall(imml(ctx.rt.i64_shl), imm(4)),
        label(retzero),
        copy(imm(0), storel(ctx.layout.hi_return().addr)),
        ret(imm(0)),
    )
}

fn gen_i64_trunc_s_f32(ctx: &mut Context) {
    let x = 0;
    let sl = 1;
    let lo = 2;

    let posval_posshift = ctx.gen.gen("i64_trunc_s_f32_posval_posshift");
    let negval = ctx.gen.gen("i64_trunc_s_f32_negval");
    let negval_posshift = ctx.gen.gen("i64_trunc_s_f32_negval_posshift");
    let nocarry = ctx.gen.gen("i64_trunc_s_f32_nocarry");
    let retzero = ctx.gen.gen("i64_trunc_s_f32_retzero");
    let retmax = ctx.gen.gen("i64_trunc_s_f32_retmax");
    let retmin = ctx.gen.gen("i64_trunc_s_f32_retmin");
    let common = ctx.gen.gen("i64_trunc_s_f32_common");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_trunc_sat_s_f32),
        fnhead_local(3),
        jisnan(lloc(x), retzero),
        jflt(lloc(x), f32_to_imm(-9223372036854775808.), retmin),
        jfge(lloc(x), f32_to_imm(9223372036854775808.), retmax),
        jump(common),
        label(retmin),
        copy(imm(-0x80000000), storel(ctx.layout.hi_return().addr)),
        ret(uimm(0)),
        label(retmax),
        copy(imm(0x7fffffff), storel(ctx.layout.hi_return().addr)),
        ret(uimm(0xffffffff)),
        label(ctx.rt.i64_trunc_s_f32),
        fnhead_local(3),
        jisnan(lloc(x), ctx.rt.trap_invalid_conversion_to_integer),
        jisinf(lloc(x), ctx.rt.trap_integer_overflow),
        label(common),
        bitand(lloc(x), uimm(0x7f800000), push()),
        ushiftr(pop(), imm(23), push()),
        sub(pop(), imm(127 + 23), sloc(sl)),
        jflt(lloc(x), f32_to_imm(0.), negval),
        floor(lloc(x), sloc(x)),
        jfge(lloc(x), uimm(0x5f000000), ctx.rt.trap_integer_overflow),
        jgt(lloc(sl), imm(0), posval_posshift),
        sub(imm(0), lloc(sl), push()),
        bitand(lloc(x), uimm(0x7fffff), push()),
        bitor(pop(), uimm(0x800000), push()),
        ushiftr(pop(), pop(), push()),
        copy(imm(0), storel(ctx.layout.hi_return().addr)),
        ret(pop()),
        label(posval_posshift),
        bitand(lloc(x), uimm(0x7fffff), push()),
        bitor(pop(), uimm(0x800000), push()),
        copy(imm(0), push()),
        copy(lloc(sl), push()),
        copy(imm(0), push()),
        tailcall(imml(ctx.rt.i64_shl), imm(4)),
        label(negval),
        ceil(lloc(x), sloc(x)),
        jfeq(lloc(x), f32_to_imm(0.), f32_to_imm(0.), retzero),
        jflt(lloc(x), uimm(0xdf000000), ctx.rt.trap_integer_overflow),
        jgt(lloc(sl), imm(0), negval_posshift),
        sub(imm(0), lloc(sl), push()),
        bitand(lloc(x), uimm(0x7fffff), push()),
        bitor(pop(), uimm(0x800000), push()),
        ushiftr(pop(), pop(), push()),
        bitxor(pop(), imm(-1), push()),
        add(pop(), imm(1), push()),
        copy(imm(-1), storel(ctx.layout.hi_return().addr)),
        ret(pop()),
        label(negval_posshift),
        bitand(lloc(x), uimm(0x7fffff), push()),
        bitor(pop(), uimm(0x800000), push()),
        copy(imm(0), push()),
        copy(lloc(sl), push()),
        copy(imm(0), push()),
        call(imml(ctx.rt.i64_shl), imm(4), sloc(lo)),
        bitxor(lloc(lo), imm(-1), sloc(lo)),
        bitxor(
            derefl(ctx.layout.hi_return().addr),
            imm(-1),
            storel(ctx.layout.hi_return().addr)
        ),
        add(lloc(lo), imm(1), sloc(lo)),
        jnz(lloc(lo), nocarry),
        add(
            derefl(ctx.layout.hi_return().addr),
            imm(1),
            storel(ctx.layout.hi_return().addr)
        ),
        label(nocarry),
        ret(lloc(lo)),
        label(retzero),
        copy(imm(0), storel(ctx.layout.hi_return().addr)),
        ret(imm(0)),
    )
}

fn gen_f32_convert_i32_u(ctx: &mut Context) {
    let x = 0;
    let sd = 1;
    let e = 2;

    let mant_digits = 24;

    let sd_small = ctx.gen.gen("f32_convert_i32_u_sd_small");
    let sd_mant_plus_1 = ctx.gen.gen("f32_convert_i32_u_sd_mant_plus_1");
    let rounding_prepared = ctx.gen.gen("f32_convert_i32_u_rounding_prepared");
    let rounding_done = ctx.gen.gen("f32_convert_i32_u_rounding_done");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.f32_convert_i32_u),
        fnhead_local(3),
        jz_ret(lloc(x), false),
        callfi(imml(ctx.rt.i32_clz), lloc(x), push()),
        sub(imm(32), pop(), sloc(sd)),
        sub(lloc(sd), imm(1), sloc(e)),
        jleu(lloc(sd), imm(mant_digits), sd_small),
        jeq(lloc(sd), imm(mant_digits + 1), sd_mant_plus_1),
        jeq(lloc(sd), imm(mant_digits + 2), rounding_prepared),
        sub(imm(32 + mant_digits + 2), lloc(sd), push()),
        ushiftr(imm(-1), pop(), push()),
        bitand(lloc(x), pop(), push()),
        callfii(imml(ctx.rt.i32_ne), pop(), imm(0), push()),
        sub(lloc(sd), imm(mant_digits + 2), push()),
        ushiftr(lloc(x), pop(), push()),
        bitor(pop(), pop(), sloc(x)),
        jump(rounding_prepared),
        label(sd_mant_plus_1),
        shiftl(lloc(x), imm(1), sloc(x)),
        label(rounding_prepared),
        bitand(lloc(x), imm(4), push()),
        callfii(imml(ctx.rt.i32_ne), pop(), imm(0), push()),
        bitor(lloc(x), pop(), sloc(x)),
        add(lloc(x), imm(1), sloc(x)),
        ushiftr(lloc(x), imm(2), sloc(x)),
        bitand(lloc(x), uimm(1 << mant_digits), push()),
        jz(pop(), rounding_done),
        ushiftr(lloc(x), imm(1), sloc(x)),
        add(lloc(e), imm(1), sloc(e)),
        jump(rounding_done),
        label(sd_small),
        sub(imm(mant_digits), lloc(sd), push()),
        shiftl(lloc(x), pop(), sloc(x)),
        label(rounding_done),
        add(lloc(e), imm(127), push()),
        shiftl(pop(), imm(mant_digits - 1), push()),
        bitand(lloc(x), uimm(0x007fffff), push()),
        bitor(pop(), pop(), push()),
        ret(pop())
    )
}

fn gen_f32_convert_i64_u(ctx: &mut Context) {
    let x_lo = 1;
    let x_hi = 0;

    let sd = 2;
    let e = 3;

    let mant_digits = 24;

    let sd_small = ctx.gen.gen("f32_convert_i64_u_sd_small");
    let sd_mant_plus_1 = ctx.gen.gen("f32_convert_i64_u_sd_mant_plus_1");
    let rounding_prepared = ctx.gen.gen("f32_convert_i64_u_rounding_prepared");
    let rounding_done = ctx.gen.gen("f32_convert_i64_u_rounding_done");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.f32_convert_i64_u),
        fnhead_local(4),
        callfii(imml(ctx.rt.i64_eqz), lloc(x_hi), lloc(x_lo), push()),
        jnz_ret(pop(), false),
        callfii(imml(ctx.rt.i64_clz), lloc(x_hi), lloc(x_lo), push()),
        sub(imm(64), pop(), sloc(sd)),
        sub(lloc(sd), imm(1), sloc(e)),
        jleu(lloc(sd), imm(mant_digits), sd_small),
        jeq(lloc(sd), imm(mant_digits + 1), sd_mant_plus_1),
        jeq(lloc(sd), imm(mant_digits + 2), rounding_prepared),
        copy(imm(-1), push()),
        copy(imm(-1), push()),
        sub(imm(64 + mant_digits + 2), lloc(sd), push()),
        copy(imm(0), push()),
        call(imml(ctx.rt.i64_shr_u), imm(4), push()),
        copy(derefl(ctx.layout.hi_return().addr), push()),
        copy(lloc(x_lo), push()),
        copy(lloc(x_hi), push()),
        call(imml(ctx.rt.i64_and), imm(4), push()),
        copy(derefl(ctx.layout.hi_return().addr), push()),
        copy(imm(0), push()),
        copy(imm(0), push()),
        call(imml(ctx.rt.i64_ne), imm(4), push()),
        copy(imm(0), push()),
        copy(lloc(x_lo), push()),
        copy(lloc(x_hi), push()),
        sub(lloc(sd), imm(mant_digits + 2), push()),
        copy(imm(0), push()),
        call(imml(ctx.rt.i64_shr_u), imm(4), push()),
        copy(derefl(ctx.layout.hi_return().addr), push()),
        call(imml(ctx.rt.i64_or), imm(4), sloc(x_lo)),
        copy(derefl(ctx.layout.hi_return().addr), sloc(x_hi)),
        jump(rounding_prepared),
        label(sd_mant_plus_1),
        copy(lloc(x_lo), push()),
        copy(lloc(x_hi), push()),
        copy(imm(1), push()),
        copy(imm(0), push()),
        call(imml(ctx.rt.i64_shl), imm(4), sloc(x_lo)),
        copy(derefl(ctx.layout.hi_return().addr), sloc(x_hi)),
        label(rounding_prepared),
        bitand(lloc(x_lo), imm(4), push()),
        callfii(imml(ctx.rt.i32_ne), pop(), imm(0), push()),
        bitor(lloc(x_lo), pop(), sloc(x_lo)),
        copy(lloc(x_lo), push()),
        copy(lloc(x_hi), push()),
        copy(imm(1), push()),
        copy(imm(0), push()),
        call(imml(ctx.rt.i64_add), imm(4), sloc(x_lo)),
        copy(derefl(ctx.layout.hi_return().addr), sloc(x_hi)),
        copy(lloc(x_lo), push()),
        copy(lloc(x_hi), push()),
        copy(imm(2), push()),
        copy(imm(0), push()),
        call(imml(ctx.rt.i64_shr_u), imm(4), sloc(x_lo)),
        copy(derefl(ctx.layout.hi_return().addr), sloc(x_hi)),
        bitand(lloc(x_lo), uimm(1 << mant_digits), push()),
        jz(pop(), rounding_done),
        copy(lloc(x_lo), push()),
        copy(lloc(x_hi), push()),
        copy(imm(1), push()),
        copy(imm(0), push()),
        call(imml(ctx.rt.i64_shr_u), imm(4), sloc(x_lo)),
        copy(derefl(ctx.layout.hi_return().addr), sloc(x_hi)),
        add(lloc(e), imm(1), sloc(e)),
        jump(rounding_done),
        label(sd_small),
        sub(imm(mant_digits), lloc(sd), push()),
        shiftl(lloc(x_lo), pop(), sloc(x_lo)),
        label(rounding_done),
        add(lloc(e), imm(127), push()),
        shiftl(pop(), imm(mant_digits - 1), push()),
        bitand(lloc(x_lo), uimm(0x007fffff), push()),
        bitor(pop(), pop(), push()),
        ret(pop())
    )
}

fn gen_f32_convert_i64_s(ctx: &mut Context) {
    let x_lo = 1;
    let x_hi = 0;

    let neg = ctx.gen.gen("f32_convert_i64_s_neg");
    let nocarry = ctx.gen.gen("f32_convert_i64_s_nocarry");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.f32_convert_i64_s),
        fnhead_local(2),
        ushiftr(lloc(x_hi), imm(31), push()),
        jnz(pop(), neg),
        copy(lloc(x_lo), push()),
        copy(lloc(x_hi), push()),
        tailcall(imml(ctx.rt.f32_convert_i64_u), imm(2)),
        label(neg),
        bitxor(lloc(x_hi), imm(-1), sloc(x_hi)),
        bitxor(lloc(x_lo), imm(-1), sloc(x_lo)),
        add(lloc(x_lo), imm(1), sloc(x_lo)),
        jnz(lloc(x_lo), nocarry),
        add(lloc(x_hi), imm(1), sloc(x_hi)),
        label(nocarry),
        callfii(
            imml(ctx.rt.f32_convert_i64_u),
            lloc(x_hi),
            lloc(x_lo),
            push()
        ),
        bitor(pop(), uimm(0x80000000), push()),
        ret(pop())
    );
}

fn gen_f64_trunc(ctx: &mut Context) {
    let x_lo = 1;
    let x_hi = 0;

    let neg = ctx.gen.gen("f64_trunc_neg");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.f64_trunc),
        fnhead_local(2),
        // Intentionally an integer comparison
        jlt(lloc(x_hi), imm(0), neg),
        dfloor(
            lloc(x_hi),
            lloc(x_lo),
            push(),
            storel(ctx.layout.hi_return().addr)
        ),
        ret(pop()),
        label(neg),
        dceil(
            lloc(x_hi),
            lloc(x_lo),
            push(),
            storel(ctx.layout.hi_return().addr)
        ),
        ret(pop())
    );
}

fn gen_f64_nearest(ctx: &mut Context) {
    let x_lo = 1;
    let x_hi = 0;

    let x_ceil_hi = 2;
    let x_ceil_lo = 3;

    let x_floor_hi = 4;
    let x_floor_lo = 5;

    let ident = ctx.gen.gen("f64_nearest_ident");
    let nan = ctx.gen.gen("f64_nearest_nan");
    let neg = ctx.gen.gen("f64_nearest_neg");
    let lehalf = ctx.gen.gen("f64_nearest_lehalf");
    let geneghalf = ctx.gen.gen("f64_nearest_geneghalf");
    let choose_floor = ctx.gen.gen("f64_nearest_choose_floor");
    let choose_ceil = ctx.gen.gen("f64_nearest_choose_ceil");
    let main_case = ctx.gen.gen("f64_nearest_maincase");

    let (half_hi, half_lo) = f64_to_imm(0.5);
    let (neghalf_hi, neghalf_lo) = f64_to_imm(-0.5);
    let (zero_hi, zero_lo) = f64_to_imm(0.);
    let (negzero_hi, negzero_lo) = f64_to_imm(-0.);

    push_all!(
        ctx.rom_items,
        label(ctx.rt.f64_nearest),
        fnhead_local(6),
        jdisnan(lloc(x_hi), lloc(x_lo), nan),
        jdisinf(lloc(x_hi), lloc(x_lo), ident),
        jdeq(
            lloc(x_hi),
            lloc(x_lo),
            zero_hi,
            zero_lo,
            zero_hi,
            zero_lo,
            ident
        ),
        jdlt(lloc(x_hi), lloc(x_lo), zero_hi, zero_lo, neg),
        jdle(lloc(x_hi), lloc(x_lo), half_hi, half_lo, lehalf),
        jump(main_case),
        label(neg),
        jdge(lloc(x_hi), lloc(x_lo), neghalf_hi, neghalf_lo, geneghalf),
        label(main_case),
        dceil(lloc(x_hi), lloc(x_lo), sloc(x_ceil_lo), sloc(x_ceil_hi)),
        dfloor(lloc(x_hi), lloc(x_lo), sloc(x_floor_lo), sloc(x_floor_hi)),
        dsub(
            lloc(x_hi),
            lloc(x_lo),
            lloc(x_floor_hi),
            lloc(x_floor_lo),
            push(),
            push()
        ),
        jdlt(pop(), pop(), half_hi, half_lo, choose_floor),
        dsub(
            lloc(x_ceil_hi),
            lloc(x_ceil_lo),
            lloc(x_hi),
            lloc(x_lo),
            push(),
            push()
        ),
        jdlt(pop(), pop(), half_hi, half_lo, choose_ceil),
        callfii(
            imml(ctx.rt.i64_ctz),
            lloc(x_ceil_hi),
            lloc(x_ceil_lo),
            push()
        ),
        callfii(
            imml(ctx.rt.i64_ctz),
            lloc(x_floor_hi),
            lloc(x_floor_lo),
            push()
        ),
        jgtu(pop(), pop(), choose_floor),
        label(choose_ceil),
        copy(lloc(x_ceil_hi), storel(ctx.layout.hi_return().addr)),
        ret(lloc(x_ceil_lo)),
        label(nan),
        bitor(
            lloc(x_hi),
            uimm(0x00080000),
            storel(ctx.layout.hi_return().addr)
        ),
        ret(lloc(x_lo)),
        label(ident),
        copy(lloc(x_hi), storel(ctx.layout.hi_return().addr)),
        ret(lloc(x_lo)),
        label(lehalf),
        copy(zero_hi, storel(ctx.layout.hi_return().addr)),
        ret(zero_lo),
        label(geneghalf),
        copy(negzero_hi, storel(ctx.layout.hi_return().addr)),
        ret(negzero_lo),
        label(choose_floor),
        copy(lloc(x_floor_hi), storel(ctx.layout.hi_return().addr)),
        ret(lloc(x_floor_lo)),
    )
}

fn gen_f64_eq(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let y_lo = 1;
    let y_hi = 0;

    let (zero_hi, zero_lo) = f64_to_imm(0.);

    push_all!(
        ctx.rom_items,
        label(ctx.rt.f64_eq),
        fnhead_local(4),
        jdeq_ret(
            lloc(x_hi),
            lloc(x_lo),
            lloc(y_hi),
            lloc(y_lo),
            zero_hi,
            zero_lo,
            true
        ),
        ret(imm(0)),
    );
}

fn gen_f64_ne(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let y_lo = 1;
    let y_hi = 0;

    let (zero_hi, zero_lo) = f64_to_imm(0.);

    push_all!(
        ctx.rom_items,
        label(ctx.rt.f64_ne),
        fnhead_local(4),
        jdne_ret(
            lloc(x_hi),
            lloc(x_lo),
            lloc(y_hi),
            lloc(y_lo),
            zero_hi,
            zero_lo,
            true
        ),
        ret(imm(0)),
    );
}

fn gen_f64_lt(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let y_lo = 1;
    let y_hi = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.f64_lt),
        fnhead_local(4),
        jdlt_ret(lloc(x_hi), lloc(x_lo), lloc(y_hi), lloc(y_lo), true),
        ret(imm(0)),
    );
}

fn gen_f64_gt(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let y_lo = 1;
    let y_hi = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.f64_gt),
        fnhead_local(4),
        jdgt_ret(lloc(x_hi), lloc(x_lo), lloc(y_hi), lloc(y_lo), true),
        ret(imm(0)),
    );
}

fn gen_f64_le(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let y_lo = 1;
    let y_hi = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.f64_le),
        fnhead_local(4),
        jdle_ret(lloc(x_hi), lloc(x_lo), lloc(y_hi), lloc(y_lo), true),
        ret(imm(0)),
    );
}

fn gen_f64_ge(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let y_lo = 1;
    let y_hi = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.f64_ge),
        fnhead_local(4),
        jdge_ret(lloc(x_hi), lloc(x_lo), lloc(y_hi), lloc(y_lo), true),
        ret(imm(0)),
    );
}

fn gen_f64_min(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let y_lo = 1;
    let y_hi = 0;

    let x_nan = ctx.gen.gen("f64_min_x_nan");
    let y_nan = ctx.gen.gen("f64_min_y_nan");
    let choose_x = ctx.gen.gen("f64_min_choose_x");
    let choose_y = ctx.gen.gen("f64_min_choose_y");
    let x_neg_zero = ctx.gen.gen("f64_min_x_neg_zero");
    let x_not_neg_zero = ctx.gen.gen("f64_min_x_not_neg_zero");
    let y_neg_zero = ctx.gen.gen("f64_min_y_neg_zero");
    let main_case = ctx.gen.gen("f64_min_main_case");

    let (inf_hi, _) = f64_to_imm(f64::INFINITY);
    let (neginf_hi, _) = f64_to_imm(f64::NEG_INFINITY);
    let (zero_hi, zero_lo) = f64_to_imm(0.);
    let (negzero_hi, negzero_lo) = f64_to_imm(-0.);

    push_all!(
        ctx.rom_items,
        label(ctx.rt.f64_min),
        fnhead_local(4),
        jdisnan(lloc(x_hi), lloc(x_lo), x_nan),
        jdisnan(lloc(y_hi), lloc(y_lo), y_nan),
        // We've ruled out NaNs already, so checking the high word is sufficient
        // for infinities.
        jeq(lloc(x_hi), neginf_hi, choose_x),
        jeq(lloc(y_hi), neginf_hi, choose_y),
        jeq(lloc(x_hi), inf_hi, choose_y),
        jeq(lloc(y_hi), inf_hi, choose_x),
        jne(lloc(x_hi), negzero_hi, x_not_neg_zero),
        jeq(lloc(x_lo), negzero_lo, x_neg_zero),
        label(x_not_neg_zero),
        jne(lloc(y_hi), negzero_hi, main_case),
        jeq(lloc(y_lo), negzero_lo, y_neg_zero),
        label(main_case),
        jdlt(lloc(x_hi), lloc(x_lo), lloc(y_hi), lloc(y_lo), choose_x),
        label(choose_y),
        copy(lloc(y_hi), storel(ctx.layout.hi_return().addr)),
        ret(lloc(y_lo)),
        label(x_nan),
        bitor(
            lloc(x_hi),
            imm(0x00080000),
            storel(ctx.layout.hi_return().addr)
        ),
        ret(lloc(x_lo)),
        label(choose_x),
        copy(lloc(x_hi), storel(ctx.layout.hi_return().addr)),
        ret(lloc(x_lo)),
        label(y_nan),
        bitor(
            lloc(y_hi),
            imm(0x00080000),
            storel(ctx.layout.hi_return().addr)
        ),
        ret(lloc(y_lo)),
        label(x_neg_zero),
        jne(lloc(y_hi), zero_hi, main_case),
        jeq(lloc(y_lo), zero_lo, choose_x),
        jump(main_case),
        label(y_neg_zero),
        jne(lloc(x_hi), zero_hi, main_case),
        jeq(lloc(x_lo), zero_lo, choose_y),
        jump(main_case),
    )
}

fn gen_f64_max(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let y_lo = 1;
    let y_hi = 0;

    let x_nan = ctx.gen.gen("f64_max_x_nan");
    let y_nan = ctx.gen.gen("f64_max_y_nan");
    let choose_x = ctx.gen.gen("f64_max_choose_x");
    let choose_y = ctx.gen.gen("f64_max_choose_y");
    let x_neg_zero = ctx.gen.gen("f64_max_x_neg_zero");
    let x_not_neg_zero = ctx.gen.gen("f64_max_x_not_neg_zero");
    let y_neg_zero = ctx.gen.gen("f64_max_y_neg_zero");
    let main_case = ctx.gen.gen("f64_max_main_case");

    let (inf_hi, _) = f64_to_imm(f64::INFINITY);
    let (neginf_hi, _) = f64_to_imm(f64::NEG_INFINITY);
    let (zero_hi, zero_lo) = f64_to_imm(0.);
    let (negzero_hi, negzero_lo) = f64_to_imm(-0.);

    push_all!(
        ctx.rom_items,
        label(ctx.rt.f64_max),
        fnhead_local(4),
        jdisnan(lloc(x_hi), lloc(x_lo), x_nan),
        jdisnan(lloc(y_hi), lloc(y_lo), y_nan),
        // We've ruled out NaNs already, so checking the high word is sufficient
        // for infinities.
        jeq(lloc(x_hi), neginf_hi, choose_y),
        jeq(lloc(y_hi), neginf_hi, choose_x),
        jeq(lloc(x_hi), inf_hi, choose_x),
        jeq(lloc(y_hi), inf_hi, choose_y),
        jne(lloc(x_hi), negzero_hi, x_not_neg_zero),
        jeq(lloc(x_lo), negzero_lo, x_neg_zero),
        label(x_not_neg_zero),
        jne(lloc(y_hi), negzero_hi, main_case),
        jeq(lloc(y_lo), negzero_lo, y_neg_zero),
        label(main_case),
        jdgt(lloc(x_hi), lloc(x_lo), lloc(y_hi), lloc(y_lo), choose_x),
        label(choose_y),
        copy(lloc(y_hi), storel(ctx.layout.hi_return().addr)),
        ret(lloc(y_lo)),
        label(x_nan),
        bitor(
            lloc(x_hi),
            imm(0x00080000),
            storel(ctx.layout.hi_return().addr)
        ),
        ret(lloc(x_lo)),
        label(choose_x),
        copy(lloc(x_hi), storel(ctx.layout.hi_return().addr)),
        ret(lloc(x_lo)),
        label(y_nan),
        bitor(
            lloc(y_hi),
            imm(0x00080000),
            storel(ctx.layout.hi_return().addr)
        ),
        ret(lloc(y_lo)),
        label(x_neg_zero),
        jne(lloc(y_hi), zero_hi, main_case),
        jeq(lloc(y_lo), zero_lo, choose_y),
        jump(main_case),
        label(y_neg_zero),
        jne(lloc(x_hi), zero_hi, main_case),
        jeq(lloc(x_lo), zero_lo, choose_x),
        jump(main_case),
    )
}

fn gen_f64_copysign(ctx: &mut Context) {
    let x_lo = 3;
    let x_hi = 2;
    let _y_lo = 1;
    let y_hi = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.f64_copysign),
        fnhead_local(4),
        bitand(lloc(y_hi), uimm(0x80000000), push()),
        bitand(lloc(x_hi), uimm(0x7fffffff), push()),
        bitor(pop(), pop(), storel(ctx.layout.hi_return().addr)),
        ret(lloc(x_lo))
    )
}

fn gen_i32_trunc_s_f64(ctx: &mut Context) {
    let x_lo = 1;
    let x_hi = 0;

    let (maxbound_hi, maxbound_lo) = f64_to_imm(2147483648.);
    let (minbound_hi, minbound_lo) = f64_to_imm(-2147483649.);
    let retmin = ctx.gen.gen("i32_trunc_s_f64_retmin");
    let retmax = ctx.gen.gen("i32_trunc_s_f64_retmax");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i32_trunc_sat_s_f64),
        fnhead_local(2),
        jdisnan_ret(lloc(x_hi), lloc(x_lo), false,),
        jdge(lloc(x_hi), lloc(x_lo), maxbound_hi, maxbound_lo, retmax),
        jdle(lloc(x_hi), lloc(x_lo), minbound_hi, minbound_lo, retmin),
        dtonumz(lloc(x_hi), lloc(x_lo), push()),
        ret(pop()),
        label(retmin),
        ret(imm(-0x80000000)),
        label(retmax),
        ret(imm(0x7fffffff)),
        label(ctx.rt.i32_trunc_s_f64),
        fnhead_local(2),
        jdisnan(
            lloc(x_hi),
            lloc(x_lo),
            ctx.rt.trap_invalid_conversion_to_integer
        ),
        jdge(
            lloc(x_hi),
            lloc(x_lo),
            maxbound_hi,
            maxbound_lo,
            ctx.rt.trap_integer_overflow
        ),
        jdle(
            lloc(x_hi),
            lloc(x_lo),
            minbound_hi,
            minbound_lo,
            ctx.rt.trap_integer_overflow
        ),
        dtonumz(lloc(x_hi), lloc(x_lo), push()),
        ret(pop()),
    )
}

fn gen_i32_trunc_u_f64(ctx: &mut Context) {
    let x_lo = 1;
    let x_hi = 0;

    let sr = 2;

    let (maxbound_hi, maxbound_lo) = f64_to_imm(4294967296.);
    let (minbound_hi, minbound_lo) = f64_to_imm(-1.);
    let (one_hi, one_lo) = f64_to_imm(1.);

    let retmax = ctx.gen.gen("i32_trunc_u_f64_retmax");
    let common = ctx.gen.gen("i32_trunc_u_f64_common");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i32_trunc_sat_u_f64),
        fnhead_local(3),
        jdisnan_ret(lloc(x_hi), lloc(x_lo), false),
        jdge(lloc(x_hi), lloc(x_lo), maxbound_hi, maxbound_lo, retmax),
        jdle_ret(lloc(x_hi), lloc(x_lo), minbound_hi, minbound_lo, false),
        jump(common),
        label(retmax),
        ret(uimm(0xffffffff)),
        label(ctx.rt.i32_trunc_u_f64),
        fnhead_local(3),
        jdisnan(
            lloc(x_hi),
            lloc(x_lo),
            ctx.rt.trap_invalid_conversion_to_integer
        ),
        jdge(
            lloc(x_hi),
            lloc(x_lo),
            maxbound_hi,
            maxbound_lo,
            ctx.rt.trap_integer_overflow
        ),
        jdle(
            lloc(x_hi),
            lloc(x_lo),
            minbound_hi,
            minbound_lo,
            ctx.rt.trap_integer_overflow
        ),
        label(common),
        jdlt_ret(lloc(x_hi), lloc(x_lo), one_hi, one_lo, false),
        dfloor(lloc(x_hi), lloc(x_lo), sloc(x_lo), sloc(x_hi)),
        bitand(lloc(x_hi), uimm(0x7ff00000), push()),
        ushiftr(pop(), imm(52 - 32), push()),
        sub(imm(1023 + 52), pop(), sloc(sr)),
        copy(lloc(x_lo), push()),
        bitand(lloc(x_hi), uimm(0x000fffff), push()),
        bitor(pop(), uimm(0x00100000), push()),
        copy(lloc(sr), push()),
        copy(imm(0), push()),
        tailcall(imml(ctx.rt.i64_shr_u), imm(4)),
    )
}

fn gen_i64_trunc_u_f64(ctx: &mut Context) {
    let x_lo = 1;
    let x_hi = 0;

    let sl = 2;

    let retmax = ctx.gen.gen("i64_trunc_u_f64_retmax");
    let retzero = ctx.gen.gen("i64_trunc_u_f64_retzero");
    let posshift = ctx.gen.gen("i64_trunc_u_f64_posshift");
    let common = ctx.gen.gen("i64_trunc_u_f64_common");

    let (maxbound_hi, maxbound_lo) = f64_to_imm(18446744073709551616.);
    let (minbound_hi, minbound_lo) = f64_to_imm(-1.);
    let (one_hi, one_lo) = f64_to_imm(1.);

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_trunc_sat_u_f64),
        fnhead_local(3),
        jdisnan(lloc(x_hi), lloc(x_lo), retzero,),
        jdge(lloc(x_hi), lloc(x_lo), maxbound_hi, maxbound_lo, retmax,),
        jdle(lloc(x_hi), lloc(x_lo), minbound_hi, minbound_lo, retzero,),
        jump(common),
        label(retmax),
        copy(uimm(0xffffffff), storel(ctx.layout.hi_return().addr)),
        ret(uimm(0xffffffff)),
        label(ctx.rt.i64_trunc_u_f64),
        fnhead_local(3),
        jdisnan(
            lloc(x_hi),
            lloc(x_lo),
            ctx.rt.trap_invalid_conversion_to_integer
        ),
        jdge(
            lloc(x_hi),
            lloc(x_lo),
            maxbound_hi,
            maxbound_lo,
            ctx.rt.trap_integer_overflow
        ),
        jdle(
            lloc(x_hi),
            lloc(x_lo),
            minbound_hi,
            minbound_lo,
            ctx.rt.trap_integer_overflow
        ),
        label(common),
        jdlt(lloc(x_hi), lloc(x_lo), one_hi, one_lo, retzero),
        dfloor(lloc(x_hi), lloc(x_lo), sloc(x_lo), sloc(x_hi)),
        bitand(lloc(x_hi), uimm(0x7ff00000), push()),
        ushiftr(pop(), imm(52 - 32), push()),
        sub(pop(), imm(1023 + 52), sloc(sl)),
        copy(lloc(x_lo), push()),
        bitand(lloc(x_hi), uimm(0x000fffff), push()),
        bitor(pop(), uimm(0x00100000), push()),
        jgt(lloc(sl), imm(0), posshift),
        sub(imm(0), lloc(sl), push()),
        copy(imm(0), push()),
        tailcall(imml(ctx.rt.i64_shr_u), imm(4)),
        label(posshift),
        copy(lloc(sl), push()),
        copy(imm(0), push()),
        tailcall(imml(ctx.rt.i64_shl), imm(4)),
        label(retzero),
        copy(imm(0), storel(ctx.layout.hi_return().addr)),
        ret(imm(0))
    )
}

fn gen_i64_trunc_s_f64(ctx: &mut Context) {
    let x_lo = 1;
    let x_hi = 0;

    let sl = 2;

    let posval_posshift = ctx.gen.gen("i64_trunc_s_f64_posval_posshift");
    let negval = ctx.gen.gen("i64_trunc_s_f64_negval");
    let negval_posshift = ctx.gen.gen("i64_trunc_s_f64_negval_posshift");
    let negshift_nocarry = ctx.gen.gen("i64_trunc_s_f64_negshift_nocarry");
    let posshift_nocarry = ctx.gen.gen("i64_trunc_s_f64_posshift_nocarry");
    let retzero = ctx.gen.gen("i64_trunc_s_f64_retzero");
    let retmin = ctx.gen.gen("i64_trunc_s_f64_retmin");
    let retmax = ctx.gen.gen("i64_trunc_s_f64_retmax");
    let common = ctx.gen.gen("i64_trunc_s_f64_common");

    let (maxbound_hi, maxbound_lo) = f64_to_imm(9223372036854775808.);
    let (minbound_hi, minbound_lo) = f64_to_imm(-9223372036854775808.);
    let (zero_hi, zero_lo) = f64_to_imm(0.);

    push_all!(
        ctx.rom_items,
        label(ctx.rt.i64_trunc_sat_s_f64),
        fnhead_local(3),
        jdisnan(lloc(x_hi), lloc(x_lo), retzero,),
        jdge(lloc(x_hi), lloc(x_lo), maxbound_hi, maxbound_lo, retmax),
        jdlt(lloc(x_hi), lloc(x_lo), minbound_hi, minbound_lo, retmin),
        jump(common),
        label(retmin),
        copy(imm(-0x80000000), storel(ctx.layout.hi_return().addr)),
        ret(uimm(0)),
        label(retmax),
        copy(imm(0x7fffffff), storel(ctx.layout.hi_return().addr)),
        ret(uimm(0xffffffff)),
        label(ctx.rt.i64_trunc_s_f64),
        fnhead_local(3),
        jdisnan(
            lloc(x_hi),
            lloc(x_lo),
            ctx.rt.trap_invalid_conversion_to_integer
        ),
        jdisinf(lloc(x_hi), lloc(x_lo), ctx.rt.trap_integer_overflow),
        label(common),
        bitand(lloc(x_hi), uimm(0x7ff00000), push()),
        ushiftr(pop(), imm(52 - 32), push()),
        sub(pop(), imm(1023 + 52), sloc(sl)),
        jdlt(lloc(x_hi), lloc(x_lo), zero_hi, zero_lo, negval),
        dfloor(lloc(x_hi), lloc(x_lo), sloc(x_lo), sloc(x_hi)),
        jdeq(
            lloc(x_hi),
            lloc(x_lo),
            zero_hi,
            zero_lo,
            zero_hi,
            zero_lo,
            retzero
        ),
        jdge(
            lloc(x_hi),
            lloc(x_lo),
            maxbound_hi,
            maxbound_lo,
            ctx.rt.trap_integer_overflow
        ),
        copy(lloc(x_lo), push()),
        bitand(lloc(x_hi), uimm(0x000fffff), push()),
        bitor(pop(), uimm(0x00100000), push()),
        jgt(lloc(sl), imm(0), posval_posshift),
        sub(imm(0), lloc(sl), push()),
        copy(imm(0), push()),
        tailcall(imml(ctx.rt.i64_shr_u), imm(4)),
        label(posval_posshift),
        copy(lloc(sl), push()),
        copy(imm(0), push()),
        tailcall(imml(ctx.rt.i64_shl), imm(4)),
        label(negval),
        dceil(lloc(x_hi), lloc(x_lo), sloc(x_lo), sloc(x_hi)),
        jdeq(
            lloc(x_hi),
            lloc(x_lo),
            zero_hi,
            zero_lo,
            zero_hi,
            zero_lo,
            retzero
        ),
        jdlt(
            lloc(x_hi),
            lloc(x_lo),
            minbound_hi,
            minbound_lo,
            ctx.rt.trap_integer_overflow
        ),
        copy(lloc(x_lo), push()),
        bitand(lloc(x_hi), uimm(0x000fffff), push()),
        bitor(pop(), uimm(0x00100000), push()),
        jgt(lloc(sl), imm(0), negval_posshift),
        sub(imm(0), lloc(sl), push()),
        copy(imm(0), push()),
        call(imml(ctx.rt.i64_shr_u), imm(4), sloc(x_lo)),
        bitxor(
            derefl(ctx.layout.hi_return().addr),
            imm(-1),
            storel(ctx.layout.hi_return().addr)
        ),
        bitxor(lloc(x_lo), imm(-1), sloc(x_lo)),
        add(lloc(x_lo), imm(1), sloc(x_lo)),
        jnz(lloc(x_lo), negshift_nocarry),
        add(
            derefl(ctx.layout.hi_return().addr),
            imm(1),
            storel(ctx.layout.hi_return().addr)
        ),
        label(negshift_nocarry),
        ret(lloc(x_lo)),
        label(negval_posshift),
        copy(lloc(sl), push()),
        copy(imm(0), push()),
        call(imml(ctx.rt.i64_shl), imm(4), sloc(x_lo)),
        bitxor(
            derefl(ctx.layout.hi_return().addr),
            imm(-1),
            storel(ctx.layout.hi_return().addr)
        ),
        bitxor(lloc(x_lo), imm(-1), sloc(x_lo)),
        add(lloc(x_lo), imm(1), sloc(x_lo)),
        jnz(lloc(x_lo), posshift_nocarry),
        add(
            derefl(ctx.layout.hi_return().addr),
            imm(1),
            storel(ctx.layout.hi_return().addr)
        ),
        label(posshift_nocarry),
        ret(lloc(x_lo)),
        label(retzero),
        copy(imm(0), storel(ctx.layout.hi_return().addr)),
        ret(imm(0))
    )
}

fn gen_f64_convert_i32_u(ctx: &mut Context) {
    let x = 0;
    let lz = 1;

    let retzero = ctx.gen.gen("f64_convert_i32_u_retzero");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.f64_convert_i32_u),
        fnhead_local(2),
        jz(lloc(x), retzero),
        callfi(imml(ctx.rt.i32_clz), lloc(x), sloc(lz)),
        add(lloc(lz), imm(1), push()),
        ushiftr(imm(-1), pop(), push()),
        bitand(pop(), lloc(x), push()),
        copy(imm(0), push()),
        add(lloc(lz), imm(52 - 31), push()),
        copy(imm(0), push()),
        call(imml(ctx.rt.i64_shl), imm(4), sloc(x)),
        sub(imm(1023 + 31), lloc(lz), push()),
        shiftl(pop(), imm(52 - 32), push()),
        bitor(
            pop(),
            derefl(ctx.layout.hi_return().addr),
            storel(ctx.layout.hi_return().addr)
        ),
        ret(lloc(x)),
        label(retzero),
        copy(imm(0), storel(ctx.layout.hi_return().addr)),
        ret(imm(0))
    );
}

fn gen_f64_convert_i64_u(ctx: &mut Context) {
    let x_lo = 1;
    let x_hi = 0;

    let sd = 2;
    let e = 3;

    let mant_digits = 53;

    let sd_small = ctx.gen.gen("f64_convert_i64_u_sd_small");
    let sd_mant_plus_1 = ctx.gen.gen("f64_convert_i64_u_sd_mant_plus_1");
    let rounding_prepared = ctx.gen.gen("f64_convert_i64_u_rounding_prepared");
    let rounding_done = ctx.gen.gen("f64_convert_i64_u_rounding_done");
    let retzero = ctx.gen.gen("f64_convert_i64_u_retzero");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.f64_convert_i64_u),
        fnhead_local(4),
        callfii(imml(ctx.rt.i64_eqz), lloc(x_hi), lloc(x_lo), push()),
        jnz(pop(), retzero),
        callfii(imml(ctx.rt.i64_clz), lloc(x_hi), lloc(x_lo), push()),
        sub(imm(64), pop(), sloc(sd)),
        sub(lloc(sd), imm(1), sloc(e)),
        jleu(lloc(sd), imm(mant_digits), sd_small),
        jeq(lloc(sd), imm(mant_digits + 1), sd_mant_plus_1),
        jeq(lloc(sd), imm(mant_digits + 2), rounding_prepared),
        copy(imm(-1), push()),
        copy(imm(-1), push()),
        sub(imm(64 + mant_digits + 2), lloc(sd), push()),
        copy(imm(0), push()),
        call(imml(ctx.rt.i64_shr_u), imm(4), push()),
        copy(derefl(ctx.layout.hi_return().addr), push()),
        copy(lloc(x_lo), push()),
        copy(lloc(x_hi), push()),
        call(imml(ctx.rt.i64_and), imm(4), push()),
        copy(derefl(ctx.layout.hi_return().addr), push()),
        copy(imm(0), push()),
        copy(imm(0), push()),
        call(imml(ctx.rt.i64_ne), imm(4), push()),
        copy(imm(0), push()),
        copy(lloc(x_lo), push()),
        copy(lloc(x_hi), push()),
        sub(lloc(sd), imm(mant_digits + 2), push()),
        copy(imm(0), push()),
        call(imml(ctx.rt.i64_shr_u), imm(4), push()),
        copy(derefl(ctx.layout.hi_return().addr), push()),
        call(imml(ctx.rt.i64_or), imm(4), sloc(x_lo)),
        copy(derefl(ctx.layout.hi_return().addr), sloc(x_hi)),
        jump(rounding_prepared),
        label(sd_mant_plus_1),
        copy(lloc(x_lo), push()),
        copy(lloc(x_hi), push()),
        copy(imm(1), push()),
        copy(imm(0), push()),
        call(imml(ctx.rt.i64_shl), imm(4), sloc(x_lo)),
        copy(derefl(ctx.layout.hi_return().addr), sloc(x_hi)),
        label(rounding_prepared),
        bitand(lloc(x_lo), imm(4), push()),
        callfii(imml(ctx.rt.i32_ne), pop(), imm(0), push()),
        bitor(lloc(x_lo), pop(), sloc(x_lo)),
        copy(lloc(x_lo), push()),
        copy(lloc(x_hi), push()),
        copy(imm(1), push()),
        copy(imm(0), push()),
        call(imml(ctx.rt.i64_add), imm(4), sloc(x_lo)),
        copy(derefl(ctx.layout.hi_return().addr), sloc(x_hi)),
        copy(lloc(x_lo), push()),
        copy(lloc(x_hi), push()),
        copy(imm(2), push()),
        copy(imm(0), push()),
        call(imml(ctx.rt.i64_shr_u), imm(4), sloc(x_lo)),
        copy(derefl(ctx.layout.hi_return().addr), sloc(x_hi)),
        bitand(lloc(x_hi), uimm(1 << (mant_digits - 32)), push()),
        jz(pop(), rounding_done),
        copy(lloc(x_lo), push()),
        copy(lloc(x_hi), push()),
        copy(imm(1), push()),
        copy(imm(0), push()),
        call(imml(ctx.rt.i64_shr_u), imm(4), sloc(x_lo)),
        copy(derefl(ctx.layout.hi_return().addr), sloc(x_hi)),
        add(lloc(e), imm(1), sloc(e)),
        jump(rounding_done),
        label(sd_small),
        copy(lloc(x_lo), push()),
        copy(lloc(x_hi), push()),
        sub(imm(mant_digits), lloc(sd), push()),
        copy(imm(0), push()),
        call(imml(ctx.rt.i64_shl), imm(4), sloc(x_lo)),
        copy(derefl(ctx.layout.hi_return().addr), sloc(x_hi)),
        label(rounding_done),
        bitand(lloc(x_hi), uimm(0x000fffff), push()),
        add(lloc(e), imm(1023), push()),
        shiftl(pop(), imm(mant_digits - 32 - 1), push()),
        bitor(pop(), pop(), storel(ctx.layout.hi_return().addr)),
        ret(lloc(x_lo)),
        label(retzero),
        copy(imm(0), storel(ctx.layout.hi_return().addr)),
        ret(imm(0))
    )
}

fn gen_f64_convert_i64_s(ctx: &mut Context) {
    let x_lo = 1;
    let x_hi = 0;

    let neg = ctx.gen.gen("f64_convert_i64_s_neg");
    let nocarry = ctx.gen.gen("f64_convert_i64_s_nocarry");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.f64_convert_i64_s),
        fnhead_local(2),
        ushiftr(lloc(x_hi), imm(31), push()),
        jnz(pop(), neg),
        copy(lloc(x_lo), push()),
        copy(lloc(x_hi), push()),
        tailcall(imml(ctx.rt.f64_convert_i64_u), imm(2)),
        label(neg),
        bitxor(lloc(x_hi), imm(-1), sloc(x_hi)),
        bitxor(lloc(x_lo), imm(-1), sloc(x_lo)),
        add(lloc(x_lo), imm(1), sloc(x_lo)),
        jnz(lloc(x_lo), nocarry),
        add(lloc(x_hi), imm(1), sloc(x_hi)),
        label(nocarry),
        callfii(
            imml(ctx.rt.f64_convert_i64_u),
            lloc(x_hi),
            lloc(x_lo),
            push()
        ),
        bitor(
            derefl(ctx.layout.hi_return().addr),
            uimm(0x80000000),
            storel(ctx.layout.hi_return().addr)
        ),
        ret(pop())
    );
}

fn gen_trap(ctx: &mut Context) {
    push_all!(
        ctx.rom_items,
        label(ctx.rt.trap_unreachable),
        debugtrap(uimm(TrapCode::Unreachable.into())),
        quit(),
        label(ctx.rt.trap_integer_overflow),
        debugtrap(uimm(TrapCode::IntegerOverflow.into())),
        quit(),
        label(ctx.rt.trap_integer_divide_by_zero),
        debugtrap(uimm(TrapCode::IntegerDivideByZero.into())),
        quit(),
        label(ctx.rt.trap_invalid_conversion_to_integer),
        debugtrap(uimm(TrapCode::InvalidConversionToInteger.into())),
        quit(),
        label(ctx.rt.trap_out_of_bounds_memory_access),
        debugtrap(uimm(TrapCode::OutOfBoundsMemoryAccess.into())),
        quit(),
        label(ctx.rt.trap_indirect_call_type_mismatch),
        debugtrap(uimm(TrapCode::IndirectCallTypeMismatch.into())),
        quit(),
        label(ctx.rt.trap_out_of_bounds_table_access),
        debugtrap(uimm(TrapCode::OutOfBoundsTableAccess.into())),
        quit(),
        label(ctx.rt.trap_undefined_element),
        debugtrap(uimm(TrapCode::UndefinedElement.into())),
        quit(),
        label(ctx.rt.trap_uninitialized_element),
        debugtrap(uimm(TrapCode::UninitializedElement.into())),
        quit(),
        label(ctx.rt.trap_call_stack_exhausted),
        debugtrap(uimm(TrapCode::CallStackExhausted.into())),
        quit(),
    )
}

fn gen_table_init_or_copy(ctx: &mut Context) {
    let d_offset = 6;
    let s_offset = 5;
    let n = 4;
    let d_addr = 3;
    let d_size = 2;
    let s_addr = 1;
    let s_size = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.table_init_or_copy),
        fnhead_local(7),
        jgtu(
            lloc(s_offset),
            lloc(s_size),
            ctx.rt.trap_out_of_bounds_table_access
        ),
        sub(lloc(s_size), lloc(s_offset), push()),
        jgtu(lloc(n), pop(), ctx.rt.trap_out_of_bounds_table_access),
        jgtu(
            lloc(d_offset),
            lloc(d_size),
            ctx.rt.trap_out_of_bounds_table_access
        ),
        sub(lloc(d_size), lloc(d_offset), push()),
        jgtu(lloc(n), pop(), ctx.rt.trap_out_of_bounds_table_access),
        shiftl(lloc(d_offset), imm(2), push()),
        add(pop(), lloc(d_addr), push()),
        shiftl(lloc(s_offset), imm(2), push()),
        add(pop(), lloc(s_addr), push()),
        shiftl(lloc(n), imm(2), push()),
        mcopy(pop(), pop(), pop()),
        ret(imm(0)),
    )
}

fn gen_table_grow(ctx: &mut Context) {
    let n = 2;
    let cur_size = 1;
    let max_size = 0;

    let old_size = 3;
    let new_size = 4;

    let fail = ctx.gen.gen("table_grow_fail");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.table_grow),
        fnhead_local(5),
        aload(lloc(cur_size), imm(0), sloc(old_size)),
        add(lloc(n), lloc(old_size), sloc(new_size)),
        jltu(lloc(new_size), lloc(old_size), fail),
        jgtu(lloc(new_size), lloc(max_size), fail),
        astore(lloc(cur_size), imm(0), lloc(new_size)),
        ret(lloc(old_size)),
        label(fail),
        ret(imm(-1)),
    )
}

fn gen_table_fill(ctx: &mut Context) {
    let i = 4;
    let val = 3;
    let n = 2;
    let table_addr = 1;
    let table_count = 0;

    let loop_label = ctx.gen.gen("table_fill_loop");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.table_fill),
        fnhead_local(5),
        jgtu(
            lloc(n),
            lloc(table_count),
            ctx.rt.trap_out_of_bounds_table_access
        ),
        sub(lloc(table_count), lloc(n), push()),
        jgtu(lloc(i), pop(), ctx.rt.trap_out_of_bounds_table_access),
        shiftl(lloc(i), imm(2), push()),
        add(lloc(table_addr), pop(), sloc(table_addr)),
        copy(imm(0), sloc(i)),
        label(loop_label),
        jeq_ret(lloc(i), lloc(n), false),
        astore(lloc(table_addr), lloc(i), lloc(val)),
        add(lloc(i), imm(1), sloc(i)),
        jump(loop_label),
    );
}

fn gen_memory_init(ctx: &mut Context) {
    let mem_offset = 4;
    let data_offset = 3;
    let n = 2;
    let data_addr = 1;
    let data_size = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.memory_init),
        fnhead_local(5),
        jgtu(
            lloc(data_offset),
            lloc(data_size),
            ctx.rt.trap_out_of_bounds_memory_access
        ),
        sub(lloc(data_size), lloc(data_offset), push()),
        jgtu(lloc(n), pop(), ctx.rt.trap_out_of_bounds_memory_access),
        jgtu(
            lloc(mem_offset),
            derefl(ctx.layout.memory().cur_size),
            ctx.rt.trap_out_of_bounds_memory_access
        ),
        sub(
            derefl(ctx.layout.memory().cur_size),
            lloc(mem_offset),
            push()
        ),
        jgtu(lloc(n), pop(), ctx.rt.trap_out_of_bounds_memory_access),
        add(lloc(mem_offset), imml(ctx.layout.memory().addr), push()),
        add(lloc(data_offset), lloc(data_addr), push()),
        mcopy(lloc(n), pop(), pop()),
        ret(imm(0)),
    )
}

fn gen_memory_copy(ctx: &mut Context) {
    let d = 2;
    let s = 1;
    let n = 0;

    let d_plus_n = 3;
    let s_plus_n = 4;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.memory_copy),
        fnhead_local(5),
        add(lloc(s), lloc(n), sloc(s_plus_n)),
        add(lloc(d), lloc(n), sloc(d_plus_n)),
        jltu(
            lloc(s_plus_n),
            lloc(s),
            ctx.rt.trap_out_of_bounds_memory_access
        ),
        jltu(
            lloc(d_plus_n),
            lloc(d),
            ctx.rt.trap_out_of_bounds_memory_access
        ),
        jgtu(
            lloc(s_plus_n),
            derefl(ctx.layout.memory().cur_size),
            ctx.rt.trap_out_of_bounds_memory_access
        ),
        jgtu(
            lloc(d_plus_n),
            derefl(ctx.layout.memory().cur_size),
            ctx.rt.trap_out_of_bounds_memory_access
        ),
        add(imml(ctx.layout.memory().addr), lloc(d), push()),
        add(imml(ctx.layout.memory().addr), lloc(s), push()),
        mcopy(lloc(n), pop(), pop()),
        ret(imm(0))
    )
}

fn gen_memory_fill(ctx: &mut Context) {
    let d = 2;
    let val = 1;
    let n = 0;

    let d_plus_n = 3;

    let memzero = ctx.gen.gen("rt_memory_fill_zero");
    let loop_start = ctx.gen.gen("rt_memory_fill_loop_start");
    let loop_done = ctx.gen.gen("rt_memory_fill_loop_done");
    let halfword_done = ctx.gen.gen("rt_memory_fill_halfword_done");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.memory_fill),
        fnhead_local(4),
        add(lloc(d), lloc(n), sloc(d_plus_n)),
        jltu(
            lloc(d_plus_n),
            lloc(d),
            ctx.rt.trap_out_of_bounds_memory_access
        ),
        jgtu(
            lloc(d_plus_n),
            derefl(ctx.layout.memory().cur_size),
            ctx.rt.trap_out_of_bounds_memory_access
        ),
        jz(lloc(1), memzero),
        bitand(lloc(val), imm(0xff), sloc(val)),
        shiftl(lloc(val), imm(8), push()),
        bitor(lloc(val), pop(), sloc(val)),
        shiftl(lloc(val), imm(16), push()),
        bitor(lloc(val), pop(), sloc(val)),
        label(loop_start),
        jltu(lloc(n), uimm(4), loop_done),
        astore(
            lloc(d),
            imml_off_shift(ctx.layout.memory().addr, 0, 2),
            lloc(val)
        ),
        sub(lloc(n), uimm(4), sloc(n)),
        add(lloc(d), uimm(4), sloc(d)),
        jump(loop_start),
        label(loop_done),
        jltu(lloc(n), uimm(2), halfword_done),
        astores(
            lloc(d),
            imml_off_shift(ctx.layout.memory().addr, 0, 1),
            lloc(val)
        ),
        sub(lloc(n), uimm(2), sloc(n)),
        add(lloc(d), uimm(2), sloc(d)),
        label(halfword_done),
        jz_ret(lloc(n), false),
        astoreb(lloc(d), imml_off(ctx.layout.memory().addr, 0), lloc(val)),
        ret(imm(0)),
        label(memzero),
        add(imml(ctx.layout.memory().addr), lloc(d), push()),
        mzero(lloc(n), pop()),
        ret(imm(0)),
    )
}

pub fn gen_memory_grow(ctx: &mut Context) {
    let growth = 0;
    let fail = ctx.gen.gen("rt_memory_grow_fail");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.memory_grow),
        fnhead_local(1),
        jgtu(lloc(growth), uimm(65535), fail),
        shiftl(lloc(growth), imm(16), sloc(growth)),
        jgtu(lloc(growth), uimm(ctx.layout.memory().max_size), fail),
        sub(uimm(ctx.layout.memory().max_size), lloc(growth), push()),
        jltu(pop(), derefl(ctx.layout.memory().cur_size), fail),
        getmemsize(push()),
        add(lloc(growth), pop(), push()),
        setmemsize(pop(), push()),
        jnz(pop(), fail),
        copy(derefl(ctx.layout.memory().cur_size), push()),
        add(
            derefl(ctx.layout.memory().cur_size),
            lloc(growth),
            storel(ctx.layout.memory().cur_size)
        ),
        ushiftr(pop(), imm(16), push()),
        ret(pop()),
        label(fail),
        ret(imm(-1)),
    );
}

pub fn gen_rt(ctx: &mut Context) {
    gen_swap(ctx);
    gen_swaps(ctx);
    gen_checkaddr(ctx);
    gen_checkglkaddr(ctx);
    gen_checkstr(ctx);
    gen_checkunistr(ctx);
    gen_memload64(ctx);
    gen_memload32(ctx);
    gen_memload16(ctx);
    gen_memload8(ctx);
    gen_memstore64(ctx);
    gen_memstore32(ctx);
    gen_memstore16(ctx);
    gen_memstore8(ctx);
    gen_swaparray(ctx);
    gen_swapglkarray(ctx);
    gen_swapunistr(ctx);
    gen_i32_div_u(ctx);
    gen_i32_rem_u(ctx);
    gen_i32_shl(ctx);
    gen_i32_shr_s(ctx);
    gen_i32_shr_u(ctx);
    gen_i32_rotl(ctx);
    gen_i32_rotr(ctx);
    gen_i32_clz(ctx);
    gen_i32_ctz(ctx);
    gen_i32_popcnt(ctx);
    gen_i32_eqz(ctx);
    gen_i32_eq(ctx);
    gen_i32_ne(ctx);
    gen_i32_lt_s(ctx);
    gen_i32_lt_u(ctx);
    gen_i32_gt_s(ctx);
    gen_i32_gt_u(ctx);
    gen_i32_le_s(ctx);
    gen_i32_le_u(ctx);
    gen_i32_ge_s(ctx);
    gen_i32_ge_u(ctx);
    gen_i64_add(ctx);
    gen_i64_sub(ctx);
    gen_i64_mul(ctx);
    gen_i64_div_u(ctx);
    gen_i64_div_s(ctx);
    gen_i64_rem_u(ctx);
    gen_i64_rem_s(ctx);
    gen_i64_and(ctx);
    gen_i64_or(ctx);
    gen_i64_xor(ctx);
    gen_i64_shl(ctx);
    gen_i64_shr_s(ctx);
    gen_i64_shr_u(ctx);
    gen_i64_rotl(ctx);
    gen_i64_rotr(ctx);
    gen_i64_eqz(ctx);
    gen_i64_eq(ctx);
    gen_i64_ne(ctx);
    gen_i64_lt_s(ctx);
    gen_i64_lt_u(ctx);
    gen_i64_gt_s(ctx);
    gen_i64_gt_u(ctx);
    gen_i64_le_s(ctx);
    gen_i64_le_u(ctx);
    gen_i64_ge_s(ctx);
    gen_i64_ge_u(ctx);
    gen_i64_clz(ctx);
    gen_i64_ctz(ctx);
    gen_i64_popcnt(ctx);
    gen_f32_trunc(ctx);
    gen_f32_nearest(ctx);
    gen_f32_eq(ctx);
    gen_f32_ne(ctx);
    gen_f32_lt(ctx);
    gen_f32_gt(ctx);
    gen_f32_le(ctx);
    gen_f32_ge(ctx);
    gen_f32_min(ctx);
    gen_f32_max(ctx);
    gen_f32_copysign(ctx);
    gen_i32_trunc_s_f32(ctx);
    gen_i32_trunc_u_f32(ctx);
    gen_i64_trunc_s_f32(ctx);
    gen_i64_trunc_u_f32(ctx);
    gen_f32_convert_i32_u(ctx);
    gen_f32_convert_i64_u(ctx);
    gen_f32_convert_i64_s(ctx);
    gen_f64_trunc(ctx);
    gen_f64_nearest(ctx);
    gen_f64_eq(ctx);
    gen_f64_ne(ctx);
    gen_f64_lt(ctx);
    gen_f64_gt(ctx);
    gen_f64_le(ctx);
    gen_f64_ge(ctx);
    gen_f64_min(ctx);
    gen_f64_max(ctx);
    gen_f64_copysign(ctx);
    gen_i32_trunc_s_f64(ctx);
    gen_i32_trunc_u_f64(ctx);
    gen_i64_trunc_u_f64(ctx);
    gen_i64_trunc_s_f64(ctx);
    gen_f64_convert_i32_u(ctx);
    gen_f64_convert_i64_u(ctx);
    gen_f64_convert_i64_s(ctx);
    gen_trap(ctx);
    gen_table_init_or_copy(ctx);
    gen_table_grow(ctx);
    gen_table_fill(ctx);
    gen_memory_init(ctx);
    gen_memory_copy(ctx);
    gen_memory_fill(ctx);
    gen_memory_grow(ctx);
}
