use crate::diagnostic::{Diagnostics, Diagnostic};
use crate::{lexer::Lexer, parser::Parser, source::Source, symbols::Symbols};
use std::cell::RefCell;
use std::error::Error;

pub struct Context {
    pub source: Source,
    pub symbols: RefCell<Symbols>,
    pub diags: RefCell<Diagnostics>
}

impl Context {
    pub fn new(source: String) -> Context {
        Context {
            source: Source::new(source),
            symbols: RefCell::new(Symbols::default()),
            diags: RefCell::new(Diagnostics::default())
        }
    }
}

pub struct Compiler {
    ctx: Context,
}

impl Compiler {
    pub fn new(source: String) -> Compiler {
        Compiler {
            ctx: Context::new(source),
        }
    }

    pub fn compile(&mut self) -> Result<(), Vec<Diagnostic>> {
        let mut lexer = Lexer::new(&self.ctx);
        
        let token_stream = lexer.tokenize();   
        if self.ctx.diags.borrow().has_diagnostics() {
            return Err(self.ctx.diags.borrow_mut().take_diagnostics())
        }


        let mut parser = Parser::new(token_stream, &self.ctx);
        let ast = match parser.parse() {
            Ok(ast) => ast,
            Err(errors) => {
                return Err(errors
                    .into_iter()
                    .map(|e| Box::<dyn Error>::from(e))
                    .collect());
            }
        };

        println!("{ast:?}");

        Ok(())
    }
}
