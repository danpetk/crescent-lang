use std::process::exit;
use std::fs;
use lang::Compiler;

fn main() {
    let filename =  std::env::args().nth(1).unwrap_or_else(||{
        eprintln!("Invalid Arguments!");
        eprintln!("Expected Usage: lang {{filename}}");
        exit(1)        
    });

    let source = fs::read_to_string(&filename).unwrap_or_else(|_| {
        eprintln!("ERROR: Failed to read file '{}'", &filename);
        exit(1);
    });

    let mut compiler = Compiler::new();

    if let Err(errors) = compiler.compile(&source) {
        for e in errors {
            eprintln!("ERROR: {e}");
        }
        exit(1);
    }

}
