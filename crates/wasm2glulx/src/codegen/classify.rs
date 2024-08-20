use walrus::ir::{self, ExtendedLoad};
use walrus::{LocalFunction, Module, RefType, ValType};

pub trait ClassifiedInstr {
    fn mnemonic(&self) -> &'static str;
    fn stack_type<'m, 's>(
        &self,
        module: &'m Module,
        localfn: &LocalFunction,
        cur_stack: &'s [ValType],
    ) -> (&'s [ValType], &'m [ValType])
    where
        'm: 's;

    fn update_stack(&self, module: &Module, localfn: &LocalFunction, stack: &mut Vec<ValType>) {
        let (params, results) = self.stack_type(module, localfn, stack);
        assert!(
            stack.len() >= params.len(),
            "Stack must contain at least as many values as are being popped"
        );
        let popped_height = stack.len() - params.len();
        assert_eq!(
            &stack[popped_height..],
            params,
            "Types on stack must match what is being popped"
        );
        stack.truncate(popped_height);
        stack.extend_from_slice(results);
    }
}
pub enum InstrClass {
    Load(Load),
    Store(Store),
    Ret(Ret),
    Block(Block),
    Loop(Loop),
    Other(Other),
}

#[derive(Debug, Clone)]
pub enum Load {
    LocalGet(ir::LocalGet),
    GlobalGet(ir::GlobalGet),
    Const(ir::Const),
    RefNull(ir::RefNull),
    RefFunc(ir::RefFunc),
    TableSize(ir::TableSize),
}

#[derive(Debug, Clone)]
pub enum Store {
    LocalSet(ir::LocalSet),
    GlobalSet(ir::GlobalSet),
    Drop(ir::Drop),
}

#[derive(Debug, Clone)]
pub enum Ret {
    Return(ir::Return),
}

#[derive(Debug, Clone)]
pub enum Block {
    Block(ir::Block),
    IfElse(Test, ir::IfElse),
}

#[derive(Debug, Clone)]
pub enum Loop {
    Loop(ir::Loop),
}

#[derive(Debug, Clone)]
pub enum Other {
    Call(ir::Call),
    CallIndirect(ir::CallIndirect),
    LocalTee(ir::LocalTee),
    Binop(ir::Binop),
    Unop(ir::Unop),
    Select(Test, ir::Select),
    Unreachable(ir::Unreachable),
    Br(ir::Br),
    BrIf(Test, ir::BrIf),
    BrTable(ir::BrTable),
    MemorySize(ir::MemorySize),
    MemoryGrow(ir::MemoryGrow),
    MemoryInit(ir::MemoryInit),
    DataDrop(ir::DataDrop),
    MemoryCopy(ir::MemoryCopy),
    MemoryFill(ir::MemoryFill),
    Load(ir::Load),
    Store(ir::Store),
    AtomicRmw(ir::AtomicRmw),
    Cmpxchg(ir::Cmpxchg),
    AtomicNotify(ir::AtomicNotify),
    AtomicWait(ir::AtomicWait),
    AtomicFence(ir::AtomicFence),
    TableGet(ir::TableGet),
    TableSet(ir::TableSet),
    TableGrow(ir::TableGrow),
    TableFill(ir::TableFill),
    RefIsNull(ir::RefIsNull),
    V128BitSelect(ir::V128Bitselect),
    I8x16Swizzle(ir::I8x16Swizzle),
    I8x16Shuffle(ir::I8x16Shuffle),
    LoadSimd(ir::LoadSimd),
    TableInit(ir::TableInit),
    ElemDrop(ir::ElemDrop),
    TableCopy(ir::TableCopy),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Test {
    I32Nez,
    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtS,
    I32LtU,
    I32GtS,
    I32GtU,
    I32LeS,
    I32LeU,
    I32GeS,
    I32GeU,
    F32Eq,
    F32Ne,
    F32Lt,
    F32Gt,
    F32Le,
    F32Ge,
}

pub enum InstrSubseq {
    Copy {
        loads: Vec<Load>,
        stores: Vec<Store>,
        ret: Option<Ret>,
    },
    Block {
        loads: Vec<Load>,
        block: Block,
    },
    Loop {
        looop: Loop,
        stores: Vec<Store>,
        ret: Option<Ret>,
    },
    Other {
        loads: Vec<Load>,
        other: Other,
        stores: Vec<Store>,
        ret: Option<Ret>,
    },
}

fn vt_singleton(vt: ValType) -> &'static [ValType] {
    match vt {
        ValType::I32 => &[ValType::I32],
        ValType::I64 => &[ValType::I64],
        ValType::F32 => &[ValType::F32],
        ValType::F64 => &[ValType::F64],
        ValType::V128 => &[ValType::V128],
        ValType::Ref(RefType::Funcref) => &[ValType::Ref(RefType::Funcref)],
        ValType::Ref(_) => &[ValType::Ref(RefType::Externref)],
    }
}

pub fn instrseq_type<'m>(
    module: &'m Module,
    localfn: &LocalFunction,
    seqid: ir::InstrSeqId,
) -> (&'m [ValType], &'m [ValType]) {
    match localfn.block(seqid).ty {
        ir::InstrSeqType::Simple(None) => (&[], &[]),
        ir::InstrSeqType::Simple(Some(vt)) => (&[], vt_singleton(vt)),
        ir::InstrSeqType::MultiValue(id) => {
            let ty = module.types.get(id);
            (ty.params(), ty.results())
        }
    }
}

impl ClassifiedInstr for InstrClass {
    fn stack_type<'m, 's>(
        &self,
        module: &'m Module,
        localfn: &LocalFunction,
        cur_stack: &'s [ValType],
    ) -> (&'s [ValType], &'m [ValType])
    where
        'm: 's {
        match self {
            InstrClass::Load(load) => load.stack_type(module, localfn, cur_stack),
            InstrClass::Store(store) => store.stack_type(module, localfn, cur_stack),
            InstrClass::Ret(ret) => ret.stack_type(module, localfn, cur_stack),
            InstrClass::Block(block) => block.stack_type(module, localfn, cur_stack),
            InstrClass::Loop(l) => l.stack_type(module, localfn, cur_stack),
            InstrClass::Other(other) => other.stack_type(module, localfn, cur_stack),
        }
    }

    fn mnemonic(&self) -> &'static str {
        match self {
            InstrClass::Load(load) => load.mnemonic(),
            InstrClass::Store(store) => store.mnemonic(),
            InstrClass::Ret(ret) => ret.mnemonic(),
            InstrClass::Block(block) => block.mnemonic(),
            InstrClass::Loop(looop) => looop.mnemonic(),
            InstrClass::Other(other) => other.mnemonic(),
        }
    }
}

impl ClassifiedInstr for Load {
    fn stack_type<'m, 's>(
        &self,
        module: &'m Module,
        _localfn: &LocalFunction,
        _cur_stack: &'s [ValType],
    ) -> (&'s [ValType], &'m [ValType])
    where
        'm: 's,
    {
        (
            &[],
            match self {
                Load::LocalGet(local_get) => vt_singleton(module.locals.get(local_get.local).ty()),
                Load::GlobalGet(global_get) => {
                    vt_singleton(module.globals.get(global_get.global).ty)
                }
                Load::Const(c) => match c.value {
                    ir::Value::I32(_) => vt_singleton(ValType::I32),
                    ir::Value::I64(_) => vt_singleton(ValType::I64),
                    ir::Value::F32(_) => vt_singleton(ValType::F32),
                    ir::Value::F64(_) => vt_singleton(ValType::F64),
                    ir::Value::V128(_) => vt_singleton(ValType::V128),
                },
                Load::RefNull(ref_null) => vt_singleton(ValType::Ref(ref_null.ty)),
                Load::RefFunc(_) => vt_singleton(ValType::Ref(RefType::Funcref)),
                Load::TableSize(_) => vt_singleton(ValType::I32),
            },
        )
    }

    fn mnemonic(&self) -> &'static str {
        match self {
            Load::LocalGet(_) => "local.get",
            Load::GlobalGet(_) => "global.get",
            Load::Const(_) => "const",
            Load::RefNull(_) => "ref.null",
            Load::RefFunc(_) => "ref.func",
            Load::TableSize(_) => "table.size",
        }
    }
}

impl ClassifiedInstr for Store {
    fn stack_type<'m, 's>(
        &self,
        module: &'m Module,
        _localfn: &LocalFunction,
        cur_stack: &'s [ValType],
    ) -> (&'s [ValType], &'m [ValType])
    where
        'm: 's,
    {
        (
            match self {
                Store::LocalSet(local_set) => vt_singleton(module.locals.get(local_set.local).ty()),
                Store::GlobalSet(global_set) => {
                    vt_singleton(module.globals.get(global_set.global).ty)
                }
                Store::Drop(_) => vt_singleton(*cur_stack.last().expect(
                    "Module validation should have prevented dropping from an empty stack",
                )),
            },
            &[],
        )
    }

    fn mnemonic(&self) -> &'static str {
        match self {
            Store::LocalSet(_) => "local.set",
            Store::GlobalSet(_) => "global.set",
            Store::Drop(_) => "drop",
        }
    }
}

impl ClassifiedInstr for Ret {
    fn stack_type<'m, 's>(
        &self,
        module: &'m Module,
        localfn: &LocalFunction,
        _cur_stack: &'s [ValType],
    ) -> (&'s [ValType], &'m [ValType])
    where
        'm: 's,
    {
        (module.types.get(localfn.ty()).results(), &[])
    }

    fn mnemonic(&self) -> &'static str {
        "ret"
    }
}

impl ClassifiedInstr for Block {
    fn stack_type<'m, 's>(
        &self,
        module: &'m Module,
        localfn: &LocalFunction,
        _cur_stack: &'s [ValType],
    ) -> (&'s [ValType], &'m [ValType])
    where
        'm: 's,
    {
        match self {
            Block::Block(block) => instrseq_type(module, localfn, block.seq),
            Block::IfElse(_, ifelse) => instrseq_type(module, localfn, ifelse.consequent),
        }
    }
    fn mnemonic(&self) -> &'static str {
        match self {
            Block::Block(_) => "block",
            Block::IfElse(_, _) => "if",
        }
    }
}

impl ClassifiedInstr for Loop {
    fn stack_type<'m, 's>(
        &self,
        module: &'m Module,
        localfn: &LocalFunction,
        _cur_stack: &'s [ValType],
    ) -> (&'s [ValType], &'m [ValType])
    where
        'm: 's,
    {
        match self {
            Loop::Loop(l) => instrseq_type(module, localfn, l.seq),
        }
    }

    fn mnemonic(&self) -> &'static str {
        "loop"
    }
}

