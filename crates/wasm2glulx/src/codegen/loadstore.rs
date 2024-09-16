use std::collections::HashSet;

use crate::common::*;
use glulx_asm::concise::*;
use glulx_asm::{LoadOperand, StoreOperand};
use walrus::{ir, ValType};

use super::classify::{Load, Store};
use super::toplevel::Frame;

#[derive(Debug, Clone)]
pub struct Credits {
    loads: Vec<LoadOperand<Label>>,
}

#[derive(Debug, Clone)]
pub struct Debts {
    stores: Vec<StoreOperand<Label>>,
    returns: Option<Returns>,
}

#[derive(Debug, Clone)]
struct Returns {
    m: usize,
    n: usize,
    hi_return: Label,
}

impl Credits {
    pub fn empty() -> Credits {
        Credits { loads: vec![] }
    }

    pub fn new(ctx: &Context, frame: &Frame, load_instrs: &[Load]) -> Credits {
        let mut loads = Vec::new();

        for instr in load_instrs {
            match instr {
                Load::LocalGet(ir::LocalGet { local: id }) => {
                    let local = ctx.module.locals.get(*id);
                    let glulx_local = *frame
                        .locals
                        .get(id)
                        .expect("All locals should have been added to the frame's map");
                    let ty = local.ty();
                    match ty {
                        ValType::I32 | ValType::F32 | ValType::Ref(_) => {
                            loads.push(lloc(glulx_local));
                        }
                        ValType::I64 | ValType::F64 => {
                            loads.push(lloc(glulx_local + 1));
                            loads.push(lloc(glulx_local));
                        }
                        ValType::V128 => {
                            loads.push(lloc(glulx_local + 3));
                            loads.push(lloc(glulx_local + 2));
                            loads.push(lloc(glulx_local + 1));
                            loads.push(lloc(glulx_local));
                        }
                    }
                }
                Load::GlobalGet(ir::GlobalGet { global: id }) => {
                    let global = ctx.module.globals.get(*id);
                    let layout = ctx.layout.global(*id);
                    match global.ty {
                        ValType::I32 | ValType::F32 | ValType::Ref(_) => {
                            assert_eq!(layout.words, 1);
                            loads.push(derefl(layout.addr));
                        }
                        ValType::I64 | ValType::F64 => {
                            assert_eq!(layout.words, 2);
                            loads.push(derefl_off(layout.addr, 4));
                            loads.push(derefl(layout.addr));
                        }
                        ValType::V128 => {
                            assert_eq!(layout.words, 4);
                            loads.push(derefl_off(layout.addr, 12));
                            loads.push(derefl_off(layout.addr, 8));
                            loads.push(derefl_off(layout.addr, 4));
                            loads.push(derefl(layout.addr));
                        }
                    }
                }
                Load::Const(ir::Const { value }) => match value {
                    walrus::ir::Value::I32(x) => {
                        loads.push(imm(*x));
                    }
                    walrus::ir::Value::I64(x) => {
                        loads.push(imm(*x as i32));
                        loads.push(imm((*x >> 32) as i32));
                    }
                    walrus::ir::Value::F32(x) => {
                        loads.push(f32_to_imm(*x));
                    }
                    walrus::ir::Value::F64(x) => {
                        let (hi, lo) = f64_to_imm(*x);
                        loads.push(lo);
                        loads.push(hi);
                    }
                    walrus::ir::Value::V128(x) => {
                        loads.push(uimm(*x as u32));
                        loads.push(uimm((*x >> 32) as u32));
                        loads.push(uimm((*x >> 64) as u32));
                        loads.push(uimm((*x >> 96) as u32));
                    }
                },
                Load::RefNull(ir::RefNull { .. }) => {
                    loads.push(imm(0));
                }
                Load::RefFunc(ir::RefFunc { func }) => {
                    loads.push(uimm(ctx.layout.func(*func).fnnum));
                }
                Load::TableSize(ir::TableSize { table: id }) => {
                    let table = ctx.layout.table(*id);
                    let addr = table.cur_count;
                    loads.push(derefl(addr));
                }
            }
        }
        Self { loads }
    }

    pub fn from_returns(ctx: &Context, result_type: &[ValType]) -> Self {
        let words: usize = result_type.iter().map(|vt| vt_words(*vt) as usize).sum();
        if words == 0 {
            return Credits::empty();
        }
        let mut loads = Vec::with_capacity(words - 1);
        for i in (0..words - 1).rev() {
            loads.push(derefl_off(ctx.layout.hi_return().addr, (4 * i).try_into().expect("hi_return offsets too large to fit in an i32 should have been rejected when building the layout")));
        }

        Credits { loads }
    }

