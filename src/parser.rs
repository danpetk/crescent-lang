use crate::ast::{ASTNode, ASTKind};
use crate::error::ParserError;
use crate::tokens::{TokenStream, TokenKind};
use crate::compiler::Context;

pub struct Parser<'ctx, 'a> {
    _context: &'ctx Context,
    token_stream: TokenStream<'a>,
    errors: Vec<ParserError>
}

impl<'ctx, 'a> Parser<'ctx, 'a> {
    pub fn new(token_stream: TokenStream<'a>,context: &'ctx Context ) -> Parser<'ctx, 'a> {
        Parser {
            _context: context,
            token_stream,
            errors: vec![]
        }
    }

    pub fn parse(&mut self) -> Result<ASTNode<'a>, Vec<ParserError>> {
        let ast = self.parse_block();
        if self.errors.len() > 0 {
            return Err(self.errors);
        };
    }

    fn parse_block(&mut self) {

    }
}

