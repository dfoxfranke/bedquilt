use glulx_asm::{concise::*, LoadOperand, StoreOperand};
use std::collections::{HashMap, HashSet};
use walrus::ir::{self, Instr, InstrSeq, InstrSeqId};
use walrus::{LocalFunction, LocalId, ValType};

use crate::common::{vt_words, Context, LabelGenerator};
use crate::{CompilationError, OverflowLocation};

use super::classify::{Block, ClassifiedInstr, Load, Loop, Other, Store};

pub struct Frame<'a, L> {
    pub function: &'a LocalFunction,
    pub function_name: Option<&'a str>,
    pub locals: &'a HashMap<LocalId, u32>,
    pub jump_targets: &'a mut HashMap<InstrSeqId, JumpTarget<L>>,
    pub jump_tables: &'a mut HashMap<L, Vec<L>>,
}
pub struct JumpTarget<L> {
    pub base: usize,
    pub arity: usize,
    pub target: L,
}

#[derive(Debug, Clone)]
pub struct Credits<L>(pub Vec<LoadOperand<L>>);

#[derive(Debug, Clone)]
pub struct Debts<L>(pub Vec<StoreOperand<L>>);

impl<L> Default for Credits<L> {
    fn default() -> Self {
        Credits(Vec::new())
    }
}

impl<L> Default for Debts<L> {
    fn default() -> Self {
        Debts(Vec::new())
    }
}

impl<L> Credits<L> {
    pub fn pop(&mut self) -> LoadOperand<L> {
        self.0.pop().unwrap_or(LoadOperand::Pop)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn gen<G>(mut self, ctx: &mut Context<G>)
    where
        G: LabelGenerator<Label = L>,
    {
        for credit in std::mem::take(&mut self.0) {
            ctx.rom_items.push(copy(credit, push()));
        }
    }

    fn prepend(mut self, mut other: Credits<L>) -> Credits<L> {
        let mut credits = std::mem::take(&mut other.0);
        credits.append(&mut self.0);
        Credits(credits)
    }
}

impl<L> Drop for Credits<L> {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            assert!(
                self.0.is_empty(),
                "Credits should be consumed before being dropped"
            )
        }
    }
}

impl<L> Debts<L> {
    pub fn pop(&mut self) -> StoreOperand<L> {
        self.0.pop().unwrap_or(StoreOperand::Push)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn gen<G>(mut self, ctx: &mut Context<G>)
    where
        G: LabelGenerator<Label = L>,
    {
        while let Some(debt) = self.0.pop() {
            ctx.rom_items.push(copy(pop(), debt));
        }
    }

    fn prepend(mut self, mut other: Debts<L>) -> Debts<L> {
        let mut debts = std::mem::take(&mut other.0);
        debts.append(&mut self.0);
        Debts(debts)
    }
}

impl<L> Drop for Debts<L> {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            assert!(
                self.0.is_empty(),
                "Debts should be satisfied before being dropped"
            )
        }
    }
}

pub fn gen_function<G>(
    ctx: &mut Context<G>,
    function: &LocalFunction,
    my_label: G::Label,
    function_name: Option<&str>,
) where
    G: LabelGenerator,
{
    let mut locals = HashMap::new();
    let mut wasm_labels = HashMap::new();
    let mut jump_tables = HashMap::new();
    let mut stack = Vec::new();
    let mut ctr: u32 = 0;

    for arg in function.args.iter().rev() {
        let local = ctx.module.locals.get(*arg);
        let words = vt_words(local.ty());
        locals.insert(*arg, ctr);
        ctr = ctr.saturating_add(words);
    }

    build_locals(
        ctx,
        function,
        &mut locals,
        &mut ctr,
        function.block(function.entry_block()),
    );

    if ctr >= 1 << 30 {
        ctx.errors
            .push(CompilationError::Overflow(OverflowLocation::Locals(
                function_name.map(|s| s.to_owned()),
            )));
        return;
    }

    let mut frame: Frame<G::Label> = Frame {
        function,
        function_name,
        locals: &locals,
        jump_targets: &mut wasm_labels,
        jump_tables: &mut jump_tables,
    };

    ctx.rom_items.push(label(my_label));
    ctx.rom_items.push(fnhead_local(ctr));

    let has_explicit_return = matches!(
        function.block(function.entry_block()).instrs.last(),
        Some((Instr::Return(_), _))
    );

    let return_debts = if has_explicit_return {
        Debts::default()
    } else {
        build_debts(
            ctx,
            &mut frame,
            ctx.module.types.get(function.ty()).results(),
            &[],
            true,
        )
    };

    gen_instrseq(
        ctx,
        &mut frame,
        function.block(function.entry_block()),
        &mut stack,
        Credits(Vec::new()),
        return_debts,
    );

    if !has_explicit_return {
        gen_return(ctx, &mut frame);
    }

    for (jump, table) in jump_tables {
        ctx.rom_items.push(label(jump));
        for l in table {
            ctx.rom_items.push(labelref(l));
        }
    }
}

