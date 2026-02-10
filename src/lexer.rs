use crate::compiler::Context;
use crate::diagnostic::{Diagnostic, DiagnosticKind};
use crate::tokens::{TokenKind, Token, TokenStream};

fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn get_keyword(identifier: &str) -> Option<TokenKind> {
    match identifier {
        "return" => Some(TokenKind::Return),
        "func" => Some(TokenKind::Func),
        "if" => Some(TokenKind::If),
        "else" => Some(TokenKind::Else),
        "while" => Some(TokenKind::While),
        "let" => Some(TokenKind::Let),
        _ => None,
    }
}

pub struct Lexer<'ctx> {
    ctx: &'ctx Context,
    start: usize,
    position: usize,
    line: i32,
}

impl<'ctx> Lexer<'ctx> {
    pub fn new(ctx: &'ctx Context) -> Lexer<'ctx> {
        Lexer {
            ctx,
            start: 0,
            position: 0,
            line: 1,
        }
    }

    pub fn tokenize(&mut self) -> TokenStream {
        let mut tokens: Vec<Token> = vec![];

        loop {
            match self.next_token() {
                Some(token) => {
                    let is_eof = token.kind == TokenKind::EOF;
                    tokens.push(token);
                    if is_eof {
                        break;
                    }
                }
                None => {}
            };
        }

        TokenStream::new(tokens)
    }

    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        self.start = self.position;

        let token = match self.advance_char() {
            None => Token {
                kind: TokenKind::EOF,
                lexeme: "".to_string(),
                line: self.line,
            },
            Some(c) => match c {
                ';' => self.make_token(TokenKind::Semi),
                ':' => self.make_token(TokenKind::Colon),
                '{' => self.make_token(TokenKind::OpenCurly),
                '}' => self.make_token(TokenKind::CloseCurly),
                '(' => self.make_token(TokenKind::OpenParen),
                ')' => self.make_token(TokenKind::CloseParen),
                '!' => {
                    let kind = self.match_switch('=', TokenKind::BangEq, TokenKind::Bang);
                    self.make_token(kind)
                }
                '=' => {
                    let kind = self.match_switch('=', TokenKind::EqEq, TokenKind::Eq);
                    self.make_token(kind)
                }
                '<' => {
                    let kind = self.match_switch('=', TokenKind::LessEq, TokenKind::LessThan);
                    self.make_token(kind)
                }
                '>' => {
                    let kind = self.match_switch('=', TokenKind::GreaterEq, TokenKind::GreaterThan);
                    self.make_token(kind)
                }
                '+' => self.make_token(TokenKind::Plus),
                '-' => self.make_token(TokenKind::Minus),
                '*' => self.make_token(TokenKind::Star),
                '/' => self.make_token(TokenKind::Slash),

                x if x.is_alphabetic() || x == '_' => self.lex_identifier(),
                x if x.is_numeric() => self.lex_literal(),
                _ => {
                    self.ctx.diags.borrow_mut().report(
                        Diagnostic {
                            line: self.line,
                            kind: DiagnosticKind::InvalidToken { lexeme: c.to_string() }
                        }
                    );
                    return None
                }
            },
        };

        Some(token)
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char()
            && c.is_whitespace()
        {
            self.advance_char();
        }
    }

    fn peek_char(&self) -> Option<char> {
        return self.ctx.source[self.position..].chars().next();
    }

    fn advance_char(&mut self) -> Option<char> {
        let c = self.ctx.source[self.position..].chars().next()?;
        if c == '\n' {
            self.line += 1;
        }
        self.position += c.len_utf8();
        Some(c)
    }

    fn match_char(&mut self, c: char) -> bool {
        if let Some(next) = self.peek_char()
            && next == c
        {
            self.advance_char();
            return true;
        }
        false
    }

    fn match_switch(&mut self, c: char, yes_kind: TokenKind, no_kind: TokenKind) -> TokenKind {
        if self.match_char(c) {
            yes_kind
        } else {
            no_kind
        }
    }

    fn lex_identifier(&mut self) -> Token {
        while let Some(c) = self.peek_char()
            && is_identifier_char(c)
        {
            self.advance_char();
        }

        let token_kind = get_keyword(self.current_lexeme()).unwrap_or(TokenKind::Identifier);

        self.make_token(token_kind)
    }

    fn lex_literal(&mut self) -> Token {
        while let Some(c) = self.peek_char()
            && c.is_numeric()
        {
            self.advance_char();
        }

        self.make_token(TokenKind::Literal)
    }

    fn current_lexeme(&self) -> &'ctx str {
        &self.ctx.source[self.start..self.position]
    }

    fn make_token(&self, kind: TokenKind) -> Token {
        Token {
            kind,
            lexeme: self.current_lexeme().to_string(),
            // span: SourceSpan {
            //     low: self.start,
            //     high: self.position
            // },
            line: self.line,
        }
    }
}
