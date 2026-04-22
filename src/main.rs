//! <h1 style="color:#B9F2FF;"><b>Diamond</b></h1>
//!
//! <h2><i>Perl but it doesn't suck ass</i></h2>
//!
//! You should check out the standard library [here](interpreter::functions).

/// Parsing Diamond code into an untyped AST.
#[doc(hidden)]
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

#[doc(hidden)]
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    input: PathBuf,
}

#[doc(hidden)]
fn main() -> Result<()> {
    let args = Args::parse();

    let string = std::fs::read_to_string(&args.input).into_diagnostic()?;

    let file = args.input.to_string_lossy();

    let stdlib_program =
        parse_di(STDLIB_HEADERS, STDLIB_PATH).expect("failed parsing headers, fuck.");
    let stdlib_walker = AstWalker::new(&stdlib_program);
    let func_table = stdlib_walker.collect_function_defs();

    let program = parse_di(&string, &file).map_err(|()| miette::miette!("parse failed"))?;

    let walker = AstWalker::new(&program);

    let mut funcs = walker.collect_function_defs();
    funcs.extend(func_table);

    let mut checker = TypeChecker::<VarGenInterpreter>::new(&funcs, &file, &string);
    let _ = checker.check_program(&program)?;

    let mut engine = Engine::new(checker.ir(), &funcs);

    engine.run();

    Ok(())
}
