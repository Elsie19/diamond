use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=stdlib/");
    println!("cargo:rerun-if-changed=STDLIB_HEADERS");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let output_path = out_dir.join("stdlib.ir");

    let stdlib_program = parse_di(STDLIB_HEADERS, STDLIB_PATH).expect("stdlib parse failed");

    let walker = AstWalker::new(&stdlib_program);
    let func_table = walker.collect_function_defs();

    let mut checker = TypeChecker::<VarGenInterpreter>::new(&func_table, "stdlib", "");

    checker
        .check_program(&stdlib_program)
        .expect("stdlib typecheck failed");

    let ir = checker.ir();

    let bytes = serialize_ir(ir);

    fs::write(&output_path, bytes).expect("failed writing stdlib.ir");
}
