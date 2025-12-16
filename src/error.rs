use std::{fmt};
use std::error::Error;


#[derive(Debug)]
pub enum LexerError {
    InvalidToken{line: i32, lexeme: String}
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidToken { line, lexeme } => 
                write!(f, "ERROR (line {line}) Unexpected token in source file: '{lexeme}'")
        }
    }
}

impl Error for LexerError {} // nothing else needed since no source


