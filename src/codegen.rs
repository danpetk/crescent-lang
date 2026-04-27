use std::{
    cell::Ref,
    fmt,
    fs::File,
    io::{BufWriter, Write},
};

use crate::{
    ast::{BinOpInfo, BinOpKind, Expr, ExprKind, UnOpKind, VarDeclInfo},
    symbols::{SymbolID, Symbols},
};

use crate::{
    ast::{FuncDeclInfo, Program, Stmt, StmtKind},
    compiler::Context,
    diagnostic::{Diagnostic, DiagnosticKind},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Register {
    Rax,
    Rbx,
    Rcx,
    Rdx,
    Rsi,
    Rdi,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Register::Rax => "%rax",
            Register::Rbx => "%rbx",
            Register::Rcx => "%rcx",
            Register::Rdx => "%rdx",
            Register::Rsi => "%rsi",
            Register::Rdi => "%rdi",
            Register::R8 => "%r8",
            Register::R9 => "%r9",
            Register::R10 => "%r10",
            Register::R11 => "%r11",
            Register::R12 => "%r12",
            Register::R13 => "%r13",
            Register::R14 => "%r14",
            Register::R15 => "%r15",
        };
        write!(f, "{s}")
    }
}

// TODO: Restructure registers to avoid needing to do this
impl Register {
    pub fn to_8bit(&self) -> &'static str {
        match self {
            Register::Rax => "%al",
            Register::Rbx => "%bl",
            Register::Rcx => "%cl",
            Register::Rdx => "%dl",
            Register::Rsi => "%sil",
            Register::Rdi => "%dil",
            Register::R8 => "%r8b",
            Register::R9 => "%r9b",
            Register::R10 => "%r10b",
            Register::R11 => "%r11b",
            Register::R12 => "%r12b",
            Register::R13 => "%r13b",
            Register::R14 => "%r14b",
            Register::R15 => "%r15b",
        }
    }
}

struct RegAlloc {
    free: Vec<Register>,
}

impl RegAlloc {
    pub fn new() -> Self {
        Self {
            free: vec![
                Register::Rax,
                Register::Rbx,
                Register::Rcx,
                Register::Rdx,
                Register::Rsi,
                Register::Rdi,
                Register::R8,
                Register::R9,
                Register::R10,
                Register::R11,
                Register::R12,
                Register::R13,
                Register::R14,
                Register::R15,
            ],
        }
    }

    pub fn alloc(&mut self) -> Register {
        let reg = self
            .free
            .pop()
            .expect("register must be freed before alloc");

        reg
    }

    pub fn free(&mut self, reg: Register) {
        self.free.push(reg);
    }
}

