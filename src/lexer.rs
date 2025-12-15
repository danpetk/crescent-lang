use crate::error::LexerError;
use crate::token::*;

fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn get_keyword(identifier: &str) -> Option<TokenKind> {
    match identifier {
        "return" => Some(TokenKind::Return),
        _ => None
    }
}

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

    pub fn next_token(&mut self) -> Result<Token<'a>, LexerError> {
        self.skip_whitespace();
        self.start = self.position;

        let token = match self.advance_char() {
            None => Token{kind: TokenKind::EOF, lexeme: "", line: self.line},
            Some(c) => match c {
                ';' => self.make_token(TokenKind::Semi),
                ':' => self.make_token(TokenKind::Colon),
                '{' => self.make_token(TokenKind::OpenCurly),
                '}' => self.make_token(TokenKind::CloseCurly),
                x if x.is_alphabetic() || x == '_' => self.lex_identifier(),
                _ => todo!()            
            }
        };

        Ok(token)
    } 

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() && c.is_whitespace() {
            self.advance_char();
        }
    }
    
    fn peek_char(&self) -> Option<char> {
        return self.source[self.position..].chars().next();
    }

    fn advance_char(&mut self) -> Option<char> {
        let c = self.source[self.position..].chars().next()?;
        if c == '\n' {
            self.line += 1;
        }
        self.position += c.len_utf8();
        Some(c)
    }

    fn lex_identifier(&mut self) -> Token<'a> {
        while let Some(c) = self.peek_char() && is_identifier_char(c) {
            self.advance_char();    
        }    

        let token_kind = get_keyword(self.current_lexeme()).unwrap_or(TokenKind::Identifier);
        self.make_token(token_kind)
    }

    fn current_lexeme(&self) -> &'a str {
        &self.source[self.start..self.position]
    }

    fn make_token(&self, kind: TokenKind) -> Token<'a> {
        Token {
            kind,
            lexeme: self.current_lexeme(),
            line: self.line
        }
    }
}