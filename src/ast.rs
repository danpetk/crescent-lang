use crate::tokens::Token;

#[derive(Debug)]
pub enum ASTKind {
    Block
}

#[derive(Debug)]
pub struct ASTNode<'a> {
    pub kind: ASTKind,
    pub token: Token<'a>,
    pub children: Vec<ASTNode<'a>>
}