impl ClassifiedInstr for Other {
    fn stack_type<'m, 's>(
        &self,
        module: &'m Module,
        localfn: &LocalFunction,
        cur_stack: &'s [ValType],
    ) -> (&'s [ValType], &'m [ValType])
    where
        'm: 's,
    {
        match self {
            Other::Call(call) => {
                let ty = module.types.get(module.funcs.get(call.func).ty());
                (ty.params(), ty.results())
            }
            Other::CallIndirect(call_indirect) => {
                let ty = module.types.get(call_indirect.ty);
                (ty.params(), ty.results())
            }
            Other::LocalTee(tee) => {
                let ty = module.locals.get(tee.local).ty();
                (vt_singleton(ty), vt_singleton(ty))
            }
            Other::Binop(binop) => match binop.op {
                ir::BinaryOp::I32Eq
                | ir::BinaryOp::I32Ne
                | ir::BinaryOp::I32LtS
                | ir::BinaryOp::I32LtU
                | ir::BinaryOp::I32GtS
                | ir::BinaryOp::I32GtU
                | ir::BinaryOp::I32LeS
                | ir::BinaryOp::I32LeU
                | ir::BinaryOp::I32GeS
                | ir::BinaryOp::I32GeU
                | ir::BinaryOp::I32Add
                | ir::BinaryOp::I32Sub
                | ir::BinaryOp::I32Mul
                | ir::BinaryOp::I32DivS
                | ir::BinaryOp::I32DivU
                | ir::BinaryOp::I32RemS
                | ir::BinaryOp::I32RemU
                | ir::BinaryOp::I32And
                | ir::BinaryOp::I32Or
                | ir::BinaryOp::I32Xor
                | ir::BinaryOp::I32Shl
                | ir::BinaryOp::I32ShrS
                | ir::BinaryOp::I32ShrU
                | ir::BinaryOp::I32Rotl
                | ir::BinaryOp::I32Rotr => (&[ValType::I32, ValType::I32], &[ValType::I32]),
                ir::BinaryOp::I64Eq
                | ir::BinaryOp::I64Ne
                | ir::BinaryOp::I64LtS
                | ir::BinaryOp::I64LtU
                | ir::BinaryOp::I64GtS
                | ir::BinaryOp::I64GtU
                | ir::BinaryOp::I64LeS
                | ir::BinaryOp::I64LeU
                | ir::BinaryOp::I64GeS
                | ir::BinaryOp::I64GeU => (&[ValType::I64, ValType::I64], &[ValType::I32]),
                ir::BinaryOp::F32Eq
                | ir::BinaryOp::F32Ne
                | ir::BinaryOp::F32Lt
                | ir::BinaryOp::F32Gt
                | ir::BinaryOp::F32Le
                | ir::BinaryOp::F32Ge => (&[ValType::F32, ValType::F32], &[ValType::I32]),
                ir::BinaryOp::F64Eq
                | ir::BinaryOp::F64Ne
                | ir::BinaryOp::F64Lt
                | ir::BinaryOp::F64Gt
                | ir::BinaryOp::F64Le
                | ir::BinaryOp::F64Ge => (&[ValType::F64, ValType::F64], &[ValType::I32]),
                ir::BinaryOp::I64Add
                | ir::BinaryOp::I64Sub
                | ir::BinaryOp::I64Mul
                | ir::BinaryOp::I64DivS
                | ir::BinaryOp::I64DivU
                | ir::BinaryOp::I64RemS
                | ir::BinaryOp::I64RemU
                | ir::BinaryOp::I64And
                | ir::BinaryOp::I64Or
                | ir::BinaryOp::I64Xor
                | ir::BinaryOp::I64Shl
                | ir::BinaryOp::I64ShrS
                | ir::BinaryOp::I64ShrU
                | ir::BinaryOp::I64Rotl
                | ir::BinaryOp::I64Rotr => (&[ValType::I64, ValType::I64], &[ValType::I64]),
                ir::BinaryOp::F32Add
                | ir::BinaryOp::F32Sub
                | ir::BinaryOp::F32Mul
                | ir::BinaryOp::F32Div
                | ir::BinaryOp::F32Min
                | ir::BinaryOp::F32Max
                | ir::BinaryOp::F32Copysign => (&[ValType::F32, ValType::F32], &[ValType::F32]),
                ir::BinaryOp::F64Add
                | ir::BinaryOp::F64Sub
                | ir::BinaryOp::F64Mul
                | ir::BinaryOp::F64Div
                | ir::BinaryOp::F64Min
                | ir::BinaryOp::F64Max
                | ir::BinaryOp::F64Copysign => (&[ValType::F64, ValType::F64], &[ValType::F64]),
                ir::BinaryOp::I8x16ReplaceLane { .. }
                | ir::BinaryOp::I16x8ReplaceLane { .. }
                | ir::BinaryOp::I32x4ReplaceLane { .. } => {
                    (&[ValType::V128, ValType::I32], &[ValType::V128])
                }
                ir::BinaryOp::I64x2ReplaceLane { .. } => {
                    (&[ValType::V128, ValType::I64], &[ValType::V128])
                }
                ir::BinaryOp::F32x4ReplaceLane { .. } => {
                    (&[ValType::V128, ValType::F32], &[ValType::V128])
                }
                ir::BinaryOp::F64x2ReplaceLane { .. } => {
                    (&[ValType::V128, ValType::F64], &[ValType::V128])
                }
                ir::BinaryOp::I8x16Eq
                | ir::BinaryOp::I8x16Ne
                | ir::BinaryOp::I8x16LtS
                | ir::BinaryOp::I8x16LtU
                | ir::BinaryOp::I8x16GtS
                | ir::BinaryOp::I8x16GtU
                | ir::BinaryOp::I8x16LeS
                | ir::BinaryOp::I8x16LeU
                | ir::BinaryOp::I8x16GeS
                | ir::BinaryOp::I8x16GeU
                | ir::BinaryOp::I16x8Eq
                | ir::BinaryOp::I16x8Ne
                | ir::BinaryOp::I16x8LtS
                | ir::BinaryOp::I16x8LtU
                | ir::BinaryOp::I16x8GtS
                | ir::BinaryOp::I16x8GtU
                | ir::BinaryOp::I16x8LeS
                | ir::BinaryOp::I16x8LeU
                | ir::BinaryOp::I16x8GeS
                | ir::BinaryOp::I16x8GeU
                | ir::BinaryOp::I32x4Eq
                | ir::BinaryOp::I32x4Ne
                | ir::BinaryOp::I32x4LtS
                | ir::BinaryOp::I32x4LtU
                | ir::BinaryOp::I32x4GtS
                | ir::BinaryOp::I32x4GtU
                | ir::BinaryOp::I32x4LeS
                | ir::BinaryOp::I32x4LeU
                | ir::BinaryOp::I32x4GeS
                | ir::BinaryOp::I32x4GeU
                | ir::BinaryOp::I64x2Eq
                | ir::BinaryOp::I64x2Ne
                | ir::BinaryOp::I64x2LtS
                | ir::BinaryOp::I64x2GtS
                | ir::BinaryOp::I64x2LeS
                | ir::BinaryOp::I64x2GeS
                | ir::BinaryOp::F32x4Eq
                | ir::BinaryOp::F32x4Ne
                | ir::BinaryOp::F32x4Lt
                | ir::BinaryOp::F32x4Gt
                | ir::BinaryOp::F32x4Le
                | ir::BinaryOp::F32x4Ge
                | ir::BinaryOp::F64x2Eq
                | ir::BinaryOp::F64x2Ne
                | ir::BinaryOp::F64x2Lt
                | ir::BinaryOp::F64x2Gt
                | ir::BinaryOp::F64x2Le
                | ir::BinaryOp::F64x2Ge
                | ir::BinaryOp::V128And
                | ir::BinaryOp::V128Or
                | ir::BinaryOp::V128Xor
                | ir::BinaryOp::V128AndNot
                | ir::BinaryOp::I8x16Shl
                | ir::BinaryOp::I8x16ShrS
                | ir::BinaryOp::I8x16ShrU
                | ir::BinaryOp::I8x16Add
                | ir::BinaryOp::I8x16AddSatS
                | ir::BinaryOp::I8x16AddSatU
                | ir::BinaryOp::I8x16Sub
                | ir::BinaryOp::I8x16SubSatS
                | ir::BinaryOp::I8x16SubSatU
                | ir::BinaryOp::I16x8Shl
                | ir::BinaryOp::I16x8ShrS
                | ir::BinaryOp::I16x8ShrU
                | ir::BinaryOp::I16x8Add
                | ir::BinaryOp::I16x8AddSatS
                | ir::BinaryOp::I16x8AddSatU
                | ir::BinaryOp::I16x8Sub
                | ir::BinaryOp::I16x8SubSatS
                | ir::BinaryOp::I16x8SubSatU
                | ir::BinaryOp::I16x8Mul
                | ir::BinaryOp::I32x4Shl
                | ir::BinaryOp::I32x4ShrS
                | ir::BinaryOp::I32x4ShrU
                | ir::BinaryOp::I32x4Add
                | ir::BinaryOp::I32x4Sub
                | ir::BinaryOp::I32x4Mul
                | ir::BinaryOp::I64x2Shl
                | ir::BinaryOp::I64x2ShrS
                | ir::BinaryOp::I64x2ShrU
                | ir::BinaryOp::I64x2Add
                | ir::BinaryOp::I64x2Sub
                | ir::BinaryOp::I64x2Mul
                | ir::BinaryOp::F32x4Add
                | ir::BinaryOp::F32x4Sub
                | ir::BinaryOp::F32x4Mul
                | ir::BinaryOp::F32x4Div
                | ir::BinaryOp::F32x4Min
                | ir::BinaryOp::F32x4Max
                | ir::BinaryOp::F32x4PMin
                | ir::BinaryOp::F32x4PMax
                | ir::BinaryOp::F64x2Add
                | ir::BinaryOp::F64x2Sub
                | ir::BinaryOp::F64x2Mul
                | ir::BinaryOp::F64x2Div
                | ir::BinaryOp::F64x2Min
                | ir::BinaryOp::F64x2Max
                | ir::BinaryOp::F64x2PMin
                | ir::BinaryOp::F64x2PMax
                | ir::BinaryOp::I8x16NarrowI16x8S
                | ir::BinaryOp::I8x16NarrowI16x8U
                | ir::BinaryOp::I16x8NarrowI32x4S
                | ir::BinaryOp::I16x8NarrowI32x4U
                | ir::BinaryOp::I8x16AvgrU
                | ir::BinaryOp::I16x8AvgrU
                | ir::BinaryOp::I8x16MinS
                | ir::BinaryOp::I8x16MinU
                | ir::BinaryOp::I8x16MaxS
                | ir::BinaryOp::I8x16MaxU
                | ir::BinaryOp::I16x8MinS
                | ir::BinaryOp::I16x8MinU
                | ir::BinaryOp::I16x8MaxS
                | ir::BinaryOp::I16x8MaxU
                | ir::BinaryOp::I32x4MinS
                | ir::BinaryOp::I32x4MinU
                | ir::BinaryOp::I32x4MaxS
                | ir::BinaryOp::I32x4MaxU
                | ir::BinaryOp::I32x4DotI16x8S
                | ir::BinaryOp::I16x8Q15MulrSatS
                | ir::BinaryOp::I16x8ExtMulLowI8x16S
                | ir::BinaryOp::I16x8ExtMulHighI8x16S
                | ir::BinaryOp::I16x8ExtMulLowI8x16U
                | ir::BinaryOp::I16x8ExtMulHighI8x16U
                | ir::BinaryOp::I32x4ExtMulLowI16x8S
                | ir::BinaryOp::I32x4ExtMulHighI16x8S
                | ir::BinaryOp::I32x4ExtMulLowI16x8U
                | ir::BinaryOp::I32x4ExtMulHighI16x8U
                | ir::BinaryOp::I64x2ExtMulLowI32x4S
                | ir::BinaryOp::I64x2ExtMulHighI32x4S
                | ir::BinaryOp::I64x2ExtMulLowI32x4U
                | ir::BinaryOp::I64x2ExtMulHighI32x4U => {
                    (&[ValType::V128, ValType::V128], &[ValType::V128])
                }
            },
            Other::Unop(unop) => match unop.op {
                ir::UnaryOp::I32Eqz
                | ir::UnaryOp::I32Clz
                | ir::UnaryOp::I32Ctz
                | ir::UnaryOp::I32Popcnt
                | ir::UnaryOp::I32Extend8S
                | ir::UnaryOp::I32Extend16S => (&[ValType::I32], &[ValType::I32]),
                ir::UnaryOp::I64Eqz
                | ir::UnaryOp::I64Clz
                | ir::UnaryOp::I64Ctz
                | ir::UnaryOp::I64Popcnt
                | ir::UnaryOp::I64Extend8S
                | ir::UnaryOp::I64Extend16S
                | ir::UnaryOp::I64Extend32S => (&[ValType::I64], &[ValType::I64]),
                ir::UnaryOp::F32Abs
                | ir::UnaryOp::F32Neg
                | ir::UnaryOp::F32Ceil
                | ir::UnaryOp::F32Floor
                | ir::UnaryOp::F32Trunc
                | ir::UnaryOp::F32Nearest
                | ir::UnaryOp::F32Sqrt => (&[ValType::F32], &[ValType::F32]),
                ir::UnaryOp::F64Abs
                | ir::UnaryOp::F64Neg
                | ir::UnaryOp::F64Ceil
                | ir::UnaryOp::F64Floor
                | ir::UnaryOp::F64Trunc
                | ir::UnaryOp::F64Nearest
                | ir::UnaryOp::F64Sqrt => (&[ValType::F64], &[ValType::F64]),
                ir::UnaryOp::I32WrapI64 => (&[ValType::I64], &[ValType::I32]),
                ir::UnaryOp::I32TruncSF32 | ir::UnaryOp::I32TruncUF32 => {
                    (&[ValType::F32], &[ValType::I32])
                }
                ir::UnaryOp::I32TruncSF64 | ir::UnaryOp::I32TruncUF64 => {
                    (&[ValType::F64], &[ValType::I32])
                }
                ir::UnaryOp::I64ExtendSI32 | ir::UnaryOp::I64ExtendUI32 => {
                    (&[ValType::I32], &[ValType::I64])
                }
                ir::UnaryOp::I64TruncSF32 | ir::UnaryOp::I64TruncUF32 => {
                    (&[ValType::F32], &[ValType::I64])
                }
                ir::UnaryOp::I64TruncSF64 | ir::UnaryOp::I64TruncUF64 => {
                    (&[ValType::F64], &[ValType::I64])
                }
                ir::UnaryOp::F32ConvertSI32
                | ir::UnaryOp::F32ConvertUI32
                | ir::UnaryOp::F32ReinterpretI32 => (&[ValType::I32], &[ValType::F32]),
                ir::UnaryOp::F32ConvertSI64 | ir::UnaryOp::F32ConvertUI64 => {
                    (&[ValType::I64], &[ValType::F32])
                }
                ir::UnaryOp::F32DemoteF64 => (&[ValType::F64], &[ValType::F32]),
                ir::UnaryOp::F64ConvertSI32 | ir::UnaryOp::F64ConvertUI32 => {
                    (&[ValType::I32], &[ValType::F64])
                }
                ir::UnaryOp::F64ConvertSI64
                | ir::UnaryOp::F64ConvertUI64
                | ir::UnaryOp::F64ReinterpretI64 => (&[ValType::I64], &[ValType::F64]),
                ir::UnaryOp::F64PromoteF32 => (&[ValType::F32], &[ValType::F64]),
                ir::UnaryOp::I32ReinterpretF32 => (&[ValType::F32], &[ValType::I32]),
                ir::UnaryOp::I64ReinterpretF64 => (&[ValType::F64], &[ValType::I64]),
                ir::UnaryOp::I8x16Splat | ir::UnaryOp::I16x8Splat | ir::UnaryOp::I32x4Splat => {
                    (&[ValType::I32], &[ValType::V128])
                }
                ir::UnaryOp::I64x2Splat => (&[ValType::I64], &[ValType::V128]),
                ir::UnaryOp::F32x4Splat => (&[ValType::F32], &[ValType::V128]),
                ir::UnaryOp::F64x2Splat => (&[ValType::F64], &[ValType::V128]),
                ir::UnaryOp::I8x16ExtractLaneS { .. }
                | ir::UnaryOp::I8x16ExtractLaneU { .. }
                | ir::UnaryOp::I16x8ExtractLaneS { .. }
                | ir::UnaryOp::I16x8ExtractLaneU { .. }
                | ir::UnaryOp::I32x4ExtractLane { .. }
                | ir::UnaryOp::V128AnyTrue
                | ir::UnaryOp::I8x16AllTrue
                | ir::UnaryOp::I16x8AllTrue
                | ir::UnaryOp::I32x4AllTrue
                | ir::UnaryOp::I64x2AllTrue
                | ir::UnaryOp::I8x16Bitmask
                | ir::UnaryOp::I16x8Bitmask
                | ir::UnaryOp::I32x4Bitmask
                | ir::UnaryOp::I64x2Bitmask => (&[ValType::V128], &[ValType::I32]),
                ir::UnaryOp::I64x2ExtractLane { .. } => (&[ValType::V128], &[ValType::I64]),
                ir::UnaryOp::F32x4ExtractLane { .. } => (&[ValType::V128], &[ValType::F32]),
                ir::UnaryOp::F64x2ExtractLane { .. } => (&[ValType::V128], &[ValType::F64]),
                ir::UnaryOp::V128Not
                | ir::UnaryOp::I8x16Abs
                | ir::UnaryOp::I8x16Popcnt
                | ir::UnaryOp::I8x16Neg
                | ir::UnaryOp::I16x8Abs
                | ir::UnaryOp::I16x8Neg
                | ir::UnaryOp::I32x4Abs
                | ir::UnaryOp::I32x4Neg
                | ir::UnaryOp::I64x2Abs
                | ir::UnaryOp::I64x2Neg
                | ir::UnaryOp::F32x4Abs
                | ir::UnaryOp::F32x4Neg
                | ir::UnaryOp::F32x4Sqrt
                | ir::UnaryOp::F32x4Ceil
                | ir::UnaryOp::F32x4Floor
                | ir::UnaryOp::F32x4Trunc
                | ir::UnaryOp::F32x4Nearest
                | ir::UnaryOp::F64x2Abs
                | ir::UnaryOp::F64x2Neg
                | ir::UnaryOp::F64x2Sqrt
                | ir::UnaryOp::F64x2Ceil
                | ir::UnaryOp::F64x2Floor
                | ir::UnaryOp::F64x2Trunc
                | ir::UnaryOp::F64x2Nearest
                | ir::UnaryOp::I16x8ExtAddPairwiseI8x16S
                | ir::UnaryOp::I16x8ExtAddPairwiseI8x16U
                | ir::UnaryOp::I32x4ExtAddPairwiseI16x8S
                | ir::UnaryOp::I32x4ExtAddPairwiseI16x8U
                | ir::UnaryOp::I64x2ExtendLowI32x4S
                | ir::UnaryOp::I64x2ExtendHighI32x4S
                | ir::UnaryOp::I64x2ExtendLowI32x4U
                | ir::UnaryOp::I64x2ExtendHighI32x4U
                | ir::UnaryOp::I32x4TruncSatF64x2SZero
                | ir::UnaryOp::I32x4TruncSatF64x2UZero
                | ir::UnaryOp::F64x2ConvertLowI32x4S
                | ir::UnaryOp::F64x2ConvertLowI32x4U
                | ir::UnaryOp::F32x4DemoteF64x2Zero
                | ir::UnaryOp::F64x2PromoteLowF32x4
                | ir::UnaryOp::I32x4TruncSatF32x4S
                | ir::UnaryOp::I32x4TruncSatF32x4U
                | ir::UnaryOp::F32x4ConvertI32x4S
                | ir::UnaryOp::F32x4ConvertI32x4U
                | ir::UnaryOp::I32TruncSSatF32
                | ir::UnaryOp::I32TruncUSatF32
                | ir::UnaryOp::I32TruncSSatF64
                | ir::UnaryOp::I32TruncUSatF64
                | ir::UnaryOp::I64TruncSSatF32
                | ir::UnaryOp::I64TruncUSatF32
                | ir::UnaryOp::I64TruncSSatF64
                | ir::UnaryOp::I64TruncUSatF64
                | ir::UnaryOp::I16x8WidenLowI8x16S
                | ir::UnaryOp::I16x8WidenLowI8x16U
                | ir::UnaryOp::I16x8WidenHighI8x16S
                | ir::UnaryOp::I16x8WidenHighI8x16U
                | ir::UnaryOp::I32x4WidenLowI16x8S
                | ir::UnaryOp::I32x4WidenLowI16x8U
                | ir::UnaryOp::I32x4WidenHighI16x8S
                | ir::UnaryOp::I32x4WidenHighI16x8U => (&[ValType::V128], &[ValType::V128]),
            },
            Other::Select(test, _) => {
                let (params, _) = test.stack_type(module, localfn, cur_stack);
                assert!(cur_stack.len() >= 2);
                let results = vt_singleton(cur_stack[cur_stack.len() - 2]);
                (params, results)
            }
            Other::Unreachable(_) => (&[], &[]),
            Other::Br(_) => (&[], &[]),
            Other::BrIf(test, _) => {
                let (params, _) = test.stack_type(module, localfn, cur_stack);
                (params, &[])
            }
            Other::BrTable(_) => (vt_singleton(ValType::I32), &[]),
            Other::MemorySize(_) => (&[], &[ValType::I32]),
            Other::MemoryGrow(_) => (vt_singleton(ValType::I32), vt_singleton(ValType::I32)),
            Other::MemoryInit(_) => (&[ValType::I32, ValType::I32, ValType::I32], &[]),
            Other::DataDrop(_) => (&[], &[]),
            Other::MemoryCopy(_) => (&[ValType::I32, ValType::I32, ValType::I32], &[]),
            Other::MemoryFill(_) => (&[ValType::I32, ValType::I32, ValType::I32], &[]),
            Other::Load(load) => match load.kind {
                ir::LoadKind::F32 => (&[ValType::I32], &[ValType::F32]),
                ir::LoadKind::F64 => (&[ValType::I32], &[ValType::F64]),
                ir::LoadKind::V128 => (&[ValType::I32], &[ValType::V128]),
                ir::LoadKind::I32 { .. }
                | ir::LoadKind::I32_8 { .. }
                | ir::LoadKind::I32_16 { .. } => (&[ValType::I32], &[ValType::I32]),
                ir::LoadKind::I64 { .. }
                | ir::LoadKind::I64_8 { .. }
                | ir::LoadKind::I64_16 { .. }
                | ir::LoadKind::I64_32 { .. } => (&[ValType::I32], &[ValType::I64]),
            },
            Other::Store(store) => match store.kind {
                ir::StoreKind::I32 { .. }
                | ir::StoreKind::I32_8 { .. }
                | ir::StoreKind::I32_16 { .. } => (&[ValType::I32, ValType::I32], &[]),

                ir::StoreKind::I64 { .. }
                | ir::StoreKind::I64_8 { .. }
                | ir::StoreKind::I64_16 { .. }
                | ir::StoreKind::I64_32 { .. } => (&[ValType::I32, ValType::I64], &[]),
                ir::StoreKind::F32 => (&[ValType::I32, ValType::F32], &[]),
                ir::StoreKind::F64 => (&[ValType::I32, ValType::F64], &[]),
                ir::StoreKind::V128 => (&[ValType::I32, ValType::V128], &[]),
            },
            Other::AtomicRmw(rmw) => match rmw.width {
                ir::AtomicWidth::I32 | ir::AtomicWidth::I32_8 | ir::AtomicWidth::I32_16 => {
                    (&[ValType::I32, ValType::I32], &[ValType::I32])
                }
                ir::AtomicWidth::I64
                | ir::AtomicWidth::I64_8
                | ir::AtomicWidth::I64_16
                | ir::AtomicWidth::I64_32 => (&[ValType::I32, ValType::I64], &[ValType::I64]),
            },
            Other::Cmpxchg(cmpxchg) => match cmpxchg.width {
                ir::AtomicWidth::I32 | ir::AtomicWidth::I32_8 | ir::AtomicWidth::I32_16 => {
                    (&[ValType::I32, ValType::I32, ValType::I32], &[ValType::I32])
                }
                ir::AtomicWidth::I64
                | ir::AtomicWidth::I64_8
                | ir::AtomicWidth::I64_16
                | ir::AtomicWidth::I64_32 => {
                    (&[ValType::I32, ValType::I64, ValType::I64], &[ValType::I64])
                }
            },
            Other::AtomicNotify(_) => (&[ValType::I32, ValType::I32], &[ValType::I32]),
            Other::AtomicWait(_) => (&[ValType::I32, ValType::I32, ValType::I32], &[ValType::I32]),
            Other::AtomicFence(_) => (&[], &[]),
            Other::TableGet(table_get) => {
                let ty = module.tables.get(table_get.table).element_ty;
                (&[ValType::I32], vt_singleton(ValType::Ref(ty)))
            }
            Other::TableSet(table_set) => {
                let ty = module.tables.get(table_set.table).element_ty;
                match ty {
                    RefType::Funcref => (&[ValType::I32, ValType::Ref(RefType::Funcref)], &[]),
                    _ => (&[ValType::I32, ValType::Ref(RefType::Externref)], &[]),
                }
            }
            Other::TableGrow(table_grow) => {
                let ty = module.tables.get(table_grow.table).element_ty;
                match ty {
                    RefType::Funcref => (
                        &[ValType::Ref(RefType::Funcref), ValType::I32],
                        &[ValType::I32],
                    ),
                    _ => (
                        &[ValType::Ref(RefType::Externref), ValType::I32],
                        &[ValType::I32],
                    ),
                }
            }
            Other::TableFill(table_fill) => {
                let ty = module.tables.get(table_fill.table).element_ty;
                match ty {
                    RefType::Funcref => (
                        &[ValType::I32, ValType::Ref(RefType::Funcref), ValType::I32],
                        &[],
                    ),
                    _ => (
                        &[ValType::I32, ValType::Ref(RefType::Externref), ValType::I32],
                        &[],
                    ),
                }
            }
            Other::RefIsNull(_) => {
                let last = cur_stack.last().unwrap();
                (vt_singleton(*last), &[ValType::I32])
            }
            Other::V128BitSelect(_) => (
                &[ValType::V128, ValType::V128, ValType::V128],
                &[ValType::V128],
            ),
            Other::I8x16Swizzle(_) => (&[ValType::V128, ValType::V128], &[ValType::V128]),
            Other::I8x16Shuffle(_) => (&[ValType::V128, ValType::V128], &[ValType::V128]),
            Other::LoadSimd(_) => (&[ValType::I32], &[ValType::V128]),
            Other::TableInit(_) => (&[ValType::I32, ValType::I32, ValType::I32], &[]),
            Other::ElemDrop(_) => (&[], &[]),
            Other::TableCopy(_) => (&[ValType::I32, ValType::I32, ValType::I32], &[]),
        }
    }

