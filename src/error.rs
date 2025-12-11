use std::{fmt};
use std::error::Error;


#[derive(Debug)]
pub enum LexerError {
    FileError{file: String}
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::FileError{file} => write!(f,"Could not read file '{}'.", file)
        }
    }
}

impl Error for LexerError {} // nothing else needed since no source


