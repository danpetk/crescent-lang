use crate::ast::{
    BinOpInfo, BinOpKind, Expr, ExprKind, FuncDeclInfo, IfInfo, Program, Stmt, StmtKind, UnOpKind,
    VarDeclInfo, WhileInfo,
};
use crate::compiler::Context;
use crate::diagnostic::{Diagnostic, DiagnosticKind};
use crate::symbols::SymbolID;
use crate::symbols::Symbols;
use crate::tokens::Token;

use std::cell::{Ref, RefMut};

#[derive(Debug, Clone, Copy)]
pub struct LoopID(usize);

impl LoopID {
    pub fn next(&mut self) -> Self {
        let current = *self;
        self.0 += 1;
        current
    }
}

pub struct SemanticAnalyzer<'ctx> {
    ctx: &'ctx Context,

    // Stuff pertaining to loops
    next_loop_id: LoopID,
    loop_id_stack: Vec<LoopID>,

    // Function
    current_function: Option<SymbolID>,
}

impl<'ctx> SemanticAnalyzer<'ctx> {
    pub fn new(ctx: &'ctx Context) -> Self {
        SemanticAnalyzer {
            ctx: ctx,
            next_loop_id: LoopID(0),
            loop_id_stack: vec![],
            current_function: None,
        }
    }

    // In the future, we could syncronize to catch multiple errors
    pub fn analyze(&mut self, ast: &mut Program) {
        for stmt in &mut ast.top {
            match self.analyze_statement(stmt) {
                Ok(_) => {}
                Err(diag) => {
                    self.ctx.diags.borrow_mut().report(diag);
                    return;
                }
            }
        }

        if let Err(diag) = self.validate_main() {
            self.ctx.diags.borrow_mut().report(diag)
        }
    }

    // TODO: Restructure this to avoid token cloning
    // instead of passing the data in the matched enum
    // we should match and then pass the whole node into the function ideally
    fn analyze_statement(&mut self, stmt: &mut Stmt) -> Result<(), Diagnostic> {
        match &mut stmt.kind {
            StmtKind::Empty => {}
            StmtKind::If(info) => self.analyze_if(info)?,
            StmtKind::ExprStmt(expr) => self.analyze_expr(expr)?,
            StmtKind::Block(stmts) => self.analyze_block(stmts)?,
            StmtKind::While(info) => self.analyze_while(info)?,
            StmtKind::VarDecl(info) => self.analyze_var(info, stmt.token.clone())?,
            StmtKind::FuncDecl(info) => self.analyze_func(info, stmt.token.clone())?,
            StmtKind::Continue(id) => self.analyze_continue(id, stmt.token.clone())?,
            StmtKind::Break(id) => self.analyze_break(id, stmt.token.clone())?,
            StmtKind::Return(expr) => self.analyze_return(expr, stmt.token.clone())?,
        }

        Ok(())
    }

    fn analyze_if(&mut self, info: &mut IfInfo) -> Result<(), Diagnostic> {
        let IfInfo {
            cond,
            do_if,
            do_else,
        } = info;

        self.analyze_expr(cond)?;
        self.analyze_statement(do_if)?;
        if let Some(do_else) = do_else {
            self.analyze_statement(do_else)?;
        }
        Ok(())
    }

    fn analyze_block(&mut self, stmts: &mut Vec<Stmt>) -> Result<(), Diagnostic> {
        self.symbols_mut().push_scope();
        self.analyze_block_inner(stmts)?;
        self.symbols_mut().pop_scope();
        Ok(())
    }

    fn analyze_block_inner(&mut self, stmts: &mut Vec<Stmt>) -> Result<(), Diagnostic> {
        for stmt in stmts {
            self.analyze_statement(stmt)?;
        }
        Ok(())
    }

    fn analyze_var(&mut self, info: &mut VarDeclInfo, var_token: Token) -> Result<(), Diagnostic> {
        let VarDeclInfo { id, ty, expr } = info;
        self.analyze_expr(expr)?;
        let symbol_id =
            self.symbols_mut()
                .register_var(&var_token, &ty, self.current_function.unwrap())?;
        *id = Some(symbol_id);
        Ok(())
    }

    fn analyze_func(
        &mut self,
        info: &mut FuncDeclInfo,
        func_token: Token,
    ) -> Result<(), Diagnostic> {
        let FuncDeclInfo {
            id,
            ty,
            params,
            body,
        } = info;

        let prev = self.current_function.take();
        let func_id = self.symbols_mut().register_func(&func_token, ty)?;
        *id = Some(func_id);
        self.current_function = *id;

        self.symbols_mut().push_scope();
        let mut param_ids = vec![];
        for param in params {
            let param_id = self
                .symbols_mut()
                .register_var(&param.token, &param.ty, func_id)?;
            param_ids.push(param_id);
        }
        self.symbols_mut().add_func_params(func_id, param_ids);

        let StmtKind::Block(stmts) = &mut body.kind else {
            unreachable!("func body must be a block")
        };
        self.analyze_block_inner(stmts)?;
        self.symbols_mut().pop_scope();

        self.current_function = prev;
        Ok(())
    }