fn build_locals<G>(
    ctx: &mut Context<G>,
    function: &LocalFunction,
    locals: &mut HashMap<LocalId, u32>,
    ctr: &mut u32,
    instrs: &InstrSeq,
) where
    G: LabelGenerator,
{
    for (instr, _) in &instrs.instrs {
        match instr {
            Instr::Block(ir::Block { seq }) | Instr::Loop(ir::Loop { seq }) => {
                build_locals(ctx, function, locals, ctr, function.block(*seq))
            }
            Instr::IfElse(ifelse) => {
                build_locals(
                    ctx,
                    function,
                    locals,
                    ctr,
                    function.block(ifelse.consequent),
                );
                build_locals(
                    ctx,
                    function,
                    locals,
                    ctr,
                    function.block(ifelse.alternative),
                );
            }
            Instr::LocalGet(ir::LocalGet { local: id })
            | Instr::LocalSet(ir::LocalSet { local: id })
            | Instr::LocalTee(ir::LocalTee { local: id }) => {
                let local = ctx.module.locals.get(*id);
                let words = vt_words(local.ty());
                if !locals.contains_key(id) {
                    locals.insert(*id, *ctr);
                    *ctr = ctr.saturating_add(words);
                }
            }
            _ => {}
        }
    }
}

fn gen_instrseq<G>(
    ctx: &mut Context<G>,
    frame: &mut Frame<G::Label>,
    instr_seq: &InstrSeq,
    stack: &mut Vec<ValType>,
    mut initial_credits: Credits<G::Label>,
    mut final_debts: Debts<G::Label>,
) where
    G: LabelGenerator,
{
    let subseqs = super::classify::subsequences(instr_seq);
    let n_subseqs = subseqs.len();

    if n_subseqs > 0 {
        for (i, subseq) in subseqs.into_iter().enumerate() {
            match subseq {
                super::classify::InstrSubseq::Copy { loads, stores, ret } => {
                    let credits = if i == 0 {
                        build_credits(ctx, frame, &loads)
                            .prepend(std::mem::take(&mut initial_credits))
                    } else {
                        build_credits(ctx, frame, &loads)
                    };

                    for load in &loads {
                        load.update_stack(ctx.module, frame.function, stack);
                    }

                    let mut debts = if i == n_subseqs - 1 {
                        build_debts(ctx, frame, stack, &stores, ret.is_some())
                            .prepend(std::mem::take(&mut final_debts))
                    } else {
                        build_debts(ctx, frame, stack, &stores, ret.is_some())
                    };

                    for store in &stores {
                        store.update_stack(ctx.module, frame.function, stack);
                    }

                    if let Some(ret) = ret {
                        // If we're explicitly returning from inside a loop at
                        // the end of a function, there will be duplicate return
                        // debts, one from the function return and one from the
                        // one at the end of the loop. The outer return debts
                        // may have nothing on the stack capable of satisfying
                        // them, so we need to trim these off to prevent
                        // gen_copies from generating pops for them.
                        let return_words: usize = ctx
                            .module
                            .types
                            .get(frame.function.ty())
                            .results()
                            .iter()
                            .map(|vt| vt_words(*vt) as usize)
                            .sum();
                        if debts.0.len() > return_words {
                            debts.0.drain(0..debts.0.len() - return_words);
                            //debts.0 = debts.0.split_off(debts.0.len() - return_words);
                        }
                        gen_copies(ctx, credits, debts);
                        ret.update_stack(ctx.module, frame.function, stack);
                        gen_return(ctx, frame);
                    } else {
                        gen_copies(ctx, credits, debts);
                    }
                }
                super::classify::InstrSubseq::Block { loads, block } => {
                    let credits = if i == 0 {
                        build_credits(ctx, frame, &loads)
                            .prepend(std::mem::take(&mut initial_credits))
                    } else {
                        build_credits(ctx, frame, &loads)
                    };

                    for load in &loads {
                        load.update_stack(ctx.module, frame.function, stack);
                    }

                    let cloned_stack = stack.clone();
                    block.update_stack(ctx.module, frame.function, stack);

                    gen_block(ctx, frame, block, cloned_stack, credits);

                    if i == n_subseqs - 1 {
                        std::mem::take(&mut final_debts).gen(ctx);
                    }
                }
                super::classify::InstrSubseq::Loop { looop, stores, ret } => {
                    if i == 0 {
                        std::mem::take(&mut initial_credits).gen(ctx);
                    }

                    let cloned_stack = stack.clone();
                    looop.update_stack(ctx.module, frame.function, stack);

                    let debts = if i == n_subseqs - 1 {
                        build_debts(ctx, frame, stack, &stores, ret.is_some())
                            .prepend(std::mem::take(&mut final_debts))
                    } else {
                        build_debts(ctx, frame, stack, &stores, ret.is_some())
                    };

                    gen_loop(ctx, frame, looop, cloned_stack, debts);

                    for store in stores {
                        store.update_stack(ctx.module, frame.function, stack);
                    }

                    if let Some(ret) = ret {
                        ret.update_stack(ctx.module, frame.function, stack);
                        gen_return(ctx, frame);
                    }
                }
                super::classify::InstrSubseq::Other {
                    loads,
                    other,
                    stores,
                    ret,
                } => {
                    let credits = if i == 0 {
                        build_credits(ctx, frame, &loads)
                            .prepend(std::mem::take(&mut initial_credits))
                    } else {
                        build_credits(ctx, frame, &loads)
                    };

                    for load in &loads {
                        load.update_stack(ctx.module, frame.function, stack);
                    }

                    let pre_height: usize = stack.iter().map(|vt| vt_words(*vt) as usize).sum();

                    other.update_stack(ctx.module, frame.function, stack);

                    let debts = if i == n_subseqs - 1 {
                        build_debts(ctx, frame, stack, &stores, ret.is_some())
                            .prepend(std::mem::take(&mut final_debts))
                    } else {
                        build_debts(ctx, frame, stack, &stores, ret.is_some())
                    };

                    gen_other(ctx, frame, other, pre_height, stack, credits, debts);

                    for store in stores {
                        store.update_stack(ctx.module, frame.function, stack);
                    }

                    if let Some(ret) = ret {
                        ret.update_stack(ctx.module, frame.function, stack);
                        gen_return(ctx, frame);
                    }
                }
            }
        }
    } else {
        gen_copies(ctx, initial_credits, final_debts);
    }
}

