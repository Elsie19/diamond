use std::{env, fs, path::PathBuf};

use parse::grammar::parse_di;
use shared::Bundle;
use type_checker::{
    core::AstWalker, pass_two::TypeChecker,
    strata::vargen_strategies::interpreter::VarGenInterpreter,
};

const STDLIB_PATH: &str = "stdlib/headers.di";
const STDLIB_HEADERS: &str = "./interpreter/src/stdlib/headers.di";

fn serialize_ir(ir: Bundle) -> Vec<u8> {
    bincode::encode_to_vec(ir, bincode::config::standard()).unwrap()
}

fn main() {
    println!("cargo:rerun-if-changed=interpreter/src/stdlib/");
    println!("cargo:rerun-if-changed=STDLIB_HEADERS");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let output_path = out_dir.join("stdlib.ir");

    let ir_text = fs::read_to_string(STDLIB_HEADERS).unwrap();

    let stdlib_program = parse_di(&ir_text, STDLIB_PATH).expect("stdlib parse failed");

    let walker = AstWalker::new(&stdlib_program);
    let func_table = walker.collect_function_defs();

    let mut checker = TypeChecker::<VarGenInterpreter>::new(&func_table, "stdlib", &String::new());

    checker
        .check_program(&stdlib_program)
        .expect("stdlib typecheck failed");

    let ir = checker.ir();

    let bytes = serialize_ir(Bundle {
        ir: ir.to_vec(),
        funcs: func_table,
    });

    fs::write(&output_path, bytes).expect("failed writing stdlib.ir");
}