    fn mnemonic(&self) -> &'static str {
        match self {
            Other::Call(_) => "call",
            Other::CallIndirect(_) => "call_indirect",
            Other::LocalTee(_) => "local.tee",
            Other::Binop(binop) => match binop.op {
                ir::BinaryOp::I32Eq => "i32.eq",
                ir::BinaryOp::I32Ne => "i32.ne",
                ir::BinaryOp::I32LtS => "i32.lt_s",
                ir::BinaryOp::I32LtU => "i32.lt_u",
                ir::BinaryOp::I32GtS => "i32.gt_s",
                ir::BinaryOp::I32GtU => "i32.gt_u",
                ir::BinaryOp::I32LeS => "i32.le_s",
                ir::BinaryOp::I32LeU => "i32.le_u",
                ir::BinaryOp::I32GeS => "i32.ge_s",
                ir::BinaryOp::I32GeU => "i32.ge_u",
                ir::BinaryOp::I64Eq => "i64.eq",
                ir::BinaryOp::I64Ne => "i64.ne",
                ir::BinaryOp::I64LtS => "i64.lt_s",
                ir::BinaryOp::I64LtU => "i64.lt_u",
                ir::BinaryOp::I64GtS => "i64.gt_s",
                ir::BinaryOp::I64GtU => "i64.gt_u",
                ir::BinaryOp::I64LeS => "i64.le_s",
                ir::BinaryOp::I64LeU => "i64.le_u",
                ir::BinaryOp::I64GeS => "i64.ge_s",
                ir::BinaryOp::I64GeU => "i64.ge_u",
                ir::BinaryOp::F32Eq => "f32.eq",
                ir::BinaryOp::F32Ne => "f32.ne",
                ir::BinaryOp::F32Lt => "f32.lt",
                ir::BinaryOp::F32Gt => "f32.gt",
                ir::BinaryOp::F32Le => "f32.le",
                ir::BinaryOp::F32Ge => "f32.ge",
                ir::BinaryOp::F64Eq => "f64.eq",
                ir::BinaryOp::F64Ne => "f64.ne",
                ir::BinaryOp::F64Lt => "f64.lt",
                ir::BinaryOp::F64Gt => "f64.gt",
                ir::BinaryOp::F64Le => "f64.le",
                ir::BinaryOp::F64Ge => "f64.ge",
                ir::BinaryOp::I32Add => "i32.add",
                ir::BinaryOp::I32Sub => "i32.sub",
                ir::BinaryOp::I32Mul => "i32.mul",
                ir::BinaryOp::I32DivS => "i32.div_s",
                ir::BinaryOp::I32DivU => "i32.div_u",
                ir::BinaryOp::I32RemS => "i32.rem_s",
                ir::BinaryOp::I32RemU => "i32.rem_u",
                ir::BinaryOp::I32And => "i32.and",
                ir::BinaryOp::I32Or => "i32.or",
                ir::BinaryOp::I32Xor => "i32.xor",
                ir::BinaryOp::I32Shl => "i32.shl",
                ir::BinaryOp::I32ShrS => "i32.shr_s",
                ir::BinaryOp::I32ShrU => "i32.shr_u",
                ir::BinaryOp::I32Rotl => "i32.rotl",
                ir::BinaryOp::I32Rotr => "i32.rotr",
                ir::BinaryOp::I64Add => "i64.add",
                ir::BinaryOp::I64Sub => "i64.sub",
                ir::BinaryOp::I64Mul => "i64.mul",
                ir::BinaryOp::I64DivS => "i64.div_s",
                ir::BinaryOp::I64DivU => "i64.div_u",
                ir::BinaryOp::I64RemS => "i64.rem_s",
                ir::BinaryOp::I64RemU => "i64.rem_u",
                ir::BinaryOp::I64And => "i64.and",
                ir::BinaryOp::I64Or => "i64.or",
                ir::BinaryOp::I64Xor => "i64.xor",
                ir::BinaryOp::I64Shl => "i64.shl",
                ir::BinaryOp::I64ShrS => "i64.shr_s",
                ir::BinaryOp::I64ShrU => "i64.shr_u",
                ir::BinaryOp::I64Rotl => "i64.rotl",
                ir::BinaryOp::I64Rotr => "i64.rotr",
                ir::BinaryOp::F32Add => "f32.add",
                ir::BinaryOp::F32Sub => "f32.sub",
                ir::BinaryOp::F32Mul => "f32.mul",
                ir::BinaryOp::F32Div => "f32.div",
                ir::BinaryOp::F32Min => "f32.min",
                ir::BinaryOp::F32Max => "f32.max",
                ir::BinaryOp::F32Copysign => "f32.copysign",
                ir::BinaryOp::F64Add => "f64.add",
                ir::BinaryOp::F64Sub => "f64.sub",
                ir::BinaryOp::F64Mul => "f64.mul",
                ir::BinaryOp::F64Div => "f64.div",
                ir::BinaryOp::F64Min => "f64.min",
                ir::BinaryOp::F64Max => "f64.max",
                ir::BinaryOp::F64Copysign => "f64.copysign",
                ir::BinaryOp::I8x16ReplaceLane { .. } => "i8x16.replace_lane",
                ir::BinaryOp::I16x8ReplaceLane { .. } => "i16x8.replace_lane",
                ir::BinaryOp::I32x4ReplaceLane { .. } => "i32x4.replace_lane",
                ir::BinaryOp::I64x2ReplaceLane { .. } => "i64x2.replace_lane",
                ir::BinaryOp::F32x4ReplaceLane { .. } => "f32x4.replace_lane",
                ir::BinaryOp::F64x2ReplaceLane { .. } => "f64x2.replace_lane",
                ir::BinaryOp::I8x16Eq => "i8x16.eq",
                ir::BinaryOp::I8x16Ne => "i8x16.ne",
                ir::BinaryOp::I8x16LtS => "i8x16.lt_s",
                ir::BinaryOp::I8x16LtU => "i8x16.lt_u",
                ir::BinaryOp::I8x16GtS => "i8x16.gt_s",
                ir::BinaryOp::I8x16GtU => "i8x16.gt_u",
                ir::BinaryOp::I8x16LeS => "i8x16.le_s",
                ir::BinaryOp::I8x16LeU => "i8x16.le_u",
                ir::BinaryOp::I8x16GeS => "i8x16.lt_s",
                ir::BinaryOp::I8x16GeU => "i8x16.ge_u",
                ir::BinaryOp::I16x8Eq => "i16x8.eq",
                ir::BinaryOp::I16x8Ne => "i16x8.ne",
                ir::BinaryOp::I16x8LtS => "i16x8.lt_s",
                ir::BinaryOp::I16x8LtU => "i16x8.lt_u",
                ir::BinaryOp::I16x8GtS => "i16x8.gt_s",
                ir::BinaryOp::I16x8GtU => "i16x8.gt_u",
                ir::BinaryOp::I16x8LeS => "i16x8.le_s",
                ir::BinaryOp::I16x8LeU => "i16x8.le_u",
                ir::BinaryOp::I16x8GeS => "i16x8.ge_s",
                ir::BinaryOp::I16x8GeU => "i16x8.ge_u",
                ir::BinaryOp::I32x4Eq => "i32x4.eq",
                ir::BinaryOp::I32x4Ne => "i32x4.ne",
                ir::BinaryOp::I32x4LtS => "i32x4.lt_s",
                ir::BinaryOp::I32x4LtU => "i32x4.lt_u",
                ir::BinaryOp::I32x4GtS => "i32x4.gt_s",
                ir::BinaryOp::I32x4GtU => "i32x4.gt_u",
                ir::BinaryOp::I32x4LeS => "i32x4.le_s",
                ir::BinaryOp::I32x4LeU => "i32x4.le_u",
                ir::BinaryOp::I32x4GeS => "i32x4.ge_s",
                ir::BinaryOp::I32x4GeU => "i32x4.ge_u",
                ir::BinaryOp::I64x2Eq => "i64x2.eq",
                ir::BinaryOp::I64x2Ne => "i64x2.ne",
                ir::BinaryOp::I64x2LtS => "i64x2.lt_s",
                ir::BinaryOp::I64x2GtS => "i64x2.gt_s",
                ir::BinaryOp::I64x2LeS => "i64x2.le_s",
                ir::BinaryOp::I64x2GeS => "i64x2.ge_s",
                ir::BinaryOp::F32x4Eq => "f32x4.eq",
                ir::BinaryOp::F32x4Ne => "f32x4.ne",
                ir::BinaryOp::F32x4Lt => "f32x4.lt",
                ir::BinaryOp::F32x4Gt => "f32x4.gt",
                ir::BinaryOp::F32x4Le => "f32x4.le",
                ir::BinaryOp::F32x4Ge => "f32x4.ge",
                ir::BinaryOp::F64x2Eq => "fx64x2.eq",
                ir::BinaryOp::F64x2Ne => "fx64x2.ne",
                ir::BinaryOp::F64x2Lt => "fx64x2.lt",
                ir::BinaryOp::F64x2Gt => "fx64x2.gt",
                ir::BinaryOp::F64x2Le => "fx64x2.le",
                ir::BinaryOp::F64x2Ge => "fx64x2.ge",
                ir::BinaryOp::V128And => "v128.and",
                ir::BinaryOp::V128Or => "v128.or",
                ir::BinaryOp::V128Xor => "v128.xor",
                ir::BinaryOp::V128AndNot => "x128.and_not",
                ir::BinaryOp::I8x16Shl => "i8x16.shl",
                ir::BinaryOp::I8x16ShrS => "i8x16.shr_s",
                ir::BinaryOp::I8x16ShrU => "i8x16.shr_u",
                ir::BinaryOp::I8x16Add => "i8x16.add",
                ir::BinaryOp::I8x16AddSatS => "i8x16.add_sat_s",
                ir::BinaryOp::I8x16AddSatU => "i8x16.add_sat_u",
                ir::BinaryOp::I8x16Sub => "i8x16.sub",
                ir::BinaryOp::I8x16SubSatS => "i8x16.sub_sat_s",
                ir::BinaryOp::I8x16SubSatU => "i8x16.sub_sat_u",
                ir::BinaryOp::I16x8Shl => "i16x8.shl",
                ir::BinaryOp::I16x8ShrS => "i16x8.shr_s",
                ir::BinaryOp::I16x8ShrU => "i16x8.shr_u",
                ir::BinaryOp::I16x8Add => "i16x8.add",
                ir::BinaryOp::I16x8AddSatS => "i16x8.add_sat_s",
                ir::BinaryOp::I16x8AddSatU => "i16x8.add_sat_u",
                ir::BinaryOp::I16x8Sub => "i16x8.sub",
                ir::BinaryOp::I16x8SubSatS => "i16x8.sub_sat_s",
                ir::BinaryOp::I16x8SubSatU => "i16x8.sub_sat_u",
                ir::BinaryOp::I16x8Mul => "i16x8.mul",
                ir::BinaryOp::I32x4Shl => "i32x4.shl",
                ir::BinaryOp::I32x4ShrS => "i32x4.shr_s",
                ir::BinaryOp::I32x4ShrU => "i32x4.shr_u",
                ir::BinaryOp::I32x4Add => "i32x4.add",
                ir::BinaryOp::I32x4Sub => "i32x4.sub",
                ir::BinaryOp::I32x4Mul => "i32x4.mul",
                ir::BinaryOp::I64x2Shl => "i64x2.shl",
                ir::BinaryOp::I64x2ShrS => "i64x2.shr_s",
                ir::BinaryOp::I64x2ShrU => "i64x2.shr_u",
                ir::BinaryOp::I64x2Add => "i64x2.add",
                ir::BinaryOp::I64x2Sub => "i64x2.sub",
                ir::BinaryOp::I64x2Mul => "i64x2.mul",
                ir::BinaryOp::F32x4Add => "f32x4.add",
                ir::BinaryOp::F32x4Sub => "f32x4.sub",
                ir::BinaryOp::F32x4Mul => "f32x4.mul",
                ir::BinaryOp::F32x4Div => "f32x4.div",
                ir::BinaryOp::F32x4Min => "f32x4.min",
                ir::BinaryOp::F32x4Max => "f32x4.max",
                ir::BinaryOp::F32x4PMin => "f32x4.pmin",
                ir::BinaryOp::F32x4PMax => "f32x4.pmax",
                ir::BinaryOp::F64x2Add => "f64x2.add",
                ir::BinaryOp::F64x2Sub => "f64x2.sub",
                ir::BinaryOp::F64x2Mul => "f64x2.mul",
                ir::BinaryOp::F64x2Div => "f64x2.div",
                ir::BinaryOp::F64x2Min => "f64x2.min",
                ir::BinaryOp::F64x2Max => "f64x2.max",
                ir::BinaryOp::F64x2PMin => "f64x2.pmin",
                ir::BinaryOp::F64x2PMax => "f64x2.pmax",
                ir::BinaryOp::I8x16NarrowI16x8S => "i8x16.narrow_i16x8_s",
                ir::BinaryOp::I8x16NarrowI16x8U => "i8x16.narrow_i16x8_u",
                ir::BinaryOp::I16x8NarrowI32x4S => "i8x16.narrow_i32x4_s",
                ir::BinaryOp::I16x8NarrowI32x4U => "i8x16.narrow_i32x4_u",
                ir::BinaryOp::I8x16AvgrU => "i8x16.avgr_u",
                ir::BinaryOp::I16x8AvgrU => "i16x8.avgr_u",
                ir::BinaryOp::I8x16MinS => "i8x16.min_s",
                ir::BinaryOp::I8x16MinU => "i8x16.min_u",
                ir::BinaryOp::I8x16MaxS => "i8x16.max_s",
                ir::BinaryOp::I8x16MaxU => "i8x16.max_u",
                ir::BinaryOp::I16x8MinS => "i8x16.min_s",
                ir::BinaryOp::I16x8MinU => "i8x16.min_u",
                ir::BinaryOp::I16x8MaxS => "i16x8.max_s",
                ir::BinaryOp::I16x8MaxU => "i16x8.max_u",
                ir::BinaryOp::I32x4MinS => "i32x4.min_s",
                ir::BinaryOp::I32x4MinU => "i32x4.min_u",
                ir::BinaryOp::I32x4MaxS => "i32x4.max_s",
                ir::BinaryOp::I32x4MaxU => "i32x4.max_u",
                ir::BinaryOp::I32x4DotI16x8S => "i32x4.dot_i16x8_s",
                ir::BinaryOp::I16x8Q15MulrSatS => "i16x8.q15mulr_sat_s",
                ir::BinaryOp::I16x8ExtMulLowI8x16S => "i16x8.extmul_low_i8x16_s",
                ir::BinaryOp::I16x8ExtMulHighI8x16S => "i16x8.extmul_high_i8x16_s",
                ir::BinaryOp::I16x8ExtMulLowI8x16U => "i16x8.extmul_low_i8x16_u",
                ir::BinaryOp::I16x8ExtMulHighI8x16U => "i16x8.extmul_high_i8x16_u",
                ir::BinaryOp::I32x4ExtMulLowI16x8S => "i32x4.extmul_low_i16x8_s",
                ir::BinaryOp::I32x4ExtMulHighI16x8S => "i32x4.extmul_high_i16x8_s",
                ir::BinaryOp::I32x4ExtMulLowI16x8U => "i32x4.extmul_low_i16x8_u",
                ir::BinaryOp::I32x4ExtMulHighI16x8U => "i32x4.extmul_high_i16x8_u",
                ir::BinaryOp::I64x2ExtMulLowI32x4S => "i64x2.extmul_low_i32x4_s",
                ir::BinaryOp::I64x2ExtMulHighI32x4S => "i64x2.extmul_high_i32x4_s",
                ir::BinaryOp::I64x2ExtMulLowI32x4U => "i64x2.extmul_low_i32x4_u",
                ir::BinaryOp::I64x2ExtMulHighI32x4U => "i64x2.extmul_high_i32x4_u",
            },
            Other::Unop(unop) => match unop.op {
                ir::UnaryOp::I32Eqz => "i32.eqz",
                ir::UnaryOp::I32Clz => "i32.clz",
                ir::UnaryOp::I32Ctz => "i32.ctz",
                ir::UnaryOp::I32Popcnt => "i32.popcnt",
                ir::UnaryOp::I64Eqz => "i64.eq",
                ir::UnaryOp::I64Clz => "i64.clz",
                ir::UnaryOp::I64Ctz => "i64.ctz",
                ir::UnaryOp::I64Popcnt => "i64.popcnt",
                ir::UnaryOp::F32Abs => "f32.abs",
                ir::UnaryOp::F32Neg => "f32.neg",
                ir::UnaryOp::F32Ceil => "f32.ceil",
                ir::UnaryOp::F32Floor => "f32.floor",
                ir::UnaryOp::F32Trunc => "f32.trunc",
                ir::UnaryOp::F32Nearest => "f32.nearest",
                ir::UnaryOp::F32Sqrt => "f32.sqrt",
                ir::UnaryOp::F64Abs => "f64.abs",
                ir::UnaryOp::F64Neg => "f64.neg",
                ir::UnaryOp::F64Ceil => "f64.ceil",
                ir::UnaryOp::F64Floor => "f64.floor",
                ir::UnaryOp::F64Trunc => "f64.trunc",
                ir::UnaryOp::F64Nearest => "f64.nearest",
                ir::UnaryOp::F64Sqrt => "f64.sqrt",
                ir::UnaryOp::I32WrapI64 => "i32.wrap_i64",
                ir::UnaryOp::I32TruncSF32 => "i32.trunc_s_f32",
                ir::UnaryOp::I32TruncUF32 => "i32.trunc_u_f32",
                ir::UnaryOp::I32TruncSF64 => "i32.trunc_s_f64",
                ir::UnaryOp::I32TruncUF64 => "i32.trunc_u_f64",
                ir::UnaryOp::I64ExtendSI32 => "i64.extend_s_i32",
                ir::UnaryOp::I64ExtendUI32 => "i64.extend_u_i32",
                ir::UnaryOp::I64TruncSF32 => "i64.trunc_s_f32",
                ir::UnaryOp::I64TruncUF32 => "i64.trunc_u_f32",
                ir::UnaryOp::I64TruncSF64 => "i64.trunc_s_f64",
                ir::UnaryOp::I64TruncUF64 => "i64.trunc_u_f64",
                ir::UnaryOp::F32ConvertSI32 => "f32.convert_s_i32",
                ir::UnaryOp::F32ConvertUI32 => "f32.convert_u_i32",
                ir::UnaryOp::F32ConvertSI64 => "f32.convert_s_i64",
                ir::UnaryOp::F32ConvertUI64 => "f32.convert_u_i64",
                ir::UnaryOp::F32DemoteF64 => "f32.demote_f64",
                ir::UnaryOp::F64ConvertSI32 => "f64.convert_s_i32",
                ir::UnaryOp::F64ConvertUI32 => "f64.convert_u_i32",
                ir::UnaryOp::F64ConvertSI64 => "f64.convert_s_i64",
                ir::UnaryOp::F64ConvertUI64 => "f64.convert_u_i64",
                ir::UnaryOp::F64PromoteF32 => "f64.promote_f32",
                ir::UnaryOp::I32ReinterpretF32 => "i32.reinterpret_f32",
                ir::UnaryOp::I64ReinterpretF64 => "i64.reinterpret_f64",
                ir::UnaryOp::F32ReinterpretI32 => "f32.reinterpret_i32",
                ir::UnaryOp::F64ReinterpretI64 => "i64.reinterpret_f64",
                ir::UnaryOp::I32Extend8S => "i32.extend8_s",
                ir::UnaryOp::I32Extend16S => "i32.extend16_s",
                ir::UnaryOp::I64Extend8S => "i64.extend8_s",
                ir::UnaryOp::I64Extend16S => "i64.extend16_s",
                ir::UnaryOp::I64Extend32S => "i64.extend32_s",
                ir::UnaryOp::I8x16Splat => "i8x16.splat",
                ir::UnaryOp::I8x16ExtractLaneS { .. } => "i8x16.extract_lane_s",
                ir::UnaryOp::I8x16ExtractLaneU { .. } => "i8x16.extract_lane_u",
                ir::UnaryOp::I16x8Splat => "i16x8.splat",
                ir::UnaryOp::I16x8ExtractLaneS { .. } => "i16x8.extract_lane_s",
                ir::UnaryOp::I16x8ExtractLaneU { .. } => "i16x8.extract_lane_u",
                ir::UnaryOp::I32x4Splat => "i32x4.splat",
                ir::UnaryOp::I32x4ExtractLane { .. } => "i32x4.extract_lane",
                ir::UnaryOp::I64x2Splat => "i64x2.splat",
                ir::UnaryOp::I64x2ExtractLane { .. } => "i64x2.extract_lane",
                ir::UnaryOp::F32x4Splat => "f32x4.splat",
                ir::UnaryOp::F32x4ExtractLane { .. } => "f32x4.extract_lane",
                ir::UnaryOp::F64x2Splat => "f64x2.splat",
                ir::UnaryOp::F64x2ExtractLane { .. } => "f64x2.extract_lane",
                ir::UnaryOp::V128Not => "v128.not",
                ir::UnaryOp::V128AnyTrue => "v128.any_true",
                ir::UnaryOp::I8x16Abs => "i8x16.abs",
                ir::UnaryOp::I8x16Popcnt => "i8x16.popcnt",
                ir::UnaryOp::I8x16Neg => "i8x16.neg",
                ir::UnaryOp::I8x16AllTrue => "i8x16.all_true",
                ir::UnaryOp::I8x16Bitmask => "i8x16.bitmask",
                ir::UnaryOp::I16x8Abs => "i16x8.abs",
                ir::UnaryOp::I16x8Neg => "i16x8.neg",
                ir::UnaryOp::I16x8AllTrue => "i16x8.all_true",
                ir::UnaryOp::I16x8Bitmask => "i16x8.bitmask",
                ir::UnaryOp::I32x4Abs => "i32x4.abs",
                ir::UnaryOp::I32x4Neg => "i32x4.neg",
                ir::UnaryOp::I32x4AllTrue => "i32x4.all_true",
                ir::UnaryOp::I32x4Bitmask => "i32x4.bitmask",
                ir::UnaryOp::I64x2Abs => "i64x2.abs",
                ir::UnaryOp::I64x2Neg => "i64x2.neg",
                ir::UnaryOp::I64x2AllTrue => "i64x2.all_true",
                ir::UnaryOp::I64x2Bitmask => "i64x2.bitmask",
                ir::UnaryOp::F32x4Abs => "f32x4.abs",
                ir::UnaryOp::F32x4Neg => "f32x4.neg",
                ir::UnaryOp::F32x4Sqrt => "f32x4.sqrt",
                ir::UnaryOp::F32x4Ceil => "f32x4.ceil",
                ir::UnaryOp::F32x4Floor => "f32x4.floor",
                ir::UnaryOp::F32x4Trunc => "f32x4.trunc",
                ir::UnaryOp::F32x4Nearest => "f32x4.nearest",
                ir::UnaryOp::F64x2Abs => "f64x2.abs",
                ir::UnaryOp::F64x2Neg => "f64x2.neg",
                ir::UnaryOp::F64x2Sqrt => "f64x2.sqrt",
                ir::UnaryOp::F64x2Ceil => "f64x2.ceil",
                ir::UnaryOp::F64x2Floor => "f64x2.floor",
                ir::UnaryOp::F64x2Trunc => "f64x2.trunc",
                ir::UnaryOp::F64x2Nearest => "f64x2.nearest",
                ir::UnaryOp::I16x8ExtAddPairwiseI8x16S => "i16x8.extadd_pairwise_i8x16_s",
                ir::UnaryOp::I16x8ExtAddPairwiseI8x16U => "i16x8.extadd_pairwise_i8x16_u",
                ir::UnaryOp::I32x4ExtAddPairwiseI16x8S => "i32x4.extadd_pairwise_i8x16_s",
                ir::UnaryOp::I32x4ExtAddPairwiseI16x8U => "i132x4.extadd_pairwise_i8x16_u",
                ir::UnaryOp::I64x2ExtendLowI32x4S => "i64x2._extend_low_i32x4_s",
                ir::UnaryOp::I64x2ExtendHighI32x4S => "i64x2._extend_high_i32x4_s",
                ir::UnaryOp::I64x2ExtendLowI32x4U => "i64x2._extend_low_i32x4_u",
                ir::UnaryOp::I64x2ExtendHighI32x4U => "i64x2._extend_high_i32x4_u",
                ir::UnaryOp::I32x4TruncSatF64x2SZero => "i32x4.trunc_sat_f64x2_s_zero",
                ir::UnaryOp::I32x4TruncSatF64x2UZero => "i32x4.trunc_sat_f64x2_u_zero",
                ir::UnaryOp::F64x2ConvertLowI32x4S => "f64x2.convert_low_i32x4_s",
                ir::UnaryOp::F64x2ConvertLowI32x4U => "f64x2.convert_low_i32x4_u",
                ir::UnaryOp::F32x4DemoteF64x2Zero => "f32x4.demote_f64x2_zero",
                ir::UnaryOp::F64x2PromoteLowF32x4 => "f64x2.promote_low_f32x4",
                ir::UnaryOp::I32x4TruncSatF32x4S => "i32x4.trunc_sat_f32x4_s",
                ir::UnaryOp::I32x4TruncSatF32x4U => "i32x4.trunc_sat_f32x4_u",
                ir::UnaryOp::F32x4ConvertI32x4S => "f32x4.convert_i32x4_s",
                ir::UnaryOp::F32x4ConvertI32x4U => "f32x4.convert_i32x4_u",
                ir::UnaryOp::I32TruncSSatF32 => "i32.trunc_s_sat_f32",
                ir::UnaryOp::I32TruncUSatF32 => "i32.trunc_u_sat_f32",
                ir::UnaryOp::I32TruncSSatF64 => "i32.trunc_s_sat_f64",
                ir::UnaryOp::I32TruncUSatF64 => "i32.trunc_u_sat_f64",
                ir::UnaryOp::I64TruncSSatF32 => "i64.trunc_s_sat_f32",
                ir::UnaryOp::I64TruncUSatF32 => "i64.trunc_u_sat_f32",
                ir::UnaryOp::I64TruncSSatF64 => "i64.trunc_s_sat_f64",
                ir::UnaryOp::I64TruncUSatF64 => "i64.trunc_u_sat_f64",
                ir::UnaryOp::I16x8WidenLowI8x16S => "i16x8.widen_low_i8x16_s",
                ir::UnaryOp::I16x8WidenLowI8x16U => "i16x8.widen_low_i8x16_u",
                ir::UnaryOp::I16x8WidenHighI8x16S => "i16x8.widen_high_i8x16_s",
                ir::UnaryOp::I16x8WidenHighI8x16U => "i16x8.widen_high_i8x16_u",
                ir::UnaryOp::I32x4WidenLowI16x8S => "i32x4.widen_low_i16x8_s",
                ir::UnaryOp::I32x4WidenLowI16x8U => "i32x4.widen_low_i16x8_u",
                ir::UnaryOp::I32x4WidenHighI16x8S => "i32x4.widen_high_i16x8_s",
                ir::UnaryOp::I32x4WidenHighI16x8U => "i32x4.widen_high_i16x8_u",
            },
            Other::Select(_, _) => "select",
            Other::Unreachable(_) => "unreachable",
            Other::Br(_) => "br",
            Other::BrIf(_, _) => "br_if",
            Other::BrTable(_) => "br_table",
            Other::MemorySize(_) => "memory.size",
            Other::MemoryGrow(_) => "memory.grow",
            Other::MemoryInit(_) => "memory.init",
            Other::DataDrop(_) => "data.drop",
            Other::MemoryCopy(_) => "memory.copy",
            Other::MemoryFill(_) => "memory.fill",
            Other::Load(load) => match load.kind {
                ir::LoadKind::I32 { atomic: true } => "i32.atomic.load",
                ir::LoadKind::I32 { atomic: false } => "i32.load",
                ir::LoadKind::I64 { atomic: true } => "i64.atomic.load",
                ir::LoadKind::I64 { atomic: false } => "i64.load",
                ir::LoadKind::F32 => "f32.load",
                ir::LoadKind::F64 => "f64.load",
                ir::LoadKind::V128 => "v128.load",
                ir::LoadKind::I32_8 {
                    kind: ExtendedLoad::ZeroExtend,
                } => "i32.load8_u",
                ir::LoadKind::I32_8 {
                    kind: ExtendedLoad::SignExtend,
                } => "i32.load8_s",
                ir::LoadKind::I32_8 {
                    kind: ExtendedLoad::ZeroExtendAtomic,
                } => "i32.atomic.load8_u",
                ir::LoadKind::I32_16 {
                    kind: ExtendedLoad::ZeroExtend,
                } => "i32.load16_u",
                ir::LoadKind::I32_16 {
                    kind: ExtendedLoad::SignExtend,
                } => "i32.load16_s",
                ir::LoadKind::I32_16 {
                    kind: ExtendedLoad::ZeroExtendAtomic,
                } => "i32.atomic.load16_u",
                ir::LoadKind::I64_8 {
                    kind: ExtendedLoad::ZeroExtend,
                } => "i64.load8_u",
                ir::LoadKind::I64_8 {
                    kind: ExtendedLoad::SignExtend,
                } => "i64.load8_s",
                ir::LoadKind::I64_8 {
                    kind: ExtendedLoad::ZeroExtendAtomic,
                } => "i64.atomic.load8_u",
                ir::LoadKind::I64_16 {
                    kind: ExtendedLoad::ZeroExtend,
                } => "i64.load16_u",
                ir::LoadKind::I64_16 {
                    kind: ExtendedLoad::SignExtend,
                } => "i64.load16_s",
                ir::LoadKind::I64_16 {
                    kind: ExtendedLoad::ZeroExtendAtomic,
                } => "i64.atomic.load16_u",
                ir::LoadKind::I64_32 {
                    kind: ExtendedLoad::ZeroExtend,
                } => "i64.load32_u",
                ir::LoadKind::I64_32 {
                    kind: ExtendedLoad::SignExtend,
                } => "i64.load32_s",
                ir::LoadKind::I64_32 {
                    kind: ExtendedLoad::ZeroExtendAtomic,
                } => "i64.atomic.load32_u",
            },
            Other::Store(store) => match store.kind {
                ir::StoreKind::I32 { atomic: true } => "i32.atomic.store",
                ir::StoreKind::I32 { atomic: false } => "i32.store",
                ir::StoreKind::I64 { atomic: true } => "i64.atomic.store",
                ir::StoreKind::I64 { atomic: false } => "i64.store",
                ir::StoreKind::F32 => "f32.store",
                ir::StoreKind::F64 => "f64.store",
                ir::StoreKind::V128 => "v128.store",
                ir::StoreKind::I32_8 { atomic: false } => "i32.store8",
                ir::StoreKind::I32_8 { atomic: true } => "i32.atomic.store8",
                ir::StoreKind::I32_16 { atomic: false } => "i32.store16",
                ir::StoreKind::I32_16 { atomic: true } => "i32.atomic.store16",
                ir::StoreKind::I64_8 { atomic: false } => "i64.store8",
                ir::StoreKind::I64_8 { atomic: true } => "i64.atomic.store8",
                ir::StoreKind::I64_16 { atomic: false } => "i64.store16",
                ir::StoreKind::I64_16 { atomic: true } => "i64.atomic.store16",
                ir::StoreKind::I64_32 { atomic: false } => "i64.store32",
                ir::StoreKind::I64_32 { atomic: true } => "i64.atomic.store32",
            },
            Other::AtomicRmw(rmw) => match rmw.op {
                ir::AtomicOp::Add => match rmw.width {
                    ir::AtomicWidth::I32 => "i32.atomic.rmw.add",
                    ir::AtomicWidth::I32_8 => "i32.atomic.rmw8.add_u",
                    ir::AtomicWidth::I32_16 => "i32.atomic.rmw16.add_u",
                    ir::AtomicWidth::I64 => "i64.atomic.rmw.add",
                    ir::AtomicWidth::I64_8 => "i64.atomic.rmw8.add_u",
                    ir::AtomicWidth::I64_16 => "i64.atomic.rmw16.add_u",
                    ir::AtomicWidth::I64_32 => "i64.atomic.rmw32.add_u",
                },
                ir::AtomicOp::Sub => match rmw.width {
                    ir::AtomicWidth::I32 => "i32.atomic.rmw.sub",
                    ir::AtomicWidth::I32_8 => "i32.atomic.rmw8.sub_u",
                    ir::AtomicWidth::I32_16 => "i32.atomic.rmw16.sub_u",
                    ir::AtomicWidth::I64 => "i64.atomic.rmw.sub",
                    ir::AtomicWidth::I64_8 => "i64.atomic.rmw8.sub_u",
                    ir::AtomicWidth::I64_16 => "i64.atomic.rmw16.sub_u",
                    ir::AtomicWidth::I64_32 => "i64.atomic.rmw32.sub_u",
                },
                ir::AtomicOp::And => match rmw.width {
                    ir::AtomicWidth::I32 => "i32.atomic.rmw.and",
                    ir::AtomicWidth::I32_8 => "i32.atomic.rmw8.and_u",
                    ir::AtomicWidth::I32_16 => "i32.atomic.rmw16.and_u",
                    ir::AtomicWidth::I64 => "i64.atomic.rmw.and",
                    ir::AtomicWidth::I64_8 => "i64.atomic.rmw8.and_u",
                    ir::AtomicWidth::I64_16 => "i64.atomic.rmw16.and_u",
                    ir::AtomicWidth::I64_32 => "i64.atomic.rmw32.and_u",
                },
                ir::AtomicOp::Or => match rmw.width {
                    ir::AtomicWidth::I32 => "i32.atomic.rmw.or",
                    ir::AtomicWidth::I32_8 => "i32.atomic.rmw8.or_u",
                    ir::AtomicWidth::I32_16 => "i32.atomic.rmw16.or_u",
                    ir::AtomicWidth::I64 => "i64.atomic.rmw.or",
                    ir::AtomicWidth::I64_8 => "i64.atomic.rmw8.or_u",
                    ir::AtomicWidth::I64_16 => "i64.atomic.rmw16.or_u",
                    ir::AtomicWidth::I64_32 => "i64.atomic.rmw32.or_u",
                },
                ir::AtomicOp::Xor => match rmw.width {
                    ir::AtomicWidth::I32 => "i32.atomic.rmw.xor",
                    ir::AtomicWidth::I32_8 => "i32.atomic.rmw8.xor_u",
                    ir::AtomicWidth::I32_16 => "i32.atomic.rmw16.xor_u",
                    ir::AtomicWidth::I64 => "i64.atomic.rmw.xor",
                    ir::AtomicWidth::I64_8 => "i64.atomic.rmw8.xor_u",
                    ir::AtomicWidth::I64_16 => "i64.atomic.rmw16.xor_u",
                    ir::AtomicWidth::I64_32 => "i64.atomic.rmw32.xor_u",
                },
                ir::AtomicOp::Xchg => match rmw.width {
                    ir::AtomicWidth::I32 => "i32.atomic.rmw.xchg",
                    ir::AtomicWidth::I32_8 => "i32.atomic.rmw8.xchg_u",
                    ir::AtomicWidth::I32_16 => "i32.atomic.rmw16.xchg_u",
                    ir::AtomicWidth::I64 => "i64.atomic.rmw.xchg",
                    ir::AtomicWidth::I64_8 => "i64.atomic.rmw8.xchg_u",
                    ir::AtomicWidth::I64_16 => "i64.atomic.rmw16.xchg_u",
                    ir::AtomicWidth::I64_32 => "i64.atomic.rmw32.xchg_u",
                },
            },
            Other::Cmpxchg(cmpxchg) => match cmpxchg.width {
                ir::AtomicWidth::I32 => "i32.atomic.rmw.cmpxchg",
                ir::AtomicWidth::I32_8 => "i32.atomic.rmw8.cmpxchg_u",
                ir::AtomicWidth::I32_16 => "i32.atomic.rmw16.cmpxchg_u",
                ir::AtomicWidth::I64 => "i64.atomic.rmw.cmpxchg",
                ir::AtomicWidth::I64_8 => "i64.atomic.rmw8.cmpxchg_u",
                ir::AtomicWidth::I64_16 => "i64.atomic.rmw16.cmpxchg_u",
                ir::AtomicWidth::I64_32 => "i64.atomic.rmw32.cmpxchg_u",
            },
            Other::AtomicNotify(_) => "memory.atomic.notify",
            Other::AtomicWait(wait) => {
                if wait.sixty_four {
                    "memory.atomic.wait64"
                } else {
                    "memory.atomic.wait32"
                }
            }
            Other::AtomicFence(_) => "atomic.fence",
            Other::TableGet(_) => "table.get",
            Other::TableSet(_) => "table.set",
            Other::TableGrow(_) => "table.grow",
            Other::TableFill(_) => "table.fill",
            Other::RefIsNull(_) => "ref.is_null",
            Other::V128BitSelect(_) => "v128.bitselect",
            Other::I8x16Swizzle(_) => "i8x16.swizzle",
            Other::I8x16Shuffle(_) => "i8x16.shuffle",
            Other::LoadSimd(load) => match load.kind {
                ir::LoadSimdKind::Splat8 => "v128.load8_splat",
                ir::LoadSimdKind::Splat16 => "v128.load16_splat",
                ir::LoadSimdKind::Splat32 => "v128.load32_splat",
                ir::LoadSimdKind::Splat64 => "v128.load64_splat",
                ir::LoadSimdKind::V128Load8x8S => "v128.load8x8_s",
                ir::LoadSimdKind::V128Load8x8U => "v128.load8x8_u",
                ir::LoadSimdKind::V128Load16x4S => "v128.load16x4_s",
                ir::LoadSimdKind::V128Load16x4U => "v128.load16x4_u",
                ir::LoadSimdKind::V128Load32x2S => "v128.load32x2_s",
                ir::LoadSimdKind::V128Load32x2U => "v128.load32x2_u",
                ir::LoadSimdKind::V128Load32Zero => "v128.load32_zero",
                ir::LoadSimdKind::V128Load64Zero => "v128.load64_zero",
                ir::LoadSimdKind::V128Load8Lane(_) => "v128.load8_lane",
                ir::LoadSimdKind::V128Load16Lane(_) => "v128.load16_lane",
                ir::LoadSimdKind::V128Load32Lane(_) => "v128.load32_lane",
                ir::LoadSimdKind::V128Load64Lane(_) => "v128.load64_lane",
                ir::LoadSimdKind::V128Store8Lane(_) => "v128.store8_lane",
                ir::LoadSimdKind::V128Store16Lane(_) => "v128.store16_lane",
                ir::LoadSimdKind::V128Store32Lane(_) => "v128.store32_lane",
                ir::LoadSimdKind::V128Store64Lane(_) => "v128.store64_lane",
            },
            Other::TableInit(_) => "table.init",
            Other::ElemDrop(_) => "elem.drop",
            Other::TableCopy(_) => "table.copy",
        }
    }
}

