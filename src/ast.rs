use crate::tokens::Token;
use crate::symbols::{Symbol};

#[derive(Debug)]
pub enum BinOpKind {
    Assign,
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
    UnOp(UnOpKind, Box<Expr>),
    Var(Symbol),
    Dummy
}

#[derive(Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub token: Token
}

impl Expr {
    pub fn var(symbol: Symbol, token: Token) -> Self {
        Expr {
            kind: ExprKind::Var(symbol),
            token
        }
    }

    pub fn binary_op(kind: BinOpKind, lhs: Expr, rhs: Expr, token: Token) -> Self {
        Expr {
            kind: ExprKind::BinOp(
                kind, 
                Box::new(lhs), 
                Box::new(rhs)
            ),
            token
        }
    }
}

// Different kinds of statements recognized in the language
#[derive(Debug)]
pub enum StmtKind {
    If(Box<Expr>, Box<Stmt>, Option<Box<Stmt>>),
    While(Box<Expr>, Box<Stmt>),
    ExprStmt(Box<Expr>),
    Block(Vec<Stmt>),
}

#[derive(Debug)]
pub struct Stmt {
    pub kind: StmtKind,
    pub token: Token
}

impl Stmt {
    pub fn block(statements: Vec<Stmt>, token: Token) -> Self {
        Stmt { 
            kind: StmtKind::Block(statements), 
            token
        }
    }

    pub fn if_else(cond: Expr, do_if: Stmt, do_else: Option<Stmt>, token: Token) -> Self {
        Stmt {
            kind: StmtKind::If(
                Box::new(cond), 
                Box::new(do_if), 
                do_else.map(Box::new)
            ),
            token
        }
    }

    pub fn while_loop(cond: Expr, statement: Stmt, token: Token) -> Self {
        Stmt {
            kind: StmtKind::While(
                Box::new(cond), 
                Box::new(statement)
            ),
            token
        }
    }
}

impl From<Expr> for Stmt {
    fn from(expr: Expr) -> Self {
        let token = expr.token.clone();
        Stmt { kind: StmtKind::ExprStmt(Box::new(expr)), token }
    }
}

#[derive(Debug)]
pub struct Root {
    pub top: Vec<Stmt>
}