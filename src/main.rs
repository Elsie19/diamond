//! <h1 style="color:#B9F2FF;"><b>Diamond</b></h1>
//!
//! <h2><i>Perl but it doesn't suck ass</i></h2>

/// Parsing Diamond code into an untyped AST.
pub mod parse;
/// Type checker.
pub mod typing;

use std::path::PathBuf;

use clap::Parser;
use miette::{IntoDiagnostic, Result};

use parse::grammar::parse_di;

use crate::typing::{core::AstWalker, pass_two::TypeChecker};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    input: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let string = std::fs::read_to_string(&args.input).into_diagnostic()?;

    let file = args.input.to_string_lossy();

    let program = parse_di(&string, &file).map_err(|_| miette::miette!("parse failed"))?;

    let walker = AstWalker::new(&program);

    let funcs = walker.collect_function_defs();

    let mut checker = TypeChecker::new(&funcs, &file, &string);
    let typed_ir = checker.check_program(&program)?;

    Ok(())
}
