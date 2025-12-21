use std::error::Error;
use crate::{lexer::Lexer, parser::Parser, symbols::Symbols};
use std::cell::RefCell;

pub struct Context {
    pub symbols: RefCell<Symbols>
}

impl Context {
    pub fn new() -> Context {
        Context {
            symbols:RefCell::new(Symbols::new())
        }
    }
}

pub struct Compiler {
    context: Context
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            context: Context::new()
        }
    }

    pub fn compile(&mut self, source: &str) -> Result<(), Vec<Box<dyn Error>>> {
        
        let mut lexer = Lexer::new(&source);
        let token_stream = match lexer.tokenize() {
            Ok(stream) => stream,
            Err(errors) => {
                return Err(errors.into_iter().map(|e| Box::<dyn Error>::from(e)).collect());
            }
        };


        let mut parser = Parser::new(token_stream, &self.context);
        let _ast = parser.parse();


        Ok(())    
    }
}