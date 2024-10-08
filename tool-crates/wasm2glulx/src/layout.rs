// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
// Copyright 2024 Daniel Fox Franke.

use crate::{common::*, CompilationError, CompilationOptions, OverflowLocation};
use std::collections::HashMap;
use walrus::{DataId, ElementId, ElementItems, FunctionId, GlobalId, Module, TableId, TypeId};

#[derive(Debug, Copy, Clone)]
pub struct TypeLayout {
    pub typenum: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct FnLayout {
    pub addr: Label,
}

#[derive(Debug, Copy, Clone)]
pub struct TableLayout {
    pub addr: Label,
    pub min_count: u32,
    pub cur_count: Label,
    pub max_count: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct GlobalLayout {
    pub addr: Label,
    pub words: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct ElemLayout {
    pub addr: Label,
    pub initial_count: u32,
    pub cur_count: Label,
}

#[derive(Debug, Copy, Clone)]
pub struct DataLayout {
    pub addr: Label,
    pub initial_size: u32,
    pub cur_size: Label,
}

#[derive(Debug, Copy, Clone)]
pub struct MemLayout {
    pub addr: Label,
    pub min_size: u32,
    pub cur_size: Label,
    pub max_size: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct GlkLayout {
    pub addr: Label,
    pub size: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct HiReturnLayout {
    pub addr: Label,
    pub size: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct TrapLayout {
    pub string_table: Label,
}

#[derive(Debug, Clone)]
pub struct Layout {
    types: HashMap<TypeId, TypeLayout>,
    funcs: HashMap<FunctionId, FnLayout>,
    tables: HashMap<TableId, TableLayout>,
    globals: HashMap<GlobalId, GlobalLayout>,
    elems: HashMap<ElementId, ElemLayout>,
    datas: HashMap<DataId, DataLayout>,
    mem: MemLayout,
    glk_area: GlkLayout,
    hi_return: HiReturnLayout,
    entrypoint: Label,
    trap: TrapLayout,
}

const MIN_HI_RETURN_WORDS: usize = 4;

impl Layout {
    pub fn new(
        options: &CompilationOptions,
        module: &Module,
        gen: &mut LabelGenerator,
    ) -> Result<Self, Vec<CompilationError>>
where {
        let mut types: HashMap<TypeId, TypeLayout> = HashMap::new();
        let mut funcs: HashMap<FunctionId, FnLayout> = HashMap::new();
        let mut tables: HashMap<TableId, TableLayout> = HashMap::new();
        let mut globals: HashMap<GlobalId, GlobalLayout> = HashMap::new();
        let mut elems: HashMap<ElementId, ElemLayout> = HashMap::new();
        let mut datas: HashMap<DataId, DataLayout> = HashMap::new();

        let mut errors: Vec<CompilationError> = Vec::new();

        for (n, t) in module.types.iter().enumerate() {
            let typenum = if let Ok(typenum) = u32::try_from(n + 1) {
                typenum
            } else {
                errors.push(CompilationError::Overflow(OverflowLocation::TypeList));
                break;
            };

            types.insert(t.id(), TypeLayout { typenum });
        }

        for f in module.funcs.iter() {
            let addr = gen.gen("function");
            funcs.insert(f.id(), FnLayout { addr });
        }

        for t in module.tables.iter() {
            let min_count = u32::try_from(t.initial).unwrap_or_else(|_| {
                errors.push(CompilationError::Overflow(OverflowLocation::Table));
                0
            });
            let max_count = u32::try_from(t.maximum.unwrap_or(u64::MAX))
                .unwrap_or(u32::MAX)
                .min(min_count.saturating_add(options.table_growth_limit));
            let addr = gen.gen("table_addr");
            let cur_count = gen.gen("table_cur_count");
            tables.insert(
                t.id(),
                TableLayout {
                    addr,
                    min_count,
                    cur_count,
                    max_count,
                },
            );
        }

        for g in module.globals.iter() {
            let addr = gen.gen("global");
            let words = g.ty.word_count();
            globals.insert(g.id(), GlobalLayout { addr, words });
        }

        for e in module.elements.iter() {
            let addr = gen.gen("element");
            let cur_count = gen.gen("element_count");
            let count_usize = match &e.items {
                ElementItems::Functions(v) => v.len(),
                ElementItems::Expressions(_, v) => v.len(),
            };
            let initial_count = u32::try_from(count_usize).unwrap_or_else(|_| {
                errors.push(CompilationError::Overflow(OverflowLocation::Element));
                0
            });
            elems.insert(
                e.id(),
                ElemLayout {
                    addr,
                    initial_count,
                    cur_count,
                },
            );
        }

        for d in module.data.iter() {
            let addr = gen.gen("data");
            let cur_size = gen.gen("data_size");
            let initial_size = u32::try_from(d.value.len()).unwrap_or_else(|_| {
                errors.push(CompilationError::Overflow(OverflowLocation::Data));
                0
            });
            datas.insert(
                d.id(),
                DataLayout {
                    addr,
                    initial_size,
                    cur_size,
                },
            );
        }

        if module.memories.iter().count() > 1 {
            errors.push(CompilationError::UnsupportedMultipleMemories);
        }

        let mem = MemLayout {
            addr: gen.gen("memory"),
            cur_size: gen.gen("memory_size"),
            min_size: if let Some(mem) = module.memories.iter().next() {
                u32::try_from(mem.initial.saturating_mul(65536)).unwrap_or_else(|_| {
                    errors.push(CompilationError::Overflow(OverflowLocation::Memory));
                    0
                })
            } else {
                0
            },
            max_size: if let Some(mem) = module.memories.iter().next() {
                if let Some(maximum) = mem.maximum {
                    u32::try_from(maximum.saturating_mul(65536)).unwrap_or(u32::MAX)
                } else {
                    u32::MAX
                }
            } else {
                0
            },
        };

        let glk_area = GlkLayout {
            addr: gen.gen("glk_area"),
            size: options.glk_area_size,
        };

        let hi_return = HiReturnLayout {
            addr: gen.gen("hi_return"),
            size: module
                .types
                .iter()
                .map(|t| t.results().word_count())
                .max()
                .unwrap_or(0)
                .max(MIN_HI_RETURN_WORDS)
                .checked_mul(4)
                .and_then(|bytes| bytes.try_into().ok())
                .unwrap_or_else(|| {
                    errors.push(CompilationError::Overflow(OverflowLocation::TypeDecl));
                    0
                }),
        };

        let entrypoint = gen.gen("entrypoint");
        let trap = TrapLayout {
            string_table: gen.gen("trap_string_table"),
        };

        if errors.is_empty() {
            Ok(Layout {
                types,
                funcs,
                tables,
                globals,
                elems,
                datas,
                mem,
                glk_area,
                hi_return,
                entrypoint,
                trap,
            })
        } else {
            Err(errors)
        }
    }

    pub fn ty(&self, id: TypeId) -> &TypeLayout {
        self.types
            .get(&id)
            .expect("Layout should contain all type IDs from module")
    }

    pub fn func(&self, id: FunctionId) -> &FnLayout {
        self.funcs
            .get(&id)
            .expect("Layout should contain all function IDs from module")
    }

    pub fn table(&self, id: TableId) -> &TableLayout {
        self.tables
            .get(&id)
            .expect("Layout should contain all table  IDs from module")
    }

    pub fn global(&self, id: GlobalId) -> &GlobalLayout {
        self.globals
            .get(&id)
            .expect("Layout should contain all global IDs from module")
    }

    pub fn element(&self, id: ElementId) -> &ElemLayout {
        self.elems
            .get(&id)
            .expect("Layout should contain all element IDs from module")
    }

    pub fn data(&self, id: DataId) -> &DataLayout {
        self.datas
            .get(&id)
            .expect("Layout should contain all data IDs from module")
    }

    pub fn memory(&self) -> &MemLayout {
        &self.mem
    }

    pub fn glk_area(&self) -> &GlkLayout {
        &self.glk_area
    }

    pub fn hi_return(&self) -> &HiReturnLayout {
        &self.hi_return
    }

    pub fn entrypoint(&self) -> Label {
        self.entrypoint
    }

    pub fn trap(&self) -> TrapLayout {
        self.trap
    }
}