fn gen_return<G>(ctx: &mut Context<G>, frame: &mut Frame<G::Label>)
where
    G: LabelGenerator,
{
    let rwords: u32 = ctx
        .module
        .types
        .get(frame.function.ty())
        .results()
        .iter()
        .copied()
        .map(vt_words)
        .sum();

    if rwords == 0 {
        ctx.rom_items.push(ret(imm(0)));
    } else {
        let rwords_offset = 4 * (rwords - 1);
        ctx.rom_items.push(ret(derefl_off(
            ctx.layout.hi_return().addr.clone(),
            rwords_offset as i32,
        )));
    }
}

fn build_credits<G>(
    ctx: &mut Context<G>,
    frame: &mut Frame<G::Label>,
    loads: &[Load],
) -> Credits<G::Label>
where
    G: LabelGenerator,
{
    let mut credits = Vec::new();

    for load in loads {
        match load {
            Load::LocalGet(ir::LocalGet { local: id }) => {
                let local = ctx.module.locals.get(*id);
                let glulx_local = *frame
                    .locals
                    .get(id)
                    .expect("All locals should have been added to the frame's map.");
                let ty = local.ty();
                match ty {
                    ValType::I32 | ValType::F32 | ValType::Ref(_) => {
                        credits.push(lloc(glulx_local));
                    }
                    ValType::I64 | ValType::F64 => {
                        credits.push(lloc(glulx_local + 1));
                        credits.push(lloc(glulx_local));
                    }
                    ValType::V128 => {
                        credits.push(lloc(glulx_local + 3));
                        credits.push(lloc(glulx_local + 2));
                        credits.push(lloc(glulx_local + 1));
                        credits.push(lloc(glulx_local));
                    }
                }
            }
            Load::GlobalGet(ir::GlobalGet { global: id }) => {
                let global = ctx.module.globals.get(*id);
                let layout = ctx.layout.global(*id);
                match global.ty {
                    ValType::I32 | ValType::F32 | ValType::Ref(_) => {
                        assert_eq!(layout.words, 1);
                        credits.push(derefl(layout.addr.clone()));
                    }
                    ValType::I64 | ValType::F64 => {
                        assert_eq!(layout.words, 2);
                        credits.push(derefl_off(layout.addr.clone(), 4));
                        credits.push(derefl(layout.addr.clone()));
                    }
                    ValType::V128 => {
                        assert_eq!(layout.words, 4);
                        credits.push(derefl_off(layout.addr.clone(), 12));
                        credits.push(derefl_off(layout.addr.clone(), 8));
                        credits.push(derefl_off(layout.addr.clone(), 4));
                        credits.push(derefl(layout.addr.clone()));
                    }
                }
            }
            Load::Const(ir::Const { value }) => match value {
                walrus::ir::Value::I32(x) => {
                    credits.push(imm(*x));
                }
                walrus::ir::Value::I64(x) => {
                    credits.push(imm(*x as i32));
                    credits.push(imm((*x >> 32) as i32));
                }
                walrus::ir::Value::F32(x) => {
                    credits.push(f32_to_imm(*x));
                }
                walrus::ir::Value::F64(x) => {
                    let (hi, lo) = f64_to_imm(*x);
                    credits.push(lo);
                    credits.push(hi);
                }
                walrus::ir::Value::V128(x) => {
                    credits.push(uimm(*x as u32));
                    credits.push(uimm((*x >> 32) as u32));
                    credits.push(uimm((*x >> 64) as u32));
                    credits.push(uimm((*x >> 96) as u32));
                }
            },
            Load::RefNull(ir::RefNull { .. }) => {
                credits.push(imm(0));
            }
            Load::RefFunc(ir::RefFunc { func }) => {
                credits.push(uimm(ctx.layout.func(*func).fnnum));
            }
            Load::TableSize(ir::TableSize { table: id }) => {
                let table = ctx.layout.table(*id);
                let addr = table.cur_count.clone();
                credits.push(derefl(addr));
            }
        }
    }

    Credits(credits)
}

