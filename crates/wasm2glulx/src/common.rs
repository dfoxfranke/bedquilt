// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
// Copyright 2024 Daniel Fox Franke.

#![macro_use]
use anyhow::anyhow;
use glulx_asm::{Item, ZeroItem};
use std::{
    fmt::{Debug, Display},
    hash::Hash,
    path::PathBuf,
};
use walrus::{GlobalId, GlobalKind, Module, ValType};

use crate::{layout::Layout, rt::RuntimeLabels, CompilationError};

macro_rules! push_all {
    ($v:expr, $($item:expr),* $(,)*) => {
        {
            $($v.push($item);)*
        }
    }
}
#[derive(Debug)]
pub struct LabelGenerator(pub usize);

#[derive(Debug, Copy, Clone)]
pub struct Label {
    desc: &'static str,
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

impl LabelGenerator {
    pub fn gen(&mut self, desc: &'static str) -> Label {
        let idx = self.0;
        self.0 += 1;
        {
            Label { desc, num: idx }
        }
    }
}
pub struct Context<'a> {
    pub options: &'a CompilationOptions,
    pub module: &'a Module,
    pub layout: &'a Layout,
    pub rt: &'a RuntimeLabels,
    pub gen: &'a mut LabelGenerator,
    pub rom_items: &'a mut Vec<Item<Label>>,
    pub ram_items: &'a mut Vec<Item<Label>>,
    pub zero_items: &'a mut Vec<ZeroItem<Label>>,
    pub errors: &'a mut Vec<CompilationError>,
}

/// The default value for `--glk-area-size`.
pub const DEFAULT_GLK_AREA_SIZE: u32 = 4096;
/// The default value for `--stack-size`.
pub const DEFAULT_STACK_SIZE: u32 = 1048576;
/// The default value for `--table-growth-limit`.
pub const DEFAULT_TABLE_GROWTH_LIMIT: u32 = 1024;

/// Options that control compilation.
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
    /// Instantiate `CompilationOptions` with its defaults.
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

    /// Set the Glk area size.
    pub fn set_glk_area_size(&mut self, size: u32) {
        self.glk_area_size = size;
    }

    /// Set the stack size.
    pub fn set_stack_size(&mut self, size: u32) {
        self.stack_size = size;
    }

    /// Set the table growth limit.
    pub fn set_table_growth_limit(&mut self, limit: u32) {
        self.table_growth_limit = limit;
    }

    /// When true, generate human-readable output instead of a story file.
    pub fn set_text(&mut self, text: bool) {
        self.text = text;
    }

    /// Set the input path.
    pub fn set_input(&mut self, input: Option<PathBuf>) {
        self.input = input;
    }

    /// Ste the output path.
    pub fn set_output(&mut self, output: Option<PathBuf>) {
        self.output = output;
    }
}

pub fn reject_global_constexpr(ctx: &mut Context, id: GlobalId) {
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

pub trait WordCount<Output> {
    fn word_count(&self) -> Output;
}

impl WordCount<u8> for ValType {
    fn word_count(&self) -> u8 {
        match self {
            ValType::I32 => 1,
            ValType::I64 => 2,
            ValType::F32 => 1,
            ValType::F64 => 2,
            ValType::V128 => 4,
            ValType::Ref(_) => 1,
        }
    }
}

impl WordCount<u32> for ValType {
    fn word_count(&self) -> u32 {
        WordCount::<u8>::word_count(self).into()
    }
}

impl WordCount<usize> for ValType {
    fn word_count(&self) -> usize {
        WordCount::<u8>::word_count(self).into()
    }
}

impl WordCount<usize> for [ValType] {
    fn word_count(&self) -> usize {
        self.iter().map(WordCount::<usize>::word_count).sum()
    }
}

impl WordCount<usize> for Vec<ValType> {
    fn word_count(&self) -> usize {
        self.as_slice().word_count()
    }
}

impl WordCount<u32> for [ValType] {
    fn word_count(&self) -> u32 {
        WordCount::<usize>::word_count(self)
            .try_into()
            .expect("Types that overflow a u32 should have been rejected during layout creation")
    }
}

impl WordCount<u32> for Vec<ValType> {
    fn word_count(&self) -> u32 {
        self.as_slice().word_count()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TrapCode {
    Unreachable,
    IntegerOverflow,
    IntegerDivideByZero,
    InvalidConversionToInteger,
    OutOfBoundsMemoryAccess,
    IndirectCallTypeMismatch,
    OutOfBoundsTableAccess,
    UndefinedElement,
    UninitializedElement,
    CallStackExhausted,
}

impl TrapCode {
    pub const ALL: &[TrapCode] = &[
        TrapCode::Unreachable,
        TrapCode::IntegerOverflow,
        TrapCode::IntegerDivideByZero,
        TrapCode::InvalidConversionToInteger,
        TrapCode::OutOfBoundsMemoryAccess,
        TrapCode::IndirectCallTypeMismatch,
        TrapCode::OutOfBoundsTableAccess,
        TrapCode::UndefinedElement,
        TrapCode::UninitializedElement,
        TrapCode::CallStackExhausted,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            TrapCode::Unreachable => "unreachable",
            TrapCode::IntegerOverflow => "integer overflow",
            TrapCode::IntegerDivideByZero => "integer divide by zero",
            TrapCode::InvalidConversionToInteger => "invalid conversion to integer",
            TrapCode::OutOfBoundsMemoryAccess => "out of bounds memory access",
            TrapCode::IndirectCallTypeMismatch => "indirect call type mismatch",
            TrapCode::OutOfBoundsTableAccess => "out of bounds table access",
            TrapCode::UndefinedElement => "undefined element",
            TrapCode::UninitializedElement => "uninitialized element",
            TrapCode::CallStackExhausted => "call stack exhausted",
        }
    }
}

impl From<TrapCode> for u32 {
    fn from(code: TrapCode) -> u32 {
        match code {
            TrapCode::Unreachable => 0,
            TrapCode::IntegerOverflow => 1,
            TrapCode::IntegerDivideByZero => 2,
            TrapCode::InvalidConversionToInteger => 3,
            TrapCode::OutOfBoundsMemoryAccess => 4,
            TrapCode::IndirectCallTypeMismatch => 5,
            TrapCode::OutOfBoundsTableAccess => 6,
            TrapCode::UndefinedElement => 7,
            TrapCode::UninitializedElement => 8,
            TrapCode::CallStackExhausted => 9,
        }
    }
}

impl Display for TrapCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