    fn analyze_while(&mut self, info: &mut WhileInfo) -> Result<(), Diagnostic> {
        let WhileInfo { id, cond, body } = info;

        *id = Some(self.next_loop_id.next());
        self.loop_id_stack.push(id.unwrap());

        self.analyze_expr(cond)?;
        self.analyze_statement(body)?;

        // TODO: Be very careful here, right now there is no problem because on error we fully stop
        // compiling but if we resync, an error above and this will never pop
        self.loop_id_stack.pop();
        Ok(())
    }

    fn analyze_continue(
        &mut self,
        id: &mut Option<LoopID>,
        token: Token,
    ) -> Result<(), Diagnostic> {
        if let Some(current_id) = self.loop_id_stack.last() {
            *id = Some(*current_id)
        } else {
            return Err(Diagnostic {
                line: token.line,
                kind: DiagnosticKind::ContinueOutsideLoop,
            });
        }
        Ok(())
    }

    fn analyze_break(&mut self, id: &mut Option<LoopID>, token: Token) -> Result<(), Diagnostic> {
        if let Some(current_id) = self.loop_id_stack.last() {
            *id = Some(*current_id)
        } else {
            return Err(Diagnostic {
                line: token.line,
                kind: DiagnosticKind::ContinueOutsideLoop,
            });
        }
        Ok(())
    }

    fn analyze_return(&mut self, expr: &mut Box<Expr>, token: Token) -> Result<(), Diagnostic> {
        self.analyze_expr(expr)?;
        if self.current_function.is_none() {
            return Err(Diagnostic {
                line: token.line,
                kind: DiagnosticKind::ContinueOutsideLoop,
            });
        }
        Ok(()) // TODO: Return here when we add more types
    }

    // TODO: Restructure this to avoid token cloning
    // instead of passing the data in the matched enum
    // we should match and then pass the whole node into the function ideally
    // I need a way to do this with the borrow checker
    fn analyze_expr(&mut self, expr: &mut Box<Expr>) -> Result<(), Diagnostic> {
        match &mut expr.kind {
            ExprKind::BinOp(info) => self.analyze_expr_binop(info)?,
            ExprKind::UnOp(kind, expr) => self.analyze_expr_unop(kind, expr)?,
            ExprKind::Var(id) => self.analyze_expr_var(id, expr.token.clone())?,
            ExprKind::Literal(num) => self.analyze_expr_literal(num)?,
        }
        Ok(())
    }

    fn analyze_expr_binop(&mut self, info: &mut BinOpInfo) -> Result<(), Diagnostic> {
        let BinOpInfo { op, lhs, rhs } = info;
        self.analyze_expr(lhs)?;
        self.analyze_expr(rhs)?;

        if matches!(op, BinOpKind::Assign) && !matches!(&lhs.kind, ExprKind::Var(_)) {
            return Err(Diagnostic {
                line: lhs.token.line,
                kind: DiagnosticKind::InvalidAssignment,
            });
        }

        Ok(())
    }

    fn analyze_expr_unop(
        &mut self,
        _kind: &mut UnOpKind,
        expr: &mut Box<Expr>,
    ) -> Result<(), Diagnostic> {
        self.analyze_expr(expr)?;
        Ok(())
    }

    fn analyze_expr_var(
        &mut self,
        id: &mut Option<SymbolID>,
        token: Token,
    ) -> Result<(), Diagnostic> {
        *id = Some(self.symbols().get_var_id(&token)?);
        Ok(())
    }

    fn analyze_expr_literal(&mut self, _num: &mut i64) -> Result<(), Diagnostic> {
        Ok(())
    }

    fn validate_main(&mut self) -> Result<(), Diagnostic> {
        let id = self.symbols().get_main_id();
        match id {
            Some(id) if self.symbols().func_info(id).params.len() == 0 => Ok(()),
            _ => Err(Diagnostic {
                line: -1,
                kind: DiagnosticKind::InvalidMain,
            }),
        }
    }

    fn symbols_mut(&mut self) -> RefMut<'ctx, Symbols> {
        self.ctx.symbols.borrow_mut()
    }

    fn symbols(&self) -> Ref<'ctx, Symbols> {
        self.ctx.symbols.borrow()
    }
}
