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
            TokenKind::Let => self.parse_let()?,
            
            _ => self.parse_expr()?.into() // No match so assume expr statement and let that find the error
        
        };

        Ok(statement)
    }

    fn parse_block(&mut self) -> Result<Stmt, ParserError> {
        self.ctx.symbols.borrow_mut().push_scope();

        let token = self.token_stream.expect(TokenKind::OpenCurly)?;
        let mut statements = vec![];
        while self.token_stream.any() && self.token_stream.peek().kind != TokenKind::CloseCurly {
            statements.push(self.parse_statement()?);
        }
        self.token_stream.expect(TokenKind::CloseCurly)?;

        self.ctx.symbols.borrow_mut().pop_scope();

        Ok(Stmt::block(statements, token))
    }

    fn parse_if(&mut self) -> Result<Stmt, ParserError> {
        let token = self.token_stream.expect(TokenKind::If)?;
        let cond = self.parse_expr()?;
        let do_if = self.parse_statement()?;
        
        let do_else = if self.token_stream.match_kind(TokenKind::Else) {
            Some(self.parse_statement()?)
        } else {
            None
        };

        Ok(Stmt::if_else(cond, do_if, do_else, token))
    }

    fn parse_while(&mut self) -> Result<Stmt, ParserError> {
        let token = self.token_stream.expect(TokenKind::While)?;
        let cond = self.parse_expr()?;
        let statement = self.parse_statement()?;
        
        Ok(Stmt::while_loop(cond, statement, token))
    }

    // ugly will be fixed
    fn parse_let(&mut self) -> Result<Stmt, ParserError> {
        self.token_stream.expect(TokenKind::Let)?;

        let ident_token = self.token_stream.expect(TokenKind::Identifier)?;
        let ident = self.ctx.source.get_spanned(&ident_token.span);

        self.token_stream.expect(TokenKind::Colon)?;
        
        let type_token = self.token_stream.expect(TokenKind::Identifier)?;
        let typee = self.ctx.source.get_spanned(&type_token.span);

        self.ctx.symbols.borrow_mut().add_var(&ident_token, ident, typee)?;

        todo!()
    }

    fn parse_expr(&mut self) ->  Result<Expr, ParserError> {
        let token = self.token_stream.expect(TokenKind::Identifier)?;

        // println!("\n\n\n{}\n\n\n", self.ctx.source.get_spanned(&token.span));
        Ok(Expr {kind: ExprKind::Dummy, token})
    }
}