    pub fn append_later(&mut self, mut other: Credits) {
        self.loads.append(&mut other.loads)
    }

    pub fn pop(&mut self) -> LoadOperand<Label> {
        self.loads.pop().unwrap_or(LoadOperand::Pop)
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.loads.len()
    }

    pub fn is_empty(&self) -> bool {
        self.loads.is_empty()
    }

    pub fn take(&mut self) -> Credits {
        Credits {
            loads: std::mem::take(&mut self.loads),
        }
    }

    pub fn gen(&mut self, ctx: &mut Context) {
        for load in std::mem::take(&mut self.loads) {
            ctx.rom_items.push(copy(load, push()));
        }
    }
}

impl Drop for Credits {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            assert!(
                self.is_empty(),
                "Credits should be consumed before dropping, left {:?}",
                self
            )
        }
    }
}

impl Debts {
    pub fn empty() -> Debts {
        Debts {
            stores: vec![],
            returns: None,
        }
    }

    pub fn new(
        ctx: &Context,
        frame: &Frame,
        mut stack: &[ValType],
        store_instrs: &[Store],
        then_return: bool,
    ) -> Debts {
        let mut stores = Vec::new();

        for store_instr in store_instrs {
            let stack_type = *stack
                .last()
                .expect("There should be something on the stack for satisfying debts");
            stack = &stack[..stack.len() - 1];

            match store_instr {
                Store::LocalSet(ir::LocalSet { local: id }) => {
                    let local = ctx.module.locals.get(*id);
                    let glulx_local = *frame
                        .locals
                        .get(id)
                        .expect("All locals should have been added to the frame's map.");
                    assert_eq!(
                        local.ty(),
                        stack_type,
                        "Type on stack shoud match type of local being stored to"
                    );
                    match stack_type {
                        ValType::I32 | ValType::F32 | ValType::Ref(_) => {
                            stores.push(sloc(glulx_local));
                        }
                        ValType::I64 | ValType::F64 => {
                            stores.push(sloc(glulx_local));
                            stores.push(sloc(glulx_local + 1));
                        }
                        ValType::V128 => {
                            stores.push(sloc(glulx_local));
                            stores.push(sloc(glulx_local + 1));
                            stores.push(sloc(glulx_local + 2));
                            stores.push(sloc(glulx_local + 3));
                        }
                    }
                }
                Store::GlobalSet(ir::GlobalSet { global: id }) => {
                    let global = ctx.module.globals.get(*id);
                    let layout = ctx.layout.global(*id);
                    assert_eq!(
                        global.ty, stack_type,
                        "Type on stack should match type of global being stored to"
                    );

                    match stack_type {
                        ValType::I32 | ValType::F32 | ValType::Ref(_) => {
                            assert_eq!(layout.words, 1);
                            stores.push(storel(layout.addr));
                        }
                        ValType::F64 | ValType::I64 => {
                            assert_eq!(layout.words, 2);
                            stores.push(storel(layout.addr));
                            stores.push(storel_off(layout.addr, 4));
                        }
                        ValType::V128 => {
                            assert_eq!(layout.words, 4);
                            stores.push(storel(layout.addr));
                            stores.push(storel_off(layout.addr, 4));
                            stores.push(storel_off(layout.addr, 8));
                            stores.push(storel_off(layout.addr, 12));
                        }
                    }
                }
                Store::Drop(_) => {
                    for _ in 0..vt_words(stack_type) {
                        stores.push(discard());
                    }
                }
            }
        }

        stores.reverse();

        let returns = if then_return {
            let ret_types = ctx.module.types.get(frame.function.ty()).results();
            assert!(
                stack.ends_with(ret_types),
                "types on stack should match return type of function"
            );
            let words: usize = ret_types.iter().map(|vt| vt_words(*vt) as usize).sum();
            Some(Returns {
                m: 0,
                n: words,
                hi_return: ctx.layout.hi_return().addr,
            })
        } else {
            None
        };

        Debts { stores, returns }
    }

    pub fn len(&self) -> usize {
        self.stores.len()
            + match &self.returns {
                None => 0,
                Some(returns) => returns.n - returns.m,
            }
    }

    pub fn is_empty(&self) -> bool {
        self.stores.is_empty()
            && match &self.returns {
                None => true,
                Some(returns) => returns.n == returns.m,
            }
    }

