//! <h1 style="color:#B9F2FF;"><b>Diamond</b></h1>
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

#[doc = include_str!("../docs/language.md")]
pub mod parse;
/// Type checker.
pub mod typing;

/// Interpreter, duh.
pub mod interpreter;

use std::path::PathBuf;

use clap::Parser;
use miette::{IntoDiagnostic, Result};

use parse::grammar::parse_di;

use crate::{
    interpreter::engine::Engine,
    typing::{
        core::AstWalker, pass_two::TypeChecker,
        strata::vargen_strategies::interpreter::VarGenInterpreter,
    },
};

#[doc(hidden)]
const STDLIB_PATH: &str = "stdlib/headers.di";
#[doc(hidden)]
const STDLIB_HEADERS: &str = include_str!("stdlib/headers.di");

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

    // PARSE STDLIB FIRST //

    let stdlib_program =
        parse_di(STDLIB_HEADERS, STDLIB_PATH).expect("failed parsing headers, fuck.");
    let stdlib_walker = AstWalker::new(&stdlib_program);
    let func_table = stdlib_walker.collect_function_defs();

    let mut stdlib_checker = TypeChecker::<VarGenInterpreter>::new(&func_table, &file, &string);
    let _ = stdlib_checker.check_program(&stdlib_program)?;

    let mut total_ir = stdlib_checker.ir().to_vec();

    // THEN PROGRAM //

    let program = parse_di(&string, &file).map_err(|()| miette::miette!("parse failed"))?;

    let walker = AstWalker::new(&program);

    let mut funcs = walker.collect_function_defs();
    funcs.extend(func_table);

    let mut checker = TypeChecker::<VarGenInterpreter>::new(&funcs, &file, &string);
    let _ = checker.check_program(&program)?;

    let program_ir = checker.ir();

    total_ir.extend(program_ir.to_vec());

    let mut engine = Engine::new(&total_ir, &funcs, &args.args);

    engine.run();

    Ok(())
}
