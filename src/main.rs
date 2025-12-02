use std::process;
use crescent::{Lexer};

fn main() {
    let filename =  std::env::args().nth(1).unwrap_or_else(||{
        eprintln!("Invalid Arguments!");
        eprintln!("Expected Usage: crescent-lang {{filename}}");
        process::exit(1)        
    });

    return compile(&filename);
}

fn compile(filename: &str) {
    let _lexer = Lexer::from_file(filename);
}
