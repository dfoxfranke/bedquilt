// SPDX-License-Identifier: Apache-2.0 WITH LLVM-Exception
// Copyright 2024 Daniel Fox Franke.
#![allow(unused_imports)]
use std::{ffi::OsString, path::PathBuf};

#[allow(dead_code)]
static BOGOGLULX_SOURCES: &[&str] = &[
    "exec.c",
    "files.c",
    "float.c",
    "funcs.c",
    "gestalt.c",
    "heap.c",
    "main.c",
    "operand.c",
    "osdepend.c",
    "search.c",
    "vm.c",
];

fn main() {
    #[cfg(feature = "spectest")]
    {
        let platform_bogoglulx_sources: Vec<PathBuf> = BOGOGLULX_SOURCES
            .iter()
            .map(|file| {
                let mut buf = PathBuf::from("bogoglulx");
                buf.push(file);
                buf
            })
            .collect();

        for src in &platform_bogoglulx_sources {
            println!(
                "cargo:rerun-if-changed={}",
                src.to_str()
                    .expect("path to bogoglulx source file should be valid UTF-8")
            );
        }

        let tool = cc::Build::new().get_compiler();
        let mut command = tool.to_command();

        let mut bogoglulx_bin =
            PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR should be set during builds"));
        bogoglulx_bin.push("bogoglulx");
        if cfg!(windows) {
            bogoglulx_bin.set_extension("exe");
        }

        if tool.is_like_clang() || tool.is_like_gnu() {
            command.arg("-o");
            command.arg(bogoglulx_bin.as_os_str());
            command.arg("-lm");
            command.args(&platform_bogoglulx_sources);
        } else if tool.is_like_msvc() {
            command.args(&platform_bogoglulx_sources);
            command.arg("/link");
            let mut out_arg = OsString::from("/OUT:");
            out_arg.push(bogoglulx_bin.as_os_str());
            command.arg(out_arg);
        } else {
            panic!("Unsupported C compiler");
        }

        let compilation_result = command.spawn().unwrap().wait().unwrap();
        if !compilation_result.success() {
            panic!("Bogoglulx compilation failed");
        }
        println!(
            "cargo:rustc-env=BOGOGLULX_BIN={}",
            std::fs::canonicalize(bogoglulx_bin)
                .unwrap()
                .to_str()
                .expect("path to bogoglulx binary should be valid UTF-8")
        );
        println!(
            "cargo:rustc-env=WASM2GLULX_MANIFEST_DIR={}",
            std::env::var("CARGO_MANIFEST_DIR").unwrap()
        );
    }
}
