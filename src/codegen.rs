use std::{
    cell::Ref,
    fs::File,
    io::{BufWriter, Write},
};

use crate::symbols::{SymbolID, Symbols};

use crate::{
    ast::{FuncDeclInfo, Program, Stmt, StmtKind},
    compiler::Context,
    diagnostic::{Diagnostic, DiagnosticKind},
};

pub struct Codegen<'ctx> {
    ctx: &'ctx Context,
    out: BufWriter<File>,
}

impl<'ctx> Codegen<'ctx> {
    pub fn try_new(ctx: &'ctx Context) -> Result<Self, Diagnostic> {
        let file = File::create(&ctx.out_path).map_err(|_| Diagnostic {
            line: -1,
            kind: DiagnosticKind::FailedOutOpen {
                path: ctx.out_path.to_owned(),
            },
        })?;
        let out = BufWriter::new(file);
        Ok(Self { ctx, out })
    }

    pub fn generate_output(&mut self, ast: &Program) {
        for stmt in &ast.top {
            match self.gen_statement(stmt) {
                Ok(_) => {}
                Err(diag) => {
                    self.ctx.diags.borrow_mut().report(diag);
                    return;
                }
            }
        }

        if self.out.flush().is_err() {
            self.ctx.diags.borrow_mut().report(Diagnostic {
                line: -1,
                kind: DiagnosticKind::WriteErr,
            });
        }
    }

    // TODO: Again.... restructure this to avoid passing individual fields in
    // instead of passing the data in the matched enum
    // we should match and then pass the whole node into the function ideally
    fn gen_statement(&mut self, stmt: &Stmt) -> Result<(), Diagnostic> {
        match &stmt.kind {
            StmtKind::FuncDecl(info) => self.gen_func(info),
            _ => todo!(),
        }
    }

    fn gen_func(&mut self, decl_info: &FuncDeclInfo) -> Result<(), Diagnostic> {
        let emitted_name = self.mangle(decl_info.id.unwrap());
        self.emit_label(&emitted_name, LabelKind::Normal)?;
        self.emit_instr("push rbp")?;
        self.emit_instr("mov rbp, rsp")?;
        self.emit_instr("pop rbp")?;
        self.emit_instr("ret")?;
        Ok(())
    }

    // TODO: Better mangling logic than whatever this is
    fn mangle(&self, id: SymbolID) -> String {
        format!("_crsnt_f{}", *id)
    }

    fn emit_label(&mut self, label: &str, kind: LabelKind) -> Result<(), Diagnostic> {
        match kind {
            LabelKind::Normal => self.emit(&format!("{label}:")),
            LabelKind::_Hidden => self.emit(&format!(".L{label}:")),
        }
    }

    fn emit_instr(&mut self, instr: &str) -> Result<(), Diagnostic> {
        self.emit(&format!("    {instr}"))
    }

    fn emit(&mut self, line: &str) -> Result<(), Diagnostic> {
        writeln!(self.out, "{line}").map_err(|_| Diagnostic {
            line: -1,
            kind: DiagnosticKind::WriteErr,
        })
    }

    fn _symbols(&self) -> Ref<'ctx, Symbols> {
        self.ctx.symbols.borrow()
    }
}

enum LabelKind {
    Normal,
    _Hidden,
}
