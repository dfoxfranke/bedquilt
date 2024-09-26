// SPDX-License-Identifier: Apache-2.0 WITH LLVM-Exception
// Copyright 2024 Daniel Fox Franke.

use glulx_asm::concise::*;
use walrus::{ImportedFunction, ValType};

use crate::common::{Context, Label, WordCount};

fn check_intrinsic_type(ctx: &mut Context, imported_func: &ImportedFunction) -> bool {
    let import = ctx.module.imports.get(imported_func.import);
    let name = &import.name;
    let ty = ctx.module.types.get(imported_func.ty);

    #[cfg(feature = "spectest")]
    if name == "spectest_result" {
        if !ty.results().is_empty() {
            ctx.errors
                .push(crate::CompilationError::IncorrectlyTypedImport {
                    import: import.clone(),
                    expected: (ty.params().to_owned(), Vec::new()),
                    actual: (ty.params().to_owned(), ty.results().to_owned()),
                });
            return false;
        } else {
            return true;
        }
    }

    let (expected_params, expected_results): (&[ValType], &[ValType]) = match name.as_str() {
        "restart" | "discardundo" => (&[], &[]),
        "random" | "glkarea_get_byte" | "glkarea_get_word" | "save" | "restore" => {
            (&[ValType::I32], &[ValType::I32])
        }
        "setrandom" | "glkarea_put_byte" | "glkarea_put_word" | "saveundo" | "restoreundo"
        | "hasundo" => (&[ValType::I32], &[]),
        "protect" => (&[ValType::I32, ValType::I32], &[]),
        "gesalt" => (&[ValType::I32, ValType::I32], &[ValType::I32]),
        "glkarea_get_bytes" | "glkarea_put_bytes" | "glkarea_get_words" | "glkarea_put_words" => {
            (&[ValType::I32, ValType::I32, ValType::I32], &[])
        }
        "expf" | "logf" | "sinf" | "cosf" | "tanf" | "asinf" | "acosf" | "atanf" => {
            (&[ValType::F32], &[ValType::F32])
        }
        "fmodf" | "powf" | "atan2f" => (&[ValType::F32, ValType::F32], &[ValType::F32]),
        "exp" | "log" | "sin" | "cos" | "tan" | "asin" | "acos" | "atan" => {
            (&[ValType::F64], &[ValType::F64])
        }
        "fmod" | "pow" | "atan2" => (&[ValType::F64, ValType::F64], &[ValType::F64]),
        _ => {
            ctx.errors.push(crate::CompilationError::UnrecognizedImport(
                ctx.module.imports.get(imported_func.import).clone(),
            ));
            return false;
        }
    };

    if ty.params() == expected_params && ty.results() == expected_results {
        true
    } else {
        ctx.errors
            .push(crate::CompilationError::IncorrectlyTypedImport {
                import: import.clone(),
                expected: (expected_params.to_owned(), expected_results.to_owned()),
                actual: (ty.params().to_owned(), ty.results().to_owned()),
            });
        false
    }
}

#[cfg(feature = "spectest")]
fn gen_spectest_result(ctx: &mut Context, imported_func: &ImportedFunction, my_label: Label) {
    let ty = ctx.module.types.get(imported_func.ty);
    let mut param_word: u32 = ty.params().word_count();

    ctx.rom_items.push(label(my_label));
    ctx.rom_items.push(fnhead_local(param_word));
    for param in ty.params() {
        match param {
            ValType::I32 | ValType::F32 | ValType::Ref(_) => {
                param_word -= 1;
                ctx.rom_items.push(streamnum(lloc(param_word)));
            }
            ValType::I64 | ValType::F64 => {
                param_word -= 2;
                ctx.rom_items.push(streamnum(lloc(param_word)));
                ctx.rom_items.push(streamnum(lloc(param_word + 1)));
            }
            ValType::V128 => {
                param_word -= 4;
                ctx.rom_items.push(streamnum(lloc(param_word)));
                ctx.rom_items.push(streamnum(lloc(param_word + 1)));
                ctx.rom_items.push(streamnum(lloc(param_word + 2)));
                ctx.rom_items.push(streamnum(lloc(param_word + 3)));
            }
        }
    }
    ctx.rom_items.push(ret(imm(0)));
}

