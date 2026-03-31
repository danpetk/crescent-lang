use crate::parser::{ParsedParam, ParsedType};
use crate::semantic::LoopID;
use crate::symbols::SymbolID;
use crate::tokens::Token;

#[derive(Debug)]
pub enum BinOpKind {
    Assign,
    Add,
    Sub,
    Mult,
    Div,
    Equals,
    NotEquals,
    LessThan,
    LessEq,
    GreaterThan,
    GreaterEq,
}

#[derive(Debug)]
pub enum UnOpKind {
    Not,
    Neg,
}

// Different kinds of expressions recognized in the language
#[derive(Debug)]
pub enum ExprKind {
    BinOp(BinOpKind, Box<Expr>, Box<Expr>),
    UnOp(UnOpKind, Box<Expr>),
    Var(Option<SymbolID>),
    Literal(i32),
}

#[derive(Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub token: Token,
}

impl Expr {
    pub fn var(token: Token) -> Self {
        Expr {
            kind: ExprKind::Var(None),
            token,
        }
    }

    pub fn lit(val: i32, token: Token) -> Self {
        Expr {
            kind: ExprKind::Literal(val),
            token,
        }
    }

    pub fn binary_op(kind: BinOpKind, lhs: Expr, rhs: Expr, token: Token) -> Self {
        Expr {
            kind: ExprKind::BinOp(kind, Box::new(lhs), Box::new(rhs)),
            token,
        }
    }

    pub fn unary_op(kind: UnOpKind, expr: Expr, token: Token) -> Self {
        Expr {
            kind: ExprKind::UnOp(kind, Box::new(expr)),
            token,
        }
    }
}

#[derive(Debug)]
pub struct FuncDeclInfo {
    pub id: Option<SymbolID>,
    pub ty: ParsedType,
    pub params: Vec<ParsedParam>,
    pub body: Box<Stmt>,
}

#[derive(Debug)]
pub struct IfInfo {
    pub cond: Box<Expr>,
    pub do_if: Box<Stmt>,
    pub do_else: Option<Box<Stmt>>,
}

#[derive(Debug)]
pub struct WhileInfo {
    pub id: Option<LoopID>,
    pub cond: Box<Expr>,
    pub body: Box<Stmt>,
}

// Different kinds of statements recognized in the language
#[derive(Debug)]
pub enum StmtKind {
    VarDecl(ParsedType, Box<Expr>),
    FuncDecl(FuncDeclInfo),
    If(IfInfo),
    While(WhileInfo),
    ExprStmt(Box<Expr>),
    Block(Vec<Stmt>),
    Return(Box<Expr>),
    Break(Option<LoopID>),
    Continue(Option<LoopID>),
    Empty,
}

#[derive(Debug)]
pub struct Stmt {
    pub kind: StmtKind,
    pub token: Token,
}

impl Stmt {
    pub fn block(statements: Vec<Stmt>, token: Token) -> Self {
        Stmt {
            kind: StmtKind::Block(statements),
            token,
        }
    }

    pub fn var_decl(ty: ParsedType, expr: Expr, token: Token) -> Self {
        Stmt {
            kind: StmtKind::VarDecl(ty, Box::new(expr)),
            token,
        }
    }

    pub fn func_decl(ty: ParsedType, params: Vec<ParsedParam>, body: Stmt, token: Token) -> Stmt {
        Stmt {
            kind: StmtKind::FuncDecl(FuncDeclInfo {
                id: None,
                ty,
                params,
                body: Box::new(body),
            }),
            token,
        }
    }

    pub fn if_else(cond: Expr, do_if: Stmt, do_else: Option<Stmt>, token: Token) -> Self {
        Stmt {
            kind: StmtKind::If(IfInfo {
                cond: Box::new(cond),
                do_if: Box::new(do_if),
                do_else: do_else.map(Box::new),
            }),
            token,
        }
    }

    pub fn while_loop(cond: Expr, body: Stmt, token: Token) -> Self {
        Stmt {
            kind: StmtKind::While(WhileInfo {
                id: None,
                cond: Box::new(cond),
                body: Box::new(body),
            }),
            token,
        }
    }

    pub fn return_stmt(expr: Expr, token: Token) -> Self {
        Stmt {
            kind: StmtKind::Return(Box::new(expr)),
            token,
        }
    }

    pub fn trivial_stmt(kind: StmtKind, token: Token) -> Self {
        Stmt { kind, token }
    }

    pub fn continue_stmt(token: Token) -> Self {
        Stmt {
            kind: StmtKind::Continue(None),
            token,
        }
    }

    pub fn break_stmt(token: Token) -> Self {
        Stmt {
            kind: StmtKind::Break(None),
            token,
        }
    }
}

impl From<Expr> for Stmt {
    fn from(expr: Expr) -> Self {
        let token = expr.token.clone();
        Stmt {
            kind: StmtKind::ExprStmt(Box::new(expr)),
            token,
        }
    }
}

#[derive(Debug)]
pub struct Program {
    pub top: Vec<Stmt>,
}
