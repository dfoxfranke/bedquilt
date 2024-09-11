use walrus::{DataId, ElementId, ElementItems, FunctionId, GlobalId, Module, TableId, TypeId};
use crate::{common::*, CompilationError, CompilationOptions, OverflowLocation};
use std::collections::HashMap;

#[derive(Debug, Copy, Clone)]
pub struct TypeLayout {
    pub typenum: u32,
    pub param_words: u32,
    pub result_words: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct FnLayout<L> {
    pub addr: L,
    pub fnnum: u32,
    pub typenum: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct TableLayout<L> {
    pub addr: L,
    pub min_count: u32,
    pub cur_count: L,
    pub max_count: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct GlobalLayout<L> {
    pub addr: L,
    pub words: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct ElemLayout<L> {
    pub addr: L,
    pub dropped: L,
    pub count: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct DataLayout<L> {
    pub addr: L,
    pub dropped: L,
    pub size: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct MemLayout<L> {
    pub addr: L,
    pub min_size: u32,
    pub cur_size: L,
}

#[derive(Debug, Copy, Clone)]
pub struct FnTypesLayout<L> {
    pub addr: L,
    pub count: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct GlkLayout<L> {
    pub addr: L,
    pub size: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct HiReturnLayout<L> {
    pub addr: L,
    pub size: u32,
}

#[derive(Debug, Clone)]
pub struct Layout<L> {
    types: HashMap<TypeId, TypeLayout>,
    funcs: HashMap<FunctionId, FnLayout<L>>,
    tables: HashMap<TableId, TableLayout<L>>,
    globals: HashMap<GlobalId, GlobalLayout<L>>,
    elems: HashMap<ElementId, ElemLayout<L>>,
    datas: HashMap<DataId, DataLayout<L>>,
    mem: MemLayout<L>,
    fntypes: FnTypesLayout<L>,
    glk_area: GlkLayout<L>,
    hi_return: HiReturnLayout<L>,
    entrypoint: L,
}

const MIN_HI_RETURN_WORDS: u32 = 4;

impl<L> Layout<L>
where
    L: Clone,
{
    pub fn new<G>(
        options: &CompilationOptions,
        module: &Module,
        gen: &mut G,
    ) -> Result<Self, Vec<CompilationError>>
    where
        G: LabelGenerator<Label = L>,
    {
        let mut types: HashMap<TypeId, TypeLayout> = HashMap::new();
        let mut funcs: HashMap<FunctionId, FnLayout<L>> = HashMap::new();
        let mut tables: HashMap<TableId, TableLayout<L>> = HashMap::new();
        let mut globals: HashMap<GlobalId, GlobalLayout<L>> = HashMap::new();
        let mut elems: HashMap<ElementId, ElemLayout<L>> = HashMap::new();
        let mut datas: HashMap<DataId, DataLayout<L>> = HashMap::new();

        let mut errors: Vec<CompilationError> = Vec::new();

        for (n, t) in module.types.iter().enumerate() {
            let typenum = if let Ok(typenum) = u32::try_from(n + 1) {
                typenum
            } else {
                errors.push(CompilationError::Overflow(OverflowLocation::TypeList));
                break;
            };
            let mut param_words: u32 = 0;
            let mut result_words: u32 = 0;

            for vt in t.params() {
                param_words = param_words.checked_add(vt_words(*vt)).unwrap_or_else(|| {
                    errors.push(CompilationError::Overflow(OverflowLocation::TypeDecl));
                    0
                });
            }

            for vt in t.results() {
                result_words = result_words.checked_add(vt_words(*vt)).unwrap_or_else(|| {
                    errors.push(CompilationError::Overflow(OverflowLocation::TypeDecl));
                    0
                });
            }

            types.insert(
                t.id(),
                TypeLayout {
                    typenum,
                    param_words,
                    result_words,
                },
            );
        }

        for (n, f) in module.funcs.iter().enumerate() {
            let typenum = types
                .get(&f.ty())
                .expect("function should have a known type number")
                .typenum;
            let fnnum = if let Ok(fnnum) = u32::try_from(n + 1) {
                fnnum
            } else {
                errors.push(CompilationError::Overflow(OverflowLocation::FnList));
                break;
            };
            let addr = gen.gen("function");
            funcs.insert(
                f.id(),
                FnLayout {
                    addr,
                    fnnum,
                    typenum,
                },
            );
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
            let words = vt_words(g.ty);
            globals.insert(g.id(), GlobalLayout { addr, words });
        }

        for e in module.elements.iter() {
            let addr = gen.gen("element");
            let dropped = gen.gen("element_dropped");
            let count_usize = match &e.items {
                ElementItems::Functions(v) => v.len(),
                ElementItems::Expressions(_, v) => v.len(),
            };
            let count = u32::try_from(count_usize).unwrap_or_else(|_| {
                errors.push(CompilationError::Overflow(OverflowLocation::Element));
                0
            });
            elems.insert(
                e.id(),
                ElemLayout {
                    addr,
                    dropped,
                    count,
                },
            );
        }

        for d in module.data.iter() {
            let addr = gen.gen("data");
            let dropped = gen.gen("data_dropped");
            let size = u32::try_from(d.value.len()).unwrap_or_else(|_| {
                errors.push(CompilationError::Overflow(OverflowLocation::Data));
                0
            });
            datas.insert(
                d.id(),
                DataLayout {
                    addr,
                    dropped,
                    size,
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
        };

        let fntypes = FnTypesLayout {
            addr: gen.gen("fntypes"),
            count: u32::try_from(funcs.len() + 1).unwrap_or_else(|_| {
                assert!(
                    !errors.is_empty(),
                    "overflow in function list should have been caught above"
                );
                0
            }),
        };

        let glk_area = GlkLayout {
            addr: gen.gen("glk_area"),
            size: options.glk_area_size,
        };

        let hi_return = HiReturnLayout {
            addr: gen.gen("hi_return"),
            size: types
                .values()
                .map(|t| t.result_words)
                .max()
                .unwrap_or(0)
                .max(MIN_HI_RETURN_WORDS)
                .checked_mul(4)
                .unwrap_or_else(|| {
                    errors.push(CompilationError::Overflow(OverflowLocation::TypeDecl));
                    0
                }),
        };

        let entrypoint = gen.gen("entrypoint");

        if errors.is_empty() {
            Ok(Layout {
                types,
                funcs,
                tables,
                globals,
                elems,
                datas,
                mem,
                fntypes,
                glk_area,
                hi_return,
                entrypoint,
            })
        } else {
            Err(errors)
        }
    }

    pub fn iter_funcs(&self) -> std::collections::hash_map::Values<FunctionId, FnLayout<L>> {
        self.funcs.values()
    }

    pub fn ty(&self, id: TypeId) -> &TypeLayout {
        self.types
            .get(&id)
            .expect("Layout should contain all type IDs from module")
    }

    pub fn func(&self, id: FunctionId) -> &FnLayout<L> {
        self.funcs
            .get(&id)
            .expect("Layout should contain all function IDs from module")
    }

    pub fn table(&self, id: TableId) -> &TableLayout<L> {
        self.tables
            .get(&id)
            .expect("Layout should contain all table  IDs from module")
    }

    pub fn global(&self, id: GlobalId) -> &GlobalLayout<L> {
        self.globals
            .get(&id)
            .expect("Layout should contain all global IDs from module")
    }

    pub fn element(&self, id: ElementId) -> &ElemLayout<L> {
        self.elems
            .get(&id)
            .expect("Layout should contain all element IDs from module")
    }

    pub fn data(&self, id: DataId) -> &DataLayout<L> {
        self.datas
            .get(&id)
            .expect("Layout should contain all data IDs from module")
    }

    pub fn memory(&self) -> &MemLayout<L> {
        &self.mem
    }

    pub fn fntypes(&self) -> &FnTypesLayout<L> {
        &self.fntypes
    }

    pub fn glk_area(&self) -> &GlkLayout<L> {
        &self.glk_area
    }

    pub fn hi_return(&self) -> &HiReturnLayout<L> {
        &self.hi_return
    }

    pub fn entrypoint(&self) -> L {
        self.entrypoint.clone()
    }
}