    pub fn pop(&mut self) -> StoreOperand<Label> {
        if let Some(store) = self.stores.pop() {
            store
        } else if let Some(returns) = &mut self.returns {
            assert!(
                returns.m < returns.n,
                "No further debts should be popped after satisfying return debts"
            );
            if returns.m + 1 < returns.n {
                let store = storel_off(returns.hi_return, (returns.m * 4).try_into().expect("hi_return offsets too large to fit in an i32 should have been rejected when building the layout"));
                returns.m += 1;
                store
            } else {
                StoreOperand::Push
            }
        } else {
            StoreOperand::Push
        }
    }

    fn pop_for_copy(&mut self) -> Option<StoreOperand<Label>> {
        if let Some(store) = self.stores.pop() {
            Some(store)
        } else if let Some(returns) = &mut self.returns {
            if returns.m + 1 < returns.n {
                let store = storel_off(returns.hi_return, (returns.m * 4).try_into().expect("hi_return offsets too large to fit in an i32 should have been rejected when building the layout"));
                returns.m += 1;
                Some(store)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn append_earlier(&mut self, mut other: Debts) {
        if other.returns.is_some() {
            self.stores = std::mem::take(&mut other.stores);
            self.returns = other.returns.take();
        } else {
            self.stores.append(&mut other.stores);
        }
    }

    pub fn take(&mut self) -> Debts {
        Debts {
            stores: std::mem::take(&mut self.stores),
            returns: self.returns.take(),
        }
    }

    pub fn gen(&mut self, ctx: &mut Context) {
        while let Some(store) = self.stores.pop() {
            ctx.rom_items.push(copy(pop(), store));
        }
        if let Some(returns) = &mut self.returns {
            if returns.n == 0 {
                ctx.rom_items.push(ret(imm(0)))
            } else {
                while returns.m + 1 < returns.n {
                    ctx.rom_items.push(copy(pop(), storel_off(returns.hi_return, (returns.m * 4).try_into().expect("hi_return offsets too large to fit in an i32 should have been rejected when building the layout"))));
                    returns.m += 1;
                }
                ctx.rom_items.push(ret(pop()));
                returns.m += 1;
            }
        }
    }
}

impl Drop for Debts {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            assert!(
                self.is_empty(),
                "Debts should be satisfied before dropping, left {:?}",
                self
            )
        }
    }
}

/// Generate an instruction sequence equivalent to pushing all credits and then
/// popping all debts.
///
/// We can optimize to skip the stack in some cases, but have to be careful
/// about stores that overwrite the locations we're loading from. We assume
/// when checking this that labels don't alias each other, which is safe because
/// all labels we're dealing with here are labels of globals and those are
/// never aliased.
pub fn gen_copies(ctx: &mut Context, mut credits: Credits, mut debts: Debts) {
    eprintln!("CREDITS: {:?}, DEBTS: {:?}", credits, debts);
    if debts.stores.is_empty() && matches!(debts.returns, Some(Returns { m: 0, n: 1, .. })) {
        debts.returns = None;
        let ret_operand = credits.pop();
        credits.gen(ctx);
        ctx.rom_items.push(ret(ret_operand));
        return;
    }

    let mut poisoned: HashSet<LoadOperand<Label>> = HashSet::new();
    let mut good_pairs = Vec::new();
    let mut oops_load = None;
    let mut oops_store = None;

    while let Some(store) = debts.pop_for_copy() {
        let load = credits.pop();

        if poisoned.contains(&load) {
            oops_load = Some(load);
            oops_store = Some(store);
            break;
        }

        let poison = match &store {
            StoreOperand::Push => unreachable!("A push operand cannot be a debt"),
            StoreOperand::Discard => None,
            StoreOperand::DerefLabel(addr) => Some(LoadOperand::DerefLabel(*addr)),
            StoreOperand::FrameAddr(addr) => Some(LoadOperand::FrameAddr(*addr)),
        };

        if let Some(poison) = poison {
            poisoned.insert(poison);
        }

        good_pairs.push((load, store));
    }

    credits.gen(ctx);
    if let Some(oops) = oops_load {
        ctx.rom_items.push(copy(oops, push()));
    }

    for (load, store) in good_pairs {
        ctx.rom_items.push(copy(load, store));
    }
    if let Some(oops) = oops_store {
        ctx.rom_items.push(copy(pop(), oops));
    }
    debts.gen(ctx);
}
