use crate::ast::{Expr, ExprKind, Stmt, StmtKind, Root};
use crate::error::ParserError;
use crate::tokens::{TokenStream, TokenKind};
use crate::compiler::Context;

pub struct Parser<'ctx> {
    ctx: &'ctx Context,
    token_stream: TokenStream,
    _errors: Vec<ParserError>
}

impl<'ctx> Parser<'ctx> {
    pub fn new(token_stream: TokenStream,context: &'ctx Context ) -> Parser<'ctx> {
        Parser {
            ctx: context,
            token_stream,
            _errors: vec![]
        }
    }

    pub fn parse(&mut self) -> Result<Root, Vec<ParserError>> {
        let mut statements = vec![];
        while self.token_stream.any() {
            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(err) => return Err(vec![err])
            }
        }
        Ok(Root{top: statements})
    }

    fn parse_statement(&mut self) -> Result<Stmt, ParserError> {
        let tok = self.token_stream.peek();

        let statement = match tok.kind {

            TokenKind::OpenCurly => self.parse_block()?,
            TokenKind::If => self.parse_if()?,
            TokenKind::While => self.parse_while()?,
            _ => self.parse_expr()?.into() // No match so assume expr statement and let that find the error
        
        };

        Ok(statement)
    }

    fn parse_block(&mut self) -> Result<Stmt, ParserError> {
        let token = self.token_stream.expect(TokenKind::OpenCurly)?;
        let mut statements = vec![];
        while self.token_stream.any() && self.token_stream.peek().kind != TokenKind::CloseCurly {
            statements.push(self.parse_statement()?);
        }
        self.token_stream.expect(TokenKind::CloseCurly)?;

        Ok(Stmt { 
            kind: StmtKind::Block(statements), 
            token
        })
    }

    fn parse_if(&mut self) -> Result<Stmt, ParserError> {
        let token = self.token_stream.expect(TokenKind::If)?;
        let cond = self.parse_expr()?;
        let if_statement = self.parse_statement()?;
        
        let else_statement = if self.token_stream.match_kind(TokenKind::Else) {
            Some(self.parse_statement()?)
        } else {
            None
        };

        Ok(Stmt {
            kind: StmtKind::If(
                Box::new(cond), 
                Box::new(if_statement), 
                else_statement.map(Box::new)
            ),
            token
        })
    }

    fn parse_while(&mut self) -> Result<Stmt, ParserError> {
        let token = self.token_stream.expect(TokenKind::While)?;
        let cond = self.parse_expr()?;
        let statement = self.parse_statement()?;
        
        Ok(Stmt {
            kind: StmtKind::While(
                Box::new(cond), 
                Box::new(statement), 
            ),
            token
        })
    }

    fn parse_expr(&mut self) ->  Result<Expr, ParserError> {
        let token = self.token_stream.expect(TokenKind::Identifier)?;

        println!("\n\n\n{}\n\n\n", self.ctx.source.get_spanned(&token.span));
        Ok(Expr {kind: ExprKind::Dummy, token})
    }
}