fn gen_glkarea_get_byte(ctx: &mut Context, my_label: Label) {
    let addr = 0;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(1),
        callfii(imml(ctx.rt.checkglkaddr), lloc(addr), imm(1), discard()),
        aloadb(imml(ctx.layout.glk_area().addr), lloc(addr), push()),
        ret(pop())
    );
}

fn gen_glkarea_put_byte(ctx: &mut Context, my_label: Label) {
    let addr = 1;
    let byte = 0;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(2),
        callfii(imml(ctx.rt.checkglkaddr), lloc(addr), imm(1), discard()),
        astoreb(imml(ctx.layout.glk_area().addr), lloc(addr), lloc(byte)),
        ret(imm(0))
    );
}

fn gen_glkarea_get_word(ctx: &mut Context, my_label: Label) {
    let glkaddr = 0;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(1),
        callfii(imml(ctx.rt.checkglkaddr), lloc(glkaddr), imm(4), discard()),
        aload(
            lloc(glkaddr),
            imml_off_shift(ctx.layout.glk_area().addr, 0, 2),
            push()
        ),
        ret(pop()),
    );
}

fn gen_glkarea_put_word(ctx: &mut Context, my_label: Label) {
    let glkaddr = 1;
    let word = 0;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(2),
        callfii(imml(ctx.rt.checkglkaddr), lloc(glkaddr), imm(4), discard()),
        astore(
            lloc(glkaddr),
            imml_off_shift(ctx.layout.glk_area().addr, 0, 2),
            lloc(word)
        ),
    );
}

fn gen_glkarea_get_bytes(ctx: &mut Context, my_label: Label) {
    let addr = 2;
    let glkaddr = 1;
    let n = 0;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(3),
        callfiii(
            imml(ctx.rt.checkaddr),
            lloc(addr),
            imm(0),
            lloc(n),
            discard()
        ),
        callfii(imml(ctx.rt.checkglkaddr), lloc(glkaddr), lloc(n), discard()),
        add(lloc(addr), imml(ctx.layout.memory().addr), push()),
        add(lloc(glkaddr), imml(ctx.layout.glk_area().addr), push()),
        mcopy(lloc(n), pop(), pop()),
        ret(imm(0))
    )
}

fn gen_glkarea_put_bytes(ctx: &mut Context, my_label: Label) {
    let glkaddr = 2;
    let addr = 1;
    let n = 0;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(3),
        callfiii(
            imml(ctx.rt.checkaddr),
            lloc(addr),
            imm(0),
            lloc(n),
            discard()
        ),
        callfii(imml(ctx.rt.checkglkaddr), lloc(glkaddr), lloc(n), discard()),
        add(lloc(glkaddr), imml(ctx.layout.glk_area().addr), push()),
        add(lloc(addr), imml(ctx.layout.memory().addr), push()),
        mcopy(lloc(n), pop(), pop()),
        ret(imm(0))
    )
}

fn gen_glkarea_get_words(ctx: &mut Context, my_label: Label) {
    let addr = 2;
    let glkaddr = 1;
    let n = 0;

    let size = 3;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(4),
        jgtu(
            lloc(n),
            uimm(0x3fffffff),
            ctx.rt.trap_out_of_bounds_memory_access
        ),
        shiftl(lloc(n), imm(2), sloc(size)),
        callfiii(
            imml(ctx.rt.checkaddr),
            lloc(addr),
            imm(0),
            lloc(size),
            discard()
        ),
        callfii(
            imml(ctx.rt.checkglkaddr),
            lloc(glkaddr),
            lloc(size),
            discard()
        ),
        add(lloc(addr), imml(ctx.layout.memory().addr), push()),
        add(lloc(glkaddr), imml(ctx.layout.glk_area().addr), push()),
        mcopy(lloc(size), pop(), pop()),
        callfii(imml(ctx.rt.swaparray), lloc(addr), lloc(n), discard()),
        ret(imm(0))
    )
}

