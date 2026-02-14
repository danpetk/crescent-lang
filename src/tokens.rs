use crate::diagnostic::{Diagnostic, DiagnosticKind};
use std::fmt::{self};

fn unexpected_token_error(actual: Token, expected: TokenKind) -> Diagnostic {
    Diagnostic {
        line: actual.line,
        kind: DiagnosticKind::UnexpectedToken {
            expected,
            found: actual.kind,
        },
    }
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
    Bang,
    Eq,
    Plus,
    Minus,
    Star,
    Slash,
    LessThan,
    GreaterThan,

    // Multi Char
    LessEq,
    BangEq,
    EqEq,
    GreaterEq,

    // Dynamic
    Identifier,
    Literal,

    // Keywords
    Return,
    Break,
    Continue,
    Func,
    If,
    Else,
    While,
    Let,

    // Special
    EOF,
}

impl TokenKind {
    pub fn starts_expression(&self) -> bool {
        // match self {
        //     // TokenKind::Identifier | TokenKind::Literal | TokenKind::OpenParen | TokenKind::Minus | TokenKind::Bang
        // }
        true
    }
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
            TokenKind::Bang => "!",

            TokenKind::Eq => "=",
            TokenKind::BangEq => "!=",
            TokenKind::EqEq => "==",
            TokenKind::Plus => "+",
            TokenKind::Minus => "-",
            TokenKind::Star => "*",
            TokenKind::Slash => "/",
            TokenKind::LessThan => "<",
            TokenKind::LessEq => "<=",
            TokenKind::GreaterThan => ">",
            TokenKind::GreaterEq => ">=",

            TokenKind::Identifier => "identifier",
            TokenKind::Literal => "literal",

            TokenKind::Return => "return",
            TokenKind::Break => "break",
            TokenKind::Continue => "continue",
            TokenKind::Func => "func",
            TokenKind::If => "if",
            TokenKind::Else => "else",
            TokenKind::While => "while",
            TokenKind::Let => "let",
            TokenKind::EOF => "EOF",
        };
        write!(f, "{rep}")
    }
}
#[derive(Debug, Clone)]
pub struct SourceSpan {
    pub low: usize,
    pub high: usize,
}

impl SourceSpan {
    pub fn dummy() -> Self {
        SourceSpan { low: 0, high: 0 }
    }
}

// Tokens holding lexemes as owned strings is something I am not proud of
// since it duplicates the source string and is not memory optimal,
// but this lets me iterate much quicker on the other parts of the code.
// I can change this later to make it more performant
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    // pub span: SourceSpan,
    pub line: i32,
}

// Nothing to do with proc_macro::TokenStream :)
pub struct TokenStream {
    tokens: Vec<Token>,
    pos: usize,
}

impl TokenStream {
    pub fn new(tokens: Vec<Token>) -> TokenStream {
        TokenStream { tokens, pos: 0 }
    }

    pub fn advance(&mut self) -> Token {
        let token = self
            .tokens
            .get(self.pos)
            .expect("advance should not allow pos to be out of bounds");
        if token.kind != TokenKind::EOF {
            self.pos += 1;
        }
        token.clone() // clone is cheap here, plus the TokenStream "serves" tokens, so it should not give ref
    }

    pub fn expect(&mut self, expected_kind: TokenKind) -> Result<Token, Diagnostic> {
        let tok = self.advance();
        if tok.kind != expected_kind {
            return Err(unexpected_token_error(tok, expected_kind));
        }
        Ok(tok)
    }

    pub fn match_kind(&mut self, expected_kind: TokenKind) -> Option<Token> {
        if self.peek().kind == expected_kind {
            return Some(self.advance());
        }
        None
    }

    pub fn peek(&self) -> Token {
        self.tokens
            .get(self.pos)
            .expect("advance should not allow pos to be out of bounds")
            .clone()
    }

    pub fn any(&self) -> bool {
        self.peek().kind != TokenKind::EOF
    }
}
