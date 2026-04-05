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

    // TODO: Remove this ugly repetition and reporting
    pub fn generate_output(&mut self, ast: &Program) {
        if self.emit(".global main").is_err() {
            self.report_write_error();
        }

        for stmt in &ast.top {
            match self.gen_statement(stmt) {
                Ok(_) => {}
                Err(diag) => {
                    self.ctx.diags.borrow_mut().report(diag);
                    return;
                }
            }
        }

        let note = "\n; comply with g++ warning\n.section .note.GNU-stack,\"\",@progbits";
        if self.emit(note).is_err() {
            self.report_write_error();
        }

        if self.out.flush().is_err() {
            self.report_write_error();
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
        let func_id = decl_info.id.unwrap();
        let emitted_name = if func_id == self.symbols().get_main_id().unwrap() {
            "main".to_string()
        } else {
            self.mangle(decl_info.id.unwrap())
        };

        let symbols = self.symbols();
        let func_info = symbols.func_info(func_id);
        let stack_size = self.align_16(func_info.stack_size);

        self.emit_blank()?;
        self.emit_label(&emitted_name, LabelKind::Normal)?;
        self.emit_instr("pushq %rbp")?;
        self.emit_instr("movq %rsp, %rbp")?;
        self.emit_instr(&format!("subq ${stack_size}, %rsp"))?;
        self.emit_instr(&format!("addq ${stack_size}, %rsp"))?;
        self.emit_instr("popq %rbp")?;
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

    fn emit_blank(&mut self) -> Result<(), Diagnostic> {
        self.emit("")
    }

    fn emit(&mut self, line: &str) -> Result<(), Diagnostic> {
        writeln!(self.out, "{line}").map_err(|_| Diagnostic {
            line: -1,
            kind: DiagnosticKind::WriteErr,
        })
    }

    fn align_16(&self, x: usize) -> usize {
        (x + 15) & !15
    }

    fn report_write_error(&self) {
        self.ctx.diags.borrow_mut().report(Diagnostic {
            line: -1,
            kind: DiagnosticKind::WriteErr,
        });
    }

    fn symbols(&self) -> Ref<'ctx, Symbols> {
        self.ctx.symbols.borrow()
    }
}

enum LabelKind {
    Normal,
    _Hidden,
}
