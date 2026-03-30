use crate::codegen::Codegen;
use crate::diagnostic::{Diagnostic, Diagnostics};
use crate::semantic::SemanticAnalyzer;
use crate::{lexer::Lexer, parser::Parser, source::Source, symbols::Symbols};
use std::cell::RefCell;

pub struct Context {
    pub source: Source,
    pub out_path: String,
    pub symbols: RefCell<Symbols>,
    pub diags: RefCell<Diagnostics>,
}

impl Context {
    pub fn new(source: String, out_path: String) -> Context {
        Context {
            source: Source::new(source),
            out_path,
            symbols: RefCell::new(Symbols::new()),
            diags: RefCell::new(Diagnostics::default()),
        }
    }
}

pub struct Compiler {
    ctx: Context,
}

impl Compiler {
    pub fn new(source: String, out_path: String) -> Compiler {
        Compiler {
            ctx: Context::new(source, out_path),
        }
    }

    // TODO: perhaps this can get less repetative later
    pub fn compile(&mut self) -> Result<(), Vec<Diagnostic>> {
        let mut lexer = Lexer::new(&self.ctx);
        let token_stream = lexer.tokenize();

        if self.ctx.diags.borrow().has_diagnostics() {
            return Err(self.ctx.diags.borrow_mut().take_diagnostics());
        }

        let mut parser = Parser::new(token_stream, &self.ctx);
        let mut ast = parser.parse();

        if self.ctx.diags.borrow().has_diagnostics() {
            return Err(self.ctx.diags.borrow_mut().take_diagnostics());
        }

        let mut semantics = SemanticAnalyzer::new(&self.ctx);
        semantics.analyze(&mut ast);

        if self.ctx.diags.borrow().has_diagnostics() {
            return Err(self.ctx.diags.borrow_mut().take_diagnostics());
        }

        println!("{ast:#?}");

        let mut _codegen = Codegen::try_new(&self.ctx).map_err(|e| vec![e])?;

        if self.ctx.diags.borrow().has_diagnostics() {
            return Err(self.ctx.diags.borrow_mut().take_diagnostics());
        }

        Ok(())
    }
}
