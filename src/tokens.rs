
#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, Clone)]
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
    pub fn new(tokens: Vec<Token<'a>>) -> TokenStream<'a> {
        TokenStream {
            tokens,
            pos: 0
        }
    }

    pub fn advance(&mut self) -> Token<'a> {
        let token = self.tokens.get(self.pos).expect("advance should not allow pos to be out of bounds");
        if token.kind != TokenKind::EOF {
            self.pos += 1;
        }
        token.clone() // clone is cheap here, plus the TokenStream "serves" tokens, so it should not give ref
    }

    pub fn peek(&self) -> Token<'a>{
        self.tokens.get(self.pos).expect("advance should not allow pos to be out of bounds").clone()
    }

    pub fn any(&self) -> bool {
        self.peek().kind != TokenKind::EOF
    }
}