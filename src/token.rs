#[derive(Debug)]
pub enum TokenKind {
    EOF,
    IDENTIFIER
}

#[derive(Debug)]
pub struct Token<'a> {
    _kind: TokenKind,
    _lexeme: &'a str,
    _line_number: i32
}