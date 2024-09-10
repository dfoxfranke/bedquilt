#![macro_use]
use anyhow::anyhow;
use glulx_asm::{Item, ZeroItem};
use std::{hash::Hash, path::PathBuf};
use walrus::{GlobalId, GlobalKind, Module, ValType};

use crate::{layout::Layout, rt::RuntimeLabels, CompilationError};

macro_rules! push_all {
    ($v:expr, $($item:expr),* $(,)*) => {
        {
            $($v.push($item);)*
        }
    }
}

pub trait LabelGenerator {
    type Label: Clone + Eq + Hash;
    fn gen(&mut self, desc: &'static str) -> Self::Label;
}

pub type ItemVec<L> = Vec<Item<L>>;
pub type ZeroItemVec<L> = Vec<ZeroItem<L>>;

pub struct Context<'a, G>
where
    G: LabelGenerator,
{
    pub options: &'a CompilationOptions,
    pub module: &'a Module,
    pub layout: &'a Layout<G::Label>,
    pub rt: &'a RuntimeLabels<G::Label>,
    pub gen: &'a mut G,
    pub rom_items: &'a mut ItemVec<G::Label>,
    pub ram_items: &'a mut ItemVec<G::Label>,
    pub zero_items: &'a mut ZeroItemVec<G::Label>,
    pub errors: &'a mut Vec<CompilationError>,
}

pub const DEFAULT_GLK_AREA_SIZE: u32 = 4096;
pub const DEFAULT_STACK_SIZE: u32 = 1048576;
pub const DEFAULT_TABLE_GROWTH_LIMIT: u32 = 1024;

#[derive(Debug, Clone)]
pub struct CompilationOptions {
    pub(crate) glk_area_size: u32,
    pub(crate) stack_size: u32,
    pub(crate) table_growth_limit: u32,
    pub(crate) text: bool,
    pub(crate) input: Option<PathBuf>,
    pub(crate) output: Option<PathBuf>,
}

impl Default for CompilationOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl CompilationOptions {
    pub fn new() -> Self {
        CompilationOptions {
            glk_area_size: DEFAULT_GLK_AREA_SIZE,
            stack_size: DEFAULT_STACK_SIZE,
            table_growth_limit: DEFAULT_TABLE_GROWTH_LIMIT,
            text: false,
            input: None,
            output: None,
        }
    }

    pub fn set_glk_area_size(&mut self, size: u32) {
        self.glk_area_size = size;
    }

    pub fn set_stack_size(&mut self, size: u32) {
        self.stack_size = size;
    }

    pub fn set_table_growth_limit(&mut self, limit: u32) {
        self.table_growth_limit = limit;
    }

    pub fn set_text(&mut self, text: bool) {
        self.text = text;
    }

    pub fn set_input(&mut self, input: Option<PathBuf>) {
        self.input = input;
    }

    pub fn set_output(&mut self, output: Option<PathBuf>) {
        self.output = output;
    }
}

pub fn vt_words(vt: ValType) -> u32 {
    match vt {
        ValType::I32 => 1,
        ValType::I64 => 2,
        ValType::F32 => 1,
        ValType::F64 => 2,
        ValType::V128 => 4,
        ValType::Ref(_) => 1,
    }
}

pub fn reject_global_constexpr<G>(ctx: &mut Context<G>, id: GlobalId)
where
    G: LabelGenerator,
{
    match &ctx.module.globals.get(id).kind {
        GlobalKind::Import(id) => ctx.errors.push(CompilationError::UnrecognizedImport(
            ctx.module.imports.get(*id).clone(),
        )),
        GlobalKind::Local(_) => {
            ctx.errors.push(CompilationError::ValidationError(anyhow!(
                "Constexprs which take their value from non-imported globals are not supported."
            )));
        }
    }
}
