use core::hash::Hash;
use std::{fmt::Display, io::{Read, Write}, sync::Arc};

use bytes::BytesMut;
use common::{Context, LabelGenerator};
use glulx_asm::AssemblerError;

mod codegen;
mod common;
mod data;
mod entrypoint;
mod error;
mod glk;
mod intrinsics;
mod layout;
mod rt;

pub use common::{
    CompilationOptions, DEFAULT_GLK_AREA_SIZE, DEFAULT_STACK_SIZE, DEFAULT_TABLE_GROWTH_LIMIT,
};
pub use error::*;

#[derive(Debug)]
struct Gen(usize);

#[derive(Debug, Clone)]
struct Label {
    desc: Arc<str>,
    num: usize,
}

impl PartialEq for Label {
    fn eq(&self, other: &Self) -> bool {
        self.num == other.num
    }
}

impl Eq for Label {}

impl Hash for Label {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.num.hash(state)
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{{{}}}", self.desc, self.num)
    }
}

impl LabelGenerator for Gen {
    type Label = Label;
    fn gen(&mut self, desc: &str) -> Self::Label {
        let idx = self.0;
        self.0 += 1;
        {
            Label {
                desc: desc.into(),
                num: idx,
            }
        }
    }
}

pub fn compile_module_to_bytes(
    options: &CompilationOptions,
    module: &walrus::Module,
) -> Result<BytesMut, Vec<CompilationError>> {
    let mut gen = Gen(0);
    let mut rom_items = common::ItemVec::new();
    let mut ram_items = common::ItemVec::new();
    let mut zero_items = common::ZeroItemVec::new();

    let layout = layout::Layout::new(options, module, &mut gen)?;
    let rt = rt::RuntimeLabels::new(&mut gen);

    let mut errors = Vec::new();

    let mut ctx = Context {
        options,
        module,
        layout: &layout,
        rt: &rt,
        gen: &mut gen,
        rom_items: &mut rom_items,
        ram_items: &mut ram_items,
        zero_items: &mut zero_items,
        errors: &mut errors,
    };

    rt::gen_rt(&mut ctx);

    for function in ctx.module.functions() {
        let fn_layout = ctx.layout.func(function.id());
        #[allow(clippy::clone_on_copy)]
        let label = fn_layout.addr.clone();
        match &function.kind {
            walrus::FunctionKind::Import(imported_function) => {
                let import = ctx.module.imports.get(imported_function.import);
                let module_name = &import.module;
                if module_name == "glk" {
                    glk::gen_glk(&mut ctx, imported_function, label);
                } else if module_name == "glulx" {
                    intrinsics::gen_intrinsic(&mut ctx, imported_function, label);
                } else {
                    ctx.errors
                        .push(CompilationError::UnrecognizedImport(import.clone()))
                }
            }
            walrus::FunctionKind::Local(local) => {
                codegen::gen_function(&mut ctx, local, label, function.name.as_deref());
            }
            walrus::FunctionKind::Uninitialized(_) => {
                unreachable!(
                    "Uninitialized functions shoud not be present in parsed and validated modules."
                )
            }
        }
    }
    entrypoint::gen_entrypoint(&mut ctx);
    data::gen_data(&mut ctx);

    if !ctx.errors.is_empty() {
        return Err(errors);
    }

    let assembly = glulx_asm::Assembly {
        rom_items: std::borrow::Cow::Borrowed(ctx.rom_items),
        ram_items: std::borrow::Cow::Borrowed(ctx.ram_items),
        zero_items: std::borrow::Cow::Borrowed(ctx.zero_items),
        stack_size: ctx.options.stack_size,
        start_func: glulx_asm::LabelRef(ctx.layout.entrypoint(), 0),
        decoding_table: None,
    };

    if ctx.options.text {
        Ok(assembly.to_string().as_str().into())
    } else {
        match assembly.assemble() {
            Ok(bytes) => Ok(bytes),
            Err(AssemblerError::Overflow) => Err(vec![CompilationError::Overflow(
                OverflowLocation::FinalAssembly,
            )]),
            Err(e) => Err(vec![CompilationError::OtherError(e.into())]),
        }
    }
}

pub fn compile(options: &CompilationOptions) -> Result<usize, Vec<CompilationError>> {
    let mut config = walrus::ModuleConfig::new();
    config.generate_synthetic_names_for_anonymous_items(true);

    let module = if let Some(pathbuf) = &options.input {
        config
            .parse_file(pathbuf)
            .map_err(|e| vec![CompilationError::ValidationError(e)])?
    } else {
        let mut stdin = std::io::stdin();
        let mut input_vec = Vec::new();
        stdin
            .read_to_end(&mut input_vec)
            .map_err(|e| vec![CompilationError::InputError(e)])?;
        config
            .parse(&input_vec)
            .map_err(|e| vec![CompilationError::ValidationError(e)])?
    };

    let bytes = compile_module_to_bytes(options, &module)?.freeze();

    if let Some(output) = &options.output {
        let mut file =
            std::fs::File::create(output).map_err(|e| vec![CompilationError::OutputError(e)])?;
        let size = bytes.len();
        file.write_all(&bytes)
            .map_err(|e| vec![CompilationError::OutputError(e)])?;
        file.flush()
            .map_err(|e| vec![CompilationError::OutputError(e)])?;
        Ok(size)
    } else {
        let mut stdout = std::io::stdout();
        let size = bytes.len();
        stdout
            .write_all(&bytes)
            .map_err(|e| vec![CompilationError::OutputError(e)])?;
        stdout
            .flush()
            .map_err(|e| vec![CompilationError::OutputError(e)])?;
        Ok(size)
    }
}
