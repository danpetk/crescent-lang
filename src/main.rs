use std::error::Error;
use std::process::{self, exit};
use lang::Lexer;

fn main() {
    let filename =  std::env::args().nth(1).unwrap_or_else(||{
        eprintln!("Invalid Arguments!");
        eprintln!("Expected Usage: lang {{filename}}");
        process::exit(1)        
    });

    if let Err(e) = compile(&filename) {
        eprintln!("ERROR: {e}");
        exit(1);
    }
}

fn compile(filename: &str) -> Result<(), Box<dyn Error>> {
    let _lexer = Lexer::from_file(filename)?;
    
    Ok(())
}
