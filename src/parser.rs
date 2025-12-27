use crate::ast::{Expr, ExprKind, Stmt, StmtKind};
use crate::error::ParserError;
use crate::tokens::{TokenStream, TokenKind};
use crate::compiler::Context;

pub struct Parser<'ctx> {
    _context: &'ctx Context,
    token_stream: TokenStream,
    errors: Vec<ParserError>
}

impl<'ctx> Parser<'ctx> {
    pub fn new(token_stream: TokenStream,context: &'ctx Context ) -> Parser<'ctx> {
        Parser {
            _context: context,
            token_stream,
            errors: vec![]
        }
    }

    pub fn parse(&mut self) -> Result<ASTNode, Vec<ParserError>> {
        todo!();
        let ast = self.parse_block();
        if self.errors.len() > 0 {
            return Err(self.errors);
        };  
    }

    fn parse_block(&mut self) {

    }
}

