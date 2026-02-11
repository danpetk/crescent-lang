use crate::diagnostic::{Diagnostic, Diagnostics};
use crate::{lexer::Lexer, parser::Parser, source::Source, symbols::Symbols};
use std::cell::RefCell;

pub struct Context {
    pub source: Source,
    pub symbols: RefCell<Symbols>,
    pub diags: RefCell<Diagnostics>,
}

impl Context {
    pub fn new(source: String) -> Context {
        Context {
            source: Source::new(source),
            symbols: RefCell::new(Symbols::default()),
            diags: RefCell::new(Diagnostics::default()),
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

    // TODO perhaps this can get less repetative later
    pub fn compile(&mut self) -> Result<(), Vec<Diagnostic>> {
        let mut lexer = Lexer::new(&self.ctx);

        let token_stream = lexer.tokenize();
        if self.ctx.diags.borrow().has_diagnostics() {
            return Err(self.ctx.diags.borrow_mut().take_diagnostics());
        }

        let mut parser = Parser::new(token_stream, &self.ctx);
        let ast = parser.parse();
        if self.ctx.diags.borrow().has_diagnostics() {
            return Err(self.ctx.diags.borrow_mut().take_diagnostics());
        }

        println!("{ast:?}");

        Ok(())
    }
}
