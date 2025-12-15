#[derive(Debug)]
pub enum TokenKind {
    EOF,
    Identifier,
    Semi,
    Colon,
    OpenCurly,
    CloseCurly,
    Return
}

#[derive(Debug)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub lexeme: &'a str,
    pub line: i32
}