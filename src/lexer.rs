use crate::error::LexerError;
use crate::token::*;


pub struct Lexer<'a> {
    source: &'a str,
    _start: usize,
    position: usize,
    line: i32
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Lexer<'a> {
        Lexer { 
           source,
            _start: 0,
            position: 0,
            line: 0
        }
    }

    pub fn next_token(&mut self) -> Result<Token, LexerError> {
        if self.at_end() {
            return Ok(Token{kind: TokenKind::EOF, lexeme: "", line_number: self.line});
        }

        todo!();
    } 

    fn at_end(&self) -> bool {
        return self.position >= self.source.len();
    }

    fn peek_char(&self) -> char {
        return self.source[self.position..].chars().next().expect("at_end() should be used to check for out-of-bounds.");
    }

    
}