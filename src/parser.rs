use crate::ast::{BinOpKind, Expr, Root, Stmt};
use crate::error::ParserError;
use crate::symbols::{Symbol};
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

            TokenKind::OpenCurly => return Ok(self.parse_block()?),
            TokenKind::If => return Ok(self.parse_if()?),
            TokenKind::While => return Ok(self.parse_while()?),
            
            TokenKind::Let => self.parse_let()?,
            _ => self.parse_expr()?.into() // No match so assume expr statement and let that find the error
        };

        self.token_stream.expect(TokenKind::Semi)?;
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
        
        let do_else = if self.token_stream.match_kind(TokenKind::Else).is_some() {
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

        let symbol = self.ctx.symbols.borrow_mut().add_local_var(&ident_token, ident, typee)?;
        let eq_token = self.token_stream.expect(TokenKind::Eq)?;

        let lhs = Expr::var(symbol, ident_token);
        let rhs = self.parse_expr()?;

        Ok(Expr::binary_op(BinOpKind::Assign, lhs, rhs, eq_token).into())
    }

    fn parse_expr(&mut self) ->  Result<Expr, ParserError> {
        Ok(self.parse_expr_recursive(None, 4)?)
    }

    fn parse_expr_recursive(&mut self, lhs: Option<Expr>, prec: u32) -> Result<Expr, ParserError> {
        if prec == 0 {
            return Ok(match lhs {
                Some(expr) => expr,
                None => self.parse_term()?
            })
        }

        let mut lhs = self.parse_expr_recursive(lhs, prec-1)?;
        let next = self.token_stream.peek();

        if let Some((op_prec, assoc_kind, op_kind)) = get_op_info(next.kind) && op_prec == prec {
            let op = self.token_stream.advance();
            let right_prec = if assoc_kind == AssocKind::Right {prec} else {prec-1};
        
            let rhs = self.parse_expr_recursive(None, right_prec)?;
            lhs = Expr::binary_op(op_kind, lhs, rhs, op);

            if assoc_kind == AssocKind::Left {
                lhs = self.parse_expr_recursive(Some(lhs), prec)?
            }
        }
        
        Ok(lhs)
    }

    fn parse_term(&mut self) -> Result<Expr, ParserError> {
        let tok = self.token_stream.advance();
        match tok.kind {
            TokenKind::Identifier => {
                let var_name = self.ctx.source.get_spanned(&tok.span);
                // let symbol = self.ctx.symbols.borrow().get_local_var(&tok, var_name)?;
                let symbol = Symbol::gg();
                Ok(Expr::var(symbol, tok))
            },
            _ => todo!()
        }
    }

}

#[derive(PartialEq)]
enum AssocKind {
    Left,
    Right,
    None
}

fn get_op_info(kind: TokenKind) -> Option<(u32, AssocKind, BinOpKind)> {
    Some(
        match kind {
            TokenKind::Eq => (4, AssocKind::Right, BinOpKind::Assign),
            TokenKind::EqEq => (3, AssocKind::None, BinOpKind::Equals),
            TokenKind::BangEq => (3, AssocKind::None, BinOpKind::NotEquals),
            TokenKind::LessThan => (3, AssocKind::None, BinOpKind::LessThan),
            TokenKind::LessEq => (3, AssocKind::None, BinOpKind::LessEq),
            TokenKind::GreaterThan => (3, AssocKind::None, BinOpKind::GreaterThan),
            TokenKind::GreaterEq => (3, AssocKind::None, BinOpKind::GreaterEq),
            TokenKind::Plus => (2, AssocKind::Left, BinOpKind::Add),
            TokenKind::Minus => (2, AssocKind::Left, BinOpKind::Sub),
            TokenKind::Star => (1, AssocKind::Left, BinOpKind::Mult),
            TokenKind::Slash => (1, AssocKind::Left, BinOpKind::Div),
            _ => { return None; }
        }
    )
}