impl Test {
    fn stack_type<'m, 's>(
        &self,
        _module: &'m Module,
        _localfn: &LocalFunction,
        _cur_stack: &'s [ValType],
    ) -> (&'s [ValType], &'m [ValType])
    where
        'm: 's,
    {
        match self {
            Test::I32Nez | Test::I32Eqz => (&[ValType::I32], &[ValType::I32]),
            Test::I32Eq
            | Test::I32Ne
            | Test::I32LtS
            | Test::I32LtU
            | Test::I32GtS
            | Test::I32GtU
            | Test::I32LeS
            | Test::I32LeU
            | Test::I32GeS
            | Test::I32GeU => (&[ValType::I32, ValType::I32], &[ValType::I32]),
            Test::F32Eq | Test::F32Ne | Test::F32Lt | Test::F32Gt | Test::F32Le | Test::F32Ge => {
                (&[ValType::F32, ValType::F32], &[ValType::I32])
            }
        }
    }

    fn from_unop(op: ir::UnaryOp) -> Option<Self> {
        match op {
            ir::UnaryOp::I32Eqz => Some(Test::I32Eqz),
            _ => None,
        }
    }

    fn from_binop(op: ir::BinaryOp) -> Option<Self> {
        match op {
            ir::BinaryOp::I32Eq => Some(Test::I32Eq),
            ir::BinaryOp::I32Ne => Some(Test::I32Ne),
            ir::BinaryOp::I32LtS => Some(Test::I32LtS),
            ir::BinaryOp::I32LtU => Some(Test::I32LtU),
            ir::BinaryOp::I32GtS => Some(Test::I32GtS),
            ir::BinaryOp::I32GtU => Some(Test::I32GtU),
            ir::BinaryOp::I32LeS => Some(Test::I32LeS),
            ir::BinaryOp::I32LeU => Some(Test::I32LeU),
            ir::BinaryOp::I32GeS => Some(Test::I32GeS),
            ir::BinaryOp::I32GeU => Some(Test::I32GeU),
            ir::BinaryOp::F32Eq => Some(Test::F32Eq),
            ir::BinaryOp::F32Ne => Some(Test::F32Ne),
            ir::BinaryOp::F32Lt => Some(Test::F32Lt),
            ir::BinaryOp::F32Gt => Some(Test::F32Gt),
            ir::BinaryOp::F32Le => Some(Test::F32Le),
            ir::BinaryOp::F32Ge => Some(Test::F32Ge),
            _ => None,
        }
    }
}

