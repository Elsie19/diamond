//! <h1 style="color:#B9F2FF;"><b>💎 Diamond 💎</b></h1>
//!
//! <q>A lightweight DSL focused on file manipulation with I/O.</q>
//!
//! ## Learning Diamond
//!
//! 1. [The Basics](`parse`)
//! 2. [Practice](`interpreter::engine`)
//!
//! ## Standard Library
//!
//! You should check out the standard library [here](interpreter::functions).

use std::{
    io::Write,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use interpreter::engine::Engine;
use miette::{IntoDiagnostic, Result};
use parse::grammar::parse_di;
use shared::{
    Bundle,
    bin_header::{FileType, binary_ir, detect_ir, get_ir},
};
use type_checker::{
    core::AstWalker,
    pass_two::TypeChecker,
    strata::{IR, vargen_strategies::interpreter::VarGenInterpreter},
};

#[doc(hidden)]
const STDLIB_IR: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/stdlib.ir"));

#[doc(hidden)]
fn load_stdlib() -> Bundle {
    let (bundle, _): (Bundle, usize) =
        bincode::decode_from_slice(STDLIB_IR, bincode::config::standard())
            .expect("failed to load bundle");

    bundle
}

#[doc(hidden)]
fn encode(ir: &[IR]) -> Vec<u8> {
    bincode::encode_to_vec(ir, bincode::config::standard()).unwrap()
}

/// A text-parsing DSL.
#[doc(hidden)]
#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[command(subcommand)]
    commands: Commands,
}

#[doc(hidden)]
#[derive(Subcommand)]
enum Commands {
    /// Run program.
    Run {
        /// Diamond program file.
        input: PathBuf,

        /// Arguments to pass into Diamond.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    Compile {
        /// Diamond program file.
        input: PathBuf,

        /// Output precompiled program.
        #[arg(short, long)]
        output: PathBuf,
    },
}

#[doc(hidden)]
fn compile_source(string: &str, file: &str) -> miette::Result<Vec<IR>> {
    let mut bundle = load_stdlib();

    let program = parse_di(string, file).map_err(|()| miette::miette!("parse failed"))?;

    let walker = AstWalker::new(&program);
    let mut funcs = walker.collect_function_defs();
    funcs.extend(bundle.funcs);

    let mut checker = TypeChecker::<VarGenInterpreter>::new(&funcs, file, &string);

    checker.check_program(&program)?;

    bundle.ir.extend_from_slice(checker.ir());

    Ok(bundle.ir)
}

#[doc(hidden)]
fn decode_bin_ir(bytes: &[u8]) -> miette::Result<Vec<IR>> {
    let bin = get_ir(bytes);

    let (ir, _): (Vec<IR>, usize) =
        bincode::decode_from_slice(bin, bincode::config::standard()).into_diagnostic()?;

    Ok(ir)
}

#[doc(hidden)]
fn main() -> Result<()> {
    let args = Args::parse();

    match args.commands {
        Commands::Run { input, args } => {
            let bytes = std::fs::read(&input).into_diagnostic()?;

            let ir = match detect_ir(&bytes) {
                FileType::Text => {
                    // SAFETY: We hope that nobody put broken UTF-8 in the file. If they did, not
                    // our fault.
                    let string = unsafe { String::from_utf8_unchecked(bytes) };
                    let file = input.to_string_lossy();
                    compile_source(&string, &file)?
                }
                FileType::Binary => decode_bin_ir(&bytes)?,
            };

            Engine::new(&args).run(ir);
        }
        Commands::Compile { input, output } => {
            let string = std::fs::read_to_string(&input).into_diagnostic()?;
            let file = input.to_string_lossy();

            let ir = compile_source(&string, &file)?;

            let bin = encode(&ir);
            let final_blob = binary_ir(&bin);

            if output == Path::new("-") {
                std::io::stdout().write_all(&final_blob).into_diagnostic()?;
                let _ = std::io::stdout().flush();
            } else {
                std::fs::write(output, final_blob).into_diagnostic()?;
            }
        }
    }

    Ok(())
}
