use crate::ast::{BinOpKind, Expr, Root, Stmt};
use crate::compiler::Context;
use crate::diagnostic::{Diagnostic, DiagnosticKind};
use crate::tokens::{TokenKind, TokenStream};

pub struct Parser<'ctx> {
    ctx: &'ctx Context,
    token_stream: TokenStream,
}

impl<'ctx> Parser<'ctx> {
    pub fn new(token_stream: TokenStream, context: &'ctx Context) -> Parser<'ctx> {
        Parser {
            ctx: context,
            token_stream,
        }
    }

    // TODO In the future, this will synchronize to report multiple errors
    pub fn parse(&mut self) -> Root {
        let mut statements = vec![];
        while self.token_stream.any() {
            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(diagnostic) => {
                    self.ctx.diags.borrow_mut().report(diagnostic);
                    break;
                }
            }
        }
        Root { top: statements }
    }

    fn parse_statement(&mut self) -> Result<Stmt, Diagnostic> {
        let tok = self.token_stream.peek();

        let statement = match tok.kind {
            TokenKind::OpenCurly => return Ok(self.parse_block()?),
            TokenKind::If => return Ok(self.parse_if()?),
            TokenKind::While => return Ok(self.parse_while()?),

            TokenKind::Let => self.parse_let()?,
            _ => self.parse_expr()?.into(), // No match so assume expr statement and let that find the error
        };

        self.token_stream.expect(TokenKind::Semi)?;
        Ok(statement)
    }

    fn parse_block(&mut self) -> Result<Stmt, Diagnostic> {
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

    fn parse_if(&mut self) -> Result<Stmt, Diagnostic> {
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

    fn parse_while(&mut self) -> Result<Stmt, Diagnostic> {
        let token = self.token_stream.expect(TokenKind::While)?;
        let cond = self.parse_expr()?;
        let statement = self.parse_statement()?;

        Ok(Stmt::while_loop(cond, statement, token))
    }

    fn parse_let(&mut self) -> Result<Stmt, Diagnostic> {
        self.token_stream.expect(TokenKind::Let)?;
        let var_token = self.token_stream.expect(TokenKind::Identifier)?;
        self.token_stream.expect(TokenKind::Colon)?;
        let type_token = self.token_stream.expect(TokenKind::Identifier)?;
        let eq_token = self.token_stream.expect(TokenKind::Eq)?;

        let rhs = self.parse_expr()?;
        let symbol = self
            .ctx
            .symbols
            .borrow_mut()
            .add_local_var(&var_token, &type_token)?;
        let lhs = Expr::var(symbol, var_token);

        Ok(Expr::binary_op(BinOpKind::Assign, lhs, rhs, eq_token).into())
    }

    fn parse_expr(&mut self) -> Result<Expr, Diagnostic> {
        Ok(self.parse_expr_recursive(None, 4)?)
    }

    fn parse_expr_recursive(&mut self, lhs: Option<Expr>, prec: u32) -> Result<Expr, Diagnostic> {
        if prec == 0 {
            return Ok(match lhs {
                Some(expr) => expr,
                None => self.parse_term()?,
            });
        }

        let mut lhs = self.parse_expr_recursive(lhs, prec - 1)?;
        let next = self.token_stream.peek();

        if let Some((op_prec, assoc_kind, op_kind)) = get_op_info(next.kind)
            && op_prec == prec
        {
            let op = self.token_stream.advance();
            let right_prec = if assoc_kind == AssocKind::Right {
                prec
            } else {
                prec - 1
            };

            let rhs = self.parse_expr_recursive(None, right_prec)?;
            lhs = Expr::binary_op(op_kind, lhs, rhs, op);

            if assoc_kind == AssocKind::Left {
                lhs = self.parse_expr_recursive(Some(lhs), prec)?
            }
        }

        Ok(lhs)
    }

    fn parse_term(&mut self) -> Result<Expr, Diagnostic> {
        let token = self.token_stream.advance();
        match token.kind {
            TokenKind::Identifier => {
                let symbol = self.ctx.symbols.borrow().get_local_var(&token)?;
                Ok(Expr::var(symbol, token))
            }
            TokenKind::Literal => {
                let val: i32 = token.lexeme.parse().map_err(|_| Diagnostic {
                    line: token.line,
                    kind: DiagnosticKind::NumLiteralTooLarge {
                        literal: token.lexeme.to_owned(),
                    },
                })?;

                Ok(Expr::lit(val, token))
            }
            _ => todo!(),
        }
    }
}

#[derive(PartialEq)]
enum AssocKind {
    Left,
    Right,
    None,
}

fn get_op_info(kind: TokenKind) -> Option<(u32, AssocKind, BinOpKind)> {
    Some(match kind {
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
        _ => {
            return None;
        }
    })
}
