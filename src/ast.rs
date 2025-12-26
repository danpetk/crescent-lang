use crate::tokens::Token;
use std::boxed::Box;

// Different kinds of expressions recognized in the language
#[derive(Debug)]
pub enum ExprKind {

}

#[derive(Debug)]
pub struct Expr {

}

#[derive(Debug)]
// Different kinds of statements recognized in the language
pub enum StmtKind {
    If(Box<Expr>, Box<Stmt>, Option<Box<Stmt>>),
    While(Box<Expr>, Box<Stmt>),
    ExprStmt(Box<Expr>),
    Block(Vec<Stmt>)
}

#[derive(Debug)]
pub struct Stmt {

}

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