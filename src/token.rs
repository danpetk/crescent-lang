#[derive(Debug)]
pub enum TokenKind {
    EOF,
    IDENTIFIER,
    SEMICOLON
}

#[derive(Debug)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub lexeme: &'a str,
    pub line: i32
}