fn build_debts<G>(
    ctx: &mut Context<G>,
    frame: &mut Frame<G::Label>,
    mut stack: &[ValType],
    stores: &[Store],
    then_return: bool,
) -> Debts<G::Label>
where
    G: LabelGenerator,
{
    let mut debts = Vec::new();

    for store in stores {
        let stack_type = *stack
            .last()
            .expect("There should be something on the stack for satisfying debts");
        stack = &stack[..stack.len() - 1];

        match store {
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
                        debts.push(sloc(glulx_local));
                    }
                    ValType::I64 | ValType::F64 => {
                        debts.push(sloc(glulx_local));
                        debts.push(sloc(glulx_local + 1));
                    }
                    ValType::V128 => {
                        debts.push(sloc(glulx_local));
                        debts.push(sloc(glulx_local + 1));
                        debts.push(sloc(glulx_local + 2));
                        debts.push(sloc(glulx_local + 3));
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
                        debts.push(storel(layout.addr.clone()));
                    }
                    ValType::F64 | ValType::I64 => {
                        assert_eq!(layout.words, 2);
                        debts.push(storel(layout.addr.clone()));
                        debts.push(storel_off(layout.addr.clone(), 4));
                    }
                    ValType::V128 => {
                        assert_eq!(layout.words, 4);
                        debts.push(storel(layout.addr.clone()));
                        debts.push(storel_off(layout.addr.clone(), 4));
                        debts.push(storel_off(layout.addr.clone(), 8));
                        debts.push(storel_off(layout.addr.clone(), 12));
                    }
                }
            }
            Store::Drop(_) => {
                for _ in 0..vt_words(stack_type) {
                    debts.push(discard());
                }
            }
        }
    }

    if then_return {
        let ret_types = ctx.module.types.get(frame.function.ty()).results();
        let mut pos = 0;

        for ret_type in ret_types.iter().rev().copied() {
            let stack_type = *stack
                .last()
                .expect("There should be something on the stack for satisfying return debts");
            stack = &stack[..stack.len() - 1];
            assert_eq!(
                ret_type, stack_type,
                "Type on stack should match function return type"
            );

            match ret_type {
                ValType::I32 | ValType::F32 | ValType::Ref(_) => {
                    debts.push(storel_off(ctx.layout.hi_return().addr.clone(), pos));
                    pos += 4;
                }
                ValType::I64 | ValType::F64 => {
                    debts.push(storel_off(ctx.layout.hi_return().addr.clone(), pos));
                    debts.push(storel_off(ctx.layout.hi_return().addr.clone(), pos + 4));
                    pos += 8;
                }
                ValType::V128 => {
                    debts.push(storel_off(ctx.layout.hi_return().addr.clone(), pos));
                    debts.push(storel_off(ctx.layout.hi_return().addr.clone(), pos + 4));
                    debts.push(storel_off(ctx.layout.hi_return().addr.clone(), pos + 8));
                    debts.push(storel_off(ctx.layout.hi_return().addr.clone(), pos + 12));
                    pos += 16;
                }
            }
        }
    }

    debts.reverse();
    Debts(debts)
}

