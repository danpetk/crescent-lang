#[derive(Debug)]
pub enum TokenKind {
    EOF,
    IDENTIFIER
}

#[derive(Debug)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub lexeme: &'a str,
    pub line_number: i32
}