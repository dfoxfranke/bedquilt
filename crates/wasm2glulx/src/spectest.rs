use anyhow::{anyhow, bail, Context, Result};
use bytes::Buf;
use hex::FromHex;
use std::{io::Write, ops::BitAnd, path::Path, process::Command};
use walrus::{ir::Value, ConstExpr, ExportId, ExportItem, FunctionBuilder, Module, ValType};
use wast::{
    core::{AbstractHeapType, HeapType, NanPattern, V128Pattern, WastArgCore, WastRetCore},
    parser::ParseBuffer,
    WastArg, WastDirective, WastExecute, WastInvoke, WastRet,
};

use crate::{compile_module_to_bytes, CompilationError};

use super::CompilationOptions;

#[derive(Debug)]
pub struct WastTest {
    pub line_col: (usize, usize),
    pub module: Vec<u8>,
    pub expected_result: ExpectedResult,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpectedResult {
    Return(Vec<ExpectedValue>),
    Trap(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ActualResult {
    Return(Vec<u8>),
    Trap(String),
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterpretedResult {
    Return(Vec<InterpretedValue>),
    Uninterpretable(Vec<u8>),
    Trap(String),
    Error(String),
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum InterpretedValue {
    I32(i32),
    I64(i64),
    F32(u32),
    F64(u64),
    I8x16([i8; 16]),
    I16x8([i16; 8]),
    I32x4([i32; 4]),
    I64x2([i64; 2]),
    F32x4([u32; 4]),
    F64x2([u64; 2]),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ExpectedValue {
    I32(i32),
    I64(i64),
    F32(F32),
    F64(F64),
    I8x16([i8; 16]),
    I16x8([i16; 8]),
    I32x4([i32; 4]),
    I64x2([i64; 2]),
    F32x4([F32; 4]),
    F64x2([F64; 2]),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum F32 {
    CanonicalNan,
    ArithmeticNan,
    Value(u32),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum F64 {
    CanonicalNan,
    ArithmeticNan,
    Value(u64),
}

impl From<NanPattern<wast::token::F32>> for F32 {
    fn from(value: NanPattern<wast::token::F32>) -> Self {
        match value {
            NanPattern::CanonicalNan => F32::CanonicalNan,
            NanPattern::ArithmeticNan => F32::ArithmeticNan,
            NanPattern::Value(x) => F32::Value(x.bits),
        }
    }
}

impl From<NanPattern<wast::token::F64>> for F64 {
    fn from(value: NanPattern<wast::token::F64>) -> Self {
        match value {
            NanPattern::CanonicalNan => F64::CanonicalNan,
            NanPattern::ArithmeticNan => F64::ArithmeticNan,
            NanPattern::Value(x) => F64::Value(x.bits),
        }
    }
}

impl InterpretedResult {
    fn interpret(expected: &ExpectedResult, actual: &ActualResult) -> InterpretedResult {
        match actual {
            ActualResult::Return(av) => match expected {
                ExpectedResult::Return(evs) => {
                    let mut iv = Vec::new();
                    let mut buf = av.as_slice();
                    for ev in evs {
                        match ev {
                            ExpectedValue::I32(_) => {
                                if buf.remaining() >= 4 {
                                    iv.push(InterpretedValue::I32(buf.get_i32()));
                                } else {
                                    return InterpretedResult::Uninterpretable(av.clone());
                                }
                            }
                            ExpectedValue::I64(_) => {
                                if buf.remaining() >= 8 {
                                    iv.push(InterpretedValue::I64(buf.get_i64()))
                                } else {
                                    return InterpretedResult::Uninterpretable(av.clone());
                                }
                            }
                            ExpectedValue::F32(_) => {
                                if buf.remaining() >= 4 {
                                    iv.push(InterpretedValue::F32(buf.get_u32()));
                                } else {
                                    return InterpretedResult::Uninterpretable(av.clone());
                                }
                            }
                            ExpectedValue::F64(_) => {
                                if buf.remaining() >= 8 {
                                    iv.push(InterpretedValue::F64(buf.get_u64()))
                                } else {
                                    return InterpretedResult::Uninterpretable(av.clone());
                                }
                            }
                            ExpectedValue::I8x16(_) => {
                                if buf.remaining() >= 16 {
                                    let mut arr = [
                                        buf.get_i8(),
                                        buf.get_i8(),
                                        buf.get_i8(),
                                        buf.get_i8(),
                                        buf.get_i8(),
                                        buf.get_i8(),
                                        buf.get_i8(),
                                        buf.get_i8(),
                                        buf.get_i8(),
                                        buf.get_i8(),
                                        buf.get_i8(),
                                        buf.get_i8(),
                                        buf.get_i8(),
                                        buf.get_i8(),
                                        buf.get_i8(),
                                        buf.get_i8(),
                                    ];
                                    arr.reverse();
                                    iv.push(InterpretedValue::I8x16(arr))
                                } else {
                                    return InterpretedResult::Uninterpretable(av.clone());
                                }
                            }
                            ExpectedValue::I16x8(_) => {
                                if buf.remaining() >= 16 {
                                    let mut arr = [
                                        buf.get_i16(),
                                        buf.get_i16(),
                                        buf.get_i16(),
                                        buf.get_i16(),
                                        buf.get_i16(),
                                        buf.get_i16(),
                                        buf.get_i16(),
                                        buf.get_i16(),
                                    ];
                                    arr.reverse();
                                    iv.push(InterpretedValue::I16x8(arr))
                                } else {
                                    return InterpretedResult::Uninterpretable(av.clone());
                                }
                            }
                            ExpectedValue::I32x4(_) => {
                                if buf.remaining() >= 16 {
                                    let mut arr = [
                                        buf.get_i32(),
                                        buf.get_i32(),
                                        buf.get_i32(),
                                        buf.get_i32(),
                                    ];
                                    arr.reverse();
                                    iv.push(InterpretedValue::I32x4(arr))
                                } else {
                                    return InterpretedResult::Uninterpretable(av.clone());
                                }
                            }
                            ExpectedValue::I64x2(_) => {
                                if buf.remaining() >= 16 {
                                    let mut arr = [buf.get_i64(), buf.get_i64()];
                                    arr.reverse();
                                    iv.push(InterpretedValue::I64x2(arr))
                                } else {
                                    return InterpretedResult::Uninterpretable(av.clone());
                                }
                            }
                            ExpectedValue::F32x4(_) => {
                                if buf.remaining() >= 16 {
                                    let mut arr = [
                                        buf.get_u32(),
                                        buf.get_u32(),
                                        buf.get_u32(),
                                        buf.get_u32(),
                                    ];
                                    arr.reverse();
                                    iv.push(InterpretedValue::F32x4(arr))
                                } else {
                                    return InterpretedResult::Uninterpretable(av.clone());
                                }
                            }
                            ExpectedValue::F64x2(_) => {
                                if buf.remaining() >= 16 {
                                    let mut arr = [buf.get_u64(), buf.get_u64()];
                                    arr.reverse();
                                    iv.push(InterpretedValue::F64x2(arr))
                                } else {
                                    return InterpretedResult::Uninterpretable(av.clone());
                                }
                            }
                        }
                    }
                    InterpretedResult::Return(iv)
                }
                ExpectedResult::Trap(_) => InterpretedResult::Uninterpretable(av.clone()),
            },
            ActualResult::Trap(e) => InterpretedResult::Trap(e.clone()),
            ActualResult::Error(e) => InterpretedResult::Error(e.clone()),
        }
    }
}

impl PartialEq<u32> for F32 {
    fn eq(&self, other: &u32) -> bool {
        match self {
            F32::CanonicalNan => other.bitand(0x7fffffff) == 0x7fc00000,
            F32::ArithmeticNan => other.bitand(0x7fc00000) == 0x7fc00000,
            F32::Value(bits) => *other == *bits,
        }
    }
}

impl PartialEq<u64> for F64 {
    fn eq(&self, other: &u64) -> bool {
        match self {
            F64::CanonicalNan => other.bitand(0x7fffffffffffffff) == 0x7ff0000000000000,
            F64::ArithmeticNan => other.bitand(0x7ff0000000000000) == 0x7ff0000000000000,
            F64::Value(bits) => *other == *bits,
        }
    }
}

impl PartialEq<ExpectedResult> for InterpretedResult {
    fn eq(&self, expected: &ExpectedResult) -> bool {
        match (self, expected) {
            (InterpretedResult::Trap(i), ExpectedResult::Trap(e)) => i == e,
            (InterpretedResult::Return(ivs), ExpectedResult::Return(evs)) => {
                if ivs.len() != evs.len() {
                    false
                } else {
                    std::iter::zip(ivs, evs).all(|(iv, ev)| match (ev, iv) {
                        (ExpectedValue::I32(x), InterpretedValue::I32(y)) => *x == *y,
                        (ExpectedValue::I64(x), InterpretedValue::I64(y)) => *x == *y,
                        (ExpectedValue::F32(x), InterpretedValue::F32(y)) => *x == *y,
                        (ExpectedValue::F64(x), InterpretedValue::F64(y)) => *x == *y,
                        (ExpectedValue::I8x16(x), InterpretedValue::I8x16(y)) => *x == *y,
                        (ExpectedValue::I16x8(x), InterpretedValue::I16x8(y)) => *x == *y,
                        (ExpectedValue::I32x4(x), InterpretedValue::I32x4(y)) => *x == *y,
                        (ExpectedValue::I64x2(x), InterpretedValue::I64x2(y)) => *x == *y,
                        (ExpectedValue::F32x4(x), InterpretedValue::F32x4(y)) => *x == *y,
                        (ExpectedValue::F64x2(x), InterpretedValue::F64x2(y)) => *x == *y,
                        _ => false,
                    })
                }
            }
            _ => false,
        }
    }
}

pub fn wast_to_tests(input: &str) -> Result<Vec<WastTest>> {
    let buffer = ParseBuffer::new(input).context("failed to lex the input")?;
    let wast = wast::parser::parse::<wast::Wast>(&buffer).context("failed to parse the input")?;
    let mut encoded_module: Vec<u8> = Vec::new();
    let mut invokes: Vec<(String, Vec<ConstExpr>)> = Vec::new();

    let mut out = Vec::new();
    for directive in wast.directives {
        match directive {
            WastDirective::Module(wast::QuoteWat::Wat(wast::Wat::Module(mut wast_module))) => {
                encoded_module = wast_module
                    .encode()
                    .context("failed to encode parsed module")?;
                invokes.clear();
            }
            WastDirective::Module(_) => {
                bail!("Encountered unsupported module pattern");
            }
            WastDirective::Invoke(WastInvoke {
                module: module_id,
                name,
                args,
                ..
            }) => {
                if module_id.is_some() {
                    bail!("Invoking imported modules is not supported");
                }

                let args_out = wast_args_to_constexprs(args)?;
                invokes.push((name.to_owned(), args_out));
            }
            WastDirective::AssertReturn {
                span,
                exec,
                results,
                ..
            } => {
                if encoded_module.is_empty() {
                    bail!("Encountered AssertReturn with no module defined");
                }
                let expected = wast_rets_to_expecteds(results)?;
                let mut module = build_module(encoded_module.as_slice(), invokes.as_slice(), exec)?;
                let (l, c) = span.linecol_in(input);

                out.push(WastTest {
                    line_col: (l + 1, c + 1),
                    module: module.emit_wasm(),
                    expected_result: ExpectedResult::Return(expected),
                });
            }
            WastDirective::AssertTrap {
                span,
                exec,
                message,
                ..
            } => {
                if encoded_module.is_empty() {
                    bail!("Encountered AssertTrap with no module defined");
                }

                let mut module = build_module(encoded_module.as_slice(), invokes.as_slice(), exec)?;
                let (l, c) = span.linecol_in(input);

                out.push(WastTest {
                    line_col: (l + 1, c + 1),
                    module: module.emit_wasm(),
                    expected_result: ExpectedResult::Trap(message.to_owned()),
                });
            }
            WastDirective::AssertExhaustion {
                span,
                call,
                message,
                ..
            } => {
                if encoded_module.is_empty() {
                    bail!("Encountered AssertExhaustion with no module defined");
                }

                let exec = WastExecute::Invoke(call);
                let mut module = build_module(encoded_module.as_slice(), invokes.as_slice(), exec)?;
                let (l, c) = span.linecol_in(input);

                out.push(WastTest {
                    line_col: (l + 1, c + 1),
                    module: module.emit_wasm(),
                    expected_result: ExpectedResult::Trap(message.to_owned()),
                });
            }
            WastDirective::AssertException { .. }
            | WastDirective::AssertInvalid { .. }
            | WastDirective::AssertUnlinkable { .. }
            | WastDirective::AssertMalformed { .. } => {}
            x => {
                bail!("Encountered unsupported directive {:?}", x);
            }
        }
    }
    Ok(out)
}

fn wast_args_to_constexprs(args: Vec<WastArg>) -> Result<Vec<ConstExpr>> {
    let mut out = Vec::with_capacity(args.len());
    for arg in args {
        out.push(match arg {
            wast::WastArg::Core(WastArgCore::I32(x)) => ConstExpr::Value(Value::I32(x)),
            wast::WastArg::Core(WastArgCore::I64(x)) => ConstExpr::Value(Value::I64(x)),
            wast::WastArg::Core(WastArgCore::F32(x)) => {
                ConstExpr::Value(Value::F32(f32::from_bits(x.bits)))
            }
            wast::WastArg::Core(WastArgCore::F64(x)) => {
                ConstExpr::Value(Value::F64(f64::from_bits(x.bits)))
            }
            wast::WastArg::Core(WastArgCore::V128(x)) => {
                ConstExpr::Value(Value::V128(u128::from_le_bytes(x.to_le_bytes())))
            }
            wast::WastArg::Core(WastArgCore::RefNull(HeapType::Abstract {
                ty: AbstractHeapType::Func,
                ..
            })) => ConstExpr::RefNull(walrus::RefType::Funcref),
            wast::WastArg::Core(WastArgCore::RefNull(HeapType::Abstract {
                ty: AbstractHeapType::Extern,
                ..
            })) => ConstExpr::RefNull(walrus::RefType::Externref),
            x => {
                bail!("Unsupported WastArg {:?}", x);
            }
        })
    }
    Ok(out)
}

fn wast_rets_to_expecteds(rets: Vec<WastRet>) -> Result<Vec<ExpectedValue>> {
    let mut out = Vec::with_capacity(rets.len());
    for ret in rets {
        out.push(match ret {
            WastRet::Core(WastRetCore::I32(x)) => ExpectedValue::I32(x),
            WastRet::Core(WastRetCore::I64(x)) => ExpectedValue::I64(x),
            WastRet::Core(WastRetCore::F32(x)) => ExpectedValue::F32(x.into()),
            WastRet::Core(WastRetCore::F64(x)) => ExpectedValue::F64(x.into()),
            WastRet::Core(WastRetCore::V128(V128Pattern::I8x16(x))) => ExpectedValue::I8x16(x),
            WastRet::Core(WastRetCore::V128(V128Pattern::I16x8(x))) => ExpectedValue::I16x8(x),
            WastRet::Core(WastRetCore::V128(V128Pattern::I32x4(x))) => ExpectedValue::I32x4(x),
            WastRet::Core(WastRetCore::V128(V128Pattern::I64x2(x))) => ExpectedValue::I64x2(x),
            WastRet::Core(WastRetCore::V128(V128Pattern::F32x4([x, y, z, w]))) => {
                ExpectedValue::F32x4([x.into(), y.into(), z.into(), w.into()])
            }
            WastRet::Core(WastRetCore::V128(V128Pattern::F64x2([x, y]))) => {
                ExpectedValue::F64x2([x.into(), y.into()])
            }
            WastRet::Core(WastRetCore::RefNull(_)) => ExpectedValue::I32(0),
            x => {
                bail!("Unsupported WastRet {:?}", x);
            }
        })
    }
    Ok(out)
}

fn build_module(
    encoded_module: &[u8],
    invokes: &[(String, Vec<ConstExpr>)],
    execute: WastExecute,
) -> Result<Module> {
    let mut module =
        Module::from_buffer(encoded_module).context("failed to build walrus module")?;

    let result_type = find_result_type(&module, &execute)?;
    let ty_id = module.types.add(&result_type, &[]);
    let (spectest_result_id, _) = module.add_import_func("glulx", "spectest_result", ty_id);
    let mut builder = FunctionBuilder::new(&mut module.types, &[], &[]);
    builder.name("glulx_main".to_owned());
    let mut body = builder.func_body();

    for (invoke, args) in invokes {
        for arg in args {
            match arg {
                ConstExpr::Value(v) => {
                    body.const_(*v);
                }
                ConstExpr::Global(g) => {
                    body.global_get(*g);
                }
                ConstExpr::RefNull(t) => {
                    body.ref_null(*t);
                }
                ConstExpr::RefFunc(f) => {
                    body.ref_func(*f);
                }
            }
        }

        let invoke_fnid = module
            .exports
            .get_func(invoke)
            .context("Failed to locate invoked function")?;
        body.call(invoke_fnid);
        let invoke_fn = module.funcs.get(invoke_fnid);
        let invoke_ty = module.types.get(invoke_fn.ty());
        for _ in invoke_ty.results() {
            body.drop();
        }
    }

    match execute {
        WastExecute::Invoke(invoke) => {
            if invoke.module.is_some() {
                bail!(
                    "At {:?}: functions from imported modules are not supported",
                    invoke.span
                );
            }

            let args = wast_args_to_constexprs(invoke.args)?;
            for arg in args {
                match arg {
                    ConstExpr::Value(v) => {
                        body.const_(v);
                    }
                    ConstExpr::Global(g) => {
                        body.global_get(g);
                    }
                    ConstExpr::RefNull(t) => {
                        body.ref_null(t);
                    }
                    ConstExpr::RefFunc(f) => {
                        body.ref_func(f);
                    }
                }
            }

            let invoke_fnid = module
                .exports
                .get_func(invoke.name)
                .context("Failed to locate invoked function")?;
            body.call(invoke_fnid);
            body.call(spectest_result_id);
        }
        WastExecute::Get {
            span,
            module: imported_module,
            global: global_name,
        } => {
            if imported_module.is_some() {
                bail!(
                    "At {:?}: functions from imported modules are not supported",
                    span
                );
            }

            let global_id = module
                .exports
                .iter()
                .find_map(|export| {
                    if export.name.as_str() == global_name {
                        match export.item {
                            ExportItem::Global(id) => Some(id),
                            _ => None,
                        }
                    } else {
                        None
                    }
                })
                .ok_or(anyhow!(
                    "At {:?}: failed to locate export {}",
                    span,
                    global_name
                ))?;

            body.global_get(global_id);
            body.call(spectest_result_id);
        }
        _ => bail!("Encountered unsupported WastExecute"),
    }

    let built = builder.finish(vec![], &mut module.funcs);
    let export_ids: Vec<ExportId> = module.exports.iter().map(|ex| ex.id()).collect();
    for id in export_ids {
        module.exports.delete(id);
    }
    module.exports.add("glulx_main", built);

    walrus::passes::gc::run(&mut module);

    Ok(module)
}

fn find_result_type(module: &Module, execute: &WastExecute) -> Result<Vec<ValType>> {
    match execute {
        WastExecute::Invoke(WastInvoke {
            span,
            module: imported_module,
            name,
            ..
        }) => {
            if imported_module.is_some() {
                bail!(
                    "At {:?}: functions from imported modules are not supported",
                    span
                );
            }

            let function_id = module.exports.get_func(name).context(format!(
                "Failed to locate exported function {} at {:?}",
                name, span
            ))?;

            let function = module.funcs.get(function_id);
            let ty_id = function.ty();
            let ty = module.types.get(ty_id);
            Ok(ty.results().to_owned())
        }
        WastExecute::Get {
            span,
            module: imported_module,
            global: global_name,
            ..
        } => {
            if imported_module.is_some() {
                bail!(
                    "At {:?}: globals from imported modules are not supported",
                    span
                );
            }

            let global_id = module
                .exports
                .iter()
                .find_map(|export| {
                    if export.name.as_str() == *global_name {
                        match export.item {
                            ExportItem::Global(id) => Some(id),
                            _ => None,
                        }
                    } else {
                        None
                    }
                })
                .ok_or(anyhow!(
                    "At {:?}: failed to locate export {}",
                    span,
                    global_name
                ))?;

            let global = module.globals.get(global_id);
            Ok(vec![global.ty])
        }
        _ => bail!("Encountered unsupported WastExecute"),
    }
}

impl WastTest {
    pub fn run(&self, workdir: &Path, stem: &str) {
        std::fs::create_dir_all(workdir).unwrap();

        let mut error_path = workdir.to_owned();
        error_path.push(stem);
        error_path.set_extension("compile_error");
        let _ = std::fs::remove_file(&error_path);

        let mut story_path = workdir.to_owned();
        story_path.push(stem);
        story_path.set_extension("ulx");
        let _ = std::fs::remove_file(&story_path);

        let mut actual_path = workdir.to_owned();
        actual_path.push(stem);
        actual_path.set_extension("actual");
        let _ = std::fs::remove_file(&actual_path);

        let mut expected_path = workdir.to_owned();
        expected_path.push(stem);
        expected_path.set_extension("expected");
        let _ = std::fs::remove_file(&expected_path);

        let mut glulxasm_path = workdir.to_owned();
        glulxasm_path.push(stem);
        glulxasm_path.set_extension("glulxasm");
        let _ = std::fs::remove_file(&glulxasm_path);

        let mut wasm_path = workdir.to_owned();
        wasm_path.push(stem);
        wasm_path.set_extension("wasm");
        let _ = std::fs::remove_file(&wasm_path);

        std::fs::write(&wasm_path, &self.module).unwrap();

        let module = walrus::Module::from_buffer(&self.module)
            .expect("WASM module bytecode produced by WAST should be valid");
        let compiled = match super::compile_module_to_bytes(&CompilationOptions::new(), &module) {
            Ok(compiled) => compiled,
            Err(ev) => {
                // Uncomment if needed to debug missing/duplicate labels, etc.
                // let mut options = CompilationOptions::new();
                // options.set_text(true);
                // if let Ok(asm_out) = compile_module_to_bytes(&options, &module) {
                //     std::fs::write(&asm_path, &asm_out).unwrap();
                // }

                if ev
                    .iter()
                    .all(|e| matches!(e, CompilationError::UnsupportedInstruction { .. }))
                {
                    let _ = std::fs::remove_file(&wasm_path);
                    return;
                }

                let mut error_out = std::fs::File::create(&error_path).unwrap();
                for e in &ev {
                    writeln!(error_out, "{e}").unwrap();
                }
                panic!("Compilation failed. First error: {}", &ev[0]);
            }
        };

        std::fs::write(&story_path, &compiled).unwrap();

        let bogoglulx_output = match Command::new(env!("BOGOGLULX_BIN"))
            .arg(&story_path)
            .output()
        {
            Ok(output) => output,
            Err(e) => panic!("bogoglulx execution failed: {e}"),
        };

        let bogoglulx_output_str = std::str::from_utf8(&bogoglulx_output.stdout)
            .expect("Bogoglulx output should be valid UTF-8");

        let actual = if let Some(index) = bogoglulx_output_str.find('!') {
            ActualResult::Trap(bogoglulx_output_str[index + 1..].to_owned())
        } else if let Some(index) = bogoglulx_output_str.find('?') {
            ActualResult::Error(bogoglulx_output_str[index + 1..].to_owned())
        } else {
            ActualResult::Return(
                <Vec<u8>>::from_hex(bogoglulx_output_str)
                    .expect("non-error returns from bogoglulx should be valid hex"),
            )
        };

        let interpreted = InterpretedResult::interpret(&self.expected_result, &actual);

        if interpreted != self.expected_result {
            std::fs::write(&actual_path, format!("{:?}", interpreted)).unwrap();
            std::fs::write(&expected_path, format!("{:?}", self.expected_result)).unwrap();
            let mut options = CompilationOptions::new();
            options.set_text(true);
            let asm_out = compile_module_to_bytes(&options, &module)
                .expect("If binary compilation succeeded, text compilation should too");
            std::fs::write(&glulxasm_path, &asm_out).unwrap();
            panic!(
                "Test result differed from expected.\nActual: {:?}\nExpected: {:?}",
                interpreted, self.expected_result
            );
        } else {
            let _ = std::fs::remove_file(&story_path);
            let _ = std::fs::remove_file(&wasm_path);
        }
    }
}
