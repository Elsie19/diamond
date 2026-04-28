//! <h1 style="color:#B9F2FF;"><b>💎 Diamond 💎</b></h1>
//!
//! <q>A lightweight DSL focused on file manipulation with I/O.</q>
//!
//! ## Learning Diamond
//!
//! Check out the [parsing module](`parse`) for more information.
//!
//! ## Standard Library
//!
//! You should check out the standard library [here](interpreter::functions).

use std::path::PathBuf;

use clap::Parser;
use interpreter::engine::Engine;
use miette::{IntoDiagnostic, Result};
use parse::grammar::parse_di;
use shared::Bundle;
use type_checker::{
    core::AstWalker, pass_two::TypeChecker,
    strata::vargen_strategies::interpreter::VarGenInterpreter,
};

const STDLIB_IR: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/stdlib.ir"));

fn load_stdlib() -> Bundle {
    let (bundle, _): (Bundle, usize) =
        bincode::decode_from_slice(STDLIB_IR, bincode::config::standard())
            .expect("failed to load bundle");

    bundle
}

/// A text-parsing DSL.
#[doc(hidden)]
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Diamond program file.
    input: PathBuf,

    /// Arguments to pass into Diamond.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

/*
* How this works is we parse the standard library and turn that into IR, then we parse the users
* code, turn that into IR, then join the two, with the stdlib coming first, then executing it all.
* Could this be made so it doesn't have to parse the standard library every time? Sure. Do I have
* the time to do that? Not really.
*/
#[doc(hidden)]
fn main() -> Result<()> {
    let args = Args::parse();

    let string = std::fs::read_to_string(&args.input).into_diagnostic()?;

    let file = args.input.to_string_lossy();

    let mut bundle = load_stdlib();

    let program = parse_di(&string, &file).map_err(|()| miette::miette!("parse failed"))?;

    let walker = AstWalker::new(&program);
    let mut funcs = walker.collect_function_defs();
    funcs.extend(bundle.funcs);

    let mut checker = TypeChecker::<VarGenInterpreter>::new(&funcs, &file, &string);

    checker.check_program(&program)?;

    bundle.ir.extend_from_slice(checker.ir());

    Engine::new(&bundle.ir, &args.args).run();

    Ok(())
}
