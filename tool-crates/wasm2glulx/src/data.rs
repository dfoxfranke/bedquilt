// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
// Copyright 2024 Daniel Fox Franke.

use anyhow::anyhow;
use bytes::{BufMut, BytesMut};
use glulx_asm::concise::*;
use walrus::{ir::Value, ConstExpr, ElementKind, GlobalKind};

use crate::{
    common::{reject_global_constexpr, Context, Label, TrapCode},
    CompilationError, OverflowLocation,
};

pub fn gen_tables(ctx: &mut Context) {
    for table in ctx.module.tables.iter() {
        if let Some(id) = table.import {
            ctx.errors.push(CompilationError::UnrecognizedImport(
                ctx.module.imports.get(id).clone(),
            ));
            continue;
        }

        let table_layout = ctx.layout.table(table.id());

        let size = if let Some(size) = table_layout.max_count.checked_mul(4) {
            size
        } else {
            ctx.errors
                .push(CompilationError::Overflow(OverflowLocation::Table));
            continue;
        };

        ctx.zero_items.push(zlabel(table_layout.addr));
        ctx.zero_items.push(zspace(size));
        if table_layout.min_count == 0 {
            ctx.zero_items.push(zlabel(table_layout.cur_count));
            ctx.zero_items.push(zspace(4));
        } else {
            let mut bytes = BytesMut::new();
            bytes.put_u32(table_layout.min_count);
            ctx.ram_items.push(label(table_layout.cur_count));
            ctx.ram_items.push(blob(bytes));
        }
    }
}

pub fn gen_globals(ctx: &mut Context) {
    for global in ctx.module.globals.iter() {
        let mut bytes = bytes::BytesMut::new();
        let mut is_zero = true;
        let global_label = ctx.layout.global(global.id()).addr;

        match &global.kind {
            GlobalKind::Import(id) => {
                ctx.errors.push(CompilationError::UnrecognizedImport(
                    ctx.module.imports.get(*id).clone(),
                ));
            }
            GlobalKind::Local(ConstExpr::Value(Value::I32(x))) => {
                bytes.put_i32(*x);
                is_zero &= *x == 0;
            }
            GlobalKind::Local(ConstExpr::Value(Value::I64(x))) => {
                bytes.put_i64(*x);
                is_zero &= *x == 0;
            }
            GlobalKind::Local(ConstExpr::Value(Value::F32(x))) => {
                bytes.put_f32(*x);
                is_zero &= x.to_bits() == 0;
            }
            GlobalKind::Local(ConstExpr::Value(Value::F64(x))) => {
                bytes.put_f64(*x);
                is_zero &= x.to_bits() == 0;
            }
            GlobalKind::Local(ConstExpr::Value(Value::V128(x))) => {
                bytes.put_u128(*x);
                is_zero &= *x == 0;
            }
            GlobalKind::Local(ConstExpr::Global(_)) => {
                ctx.errors.push(CompilationError::ValidationError(anyhow!(
                    "Globals which take their initial value from other globals are not supported."
                )));
            }
            GlobalKind::Local(ConstExpr::RefNull(_)) => {
                bytes.put_u32(0);
            }
            GlobalKind::Local(ConstExpr::RefFunc(f)) => {
                if global.mutable {
                    ctx.ram_items.push(label(global_label));
                    ctx.ram_items.push(labelref(ctx.layout.func(*f).addr));
                } else {
                    ctx.rom_items.push(label(global_label));
                    ctx.rom_items.push(labelref(ctx.layout.func(*f).addr));
                }
                continue;
            }
        }

        if is_zero {
            ctx.zero_items.push(zlabel(global_label));
            ctx.zero_items.push(zspace(
                bytes
                    .len()
                    .try_into()
                    .expect("Length of a global should always fit in a u32"),
            ));
        } else if global.mutable {
            ctx.ram_items.push(label(global_label));
            ctx.ram_items.push(blob(bytes));
        } else {
            ctx.rom_items.push(label(global_label));
            ctx.rom_items.push(blob(bytes));
        }
    }
}

