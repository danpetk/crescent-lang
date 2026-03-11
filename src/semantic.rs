use crate::ast::{Expr, Root, Stmt, StmtKind};
use crate::compiler::Context;
use crate::diagnostic::Diagnostic;
use crate::id::LoopID;

pub struct SemanticAnalyzer<'ctx> {
    ctx: &'ctx Context,

    // Stuff pertaining to loops
    next_loop_id: LoopID,
    loop_id_stack: Vec<LoopID>,
}

impl<'ctx> SemanticAnalyzer<'ctx> {
    pub fn new(ctx: &'ctx Context) -> Self {
        SemanticAnalyzer {
            ctx: ctx,
            next_loop_id: LoopID(0),
            loop_id_stack: vec![],
        }
    }

    // In the future, we could syncronize to catch multiple errors
    pub fn analyze(&mut self, ast: &mut Root) {
        for stmt in &mut ast.top {
            match self.analyze_statement(stmt) {
                Ok(_) => {}
                Err(diagnostic) => {
                    self.ctx.diags.borrow_mut().report(diagnostic);
                    break;
                }
            }
        }
    }

    fn analyze_statement(&mut self, stmt: &mut Stmt) -> Result<(), Diagnostic> {
        match &mut stmt.kind {
            StmtKind::Empty => {}
            StmtKind::If(cond, do_if, do_else) => self.analyze_if(cond, do_if, do_else)?,
            StmtKind::ExprStmt(expr) => self.analyze_expr(expr)?,
            StmtKind::Block(stmts) => self.analyze_block(stmts)?,
            StmtKind::While(id, expr, stmt) => self.analyze_while(id, expr, stmt)?,
            _ => todo!(),
        }

        Ok(())
    }

    fn analyze_if(
        &mut self,
        cond: &mut Box<Expr>,
        do_if: &mut Box<Stmt>,
        do_else: &mut Option<Box<Stmt>>,
    ) -> Result<(), Diagnostic> {
        self.analyze_expr(cond)?;
        self.analyze_statement(do_if)?;
        if let Some(do_else) = do_else {
            self.analyze_statement(do_else)?;
        }
        Ok(())
    }

    fn analyze_block(&mut self, stmts: &mut Vec<Stmt>) -> Result<(), Diagnostic> {
        for stmt in stmts {
            self.analyze_statement(stmt)?
        }
        Ok(())
    }

    fn analyze_while(
        &mut self,
        id: &mut LoopID,
        expr: &mut Box<Expr>,
        stmt: &mut Box<Stmt>,
    ) -> Result<(), Diagnostic> {
        *id = self.next_loop_id.next();
        self.loop_id_stack.push(*id);

        self.analyze_expr(expr)?;
        self.analyze_statement(stmt)?;

        self.loop_id_stack.pop();
        Ok(())
    }

    fn analyze_expr(&mut self, _expr: &Box<Expr>) -> Result<(), Diagnostic> {
        // Nothing for now!
        Ok(())
    }
}
