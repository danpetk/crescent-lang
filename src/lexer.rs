use std::io;
use std::fs;

pub struct Lexer {
    _source: String
}

impl Lexer {
    pub fn from_file(filename: &str) -> Result<Lexer, io::Error> {
        Ok(Lexer { 
            _source: fs::read_to_string(filename)? 
        })
    }
}