use glulx_asm::concise::*;
use walrus::{ImportedFunction, ValType};

use crate::common::{vt_words, Context, LabelGenerator};

pub fn gen_intrinsic<G>(ctx: &mut Context<G>, imported_func: &ImportedFunction, my_label: G::Label)
where
    G: LabelGenerator,
{
    let import = ctx.module.imports.get(imported_func.import);
    let name = &import.name;

    match name.as_str() {
        "spectest_result" => {
            let ty = ctx.module.types.get(imported_func.ty);

            if !ty.results().is_empty() {
                ctx.errors
                    .push(crate::CompilationError::IncorrectlyTypedImport {
                        import: import.clone(),
                        expected: (ty.params().to_owned(), Vec::new()),
                        actual: (ty.params().to_owned(), ty.results().to_owned()),
                    });
                return;
            }

            let mut param_word: u32 = ty.params().iter().map(|vt| vt_words(*vt)).sum();

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
        _ => {
            ctx.errors.push(crate::CompilationError::UnrecognizedImport(
                ctx.module.imports.get(imported_func.import).clone(),
            ));
        }
    }
}