/// Generate an instruction sequence equivalent to pushing all credits and then
/// popping all debts.
///
/// We can optimize to skip the stack in some cases, but have to be careful
/// about stores that overwrite the locations we're loading from. We assume
/// when checking this that labels don't alias each other, which is safe because
/// all labels we're dealing with here are labels of globals and those are
/// never aliased.
pub fn gen_copies<G>(
    ctx: &mut Context<G>,
    mut credits: Credits<G::Label>,
    mut debts: Debts<G::Label>,
) where
    G: LabelGenerator,
{
    let mut poisoned: HashSet<LoadOperand<G::Label>> = HashSet::new();
    let mut good_pairs = Vec::new();
    while !debts.is_empty() && !credits.is_empty() {
        let load = credits.pop();
        let store = debts.pop();

        let poison = match &store {
            StoreOperand::Push => unreachable!("A push operand cannot be a debt"),
            StoreOperand::Discard => None,
            StoreOperand::DerefLabel(addr) => Some(LoadOperand::DerefLabel(addr.clone())),
            StoreOperand::FrameAddr(addr) => Some(LoadOperand::FrameAddr(*addr)),
        };

        if let Some(poison) = poison {
            if poisoned.contains(&poison) {
                credits.0.push(load);
                debts.0.push(store);
                break;
            }

            poisoned.insert(poison);
        }
        good_pairs.push((load, store));
    }

    credits.gen(ctx);
    for (load, store) in good_pairs {
        ctx.rom_items.push(copy(load, store));
    }
    debts.gen(ctx);
}

