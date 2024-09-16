use std::collections::HashMap;
use std::sync::OnceLock;

use walrus::{ImportedFunction, ValType};

use crate::common::*;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
enum GlkParam {
    /// Parameter is a scalar, not a pointer
    Scalar,
    /// Parameter is a pointer to a scalar value this many words long
    ScalarPtr(u32),
    /// Parameter a pointer to an array of bytes, with length given by the
    /// indicated argument
    ByteArrayPtr(u32),
    /// Parameter a pointer to an array of words, with length given by the
    /// indicated argument
    WordArrayPtr(u32),
    /// Parameter is a pointer to a string terminated by a null byte
    Lat1Ptr,
    /// Parameter is a pointer to a string terminated by a null word
    UnicodePtr,
    /// Parameter is a pointer to an byte array in Glk-owned memory, with
    /// length given by the indicated argument.
    OwnedByteArrayPtr(u32),
    /// Parameter is a pointer to a word array in Glk-owned memory, with
    /// length given by the indicated argument.
    OwnedWordArrayPtr(u32),
}

#[derive(Debug, Copy, Clone)]
struct GlkFunction {
    name: &'static str,
    selector: u16,
    params: &'static [GlkParam],
    has_return: bool,
}

static GLK_FUNCTIONS: &[GlkFunction] = [
    GlkFunction {
        name: "exit",
        selector: 0x0001,
        params: &[],
        has_return: false,
    },
    GlkFunction {
        name: "tick",
        selector: 0x0003,
        params: &[],
        has_return: false,
    },
    GlkFunction {
        name: "gestalt",
        selector: 0x0004,
        params: &[GlkParam::Scalar, GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "gestalt_ext",
        selector: 0x0005,
        params: &[
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::WordArrayPtr(3),
            GlkParam::Scalar,
        ],
        has_return: true,
    },
    GlkFunction {
        name: "window_open",
        selector: 0x0023,
        params: &[
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
        ],
        has_return: true,
    },
    GlkFunction {
        name: "set_window",
        selector: 0x002f,
        params: &[GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "put_char_uni",
        selector: 0x0128,
        params: &[GlkParam::Scalar],
        has_return: false,
    },
]
.as_slice();

fn get_glk_function(name: &str) -> Option<GlkFunction> {
    static GLK_FUNCTION_MAP: OnceLock<HashMap<&'static str, GlkFunction>> = OnceLock::new();

    let map = GLK_FUNCTION_MAP.get_or_init(|| GLK_FUNCTIONS.iter().map(|v| (v.name, *v)).collect());

    map.get(name).copied()
}

impl GlkFunction {
    fn codegen(&self, ctx: &mut Context, my_label: Label) {
        use glulx_asm::concise::*;
        let nargs: u32 = self.params.len().try_into().unwrap();
        let mem = ctx.layout.memory();
        let glk_area = ctx.layout.glk_area();

        ctx.rom_items.push(label(my_label));
        ctx.rom_items.push(fnhead_local(nargs));
        for (num, param) in self.params.iter().copied().rev().enumerate() {
            let num: u32 = num.try_into().unwrap();
            match param {
                GlkParam::Scalar => {
                    ctx.rom_items.push(copy(lloc(num), push()));
                }
                GlkParam::ByteArrayPtr(_) | GlkParam::Lat1Ptr => {
                    ctx.rom_items.push(add(lloc(num), imml(mem.addr), push()));
                }
                GlkParam::ScalarPtr(n) => {
                    ctx.rom_items.push(callfii(
                        imml(ctx.rt.swaparray),
                        lloc(num),
                        uimm(n),
                        discard(),
                    ));
                    ctx.rom_items.push(add(lloc(num), imml(mem.addr), push()));
                }
                GlkParam::WordArrayPtr(sizearg) => {
                    ctx.rom_items.push(callfii(
                        imml(ctx.rt.swaparray),
                        lloc(num),
                        lloc(sizearg),
                        discard(),
                    ));
                    ctx.rom_items.push(add(lloc(num), imml(mem.addr), push()));
                }
                GlkParam::UnicodePtr => {
                    ctx.rom_items
                        .push(callfi(imml(ctx.rt.swapunistr), lloc(num), discard()));
                    ctx.rom_items.push(add(lloc(num), imml(mem.addr), push()));
                }
                GlkParam::OwnedByteArrayPtr(_) | GlkParam::OwnedWordArrayPtr(_) => {
                    ctx.rom_items
                        .push(add(lloc(num), imml(glk_area.addr), push()));
                }
            }
        }
        ctx.rom_items.push(glk(
            uimm(self.selector.into()),
            uimm(nargs),
            if self.has_return { push() } else { discard() },
        ));
        for (num, param) in self.params.iter().copied().rev().enumerate() {
            let num: u32 = num.try_into().unwrap();
            match param {
                GlkParam::ScalarPtr(n) => {
                    ctx.rom_items.push(callfii(
                        imml(ctx.rt.swaparray),
                        lloc(num),
                        uimm(n),
                        discard(),
                    ));
                }
                GlkParam::WordArrayPtr(sizearg) => {
                    ctx.rom_items.push(callfii(
                        imml(ctx.rt.swaparray),
                        lloc(num),
                        lloc(sizearg),
                        discard(),
                    ));
                }
                GlkParam::UnicodePtr => {
                    ctx.rom_items
                        .push(callfi(imml(ctx.rt.swapunistr), lloc(num), discard()));
                }
                _ => {}
            }
        }
        if self.has_return {
            ctx.rom_items.push(ret(pop()));
        } else {
            ctx.rom_items.push(ret(imm(0)));
        }
    }
}

pub fn gen_glk(ctx: &mut Context, imported_func: &ImportedFunction, label: Label) {
    let import = ctx.module.imports.get(imported_func.import);
    let name = &import.name;

    if let Some(function) = get_glk_function(name.as_str()) {
        let expected_params = vec![ValType::I32; function.params.len()];
        let expected_results = if function.has_return {
            vec![ValType::I32]
        } else {
            vec![]
        };
        let ty = ctx.module.types.get(imported_func.ty);
        if expected_params == ty.params() && expected_results == ty.results() {
            function.codegen(ctx, label);
        } else {
            ctx.errors
                .push(crate::CompilationError::IncorrectlyTypedImport {
                    import: ctx.module.imports.get(imported_func.import).clone(),
                    expected: (expected_params, expected_results),
                    actual: (ty.params().to_owned(), ty.results().to_owned()),
                });
        }
    } else {
        ctx.errors
            .push(crate::CompilationError::UnrecognizedImport(import.clone()))
    }
}
