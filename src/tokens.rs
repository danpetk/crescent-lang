use std::fmt::{self};
use crate::error::ParserError;

fn unexpected_token_error(actual: Token, expected: TokenKind) -> ParserError {
    ParserError::UnexpectedToken { line: actual.line, expected, found: actual.kind}
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {

    // Single Char
    Semi,
    Colon,
    OpenCurly,
    CloseCurly,
    OpenParen,
    CloseParen,
    Comma,
    Bang,
    
    // Multi Char


    // Dynamic
    Identifier,
    
    // Keywords
    Return,
    Func,
    
    // Special
    EOF,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rep = match self {
            TokenKind::Semi => ";",
            TokenKind::Colon => ":", 
            TokenKind::OpenCurly => "{",
            TokenKind::CloseCurly => "}",
            TokenKind::OpenParen => "(",
            TokenKind::CloseParen => ")",
            TokenKind::Comma => ",",
            TokenKind::Bang => "!",
            TokenKind::Identifier => "identifier",
            TokenKind::Return => "return",
            TokenKind::Func => "func",
            TokenKind::EOF => "EOF"
        };
        write!(f, "{rep}")
    }
}

#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub lexeme: &'a str,
    pub line: i32,
}

// Nothing to do with proc_macro::TokenStream :)
pub struct TokenStream<'a> {
    tokens: Vec<Token<'a>>,
    pos: usize
}

impl<'a> TokenStream<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> TokenStream<'a> {
        TokenStream {
            tokens,
            pos: 0
        }
    }

    pub fn advance(&mut self) -> Token<'a> {
        let token = self.tokens.get(self.pos).expect("advance should not allow pos to be out of bounds");
        if token.kind != TokenKind::EOF {
            self.pos += 1;
        }
        token.clone() // clone is cheap here, plus the TokenStream "serves" tokens, so it should not give ref
    }

    pub fn expect(&mut self, expected_kind: TokenKind) -> Result<Token<'a>, ParserError> {
        let tok = self.advance();
        if tok.kind != expected_kind {
            return Err(unexpected_token_error(tok, expected_kind))
        }
        Ok(tok)
    }

    pub fn peek(&self) -> Token<'a>{
        self.tokens.get(self.pos).expect("advance should not allow pos to be out of bounds").clone()
    }

    pub fn any(&self) -> bool {
        self.peek().kind != TokenKind::EOF
    }
}