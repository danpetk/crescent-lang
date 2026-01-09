use std::error::Error;
use crate::{lexer::Lexer, parser::Parser, source::Source, symbols::Symbols};
use std::cell::RefCell;

pub struct Context {
    pub source: Source,
    pub symbols: RefCell<Symbols>
}

impl Context {
    pub fn new(source: String) -> Context {
        Context {
            source: Source::new(source),
            symbols: RefCell::new(Symbols::new())
        }
    }
}

pub struct Compiler {
    context: Context
}

impl Compiler {
    pub fn new(source: String) -> Compiler {
        Compiler {
            context: Context::new(source)
        }
    }

    pub fn compile(&mut self) -> Result<(), Vec<Box<dyn Error>>> {
        
        let mut lexer = Lexer::new(&self.context);
        let token_stream = match lexer.tokenize() {
            Ok(stream) => stream,
            Err(errors) => {
                return Err(errors.into_iter().map(|e| Box::<dyn Error>::from(e)).collect());
            }
        };

        let mut parser = Parser::new(token_stream, &self.context);
        let ast = match parser.parse() {
            Ok(ast) => ast,
            Err(errors) => {
                return Err(errors.into_iter().map(|e| Box::<dyn Error>::from(e)).collect());
            }
        };

        println!("{ast:?}");

        Ok(())    
    }
}