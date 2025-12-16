use std::error::Error;
use std::process::exit;
use std::fs;
use lang::{Lexer, TokenKind, };

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
    let mut lexer = Lexer::new(&source);
    
    let x = lexer.tokenize();

    match x {
        Ok(mut x) => {
            loop {
                let y = x.advance();
                println!("{y:?}");
                if y.kind == TokenKind::EOF {
                    break;
                }
            }
        },
        Err(x) => {
            for error in x {
                println!("{error}")
            }
        }
    }
        
    
    Ok(())
}
