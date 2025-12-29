use std::os::linux::raw::stat;

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

    pub fn parse(&mut self) -> Result<Stmt, Vec<ParserError>> {
        let mut statements = vec![];
        while self.token_stream.any() {
            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(err) => return Err(vec![err])
            }
        }
        Ok(Stmt{})
    }

    fn parse_statement(&mut self) -> Result<Stmt, ParserError> {
        todo!()
    }

    fn parse_block(&mut self) {

    }
}

