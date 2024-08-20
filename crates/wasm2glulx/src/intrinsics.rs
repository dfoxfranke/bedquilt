use walrus::ImportedFunction;

use crate::common::{Context, LabelGenerator};

pub fn gen_intrinsic<G>(ctx: &mut Context<G>, imported_func: &ImportedFunction, _label: G::Label)
where
    G: LabelGenerator,
{
    ctx.errors.push(crate::CompilationError::UnrecognizedImport(
        ctx.module.imports.get(imported_func.import).clone(),
    ));
}
