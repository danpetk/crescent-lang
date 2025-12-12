use std::error::Error;
use std::process::exit;
use std::fs;
use lang::Lexer;

fn main() {
    let filename =  std::env::args().nth(1).unwrap_or_else(||{
        eprintln!("Invalid Arguments!");
        eprintln!("Expected Usage: lang {{filename}}");
        exit(1)        
    });

    let source = fs::read_to_string(&filename).unwrap_or_else(|_| {
        eprintln!("ERROR: Failed to read file '{}'.", &filename);
        exit(1);
    });


    if let Err(e) = compile(&source) {
        eprintln!("ERROR: {e}");
        exit(1);
    }
}

fn compile(source: &str) -> Result<(), Box<dyn Error>> {
    let _lexer = Lexer::new(&source);
    
    Ok(())
}
