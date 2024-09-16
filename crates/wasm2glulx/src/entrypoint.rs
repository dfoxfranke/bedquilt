use glulx_asm::concise::*;
use walrus::{ir::Value, ConstExpr, DataKind, ElementKind};

use crate::{
    common::{reject_global_constexpr, Context},
    CompilationError,
};

pub fn gen_entrypoint(ctx: &mut Context) {
    ctx.rom_items.push(label(ctx.layout.entrypoint()));
    ctx.rom_items.push(fnhead_local(0));
    ctx.rom_items.push(setiosys(imm(2), imm(0)));

    for element in ctx.module.elements.iter() {
        if let ElementKind::Active {
            table,
            offset: offset_expr,
        } = &element.kind
        {
            let elem_layout = ctx.layout.element(element.id());
            let table_layout = ctx.layout.table(*table);
            let table_offset = match offset_expr {
                ConstExpr::Value(Value::I32(offset)) => *offset,
                ConstExpr::Global(id) => {
                    reject_global_constexpr(ctx, *id);
                    continue;
                },
                _ => unreachable!("Table offset constexprs which are not i32 should have been rejected during module validation")
            };

            push_all!(
                ctx.rom_items,
                copy(derefl(elem_layout.cur_count), push()),
                copy(imm(0), push()),
                copy(imm(table_offset), push()),
                copy(imml(table_layout.addr), push()),
                copy(derefl(table_layout.cur_count), push()),
                copy(imml(elem_layout.addr), push()),
                copy(derefl(elem_layout.cur_count), push()),
                call(imml(ctx.rt.table_init), imm(7), discard()),
                copy(imm(0), storel(elem_layout.cur_count)),
            );
        }
    }

    for data in ctx.module.data.iter() {
        if let DataKind::Active {
            memory: _,
            offset: offset_expr,
        } = &data.kind
        {
            let data_layout = ctx.layout.data(data.id());
            let mem_offset = match offset_expr {
                ConstExpr::Value(Value::I32(offset)) => *offset,
                ConstExpr::Global(id) => {
                    reject_global_constexpr(ctx, *id);
                    continue;
                },
                _ => unreachable!("Data offset constexprs which are not i32 should have been rejected during module validation")
            };

            push_all!(
                ctx.rom_items,
                copy(derefl(data_layout.cur_size), push()),
                copy(imm(0), push()),
                copy(imm(mem_offset), push()),
                copy(imml(data_layout.addr), push()),
                copy(derefl(data_layout.cur_size), push()),
                call(imml(ctx.rt.data_init), imm(5), discard()),
                copy(imm(9), storel(data_layout.cur_size)),
            );
        }
    }

    if let Ok(interrupt_handler) = ctx.module.exports.get_func("glulx_interrupt_handler") {
        let ty = ctx
            .module
            .types
            .get(ctx.module.funcs.get(interrupt_handler).ty());
        if !ty.params().is_empty() || !ty.results().is_empty() {
            ctx.errors.push(CompilationError::IncorrectlyTypedExport {
                export: ctx
                    .module
                    .exports
                    .get_exported_func(interrupt_handler)
                    .unwrap()
                    .clone(),
                expected: (Vec::new(), Vec::new()),
                actual: (ty.params().to_owned(), ty.results().to_owned()),
            });
        }

        let addr = ctx.layout.func(interrupt_handler).addr;
        push_all!(
            ctx.rom_items,
            copy(imml(addr), push()),
            glk(
                imm(0x0002), /*glk_interrupt_handler*/
                imm(1),
                discard()
            ),
        );
    }

    match (
        ctx.module.start,
        ctx.module.exports.get_func("glulx_main").ok(),
    ) {
        (Some(start), Some(glulx_main)) if start != glulx_main => {
            let glulx_main_ty = ctx.module.types.get(ctx.module.funcs.get(glulx_main).ty());
            if !glulx_main_ty.params().is_empty() || !glulx_main_ty.results().is_empty() {
                ctx.errors.push(CompilationError::IncorrectlyTypedExport {
                    export: ctx
                        .module
                        .exports
                        .get_exported_func(glulx_main)
                        .unwrap()
                        .clone(),
                    expected: (Vec::new(), Vec::new()),
                    actual: (
                        glulx_main_ty.params().to_owned(),
                        glulx_main_ty.results().to_owned(),
                    ),
                });
            }

            let start_addr = ctx.layout.func(start).addr;
            let glulx_main_addr = ctx.layout.func(glulx_main).addr;
            push_all!(
                ctx.rom_items,
                call(imml(start_addr), imm(0), discard()),
                tailcall(imml(glulx_main_addr), imm(0)),
            );
        }
        (Some(start), _) => {
            let start_addr = ctx.layout.func(start).addr;
            ctx.rom_items.push(tailcall(imml(start_addr), imm(0)));
        }
        (None, Some(glulx_main)) => {
            let glulx_main_ty = ctx.module.types.get(ctx.module.funcs.get(glulx_main).ty());
            if !glulx_main_ty.params().is_empty() || !glulx_main_ty.results().is_empty() {
                ctx.errors.push(CompilationError::IncorrectlyTypedExport {
                    export: ctx
                        .module
                        .exports
                        .get_exported_func(glulx_main)
                        .unwrap()
                        .clone(),
                    expected: (Vec::new(), Vec::new()),
                    actual: (
                        glulx_main_ty.params().to_owned(),
                        glulx_main_ty.results().to_owned(),
                    ),
                });
            }
            let glulx_main_addr = ctx.layout.func(glulx_main).addr;
            ctx.rom_items.push(tailcall(imml(glulx_main_addr), imm(0)));
        }
        (None, None) => {
            ctx.errors.push(CompilationError::NoEntrypoint);
        }
    }
}
