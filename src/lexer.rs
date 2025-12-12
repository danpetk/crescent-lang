use crate::error::LexerError;
use crate::token::*;


pub struct Lexer<'a> {
    source: &'a str,
    start: usize,
    position: usize,
    line: i32
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Lexer<'a> {
        Lexer { 
            source,
            start: 0,
            position: 0,
            line: 1
        }
    }

    pub fn next_token(&mut self) -> Result<Token, LexerError> {
        if self.at_end() {
            return Ok(Token{kind: TokenKind::EOF, lexeme: "", line: self.line});
        }

        let token = match self.advance_char() {
            x if x == ';' => self.make_token(TokenKind::SEMICOLON),
            _ => panic!()
        };


        Ok(token)
    } 

    fn at_end(&self) -> bool {
        return self.position >= self.source.len();
    }

    fn peek_char(&self) -> char {
        return self.source[self.position..].chars().next().expect("at_end() should be used before peek_char()");
    }

    pub fn advance_char(&mut self) -> char {
        let c = self.source[self.position..].chars().next().expect("at_end() should be used before advance_char()");
        self.position += c.len_utf8();
        c
    }

    fn make_token(&self, kind: TokenKind) -> Token {
        Token {
            kind,
            lexeme: &self.source[self.start..self.position],
            line: self.line
        }
    }

    
}