fn gen_glkarea_put_words(ctx: &mut Context, my_label: Label) {
    let glkaddr = 2;
    let addr = 1;
    let n = 0;

    let size = 3;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(4),
        jgtu(
            lloc(n),
            uimm(0x3fffffff),
            ctx.rt.trap_out_of_bounds_memory_access
        ),
        shiftl(lloc(n), imm(2), sloc(size)),
        callfiii(
            imml(ctx.rt.checkaddr),
            lloc(addr),
            imm(0),
            lloc(size),
            discard()
        ),
        callfii(
            imml(ctx.rt.checkglkaddr),
            lloc(glkaddr),
            lloc(size),
            discard()
        ),
        add(lloc(glkaddr), imml(ctx.layout.glk_area().addr), push()),
        add(lloc(addr), imml(ctx.layout.memory().addr), push()),
        mcopy(lloc(size), pop(), pop()),
        callfii(imml(ctx.rt.swapglkarray), lloc(glkaddr), lloc(n), discard()),
        ret(imm(0))
    )
}

pub fn gen_random(ctx: &mut Context, my_label: Label) {
    let arg = 0;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(1),
        random(lloc(arg), push()),
        ret(pop())
    )
}

pub fn gen_setrandom(ctx: &mut Context, my_label: Label) {
    let arg = 0;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(1),
        setrandom(lloc(arg)),
        ret(imm(0))
    )
}

pub fn gen_fmodf(ctx: &mut Context, my_label: Label) {
    let x = 1;
    let y = 0;
    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(2),
        fmod(lloc(x), lloc(y), push(), discard()),
        ret(pop())
    );
}

pub fn gen_expf(ctx: &mut Context, my_label: Label) {
    let x = 0;
    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(1),
        exp(lloc(x), push()),
        ret(pop())
    );
}

pub fn gen_logf(ctx: &mut Context, my_label: Label) {
    let x = 0;
    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(1),
        log(lloc(x), push()),
        ret(pop())
    );
}

pub fn gen_powf(ctx: &mut Context, my_label: Label) {
    let x = 1;
    let y = 0;
    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(2),
        pow(lloc(x), lloc(y), push()),
        ret(pop())
    );
}

pub fn gen_sinf(ctx: &mut Context, my_label: Label) {
    let x = 0;
    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(1),
        sin(lloc(x), push()),
        ret(pop())
    );
}

pub fn gen_cosf(ctx: &mut Context, my_label: Label) {
    let x = 0;
    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(1),
        cos(lloc(x), push()),
        ret(pop())
    );
}

pub fn gen_tanf(ctx: &mut Context, my_label: Label) {
    let x = 0;
    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(1),
        tan(lloc(x), push()),
        ret(pop())
    );
}

pub fn gen_asinf(ctx: &mut Context, my_label: Label) {
    let x = 0;
    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(1),
        asin(lloc(x), push()),
        ret(pop())
    );
}

pub fn gen_acosf(ctx: &mut Context, my_label: Label) {
    let x = 0;
    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(1),
        acos(lloc(x), push()),
        ret(pop())
    );
}

pub fn gen_atanf(ctx: &mut Context, my_label: Label) {
    let x = 0;
    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(1),
        atan(lloc(x), push()),
        ret(pop())
    );
}

pub fn gen_atan2f(ctx: &mut Context, my_label: Label) {
    let x = 1;
    let y = 0;
    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(2),
        pow(lloc(x), lloc(y), push()),
        ret(pop())
    );
}

pub fn gen_fmod(ctx: &mut Context, my_label: Label) {
    let x_lo = 3;
    let x_hi = 2;
    let y_lo = 1;
    let y_hi = 0;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(4),
        dmodr(
            lloc(x_hi),
            lloc(x_lo),
            lloc(y_hi),
            lloc(y_lo),
            push(),
            storel(ctx.layout.hi_return().addr)
        ),
        ret(pop())
    )
}

