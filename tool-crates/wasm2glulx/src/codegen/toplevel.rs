// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
// Copyright 2024 Daniel Fox Franke.

use glulx_asm::concise::*;
use std::collections::HashMap;
use walrus::ir::{self, InstrSeq, InstrSeqId};
use walrus::{LocalFunction, LocalId, ValType};

use crate::common::{Context, Label, WordCount};
use crate::{CompilationError, OverflowLocation};

use super::classify::{
    subsequences, Block, ClassifiedInstr, InstrSubseq, Load, Loop, Other, Store, Terminal,
};
use super::loadstore::{gen_copies, Credits, Debts};

pub struct Frame<'a> {
    pub function: &'a LocalFunction,
    pub function_name: Option<&'a str>,
    pub locals: &'a HashMap<LocalId, u32>,
    pub jump_targets: &'a mut HashMap<InstrSeqId, JumpTarget>,
    pub jump_tables: &'a mut HashMap<Label, Vec<Label>>,
}
pub struct JumpTarget {
    pub base: usize,
    pub arity: usize,
    pub target: Label,
}

struct LocalsBuilder<'a> {
    ctx: &'a Context<'a>,
    locals: &'a mut HashMap<LocalId, u32>,
    ctr: &'a mut u32,
}

impl<'a> ir::Visitor<'_> for LocalsBuilder<'a> {
    fn visit_local_id(&mut self, id: &LocalId) {
        let local = self.ctx.module.locals.get(*id);
        let words = local.ty().word_count();
        if !self.locals.contains_key(id) {
            self.locals.insert(*id, *self.ctr);
            *self.ctr = self.ctr.saturating_add(words);
        }
    }
}

struct BranchToEntrySearcher {
    found: bool,
    entry: InstrSeqId,
}

impl ir::Visitor<'_> for BranchToEntrySearcher {
    fn visit_br(&mut self, instr: &ir::Br) {
        if instr.block == self.entry {
            self.found = true;
        }
    }

    fn visit_br_if(&mut self, instr: &ir::BrIf) {
        if instr.block == self.entry {
            self.found = true;
        }
    }

    fn visit_br_table(&mut self, instr: &ir::BrTable) {
        for block in &instr.blocks {
            if *block == self.entry {
                self.found = true;
            }
        }

        if instr.default == self.entry {
            self.found = true;
        }
    }
}

pub fn gen_function(
    ctx: &mut Context,
    function: &LocalFunction,
    my_label: Label,
    function_name: Option<&str>,
) {
    let mut locals = HashMap::new();
    let mut wasm_labels = HashMap::new();
    let mut jump_tables = HashMap::new();
    let mut ctr: u32 = 0;

    for arg in function.args.iter().rev() {
        let local = ctx.module.locals.get(*arg);
        let words = local.ty().word_count();
        locals.insert(*arg, ctr);
        ctr = ctr.saturating_add(words);
    }

    let mut locals_builder = LocalsBuilder {
        ctx,
        locals: &mut locals,
        ctr: &mut ctr,
    };

    ir::dfs_in_order(&mut locals_builder, function, function.entry_block());

    if ctr >= 1 << 30 {
        ctx.errors
            .push(CompilationError::Overflow(OverflowLocation::Locals(
                function_name.map(|s| s.to_owned()),
            )));
        return;
    }

    let mut frame = Frame {
        function,
        function_name,
        locals: &locals,
        jump_targets: &mut wasm_labels,
        jump_tables: &mut jump_tables,
    };

    ctx.rom_items.push(label(my_label));
    ctx.rom_items.push(fnhead_local(ctr));

    let mut branch_to_entry_searcher = BranchToEntrySearcher {
        found: false,
        entry: function.entry_block(),
    };

    ir::dfs_in_order(
        &mut branch_to_entry_searcher,
        function,
        function.entry_block(),
    );

    let mut debts = Debts::new(
        ctx,
        &frame,
        ctx.module.types.get(function.ty()).results(),
        &[],
        true,
    );
    if branch_to_entry_searcher.found {
        let block = Block::Block(ir::Block {
            seq: function.entry_block(),
        });
        gen_block(ctx, &mut frame, block, vec![], Credits::empty());
        debts.gen(ctx);
    } else {
        let mut stack = Vec::new();
        let block = frame.function.block(function.entry_block());
        gen_instrseq(ctx, &mut frame, block, &mut stack, Credits::empty(), debts);
    }

    for (jump, table) in jump_tables {
        ctx.rom_items.push(label(jump));
        for l in table {
            ctx.rom_items.push(labelref(l));
        }
    }
}