fn gen_block<G>(
    ctx: &mut Context<G>,
    frame: &mut Frame<G::Label>,
    block: Block,
    mut stack: Vec<ValType>,
    credits: Credits<G::Label>,
) where
    G: LabelGenerator,
{
    let (params, results) = block.stack_type(ctx.module, frame.function, stack.as_slice());
    let stack_height: usize = stack.iter().map(|vt| vt_words(*vt) as usize).sum();
    let param_len: usize = params.iter().map(|vt| vt_words(*vt) as usize).sum();
    let arity: usize = results.iter().map(|vt| vt_words(*vt) as usize).sum();
    let base = stack_height - param_len;

    let target = ctx.gen.gen("endblock");

    match &block {
        Block::Block(ir::Block { seq: id }) => {
            frame.jump_targets.insert(
                *id,
                JumpTarget {
                    base,
                    arity,
                    target: target.clone(),
                },
            );
            let seq = frame.function.block(*id);
            gen_instrseq(ctx, frame, seq, &mut stack, credits, Debts::default());
        }
        Block::IfElse(
            test,
            ir::IfElse {
                consequent: cid,
                alternative: aid,
            },
        ) => {
            frame.jump_targets.insert(
                *cid,
                JumpTarget {
                    base,
                    arity,
                    target: target.clone(),
                },
            );
            frame.jump_targets.insert(
                *aid,
                JumpTarget {
                    base,
                    arity,
                    target: target.clone(),
                },
            );
            let test_target = ctx.gen.gen("consequent");
            super::control::gen_test(ctx, *test, test_target.clone(), credits);
            test.update_stack(ctx.module, frame.function, &mut stack);
            let mut cloned_stack = stack.clone();
            let alternative = frame.function.block(*aid);
            gen_instrseq(
                ctx,
                frame,
                alternative,
                &mut stack,
                Credits::default(),
                Debts::default(),
            );
            ctx.rom_items.push(jump(target.clone()));
            ctx.rom_items.push(label(test_target));
            let consequent = frame.function.block(*cid);
            gen_instrseq(
                ctx,
                frame,
                consequent,
                &mut cloned_stack,
                Credits::default(),
                Debts::default(),
            );
        }
    }

    ctx.rom_items.push(label(target));
}

fn gen_loop<G>(
    ctx: &mut Context<G>,
    frame: &mut Frame<G::Label>,
    looop: Loop,
    mut stack: Vec<ValType>,
    debts: Debts<G::Label>,
) where
    G: LabelGenerator,
{
    let Loop::Loop(ir::Loop { seq: id }) = looop;
    let seq = frame.function.block(id);
    let (params, _) = looop.stack_type(ctx.module, frame.function, stack.as_slice());

    let arity: usize = params.iter().map(|vt| vt_words(*vt) as usize).sum();
    let stack_height: usize = stack.iter().map(|vt| vt_words(*vt) as usize).sum();
    let base: usize = stack_height - arity;
    let target = ctx.gen.gen("loop");
    frame.jump_targets.insert(
        id,
        JumpTarget {
            base,
            arity,
            target: target.clone(),
        },
    );
    ctx.rom_items.push(label(target));
    gen_instrseq(ctx, frame, seq, &mut stack, Credits::default(), debts);
}

fn gen_other<G>(
    ctx: &mut Context<G>,
    frame: &mut Frame<G::Label>,
    other: Other,
    pre_height: usize,
    post_stack: &[ValType],
    credits: Credits<G::Label>,
    debts: Debts<G::Label>,
) where
    G: LabelGenerator,
{
    match &other {
        Other::Br(br) => {
            super::control::gen_br(ctx, frame, br, pre_height, credits, debts);
        }
        Other::BrIf(test, br_if) => {
            super::control::gen_br_if(ctx, frame, *test, br_if, pre_height, credits, debts);
        }
        Other::BrTable(br_table) => {
            super::control::gen_br_table(ctx, frame, br_table, pre_height, credits, debts);
        }
        Other::Call(call) => {
            super::control::gen_call(ctx, frame, call, credits, debts);
        }
        Other::Select(test, select) => {
            super::control::gen_select(ctx, frame, *test, select, post_stack, credits, debts);
        }
        _ => {
            credits.gen(ctx);
            ctx.errors.push(CompilationError::UnsupportedInstruction {
                function: frame.function_name.map(|s| s.to_owned()),
                instr: other.mnemonic(),
            });
            debts.gen(ctx);
        }
    }
}
