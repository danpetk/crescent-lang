use crate::ast::{Root, Stmt};
use crate::compiler::Context;

pub struct SemanticAnalyzer<'ctx> {
    _ctx: &'ctx Context,
}

impl<'ctx> SemanticAnalyzer<'ctx> {
    pub fn new(ctx: &'ctx Context) -> Self {
        SemanticAnalyzer { _ctx: ctx }
    }

    pub fn analyze(&mut self, ast: &mut Root) {
        for stmt in &mut ast.top {
            self.analyze_statement(stmt);
        }
    }

    fn analyze_statement(&mut self, _stmt: &mut Stmt) {}
}