fn make_credits(
    ctx: &Context,
    frame: &Frame,
    initial: &mut Credits,
    load_instrs: &[Load],
    use_initial: bool,
) -> Credits {
    if use_initial {
        let mut credits = initial.take();
        let more_credits = Credits::new(ctx, frame, load_instrs);
        credits.append_later(more_credits);
        credits
    } else {
        Credits::new(ctx, frame, load_instrs)
    }
}

fn make_debts(
    ctx: &Context,
    frame: &Frame,
    stack: &[ValType],
    final_debts: &mut Debts,
    store_instrs: &[Store],
    then_return: bool,
    use_final: bool,
) -> Debts {
    if use_final {
        let mut debts = final_debts.take();
        let more_debts = Debts::new(ctx, frame, stack, store_instrs, then_return);
        debts.append_earlier(more_debts);
        debts
    } else {
        Debts::new(ctx, frame, stack, store_instrs, then_return)
    }
}

fn gen_instrseq(
    ctx: &mut Context,
    frame: &mut Frame,
    instr_seq: &InstrSeq,
    stack: &mut Vec<ValType>,
    mut initial_credits: Credits,
    mut final_debts: Debts,
) {
    let subseqs = subsequences(instr_seq);
    let n_subseqs = subseqs.len();

    if n_subseqs == 0 {
        gen_copies(ctx, initial_credits, final_debts);
        return;
    }

    for (i, subseq) in subseqs.into_iter().enumerate() {
        match subseq {
            InstrSubseq::Copy { loads, stores, ret } => {
                let credits = make_credits(ctx, frame, &mut initial_credits, &loads, i == 0);
                for load in &loads {
                    load.update_stack(ctx.module, frame.function, stack);
                }
                let debts = make_debts(
                    ctx,
                    frame,
                    stack,
                    &mut final_debts,
                    &stores,
                    ret.is_some(),
                    i == n_subseqs - 1,
                );
                gen_copies(ctx, credits, debts);
                for store in &stores {
                    store.update_stack(ctx.module, frame.function, stack);
                }
                if let Some(ret) = ret {
                    ret.update_stack(ctx.module, frame.function, stack);
                }
            }
            InstrSubseq::Block { loads, block } => {
                let credits = make_credits(ctx, frame, &mut initial_credits, &loads, i == 0);
                for load in &loads {
                    load.update_stack(ctx.module, frame.function, stack);
                }
                let cloned_stack = stack.clone();
                block.update_stack(ctx.module, frame.function, stack);
                gen_block(ctx, frame, block, cloned_stack, credits);
                if i == n_subseqs - 1 {
                    final_debts.gen(ctx);
                }
            }
            InstrSubseq::Loop { looop, stores, ret } => {
                if i == 0 {
                    initial_credits.gen(ctx);
                }
                let cloned_stack = stack.clone();
                looop.update_stack(ctx.module, frame.function, stack);
                let debts = make_debts(
                    ctx,
                    frame,
                    stack,
                    &mut final_debts,
                    &stores,
                    ret.is_some(),
                    i == n_subseqs - 1,
                );

                gen_loop(ctx, frame, looop, cloned_stack, debts);
                for store in &stores {
                    store.update_stack(ctx.module, frame.function, stack);
                }
                if let Some(ret) = ret {
                    ret.update_stack(ctx.module, frame.function, stack);
                }
            }
            InstrSubseq::Other {
                loads,
                other,
                stores,
                ret,
            } => {
                let credits = make_credits(ctx, frame, &mut initial_credits, &loads, i == 0);
                for load in &loads {
                    load.update_stack(ctx.module, frame.function, stack);
                }

                let pre_height: usize = stack.word_count();
                other.update_stack(ctx.module, frame.function, stack);

                let debts = make_debts(
                    ctx,
                    frame,
                    stack,
                    &mut final_debts,
                    &stores,
                    ret.is_some(),
                    i == n_subseqs - 1,
                );

                gen_other(ctx, frame, other, pre_height, stack, credits, debts);
                for store in &stores {
                    store.update_stack(ctx.module, frame.function, stack);
                }
                if let Some(ret) = ret {
                    ret.update_stack(ctx.module, frame.function, stack);
                }
            }
            InstrSubseq::Terminal { loads, terminal } => {
                let credits = make_credits(ctx, frame, &mut initial_credits, &loads, i == 0);
                for load in &loads {
                    load.update_stack(ctx.module, frame.function, stack);
                }
                let pre_height: usize = stack.word_count();
                terminal.update_stack(ctx.module, frame.function, stack);
                gen_terminal(ctx, frame, terminal, pre_height, credits);
                final_debts.declare_bankruptcy();
                return;
            }
        }
    }
}