pub fn gen_exp(ctx: &mut Context, my_label: Label) {
    let x_lo = 1;
    let x_hi = 0;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(2),
        dexp(
            lloc(x_hi),
            lloc(x_lo),
            push(),
            storel(ctx.layout.hi_return().addr)
        ),
        ret(pop())
    );
}

pub fn gen_log(ctx: &mut Context, my_label: Label) {
    let x_lo = 1;
    let x_hi = 0;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(2),
        dlog(
            lloc(x_hi),
            lloc(x_lo),
            push(),
            storel(ctx.layout.hi_return().addr)
        ),
        ret(pop())
    );
}

pub fn gen_pow(ctx: &mut Context, my_label: Label) {
    let x_lo = 3;
    let x_hi = 2;
    let y_lo = 1;
    let y_hi = 0;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(4),
        dpow(
            lloc(x_hi),
            lloc(x_lo),
            lloc(y_hi),
            lloc(y_lo),
            push(),
            storel(ctx.layout.hi_return().addr)
        ),
        ret(pop())
    )
}

pub fn gen_sin(ctx: &mut Context, my_label: Label) {
    let x_lo = 1;
    let x_hi = 0;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(2),
        dsin(
            lloc(x_hi),
            lloc(x_lo),
            push(),
            storel(ctx.layout.hi_return().addr)
        ),
        ret(pop())
    );
}

pub fn gen_cos(ctx: &mut Context, my_label: Label) {
    let x_lo = 1;
    let x_hi = 0;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(2),
        dcos(
            lloc(x_hi),
            lloc(x_lo),
            push(),
            storel(ctx.layout.hi_return().addr)
        ),
        ret(pop())
    );
}

pub fn gen_tan(ctx: &mut Context, my_label: Label) {
    let x_lo = 1;
    let x_hi = 0;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(2),
        dtan(
            lloc(x_hi),
            lloc(x_lo),
            push(),
            storel(ctx.layout.hi_return().addr)
        ),
        ret(pop())
    );
}

pub fn gen_asin(ctx: &mut Context, my_label: Label) {
    let x_lo = 1;
    let x_hi = 0;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(2),
        dasin(
            lloc(x_hi),
            lloc(x_lo),
            push(),
            storel(ctx.layout.hi_return().addr)
        ),
        ret(pop())
    );
}

pub fn gen_acos(ctx: &mut Context, my_label: Label) {
    let x_lo = 1;
    let x_hi = 0;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(2),
        dacos(
            lloc(x_hi),
            lloc(x_lo),
            push(),
            storel(ctx.layout.hi_return().addr)
        ),
        ret(pop())
    );
}

pub fn gen_atan(ctx: &mut Context, my_label: Label) {
    let x_lo = 1;
    let x_hi = 0;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(2),
        datan(
            lloc(x_hi),
            lloc(x_lo),
            push(),
            storel(ctx.layout.hi_return().addr)
        ),
        ret(pop())
    );
}

pub fn gen_atan2(ctx: &mut Context, my_label: Label) {
    let x_lo = 3;
    let x_hi = 2;
    let y_lo = 1;
    let y_hi = 0;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(4),
        datan2(
            lloc(x_hi),
            lloc(x_lo),
            lloc(y_hi),
            lloc(y_lo),
            push(),
            storel(ctx.layout.hi_return().addr)
        ),
        ret(pop())
    )
}

pub fn gen_restart(ctx: &mut Context, my_label: Label) {
    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(0),
        restart(),
        ret(imm(0)),
    );
}

pub fn gen_save(ctx: &mut Context, my_label: Label) {
    let stream = 0;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(1),
        save(lloc(stream), push()),
        ret(pop()),
    );
}

pub fn gen_restore(ctx: &mut Context, my_label: Label) {
    let stream = 0;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(1),
        save(lloc(stream), push()),
        ret(pop()),
    );
}

pub fn gen_saveundo(ctx: &mut Context, my_label: Label) {
    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(0),
        saveundo(push()),
        ret(pop()),
    );
}

pub fn gen_restoreundo(ctx: &mut Context, my_label: Label) {
    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(0),
        restoreundo(push()),
        ret(pop()),
    );
}

