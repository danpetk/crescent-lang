use crate::ast::{ASTNode, ASTKind};
use crate::tokens::TokenStream;
use crate::compiler::Context;

pub struct Parser<'ctx, 'a> {
    context: &'ctx Context,
    token_stream: TokenStream<'a>
}

impl<'ctx, 'a> Parser<'ctx, 'a> {
    pub fn new(token_stream: TokenStream<'a>,context: &'ctx Context ) -> Parser<'ctx, 'a> {
        Parser {
            context,
            token_stream
        }
    }

    pub fn parse(&mut self) -> ASTNode<'a> {
        while self.token_stream.any() {

            let tok = ASTNode {kind: ASTKind::Block, token: self.token_stream.advance(), children: vec![] };
            println!("{tok:?}");
        }
        todo!()
    }
}