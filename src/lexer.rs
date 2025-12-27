use crate::error::LexerError;
use crate::tokens::*;

fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn get_keyword(identifier: &str) -> Option<TokenKind> {
    match identifier {
        "return" => Some(TokenKind::Return),
        "func" => Some(TokenKind::Func),
        _ => None
    }
}

pub struct Lexer<'src> {
    source: &'src str,
    start: usize,
    position: usize,
    line: i32
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Lexer<'src> {
        Lexer { 
            source,
            start: 0,
            position: 0,
            line: 1
        }
    }

    pub fn tokenize(&mut self) -> Result<TokenStream, Vec<LexerError>> {
        let mut tokens: Vec<Token> = vec![];
        let mut errors: Vec<LexerError> = vec![];
        
        loop {
            match self.next_token() {
                Ok(token) => {
                    if token.kind == TokenKind::EOF{
                        tokens.push(token);
                        break;
                    }
                    tokens.push(token);
                },
                Err(error) => {
                    errors.push(error);
                }
            };
        }

        if errors.is_empty() {
            Ok(TokenStream::new(tokens))
        } else {
            Err(errors)
        }
    }

    fn next_token(&mut self) -> Result<Token, LexerError> {
        self.skip_whitespace();
        self.start = self.position;

        let token = match self.advance_char() {
            None => Token{kind: TokenKind::EOF, line: self.line},
            Some(c) => match c {
                ';' => self.make_token(TokenKind::Semi),
                ':' => self.make_token(TokenKind::Colon),
                '{' => self.make_token(TokenKind::OpenCurly),
                '}' => self.make_token(TokenKind::CloseCurly),
                ',' => self.make_token(TokenKind::Comma),
                '(' => self.make_token(TokenKind::OpenParen),
                ')' => self.make_token(TokenKind::CloseParen),
                x if x.is_alphabetic() || x == '_' => self.lex_identifier(),
                _ => return Err(LexerError::InvalidToken { line: self.line, lexeme: c.to_string() })          
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

    fn lex_identifier(&mut self) -> Token {
        while let Some(c) = self.peek_char() && is_identifier_char(c) {
            self.advance_char();    
        }    

        let token_kind = get_keyword(self.current_lexeme())
            .unwrap_or(TokenKind::Identifier);
        
        self.make_token(token_kind)
    }

    fn current_lexeme(&self) -> &'src str {
        &self.source[self.start..self.position]
    }

    fn make_token(&self, kind: TokenKind) -> Token {
        Token {
            kind,
            // lexeme: self.current_lexeme(),
            line: self.line
        }
    }
}