fn gen_block(
    ctx: &mut Context,
    frame: &mut Frame,
    block: Block,
    mut stack: Vec<ValType>,
    credits: Credits,
) {
    let (params, results) = block.stack_type(ctx.module, frame.function, stack.as_slice());
    let stack_height: usize = stack.word_count();
    let param_len: usize = params.word_count();
    let arity: usize = results.word_count();
    let base = stack_height - param_len;

    let target = ctx.gen.gen("endblock");

    match &block {
        Block::Block(ir::Block { seq: id }) => {
            frame.jump_targets.insert(
                *id,
                JumpTarget {
                    base,
                    arity,
                    target,
                },
            );
            let seq = frame.function.block(*id);
            gen_instrseq(ctx, frame, seq, &mut stack, credits, Debts::empty());
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
                    target,
                },
            );
            frame.jump_targets.insert(
                *aid,
                JumpTarget {
                    base,
                    arity,
                    target,
                },
            );
            let test_target = ctx.gen.gen("consequent");
            super::control::gen_test(ctx, *test, test_target, credits);
            test.update_stack(ctx.module, frame.function, &mut stack);
            let mut cloned_stack = stack.clone();
            let alternative = frame.function.block(*aid);
            gen_instrseq(
                ctx,
                frame,
                alternative,
                &mut stack,
                Credits::empty(),
                Debts::empty(),
            );
            ctx.rom_items.push(jump(target));
            ctx.rom_items.push(label(test_target));
            let consequent = frame.function.block(*cid);
            gen_instrseq(
                ctx,
                frame,
                consequent,
                &mut cloned_stack,
                Credits::empty(),
                Debts::empty(),
            );
        }
    }

    ctx.rom_items.push(label(target));
}

fn gen_loop(
    ctx: &mut Context,
    frame: &mut Frame,
    looop: Loop,
    mut stack: Vec<ValType>,
    debts: Debts,
) {
    let Loop::Loop(ir::Loop { seq: id }) = looop;
    let seq = frame.function.block(id);
    let (params, _) = looop.stack_type(ctx.module, frame.function, stack.as_slice());

    let arity: usize = params.word_count();
    let stack_height: usize = stack.word_count();
    let base: usize = stack_height - arity;
    let target = ctx.gen.gen("loop");
    frame.jump_targets.insert(
        id,
        JumpTarget {
            base,
            arity,
            target,
        },
    );
    ctx.rom_items.push(label(target));
    gen_instrseq(ctx, frame, seq, &mut stack, Credits::empty(), debts);
}

