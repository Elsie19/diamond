//! <h1 style="color:#B9F2FF;"><b>Diamond</b></h1>
//!
//! <h2><i>Perl but it doesn't suck ass</i></h2>

/// Parsing Diamond code into an untyped AST.
pub mod parse;
/// Type checker.
pub mod typing;

/*
* Steps are:
*
* 1. Parsing
* 1.1 Aliasing
* 2. Type Checking
*/

fn main() {
    println!("Hello, world!");
}
