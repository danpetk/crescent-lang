
#[derive(Debug, PartialEq)]
pub enum TokenKind {

    // Single Char
    Semi,
    Colon,
    OpenCurly,
    CloseCurly,
    OpenParen,
    CloseParen,
    Comma,
    
    // Multi Char


    // Dynamic
    Identifier,
    
    // Keywords
    Return,
    Func,
    
    // Special
    EOF,
}

#[derive(Debug)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub lexeme: &'a str,
    pub line: i32,
}

// Nothing to do with proc_macro::TokenStream :)
pub struct TokenStream<'a> {
    tokens: Vec<Token<'a>>,
    pos: usize
}

impl<'a> TokenStream<'a> {
    pub fn new(tokens: Vec<Token>) -> TokenStream {
        TokenStream {
            tokens,
            pos: 0
        }
    }

    pub fn peek(&self) -> &Token<'a> {
        &self.tokens.get(self.pos).expect("advance should not allow pos to be out of bounds")
    }
}