fn gen_other(
    ctx: &mut Context,
    frame: &mut Frame,
    other: Other,
    pre_height: usize,
    post_stack: &[ValType],
    mut credits: Credits,
    mut debts: Debts,
) {
    match &other {
        Other::Binop(binop) => {
            super::arith::gen_binop(ctx, frame, binop, credits, debts);
        }
        Other::BrIf(test, br_if) => {
            super::control::gen_br_if(ctx, frame, *test, br_if, pre_height, credits, debts);
        }
        Other::Call(call) => {
            super::control::gen_call(ctx, frame, call, credits, debts);
        }
        Other::CallIndirect(call_indirect) => {
            super::control::gen_call_indirect(ctx, frame, call_indirect, credits, debts);
        }
        Other::DataDrop(data_drop) => {
            super::memory::gen_data_drop(ctx, frame, data_drop, credits, debts);
        }
        Other::ElemDrop(elem_drop) => {
            super::table::gen_elem_drop(ctx, frame, elem_drop, credits, debts);
        }
        Other::Load(load) => {
            super::memory::gen_load(ctx, frame, load, credits, debts);
        }
        Other::LocalTee(local_tee) => {
            super::loadstore::gen_local_tee(ctx, frame, local_tee, credits, debts);
        }
        Other::MemoryCopy(memory_copy) => {
            super::memory::gen_memory_copy(ctx, frame, memory_copy, credits, debts);
        }
        Other::MemoryGrow(memory_grow) => {
            super::memory::gen_memory_grow(ctx, frame, memory_grow, credits, debts);
        }
        Other::MemoryFill(memory_fill) => {
            super::memory::gen_memory_fill(ctx, frame, memory_fill, credits, debts);
        }
        Other::MemoryInit(memory_init) => {
            super::memory::gen_memory_init(ctx, frame, memory_init, credits, debts);
        }
        Other::MemorySize(memory_size) => {
            super::memory::gen_memory_size(ctx, frame, memory_size, credits, debts);
        }
        Other::RefIsNull(_ref_is_null) => {
            // This doesn't fit into any good category so just implement it directly.
            let r = credits.pop();
            let out = debts.pop();
            credits.gen(ctx);
            ctx.rom_items.push(callfi(imml(ctx.rt.i32_eqz), r, out));
            debts.gen(ctx);
        }
        Other::Select(test, select) => {
            super::control::gen_select(ctx, frame, *test, select, post_stack, credits, debts);
        }
        Other::Store(store) => {
            super::memory::gen_store(ctx, frame, store, credits, debts);
        }
        Other::TableCopy(table_copy) => {
            super::table::gen_table_copy(ctx, frame, table_copy, credits, debts);
        }
        Other::TableFill(table_fill) => {
            super::table::gen_table_fill(ctx, frame, table_fill, credits, debts);
        }
        Other::TableGet(table_get) => {
            super::table::gen_table_get(ctx, frame, table_get, credits, debts);
        }
        Other::TableGrow(table_grow) => {
            super::table::gen_table_grow(ctx, frame, table_grow, credits, debts);
        }
        Other::TableInit(table_init) => {
            super::table::gen_table_init(ctx, frame, table_init, credits, debts);
        }
        Other::TableSet(table_set) => {
            super::table::gen_table_set(ctx, frame, table_set, credits, debts);
        }
        Other::Unop(unop) => {
            super::arith::gen_unop(ctx, frame, unop, credits, debts);
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

fn gen_terminal(
    ctx: &mut Context,
    frame: &mut Frame,
    terminal: Terminal,
    pre_height: usize,
    mut credits: Credits,
) {
    match &terminal {
        Terminal::Br(br) => {
            super::control::gen_br(ctx, frame, br, pre_height, credits);
        }
        Terminal::BrTable(br_table) => {
            super::control::gen_br_table(ctx, frame, br_table, pre_height, credits);
        }
        Terminal::Unreachable(unreachable) => {
            super::control::gen_unreachable(ctx, frame, unreachable, credits);
        }
        _ => {
            credits.gen(ctx);
            ctx.errors.push(CompilationError::UnsupportedInstruction {
                function: frame.function_name.map(|s| s.to_owned()),
                instr: terminal.mnemonic(),
            });
        }
    }
}
