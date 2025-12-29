use crate::tokens::Token;
use std::boxed::Box;

#[derive(Debug)]
pub enum BinOpKind {
    Add,
}

#[derive(Debug)]
pub enum UnOpKind {
    Not,
}

// Different kinds of expressions recognized in the language
#[derive(Debug)]
pub enum ExprKind {
    BinOp(BinOpKind, Box<Expr>, Box<Expr>),
    UnOp(UnOpKind, Box<Expr>)
}

#[derive(Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub token: Token
}

// Different kinds of statements recognized in the language
#[derive(Debug)]
pub enum StmtKind {
    If(Box<Expr>, Box<Stmt>, Option<Box<Stmt>>),
    While(Box<Expr>, Box<Stmt>),
    ExprStmt(Box<Expr>),
    Block(Vec<Stmt>),
    File(Vec<Stmt>)
}

#[derive(Debug)]
pub struct Stmt {
    pub kind: StmtKind,
    pub token: Token

}