pub fn gen_elems(ctx: &mut Context) {
    for elem in ctx.module.elements.iter() {
        if matches!(elem.kind, ElementKind::Declared) {
            continue;
        }

        let layout = ctx.layout.element(elem.id());
        ctx.rom_items.push(label(layout.addr));

        match &elem.items {
            walrus::ElementItems::Functions(v) => {
                for id in v {
                    ctx.rom_items.push(labelref(ctx.layout.func(*id).addr));
                }
            }

            walrus::ElementItems::Expressions(_, v) => {
                for expr in v {
                    match expr {
                        ConstExpr::Value(_) => unreachable!("Non-reference types in an element segment should have been caught during module validation"),
                        ConstExpr::Global(id) => {
                            reject_global_constexpr(ctx, *id);
                        }
                        ConstExpr::RefNull(_) => {
                            ctx.rom_items.push(blob([0u8; 4].as_slice()));
                        }
                        ConstExpr::RefFunc(id) => {
                            ctx.rom_items.push(labelref(ctx.layout.func(*id).addr));
                        }
                    }
                }
            }
        }

        ctx.ram_items.push(label(layout.cur_count));
        ctx.ram_items
            .push(blob(Vec::from(layout.initial_count.to_be_bytes())));
    }
}

pub fn gen_datas(ctx: &mut Context) {
    for data in ctx.module.data.iter() {
        let layout = ctx.layout.data(data.id());
        ctx.rom_items.push(label(layout.addr));
        ctx.rom_items.push(blob(data.value.clone()));
        ctx.ram_items.push(label(layout.cur_size));
        ctx.ram_items
            .push(blob(Vec::from(layout.initial_size.to_be_bytes())));
    }
}

pub fn gen_trap(ctx: &mut Context) {
    let table: Vec<(Label, TrapCode)> = TrapCode::ALL
        .iter()
        .map(|code| (ctx.gen.gen("trap_string"), *code))
        .collect();

    ctx.rom_items.push(label(ctx.layout.trap().string_table));
    for (l, _) in &table {
        ctx.rom_items.push(labelref(*l));
    }

    for (l, code) in &table {
        ctx.rom_items.push(label(*l));
        ctx.rom_items.push(mystery_string(&code.as_str()));
    }
}

pub fn gen_hi_return(ctx: &mut Context) {
    ctx.zero_items.push(zlabel(ctx.layout.hi_return().addr));
    ctx.zero_items.push(zspace(ctx.layout.hi_return().size));
}

pub fn gen_glk_area(ctx: &mut Context) {
    ctx.zero_items.push(zalign(4));
    ctx.zero_items.push(zlabel(ctx.layout.glk_area().addr));
    ctx.zero_items.push(zspace(ctx.layout.glk_area().size));
}

pub fn gen_memory(ctx: &mut Context) {
    let mut bytes = BytesMut::with_capacity(4);
    let mem = ctx.layout.memory();
    bytes.put_u32(mem.min_size);

    ctx.ram_items.push(label(mem.cur_size));
    ctx.ram_items.push(blob(bytes));
    // Add four bytes of padding before the start of memory, so that the
    // prefix-patches that we write when calling Glk string functions don't
    // overwrite anything that isn't memory.
    ctx.zero_items.push(zspace(4));
    ctx.zero_items.push(zalign(4));
    ctx.zero_items.push(zlabel(mem.addr));
    ctx.zero_items.push(zspace(mem.min_size));
}

pub fn gen_data(ctx: &mut Context) {
    gen_trap(ctx);
    gen_tables(ctx);
    gen_globals(ctx);
    gen_elems(ctx);
    gen_datas(ctx);
    gen_hi_return(ctx);
    gen_glk_area(ctx);
    gen_memory(ctx);
}
