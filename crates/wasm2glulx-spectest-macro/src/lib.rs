extern crate proc_macro;
use std::path::PathBuf;

use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};
use syn::{parse_macro_input, LitStr};
use wasm2glulx::spectest::{ExpectedResult, F32, F64};

#[proc_macro]
pub fn spectest(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as LitStr).value();
    let mut input_path = PathBuf::from(std::env::var_os("WASM2GLULX_MANIFEST_DIR").unwrap());
    input_path.push(input);
    let wast = std::fs::read_to_string(&input_path).unwrap();
    let tests = wasm2glulx::spectest::wast_to_tests(wast.as_str()).unwrap();
    let mut out = Vec::new();
    let stem = input_path
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .replace('-', "_");

    for test in tests {
        let module = Literal::byte_string(&test.module);
        let expected = expected_result_to_syn(test.expected_result);
        let l = Literal::usize_suffixed(test.line_col.0);
        let c = Literal::usize_suffixed(test.line_col.1);

        let ident = format_ident!("{}_L{}_C{}", stem, test.line_col.0, test.line_col.1);
        let stem_literal =
            Literal::string(format!("{}_L{}_C{}", stem, test.line_col.0, test.line_col.1).as_str());

        out.push(quote! {
            #[test]
            fn #ident() {
                let test = ::wasm2glulx::spectest::WastTest {
                    line_col: (#l,#c),
                    module: #module.as_slice().to_owned(),
                    expected_result: #expected
                };

                test.run(env!("CARGO_TARGET_TMPDIR").as_ref(), #stem_literal);
            }
        });
    }
    let result = quote! { #(#out)* };
    result.into()
}

fn f32_to_syn(x: F32) -> TokenStream {
    match x {
        F32::CanonicalNan => quote! { ::wasm2glulx::spectest::F32::CanonicalNan },
        F32::ArithmeticNan => quote! { ::wasm2glulx::spectest::F32::ArithmeticNan },
        F32::Value(x) => {
            let lit = Literal::u32_suffixed(x);
            quote! { ::wasm2glulx::spectest::F32::Value(#lit) }
        }
    }
}

fn f64_to_syn(x: F64) -> TokenStream {
    match x {
        F64::CanonicalNan => quote! { ::wasm2glulx::spectest::F64::CanonicalNan },
        F64::ArithmeticNan => quote! { ::wasm2glulx::spectest::F64::ArithmeticNan },
        F64::Value(x) => {
            let lit = Literal::u64_suffixed(x);
            quote! { ::wasm2glulx::spectest::F64::Value(#lit) }
        }
    }
}

fn expected_result_to_syn(expected: ExpectedResult) -> TokenStream {
    match expected {
        ExpectedResult::Return(ret) => {
            let ret_literals: Vec<TokenStream> = ret
                .into_iter()
                .map(|v| match v {
                    wasm2glulx::spectest::ExpectedValue::I32(x) => {
                        let lit = Literal::i32_suffixed(x);
                        quote! { wasm2glulx::spectest::ExpectedValue::I32(#lit) }
                    }
                    wasm2glulx::spectest::ExpectedValue::I64(x) => {
                        let lit = Literal::i64_suffixed(x);
                        quote! { wasm2glulx::spectest::ExpectedValue::I64(#lit) }
                    }
                    wasm2glulx::spectest::ExpectedValue::F32(x) => {
                        let lit = f32_to_syn(x);
                        quote! { wasm2glulx::spectest::ExpectedValue::F32(#lit) }
                    }
                    wasm2glulx::spectest::ExpectedValue::F64(x) => {
                        let lit = f64_to_syn(x);
                        quote! { wasm2glulx::spectest::ExpectedValue::F64(#lit) }
                    }
                    wasm2glulx::spectest::ExpectedValue::I8x16(v) => {
                        let lits = v.map(Literal::i8_suffixed);
                        quote! { wasm2glulx::spectest::ExpectedValue::I8x16([#(#lits),*])}
                    }
                    wasm2glulx::spectest::ExpectedValue::I16x8(v) => {
                        let lits = v.map(Literal::i16_suffixed);
                        quote! { wasm2glulx::spectest::ExpectedValue::I16x8([#(#lits),*])}
                    }
                    wasm2glulx::spectest::ExpectedValue::I32x4(v) => {
                        let lits = v.map(Literal::i32_suffixed);
                        quote! { wasm2glulx::spectest::ExpectedValue::I32x4([#(#lits),*])}
                    }
                    wasm2glulx::spectest::ExpectedValue::I64x2(v) => {
                        let lits = v.map(Literal::i64_suffixed);
                        quote! { wasm2glulx::spectest::ExpectedValue::I64x2([#(#lits),*])}
                    }
                    wasm2glulx::spectest::ExpectedValue::F32x4(v) => {
                        let lits = v.map(f32_to_syn);
                        quote! { wasm2glulx::spectest::ExpectedValue::F32x4([#(#lits),*])}
                    }
                    wasm2glulx::spectest::ExpectedValue::F64x2(v) => {
                        let lits = v.map(f64_to_syn);
                        quote! { wasm2glulx::spectest::ExpectedValue::F64x2([#(#lits),*])}
                    }
                })
                .collect();

            quote! {::wasm2glulx::spectest::ExpectedResult::Return(vec![#(#ret_literals),*]) }
        }
        ExpectedResult::Trap(trap) => {
            let trap_literal = proc_macro2::Literal::string(trap.as_str());
            quote! {::wasm2glulx::spectest::ExpectedResult::Trap(#trap_literal.to_string())}
        }
    }
}
