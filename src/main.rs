use crescent_lang::Compiler;
use std::fs;
use std::process::exit;

fn main() {
    let filename = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Invalid Arguments!");
        eprintln!("Expected Usage: lang {{filename}} {{out_file [defaults to out.crsnt]}}");
        exit(1)
    });

    let out_path = std::env::args().nth(1).unwrap_or("out.crsnt".to_string());

    let source = fs::read_to_string(&filename).unwrap_or_else(|_| {
        eprintln!("ERROR: Failed to read file '{}'", &filename);
        exit(1);
    });

    let mut compiler = Compiler::new(source, out_path);

    if let Err(errors) = compiler.compile() {
        for e in errors {
            eprintln!("{e}");
        }
        exit(1);
    }
}
