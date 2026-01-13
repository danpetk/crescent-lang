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
    UnexpectedToken{line: i32, expected: TokenKind, found: TokenKind},
    VarRedeclared{line_redec: i32, line_orig: i32, var_name: String},
    VarUnknown{line: i32, var_name: String}
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnexpectedToken {line, expected, found} => 
                write!(f, "ERROR (line {line}): Expected token '{expected}', found '{found}'"),
            Self::VarRedeclared { line_redec, line_orig, var_name } => 
                write!(f, "ERROR (line {line_redec}): Variable '{var_name}' redeclared. (Orignally declared on line {line_orig})"),
            Self::VarUnknown { line, var_name } => 
                write!(f, "ERROR (line {line}): Unknown variable '{var_name}'")
        }
    }
}

impl Error for ParserError {} // nothing else needed since no source