pub fn classify(seq: &ir::InstrSeq) -> Vec<InstrClass> {
    let mut out = Vec::with_capacity(seq.len());
    let mut seqiter = seq.iter().peekable();
    while let Some((instr, _)) = seqiter.next() {
        match instr {
            ir::Instr::Block(block) => out.push(InstrClass::Block(Block::Block(block.clone()))),
            ir::Instr::Loop(l) => out.push(InstrClass::Loop(Loop::Loop(l.clone()))),
            ir::Instr::Call(call) => out.push(InstrClass::Other(Other::Call(call.clone()))),
            ir::Instr::CallIndirect(call_indirect) => out.push(InstrClass::Other(
                Other::CallIndirect(call_indirect.clone()),
            )),
            ir::Instr::LocalGet(local_get) => {
                out.push(InstrClass::Load(Load::LocalGet(local_get.clone())))
            }
            ir::Instr::LocalSet(local_set) => {
                out.push(InstrClass::Store(Store::LocalSet(local_set.clone())))
            }
            ir::Instr::LocalTee(local_tee) => {
                out.push(InstrClass::Other(Other::LocalTee(local_tee.clone())))
            }
            ir::Instr::GlobalGet(global_get) => {
                out.push(InstrClass::Load(Load::GlobalGet(global_get.clone())))
            }
            ir::Instr::GlobalSet(global_set) => {
                out.push(InstrClass::Store(Store::GlobalSet(global_set.clone())))
            }
            ir::Instr::Const(c) => out.push(InstrClass::Load(Load::Const(c.clone()))),
            ir::Instr::Binop(binop) => {
                if let Some(test) = Test::from_binop(binop.op) {
                    match seqiter.peek() {
                        Some((ir::Instr::Select(select), _)) => {
                            out.push(InstrClass::Other(Other::Select(test, select.clone())));
                            seqiter.next();
                        }
                        Some((ir::Instr::BrIf(brif), _)) => {
                            out.push(InstrClass::Other(Other::BrIf(test, brif.clone())));
                            seqiter.next();
                        }
                        Some((ir::Instr::IfElse(ifelse), _)) => {
                            out.push(InstrClass::Block(Block::IfElse(test, ifelse.clone())));
                            seqiter.next();
                        }
                        _ => {
                            out.push(InstrClass::Other(Other::Binop(binop.clone())));
                        }
                    }
                } else {
                    out.push(InstrClass::Other(Other::Binop(binop.clone())));
                }
            }
            ir::Instr::Unop(unop) => {
                if let Some(test) = Test::from_unop(unop.op) {
                    match seqiter.peek() {
                        Some((ir::Instr::Select(select), _)) => {
                            out.push(InstrClass::Other(Other::Select(test, select.clone())));
                            seqiter.next();
                        }
                        Some((ir::Instr::BrIf(brif), _)) => {
                            out.push(InstrClass::Other(Other::BrIf(test, brif.clone())));
                            seqiter.next();
                        }
                        Some((ir::Instr::IfElse(ifelse), _)) => {
                            out.push(InstrClass::Block(Block::IfElse(test, ifelse.clone())));
                            seqiter.next();
                        }
                        _ => {
                            out.push(InstrClass::Other(Other::Unop(unop.clone())));
                        }
                    }
                } else {
                    out.push(InstrClass::Other(Other::Unop(unop.clone())));
                }
            }
            ir::Instr::Select(select) => {
                out.push(InstrClass::Other(Other::Select(
                    Test::I32Nez,
                    select.clone(),
                )));
            }
            ir::Instr::Unreachable(unreachable) => {
                out.push(InstrClass::Other(Other::Unreachable(unreachable.clone())));
            }
            ir::Instr::Br(br) => out.push(InstrClass::Other(Other::Br(br.clone()))),
            ir::Instr::BrIf(brif) => {
                out.push(InstrClass::Other(Other::BrIf(Test::I32Nez, brif.clone())));
            }
            ir::Instr::IfElse(ifelse) => out.push(InstrClass::Block(Block::IfElse(
                Test::I32Nez,
                ifelse.clone(),
            ))),
            ir::Instr::BrTable(brtable) => {
                out.push(InstrClass::Other(Other::BrTable(brtable.clone())));
            }
            ir::Instr::Drop(drop) => out.push(InstrClass::Store(Store::Drop(drop.clone()))),
            ir::Instr::Return(ret) => out.push(InstrClass::Ret(Ret::Return(ret.clone()))),
            ir::Instr::MemorySize(memsize) => {
                out.push(InstrClass::Other(Other::MemorySize(memsize.clone())));
            }
            ir::Instr::MemoryGrow(memgrow) => {
                out.push(InstrClass::Other(Other::MemoryGrow(memgrow.clone())));
            }
            ir::Instr::MemoryInit(meminit) => {
                out.push(InstrClass::Other(Other::MemoryInit(meminit.clone())));
            }
            ir::Instr::DataDrop(datadrop) => {
                out.push(InstrClass::Other(Other::DataDrop(datadrop.clone())));
            }
            ir::Instr::MemoryCopy(memcopy) => {
                out.push(InstrClass::Other(Other::MemoryCopy(memcopy.clone())));
            }
            ir::Instr::MemoryFill(memfill) => {
                out.push(InstrClass::Other(Other::MemoryFill(memfill.clone())));
            }
            ir::Instr::Load(load) => {
                out.push(InstrClass::Other(Other::Load(load.clone())));
            }
            ir::Instr::Store(store) => {
                out.push(InstrClass::Other(Other::Store(store.clone())));
            }
            ir::Instr::AtomicRmw(rmw) => {
                out.push(InstrClass::Other(Other::AtomicRmw(rmw.clone())));
            }
            ir::Instr::Cmpxchg(cmpxchg) => {
                out.push(InstrClass::Other(Other::Cmpxchg(cmpxchg.clone())));
            }
            ir::Instr::AtomicNotify(notify) => {
                out.push(InstrClass::Other(Other::AtomicNotify(notify.clone())));
            }
            ir::Instr::AtomicWait(wait) => {
                out.push(InstrClass::Other(Other::AtomicWait(wait.clone())));
            }
            ir::Instr::AtomicFence(fence) => {
                out.push(InstrClass::Other(Other::AtomicFence(fence.clone())));
            }
            ir::Instr::TableGet(table_get) => {
                out.push(InstrClass::Other(Other::TableGet(table_get.clone())));
            }
            ir::Instr::TableSet(table_set) => {
                out.push(InstrClass::Other(Other::TableSet(table_set.clone())));
            }
            ir::Instr::TableGrow(table_grow) => {
                out.push(InstrClass::Other(Other::TableGrow(table_grow.clone())));
            }
            ir::Instr::TableSize(table_size) => {
                out.push(InstrClass::Load(Load::TableSize(table_size.clone())));
            }
            ir::Instr::TableFill(table_fill) => {
                out.push(InstrClass::Other(Other::TableFill(table_fill.clone())));
            }
            ir::Instr::RefNull(ref_null) => {
                out.push(InstrClass::Load(Load::RefNull(ref_null.clone())));
            }
            ir::Instr::RefIsNull(ref_is_null) => {
                out.push(InstrClass::Other(Other::RefIsNull(ref_is_null.clone())));
            }
            ir::Instr::RefFunc(ref_func) => {
                out.push(InstrClass::Load(Load::RefFunc(ref_func.clone())));
            }
            ir::Instr::V128Bitselect(bitsel) => {
                out.push(InstrClass::Other(Other::V128BitSelect(bitsel.clone())));
            }
            ir::Instr::I8x16Swizzle(swizzle) => {
                out.push(InstrClass::Other(Other::I8x16Swizzle(swizzle.clone())));
            }
            ir::Instr::I8x16Shuffle(shuffle) => {
                out.push(InstrClass::Other(Other::I8x16Shuffle(shuffle.clone())));
            }
            ir::Instr::LoadSimd(load_simd) => {
                out.push(InstrClass::Other(Other::LoadSimd(load_simd.clone())));
            }
            ir::Instr::TableInit(table_init) => {
                out.push(InstrClass::Other(Other::TableInit(table_init.clone())));
            }
            ir::Instr::ElemDrop(elem_drop) => {
                out.push(InstrClass::Other(Other::ElemDrop(elem_drop.clone())));
            }
            ir::Instr::TableCopy(table_copy) => {
                out.push(InstrClass::Other(Other::TableCopy(table_copy.clone())));
            }
        }
    }

    out
}