pub fn gen_hasundo(ctx: &mut Context, my_label: Label) {
    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(0),
        hasundo(push()),
        ret(pop()),
    );
}

pub fn gen_discardundo(ctx: &mut Context, my_label: Label) {
    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(0),
        discardundo(),
        ret(imm(0)),
    );
}

pub fn gen_protect(ctx: &mut Context, my_label: Label) {
    let addr = 1;
    let n = 0;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(2),
        callfiii(
            imml(ctx.rt.checkaddr),
            lloc(addr),
            imm(0),
            lloc(n),
            discard()
        ),
        add(imml(ctx.layout.memory().addr), lloc(addr), push()),
        protect(pop(), lloc(n)),
        ret(imm(0)),
    );
}

pub fn gen_gestalt(ctx: &mut Context, my_label: Label) {
    let number = 1;
    let extra = 0;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(2),
        gestalt(lloc(number), lloc(extra), push()),
        ret(pop()),
    );
}

pub fn gen_intrinsic(ctx: &mut Context, imported_func: &ImportedFunction, my_label: Label) {
    let import = ctx.module.imports.get(imported_func.import);
    let name = &import.name;

    if check_intrinsic_type(ctx, imported_func) {
        match name.as_str() {
            #[cfg(feature = "spectest")]
            "spectest_result" => gen_spectest_result(ctx, imported_func, my_label),
            "glkarea_get_byte" => gen_glkarea_get_byte(ctx, my_label),
            "glkarea_get_word" => gen_glkarea_get_word(ctx, my_label),
            "glkarea_get_bytes" => gen_glkarea_get_bytes(ctx, my_label),
            "glkarea_get_words" => gen_glkarea_get_words(ctx, my_label),
            "glkarea_put_byte" => gen_glkarea_put_byte(ctx, my_label),
            "glkarea_put_word" => gen_glkarea_put_word(ctx, my_label),
            "glkarea_put_bytes" => gen_glkarea_put_bytes(ctx, my_label),
            "glkarea_put_words" => gen_glkarea_put_words(ctx, my_label),
            "random" => gen_random(ctx, my_label),
            "setrandom" => gen_setrandom(ctx, my_label),
            "fmodf" => gen_fmodf(ctx, my_label),
            "expf" => gen_expf(ctx, my_label),
            "logf" => gen_logf(ctx, my_label),
            "powf" => gen_powf(ctx, my_label),
            "sinf" => gen_sinf(ctx, my_label),
            "cosf" => gen_cosf(ctx, my_label),
            "tanf" => gen_tanf(ctx, my_label),
            "asinf" => gen_asinf(ctx, my_label),
            "acosf" => gen_acosf(ctx, my_label),
            "atanf" => gen_atanf(ctx, my_label),
            "atan2f" => gen_atan2f(ctx, my_label),
            "fmod" => gen_fmod(ctx, my_label),
            "exp" => gen_exp(ctx, my_label),
            "log" => gen_log(ctx, my_label),
            "pow" => gen_pow(ctx, my_label),
            "sin" => gen_sin(ctx, my_label),
            "cos" => gen_cos(ctx, my_label),
            "tan" => gen_tan(ctx, my_label),
            "asin" => gen_asin(ctx, my_label),
            "acos" => gen_acos(ctx, my_label),
            "atan" => gen_atan(ctx, my_label),
            "atan2" => gen_atan2(ctx, my_label),
            "restart" => gen_restart(ctx, my_label),
            "save" => gen_save(ctx, my_label),
            "restore" => gen_restore(ctx, my_label),
            "saveundo" => gen_saveundo(ctx, my_label),
            "restoreundo" => gen_restoreundo(ctx, my_label),
            "hasundo" => gen_hasundo(ctx, my_label),
            "discardundo" => gen_discardundo(ctx, my_label),
            "protect" => gen_protect(ctx, my_label),
            "gestalt" => gen_gestalt(ctx, my_label),
            _ => unreachable!(
                "Unrecognized intrinsic function should have returned false from type check"
            ),
        }
    }
}
