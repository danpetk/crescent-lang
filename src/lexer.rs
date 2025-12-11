use std::fs;
use crate::error::LexerError;
pub struct Lexer {
    _source: String,
    _start: usize,
    _position: usize,
    _line: i32
}

impl Lexer {
    pub fn from_file(filename: &str) -> Result<Lexer, LexerError> {
        let source = fs::read_to_string(filename).map_err(|_| LexerError::FileError {
                file: filename.to_string()
        })?;
        
        Ok(Lexer { 
            _source: source,
            _start: 0,
            _position: 0,
            _line: 0
        })
    }

    // pub fn next_token(&mut self) -> 
    
}