pub fn subsequences(seq: &ir::InstrSeq) -> Vec<InstrSubseq> {
    enum State {
        Start,
        SeenLoad,
        SeenNucleus,
    }

    let mut subseqs = Vec::new();
    let mut loads = Vec::new();
    let mut block = None;
    let mut looop = None;
    let mut other = None;
    let mut stores = Vec::new();
    let mut ret = None;
    let mut state = State::Start;

    macro_rules! subseq_done {
        () => {
            if let Some(block) = block.take() {
                subseqs.push(InstrSubseq::Block {
                    block,
                    loads: std::mem::take(&mut loads),
                })
            } else if let Some(looop) = looop.take() {
                subseqs.push(InstrSubseq::Loop {
                    looop,
                    stores: std::mem::take(&mut stores),
                    ret: ret.take(),
                })
            } else if let Some(other) = other.take() {
                subseqs.push(InstrSubseq::Other {
                    other,
                    loads: std::mem::take(&mut loads),
                    stores: std::mem::take(&mut stores),
                    ret: ret.take(),
                })
            } else {
                subseqs.push(InstrSubseq::Copy {
                    loads: std::mem::take(&mut loads),
                    stores: std::mem::take(&mut stores),
                    ret: ret.take(),
                })
            }
        };
    }

    for class in classify(seq) {
        match state {
            State::Start => match class {
                InstrClass::Load(load) => {
                    loads.push(load);
                    state = State::SeenLoad;
                }
                InstrClass::Store(store) => {
                    stores.push(store);
                    state = State::SeenNucleus;
                }
                InstrClass::Ret(this_ret) => {
                    ret = Some(this_ret);
                    subseq_done!();
                    state = State::Start;
                    break;
                }
                InstrClass::Block(this_block) => {
                    block = Some(this_block);
                    subseq_done!();
                    state = State::Start;
                }
                InstrClass::Loop(this_loop) => {
                    looop = Some(this_loop);
                    state = State::SeenNucleus;
                }
                InstrClass::Other(this_other) => {
                    other = Some(this_other);
                    state = State::SeenNucleus;
                }
            },
            State::SeenLoad => match class {
                InstrClass::Load(load) => {
                    loads.push(load);
                    state = State::SeenLoad;
                }
                InstrClass::Store(store) => {
                    stores.push(store);
                    state = State::SeenNucleus;
                }
                InstrClass::Ret(this_ret) => {
                    ret = Some(this_ret);
                    subseq_done!();
                    state = State::Start;
                    break;
                }
                InstrClass::Block(this_block) => {
                    block = Some(this_block);
                    subseq_done!();
                    state = State::Start;
                }
                InstrClass::Loop(this_loop) => {
                    subseq_done!();
                    looop = Some(this_loop);
                    state = State::SeenNucleus;
                }
                InstrClass::Other(this_other) => {
                    other = Some(this_other);
                    state = State::SeenNucleus;
                }
            },
            State::SeenNucleus => match class {
                InstrClass::Load(load) => {
                    subseq_done!();
                    loads.push(load);
                    state = State::SeenLoad;
                }
                InstrClass::Store(store) => {
                    stores.push(store);
                }
                InstrClass::Ret(this_ret) => {
                    ret = Some(this_ret);
                    subseq_done!();
                    state = State::Start;
                    break;
                }
                InstrClass::Block(this_block) => {
                    subseq_done!();
                    block = Some(this_block);
                    state = State::SeenNucleus;
                }
                InstrClass::Loop(this_loop) => {
                    subseq_done!();
                    looop = Some(this_loop);
                    state = State::SeenNucleus;
                }
                InstrClass::Other(this_other) => {
                    subseq_done!();
                    other = Some(this_other);
                    state = State::SeenNucleus;
                }
            },
        }
    }

    match state {
        State::Start => {}
        _ => {
            subseq_done!();
        }
    }

    subseqs
}
