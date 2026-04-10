//! <h1 style="color:#B9F2FF;"><b>Diamond</b></h1>
//!
//! <h2><i>Perl but it doesn't suck ass</i></h2>

/// Parsing Diamond code into an untyped AST.
pub mod parse;
/// Type checker.
pub mod typing;

/// Standard library.
pub mod stdlib;

use std::path::PathBuf;

use clap::Parser;
use miette::{IntoDiagnostic, Result};

use parse::grammar::parse_di;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    input: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let string = std::fs::read_to_string(&args.input).into_diagnostic()?;

    let file = args.input.to_string_lossy();

    let program = parse_di(&string, &file).map_err(|_| std::process::exit(1));

    dbg!(program).into_diagnostic()?;

    Ok(())
}
