use std::{fmt};
use std::error::Error;

use crate::tokens::{TokenKind};

#[derive(Debug)]
pub enum LexerError {
    InvalidToken{line: i32, lexeme: String}
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidToken { line, lexeme } => 
                write!(f, "ERROR (line {line}): Unexpected token in source file: '{lexeme}'")
        }
    }
}

impl Error for LexerError {} // nothing else needed since no source

#[derive(Debug)]
pub enum ParserError {
    UnexpectedToken{line: i32, expected: TokenKind, found: TokenKind}
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnexpectedToken {line, expected, found} => 
                write!(f, "ERROR (line {line}): Expected token '{expected}', found '{found}'")
        }
    }
}

impl Error for ParserError {} // nothing else needed since no source
