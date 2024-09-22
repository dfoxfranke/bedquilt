use glulx_asm::concise::*;
use walrus::{ImportedFunction, ValType};

use crate::common::{Context, Label, WordCount};

fn check_intrinsic_type(ctx: &mut Context, imported_func: &ImportedFunction) -> bool {
    let import = ctx.module.imports.get(imported_func.import);
    let name = &import.name;
    let ty = ctx.module.types.get(imported_func.ty);

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
        "random" | "glkarea_get_byte" | "glkarea_get_word" => (&[ValType::I32], &[ValType::I32]),
        "setrandom" | "glkarea_put_byte" | "glkarea_put_word" => (&[ValType::I32], &[]),
        "glkarea_get_bytes" | "glkarea_put_bytes" | "glkarea_get_words" | "glkarea_put_words" => {
            (&[ValType::I32, ValType::I32, ValType::I32], &[])
        }
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
    let byte = 1;
    let addr = 0;

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
    let addr = 0;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(1),
        callfii(imml(ctx.rt.checkglkaddr), lloc(addr), imm(4), discard()),
        aload(
            lloc(addr),
            imml_off_shift(ctx.layout.glk_area().addr, 0, 2),
            push()
        ),
        tailcall(imml(ctx.rt.swap), imm(2)),
    );
}

fn gen_glkarea_put_word(ctx: &mut Context, my_label: Label) {
    let word = 1;
    let addr = 0;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(2),
        callfii(imml(ctx.rt.checkglkaddr), lloc(addr), imm(4), discard()),
        callfi(imml(ctx.rt.swap), lloc(word), push()),
        astore(
            lloc(addr),
            imml_off_shift(ctx.layout.glk_area().addr, 0, 2),
            pop()
        ),
    );
}

fn gen_glkarea_get_bytes(ctx: &mut Context, my_label: Label) {
    let n = 0;
    let glkaddr = 1;
    let addr = 2;

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
    let n = 2;
    let addr = 1;
    let glkaddr = 0;

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
    let n = 2;
    let glkaddr = 1;
    let addr = 0;

    let size = 3;
    let absaddr = 4;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(5),
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
        add(lloc(addr), imml(ctx.layout.memory().addr), sloc(absaddr)),
        add(lloc(glkaddr), imml(ctx.layout.glk_area().addr), push()),
        mcopy(lloc(size), pop(), lloc(absaddr)),
        callfii(imml(ctx.rt.swaparray), lloc(addr), lloc(n), discard()),
        ret(imm(0))
    )
}

fn gen_glkarea_put_words(ctx: &mut Context, my_label: Label) {
    let n = 2;
    let addr = 1;
    let glkaddr = 0;

    let size = 3;
    let absglkaddr = 4;

    push_all!(
        ctx.rom_items,
        label(my_label),
        fnhead_local(5),
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
        add(
            lloc(glkaddr),
            imml(ctx.layout.glk_area().addr),
            sloc(absglkaddr)
        ),
        add(lloc(addr), imml(ctx.layout.memory().addr), push()),
        mcopy(lloc(size), pop(), lloc(absglkaddr)),
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

pub fn gen_intrinsic(ctx: &mut Context, imported_func: &ImportedFunction, my_label: Label) {
    let import = ctx.module.imports.get(imported_func.import);
    let name = &import.name;

    if check_intrinsic_type(ctx, imported_func) {
        match name.as_str() {
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
            _ => unreachable!(
                "Unrecognized intrinsic function should have returned false from type check"
            ),
        }
    }
}
