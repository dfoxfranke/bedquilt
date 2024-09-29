// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
// Copyright 2024 Daniel Fox Franke.

use std::{
    ffi::OsString,
    io::IsTerminal,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::{CommandFactory, Parser, ValueHint};
use wasm2glulx::{
    compile, CompilationOptions, DEFAULT_GLK_AREA_SIZE, DEFAULT_STACK_SIZE,
    DEFAULT_TABLE_GROWTH_LIMIT,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, max_term_width = 72)]
struct Args {
    /// Name of output file, or "-" for stdout
    ///
    /// The default is stdout if the input comes from stdin. Otherwise, the
    /// default is to strip any .wasm suffix from the input file name, add a
    /// .ulx suffix, and output it to the current directory.
    #[arg(short, long, value_name="FILE", value_hint = ValueHint::FilePath)]
    output: Option<PathBuf>,

    /// Size (in bytes) of the GLK area
    #[arg(long, default_value_t = DEFAULT_GLK_AREA_SIZE, value_name="SIZE")]
    glk_area_size: u32,
    /// Size (in bytes) of the program stack
    #[arg(long, default_value_t = DEFAULT_STACK_SIZE, value_name="SIZE")]
    stack_size: u32,
    /// Output human-readable assembly rather than a story file
    #[arg(long, default_value_t = false)]
    text: bool,

    /// Growth limit (in entries) for tables
    ///
    /// If the input module specifies a lower limit, the lower one will be used.
    /// Most programs don't use growable tables and will specify a maximum size
    /// the same as the initial one, so this option is usually ignored.
    #[arg(long, default_value_t = DEFAULT_TABLE_GROWTH_LIMIT, value_name="N")]
    table_growth_limit: u32,

    /// Path to WASM module, or "-" (default) for stdin
    #[arg(index = 1, value_name = "INPUT-FILE")]
    input: Option<PathBuf>,
}

fn main() -> ExitCode {
    let args = Args::parse();
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let stderr = std::io::stderr();

    if args.input.is_none() && stdin.is_terminal() {
        eprintln!("\u{1b}[1m\u{1b}[31mwasm2glulx: reading input file from stdin, but stdin is a tty. Add \"-\" to the command line if you want to force this.\u{1b}[39m\u{1b}[22m");
        if stdout.is_terminal() {
            let _ = Args::command().print_help();
        }
        return ExitCode::FAILURE;
    }

    if !args.text
        && (args.input.is_none() || args.input.as_deref() == Some(Path::new("-")))
        && args.output.is_none()
        && stdout.is_terminal()
    {
        eprintln!("\u{1b}[1m\u{1b}[31mwasm2glulx: writing output to stdout, but stdout is a tty. Add \"-o -\" to the command line if you want to force this.\u{1b}[39m\u{1b}[22m");
        return ExitCode::FAILURE;
    }

    let input = if args.input.as_deref() == Some(Path::new("-")) {
        None
    } else {
        args.input
    };
    let output = if args.output.as_deref() == Some(Path::new("-")) {
        None
    } else if args.output.is_none() && input.is_some() {
        let input = input.as_deref().unwrap();
        let mut basename = input
            .file_name()
            // A path with no basename is directory and this is going to fail
            // later, but just continue with something for the time being.
            .unwrap_or("".as_ref())
            .as_encoded_bytes()
            .to_owned();
        if basename.ends_with(b".wasm") {
            basename.truncate(basename.len() - 5);
        }
        if args.text {
            basename.extend_from_slice(b".glulxasm");
        } else {
            basename.extend_from_slice(b".ulx");
        }
        Some(PathBuf::from(unsafe {
            // SAFETY: On all platforms, OsStrings are a self-synchronizing
            // superset of UTF-8, so the above manipulations preserve validity.
            OsString::from_encoded_bytes_unchecked(basename)
        }))
    } else {
        args.output
    };

    let mut options = CompilationOptions::new();
    options.set_glk_area_size(args.glk_area_size);
    options.set_stack_size(args.stack_size);
    options.set_table_growth_limit(args.table_growth_limit);
    options.set_text(args.text);
    options.set_input(input);
    options.set_output(output);

    match compile(&options) {
        Ok(_) => ExitCode::SUCCESS,
        Err(errv) => {
            if stderr.is_terminal() {
                eprintln!(
                    "\u{1b}[1m\u{1b}[31mwasm2glulx: {} error{} encountered\u{1b}[39m\u{1b}[22m",
                    errv.len(),
                    if errv.len() > 1 { "s" } else { "" }
                );
                for err in errv {
                    eprintln!("\u{1b}[1m*\u{1b}[22m {err}");
                }
            } else {
                eprintln!(
                    "wasm2glulx: {} error{} encountered",
                    errv.len(),
                    if errv.len() > 1 { "s" } else { "" }
                );
                for err in errv {
                    eprintln!("* {err}");
                }
            }
            ExitCode::FAILURE
        }
    }
}
