use crate::ast::{BinOpKind, Expr, Program, Stmt, StmtKind, UnOpKind};
use crate::compiler::Context;
use crate::diagnostic::{Diagnostic, DiagnosticKind};
use crate::symbols::GenericType;
use crate::tokens::{Token, TokenKind, TokenStream};

pub type ParsedType = GenericType<Token>;

#[derive(Debug)]
pub struct ParsedParam {
    pub token: Token,
    pub ty: ParsedType,
}

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

    // TODO: In the future, this will synchronize to report multiple errors
    pub fn parse(&mut self) -> Program {
        let mut statements = vec![];
        while self.token_stream.any() {
            match self.parse_func() {
                Ok(stmt) => statements.push(stmt),
                Err(diagnostic) => {
                    self.ctx.diags.borrow_mut().report(diagnostic);
                    break;
                }
            }
        }
        Program { top: statements }
    }

    fn parse_statement(&mut self) -> Result<Stmt, Diagnostic> {
        let tok = self.token_stream.peek();

        let statement = match tok.kind {
            // Tokens without semicolons
            TokenKind::OpenCurly => return Ok(self.parse_block()?),
            TokenKind::If => return Ok(self.parse_if()?),
            TokenKind::While => return Ok(self.parse_while()?),

            // Tokens with semicolons
            TokenKind::Let => self.parse_let()?,
            TokenKind::Return => self.parse_return()?,
            TokenKind::Continue => self.parse_continue()?,
            TokenKind::Break => self.parse_break()?,
            TokenKind::Semi => self.parse_empty()?,

            // TODO: Allow this later
            TokenKind::Func => {
                return Err(Diagnostic {
                    line: tok.line,
                    kind: DiagnosticKind::FuncInScope,
                });
            }

            // TODO: Refactor to only allow starting expr tokens to avoid _
            _ => self.parse_expr()?.into(), // No match so assume expr statement and let that find the error
        };

        self.token_stream.expect(TokenKind::Semi)?;
        Ok(statement)
    }

    fn parse_block(&mut self) -> Result<Stmt, Diagnostic> {
        let token = self.token_stream.expect(TokenKind::OpenCurly)?;
        let mut statements = vec![];
        while self.token_stream.any() && self.token_stream.peek().kind != TokenKind::CloseCurly {
            statements.push(self.parse_statement()?);
        }
        self.token_stream.expect(TokenKind::CloseCurly)?;

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
        self.token_stream.expect(TokenKind::Eq)?;
        let rhs = self.parse_expr()?;

        Ok(Stmt::var_decl(
            ParsedType::Named(type_token),
            rhs,
            var_token,
        ))
    }

    fn parse_func(&mut self) -> Result<Stmt, Diagnostic> {
        self.token_stream.expect(TokenKind::Func)?;
        let func_token = self.token_stream.expect(TokenKind::Identifier)?;
        self.token_stream.expect(TokenKind::OpenParen)?;
        let mut params = vec![];

        while self.token_stream.peek().kind != TokenKind::CloseParen {
            let token = self.token_stream.expect(TokenKind::Identifier)?;
            self.token_stream.expect(TokenKind::Colon)?;
            let type_token = self.token_stream.expect(TokenKind::Identifier)?;
            params.push(ParsedParam {
                token,
                ty: ParsedType::Named(type_token),
            });
            if self.token_stream.peek().kind != TokenKind::CloseParen {
                self.token_stream.expect(TokenKind::Comma)?;
            }
        }

        self.token_stream.expect(TokenKind::CloseParen)?;
        self.token_stream.expect(TokenKind::Colon)?;
        let ret_token = self.token_stream.expect(TokenKind::Identifier)?;
        let body = self.parse_block()?;

        Ok(Stmt::func_decl(
            ParsedType::Named(ret_token),
            params,
            body,
            func_token,
        ))
    }

    fn parse_return(&mut self) -> Result<Stmt, Diagnostic> {
        let token = self.token_stream.expect(TokenKind::Return)?;
        let expr = self.parse_expr()?;

        Ok(Stmt::return_stmt(expr, token))
    }

    fn parse_continue(&mut self) -> Result<Stmt, Diagnostic> {
        let token = self.token_stream.expect(TokenKind::Continue)?;
        Ok(Stmt::continue_stmt(token))
    }

    fn parse_break(&mut self) -> Result<Stmt, Diagnostic> {
        let token = self.token_stream.expect(TokenKind::Break)?;
        Ok(Stmt::break_stmt(token))
    }

    fn parse_empty(&mut self) -> Result<Stmt, Diagnostic> {
        let token = self.token_stream.expect(TokenKind::Semi)?;
        Ok(Stmt::trivial_stmt(StmtKind::Empty, token))
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
                if self.token_stream.peek().kind != TokenKind::OpenParen {
                    return Ok(Expr::var(token));
                }

                self.token_stream.expect(TokenKind::OpenParen)?;
                let mut args = vec![];

                while self.token_stream.peek().kind != TokenKind::CloseParen {
                    let expr = self.parse_expr()?;
                    args.push(expr);

                    if self.token_stream.peek().kind != TokenKind::CloseParen {
                        self.token_stream.expect(TokenKind::Comma)?;
                    }
                }

                self.token_stream.expect(TokenKind::CloseParen)?;

                Ok(Expr::func(args, token))
            }
            TokenKind::Literal => {
                let val: i64 = token.lexeme.parse().map_err(|_| Diagnostic {
                    line: token.line,
                    kind: DiagnosticKind::NumLiteralTooLarge {
                        literal: token.lexeme.to_owned(),
                    },
                })?;
                Ok(Expr::lit(val, token))
            }
            TokenKind::OpenParen => {
                let expr = self.parse_expr()?;
                self.token_stream.expect(TokenKind::CloseParen)?;
                Ok(expr)
            }
            TokenKind::Bang => {
                let expr = self.parse_expr()?;
                Ok(Expr::unary_op(UnOpKind::Not, expr, token))
            }
            TokenKind::Minus => {
                let expr = self.parse_expr()?;
                Ok(Expr::unary_op(UnOpKind::Neg, expr, token))
            }
            _ => Err(Diagnostic {
                line: token.line,
                kind: DiagnosticKind::UnexpectedTokenInExpression { found: token.kind },
            }),
        }
    }
}

#[derive(PartialEq)]
enum AssocKind {
    Left,
    Right,
    None,
}

// </3
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