pub struct Codegen<'ctx> {
    ctx: &'ctx Context,
    out: BufWriter<File>,
    ra: RegAlloc,
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
        Ok(Self {
            ctx,
            out,
            ra: RegAlloc::new(),
        })
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

        let note = "\n# comply with g++ warning\n.section .note.GNU-stack,\"\",@progbits";
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
            StmtKind::Block(stmts) => self.gen_block(stmts),
            StmtKind::VarDecl(info) => self.gen_var_decl(info),
            StmtKind::ExprStmt(expr) => {
                let reg = self.gen_expr(expr)?;
                self.ra.free(reg);
                Ok(())
            }
            _ => todo!("stmt"),
        }
    }

    fn gen_func(&mut self, decl_info: &FuncDeclInfo) -> Result<(), Diagnostic> {
        self.ra = RegAlloc::new(); // Reset allocater for function
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
        self.emit_blank()?;

        self.gen_statement(&decl_info.body)?;

        self.emit_blank()?;
        self.emit_instr(&format!("addq ${stack_size}, %rsp"))?;
        self.emit_instr("popq %rbp")?;
        self.emit_instr("ret")?;
        Ok(())
    }

    fn gen_block(&mut self, stmts: &Vec<Stmt>) -> Result<(), Diagnostic> {
        for stmt in stmts {
            self.gen_statement(stmt)?;
            self.emit_blank()?;
        }
        Ok(())
    }

    fn gen_var_decl(&mut self, info: &VarDeclInfo) -> Result<(), Diagnostic> {
        let var_id = info.id.unwrap();
        let expr = &info.expr;
        let cr = self.gen_expr(expr)?;
        let store_offset = self.symbols().var_info(var_id).stack_offset + 8;

        self.emit_instr(&format!("movq {cr}, -{store_offset}(%rbp)"))?;

        self.ra.free(cr);
        Ok(())
    }

    fn gen_expr(&mut self, expr: &Expr) -> Result<Register, Diagnostic> {
        match &expr.kind {
            ExprKind::Literal(val) => self.gen_expr_literal(*val),
            ExprKind::Var(id) => self.gen_expr_var(id.unwrap()),
            ExprKind::UnOp(op, expr) => self.gen_expr_unop(*op, expr),
            ExprKind::BinOp(info) => self.gen_expr_binop(info),
        }
    }

    fn gen_expr_literal(&mut self, val: i64) -> Result<Register, Diagnostic> {
        let r = self.ra.alloc();
        self.emit_instr(&format!("movq ${val}, {r}"))?;
        Ok(r)
    }

    fn gen_expr_var(&mut self, id: SymbolID) -> Result<Register, Diagnostic> {
        let load_offset = self.symbols().var_info(id).stack_offset + 8;
        let r = self.ra.alloc();
        self.emit_instr(&format!("movq -{load_offset}(%rbp), {r}"))?;
        Ok(r)
    }

    fn gen_expr_unop(&mut self, op: UnOpKind, expr: &Expr) -> Result<Register, Diagnostic> {
        let cr = self.gen_expr(expr)?;
        match op {
            UnOpKind::Neg => self.emit_instr(&format!("negq {cr}"))?,
            UnOpKind::Not => {
                self.emit_instr(&format!("testq {cr}, {cr}"))?;
                self.emit_instr(&format!("sete {}", cr.to_8bit()))?;
                self.emit_instr(&format!("movezbq {}, {cr}", cr.to_8bit()))?;
            }
        };
        Ok(cr)
    }

    fn gen_expr_binop(&mut self, info: &BinOpInfo) -> Result<Register, Diagnostic> {
        let BinOpInfo { op, lhs, rhs } = info;

        // Explicitly handle assignment case
        if matches!(op, BinOpKind::Assign) {
            let ExprKind::Var(id) = lhs.kind else {
                panic!("gen assign w/o var");
            };

            let store_offset = self.symbols().var_info(id.unwrap()).stack_offset + 8;
            let cr = self.gen_expr(rhs)?;

            self.emit_instr(&format!("movq {cr}, -{store_offset}(%rbp)"))?;
            self.ra.free(cr);
            return Ok(cr);
        }

        let lhsr = self.gen_expr(lhs)?;
        let rhsr = self.gen_expr(rhs)?;

        let is_cmp = matches!(
            op,
            BinOpKind::Equals
                | BinOpKind::NotEquals
                | BinOpKind::LessThan
                | BinOpKind::GreaterThan
                | BinOpKind::LessEq
                | BinOpKind::GreaterEq
        );

        if is_cmp {
            self.emit_instr(&format!("cmpq {rhsr}, {lhsr}"))?;
        }

        let lhsr_8bit = lhsr.to_8bit();
        match op {
            BinOpKind::Assign => unreachable!(),
            BinOpKind::Add => self.emit_instr(&format!("addq {rhsr}, {lhsr}"))?,
            BinOpKind::Sub => self.emit_instr(&format!("subq {rhsr}, {lhsr}"))?,
            BinOpKind::Mult => self.emit_instr(&format!("imulq {rhsr}, {lhsr}"))?,
            BinOpKind::Div => {
                // TODO: Revisit with when register spilling is done
                self.emit_instr("pushq %rax")?;
                self.emit_instr("pushq %rdx")?;
                self.emit_instr(&format!("movq {lhsr}, %rax"))?;
                self.emit_instr("cqto")?;
                self.emit_instr(&format!("idivq {rhsr}"))?;
                self.emit_instr(&format!("movq %rax, {lhsr}"))?;
                self.emit_instr("popq %rdx")?;
                self.emit_instr("popq %rax")?;
            }
            BinOpKind::Equals => self.emit_instr(&format!("sete {lhsr_8bit}"))?,
            BinOpKind::NotEquals => self.emit_instr(&format!("setne {lhsr_8bit}"))?,
            BinOpKind::LessThan => self.emit_instr(&format!("setl {lhsr_8bit}"))?,
            BinOpKind::GreaterThan => self.emit_instr(&format!("setg {lhsr_8bit}"))?,
            BinOpKind::LessEq => self.emit_instr(&format!("setle {lhsr_8bit}"))?,
            BinOpKind::GreaterEq => self.emit_instr(&format!("setge {lhsr_8bit}"))?,
        };

        if is_cmp {
            self.emit_instr(&format!("movzbq {lhsr_8bit}, {lhsr}"))?;
        }

        self.ra.free(rhsr);
        Ok(lhsr)
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
