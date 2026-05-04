// Alot of stuff in codegen is kind of sloppy and hardcoded
// I'm not proud of it and will perhaps fix it if i plan
// to expand

use std::{
    cell::Ref,
    collections::{HashMap, VecDeque},
    fmt,
    fs::File,
    io::{BufWriter, Write},
};

use crate::{
    ast::{
        BinOpInfo, BinOpKind, Expr, ExprKind, IfInfo, UnOpInfo, UnOpKind, VarDeclInfo, WhileInfo,
    },
    semantic::{IfID, LoopID},
    symbols::{SymbolID, Symbols},
};

use crate::{
    ast::{FuncDeclInfo, Program, Stmt, StmtKind},
    compiler::Context,
    diagnostic::{Diagnostic, DiagnosticKind},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

const ALL_REGISTERS: &[Register] = {
    use Register::*;
    &[
        R15, R14, R13, R12, R11, R10, R9, R8, Rdi, Rsi, Rdx, Rcx, Rbx, Rax,
    ]
};

// Let this entire structure be a lesson as to why you should use a real IR
// instead of just walking the tree directly when making a compiler
struct RegAlloc {
    free: Vec<Register>,
    in_use: VecDeque<Register>,
    spilled: HashMap<Register, Vec<usize>>,
    next_slot: usize,
}

impl RegAlloc {
    pub fn new(init_slot: usize) -> Self {
        Self {
            free: ALL_REGISTERS.to_vec(),
            in_use: VecDeque::new(),
            spilled: ALL_REGISTERS.iter().map(|&r| (r, vec![])).collect(),
            next_slot: init_slot,
        }
    }

    pub fn alloc(&mut self, out: &mut BufWriter<File>) -> Result<Register, Diagnostic> {
        if self.free.is_empty() {
            let spill_slot = self.next_slot;
            if spill_slot % 16 == 0 {
                // Make more space for spillage, we dont reuse stack space :0
                self.emit_instr(&format!("subq $16, %rsp"), out)?;
            }
            self.next_slot += 8;

            let victim = self.get_victim();
            self.spilled.get_mut(&victim).unwrap().push(spill_slot);
            let store_offset = spill_slot + 8;
            self.emit_instr(&format!("movq {victim}, -{store_offset}(%rbp)"), out)?;

            return Ok(victim);
        }

        let reg = self.free.pop().unwrap();
        self.in_use.push_back(reg);
        Ok(reg)
    }

    pub fn free(&mut self, reg: Register, out: &mut BufWriter<File>) -> Result<(), Diagnostic> {
        if let Some(spill_slot) = self.spilled.get_mut(&reg).unwrap().pop() {
            let load_offset = spill_slot + 8;
            self.emit_instr(&format!("movq -{load_offset}(%rbp), {reg}"), out)?;

            return Ok(());
        }

        // Here we expect this item to exist in the in_use vec, so we unwrap to enforce this
        // invariant
        let pos = self.in_use.iter().position(|&r| r == reg).unwrap();
        self.in_use.remove(pos);
        self.free.push(reg);
        Ok(())
    }

    pub fn push_div(&self, dstr: Register, out: &mut BufWriter<File>) -> Result<(), Diagnostic> {
        let rax_free: bool = self.free.contains(&Register::Rax);
        let rdx_free: bool = self.free.contains(&Register::Rdx);

        if dstr != Register::Rax && !rax_free {
            self.emit_instr("pushq %rax", out)?;
        }

        if dstr != Register::Rbx && !rdx_free {
            self.emit_instr("pushq %rdx", out)?;
        }

        Ok(())
    }

    pub fn pop_div(&self, dstr: Register, out: &mut BufWriter<File>) -> Result<(), Diagnostic> {
        let rax_free: bool = self.free.contains(&Register::Rax);
        let rdx_free: bool = self.free.contains(&Register::Rdx);

        if dstr != Register::Rbx && !rdx_free {
            self.emit_instr("popq %rdx", out)?;
        }

        if dstr != Register::Rax && !rax_free {
            self.emit_instr("popq %rax", out)?;
        }

        Ok(())
    }
    fn get_victim(&mut self) -> Register {
        // Shuffle to front
        let victim = self.in_use.pop_front().unwrap();
        self.in_use.push_back(victim);
        victim
    }

    fn emit_instr(&self, instr: &str, out: &mut BufWriter<File>) -> Result<(), Diagnostic> {
        self.emit(&format!("    {instr}"), out)
    }

    fn emit(&self, line: &str, out: &mut BufWriter<File>) -> Result<(), Diagnostic> {
        writeln!(out, "{line}").map_err(|_| Diagnostic {
            line: -1,
            kind: DiagnosticKind::WriteErr,
        })
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
            ra: RegAlloc::new(0),
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
            StmtKind::Empty => Ok(()),
            StmtKind::FuncDecl(info) => self.gen_func(info),
            StmtKind::Block(stmts) => self.gen_block(stmts),
            StmtKind::VarDecl(info) => self.gen_var_decl(info),
            StmtKind::If(info) => self.gen_if(info),
            StmtKind::While(info) => self.gen_while(info),
            StmtKind::Return(expr) => self.gen_return(expr),
            StmtKind::Continue(id) => self.gen_continue(id.unwrap()),
            StmtKind::Break(id) => self.gen_break(id.unwrap()),
            StmtKind::ExprStmt(expr) => {
                let reg = self.gen_expr(expr)?;
                self.ra.free(reg, &mut self.out)?;
                Ok(())
            }
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

        self.ra = RegAlloc::new(stack_size); // Reset allocater for function

        self.emit_blank()?;
        self.emit_label(&emitted_name)?;
        self.emit_instr("pushq %rbp")?;
        self.emit_instr("movq %rsp, %rbp")?;
        self.emit_instr(&format!("subq ${stack_size}, %rsp"))?;
        self.emit_blank()?;

        self.gen_statement(&decl_info.body)?;

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

        self.ra.free(cr, &mut self.out)?;
        Ok(())
    }

    fn gen_if(&mut self, info: &IfInfo) -> Result<(), Diagnostic> {
        let IfInfo {
            id,
            cond,
            do_if,
            do_else,
        } = info;
        let id = id.unwrap();

        let (if_else, if_end) = self.if_labels(id);

        let er = self.gen_expr(cond)?;

        self.emit_blank()?;
        self.emit_instr(&format!("testq {er}, {er}"))?;
        self.ra.free(er, &mut self.out)?;

        if do_else.is_some() {
            self.emit_instr(&format!("je {if_else}"))?;
            self.emit_blank()?;

            self.gen_statement(do_if)?;
            self.emit_instr(&format!("jmp {if_end}"))?;
            self.emit_blank()?;

            self.emit_label(&if_else)?;
            self.emit_blank()?;

            self.gen_statement(do_else.as_ref().unwrap())?;
        } else {
            self.emit_instr(&format!("je {if_end}"))?;
            self.gen_statement(do_if)?;
        }

        self.emit_label(&if_end)?;
        Ok(())
    }

    fn gen_while(&mut self, info: &WhileInfo) -> Result<(), Diagnostic> {
        let WhileInfo { id, cond, body } = info;
        let id = id.unwrap();

        let (loop_start, loop_end) = self.loop_labels(id);

        self.emit_label(&loop_start)?;
        self.emit_blank()?;

        let er = self.gen_expr(cond)?;
        self.emit_instr(&format!("testq {er}, {er}"))?;
        self.ra.free(er, &mut self.out)?;
        self.emit_instr(&format!("je {loop_end}"))?;
        self.emit_blank()?;

        self.gen_statement(body)?;

        self.emit_instr(&format!("jmp {loop_start}"))?;
        self.emit_blank()?;
        self.emit_label(&loop_end)?;

        Ok(())
    }

    fn gen_return(&mut self, expr: &Expr) -> Result<(), Diagnostic> {
        let cr = self.gen_expr(expr)?;
        self.emit_instr(&format!("movq {cr}, %rax"))?;
        self.emit_instr("leave")?;
        self.emit_instr("ret")?;
        self.ra.free(cr, &mut self.out)?;
        Ok(())
    }

    fn gen_continue(&mut self, id: LoopID) -> Result<(), Diagnostic> {
        let (loop_start, _) = self.loop_labels(id);
        self.emit_instr(&format!("jmp {loop_start}"))?;
        Ok(())
    }

    fn gen_break(&mut self, id: LoopID) -> Result<(), Diagnostic> {
        let (_, loop_end) = self.loop_labels(id);
        self.emit_instr(&format!("jmp {loop_end}"))?;
        Ok(())
    }

    fn gen_expr(&mut self, expr: &Expr) -> Result<Register, Diagnostic> {
        match &expr.kind {
            ExprKind::Literal(val) => self.gen_expr_literal(*val),
            ExprKind::Var(id) => self.gen_expr_var(id.unwrap()),
            ExprKind::UnOp(info) => self.gen_expr_unop(info),
            ExprKind::BinOp(info) => self.gen_expr_binop(info),
            _ => todo!(),
        }
    }

    fn gen_expr_literal(&mut self, val: i64) -> Result<Register, Diagnostic> {
        let r = self.ra.alloc(&mut self.out)?;
        self.emit_instr(&format!("movq ${val}, {r}"))?;
        Ok(r)
    }

    fn gen_expr_var(&mut self, id: SymbolID) -> Result<Register, Diagnostic> {
        let load_offset = self.symbols().var_info(id).stack_offset + 8;
        let r = self.ra.alloc(&mut self.out)?;
        self.emit_instr(&format!("movq -{load_offset}(%rbp), {r}"))?;
        Ok(r)
    }

    fn gen_expr_unop(&mut self, info: &UnOpInfo) -> Result<Register, Diagnostic> {
        let UnOpInfo { op, expr } = info;
        let cr = self.gen_expr(expr)?;
        match op {
            UnOpKind::Neg => self.emit_instr(&format!("negq {cr}"))?,
            UnOpKind::Not => {
                self.emit_instr(&format!("testq {cr}, {cr}"))?;
                self.emit_instr(&format!("sete {}", cr.to_8bit()))?;
                self.emit_instr(&format!("movzbq {}, {cr}", cr.to_8bit()))?;
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
                self.ra.push_div(lhsr, &mut self.out)?;

                self.emit_instr(&format!("movq {lhsr}, %rax"))?;
                self.emit_instr("cqto")?;
                self.emit_instr(&format!("idivq {rhsr}"))?;
                self.emit_instr(&format!("movq %rax, {lhsr}"))?;

                self.ra.pop_div(lhsr, &mut self.out)?;
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

        self.ra.free(rhsr, &mut self.out)?;
        Ok(lhsr)
    }

    // TODO: Better mangling logic than whatever this is
    fn mangle(&self, id: SymbolID) -> String {
        format!("_crsnt_f{}", *id)
    }

    fn loop_labels(&self, id: LoopID) -> (String, String) {
        (format!(".L{id}_start"), format!(".L{id}_end"))
    }

    fn if_labels(&self, id: IfID) -> (String, String) {
        (format!(".L{id}_else"), format!(".L{id}_end"))
    }

    fn emit_label(&mut self, label: &str) -> Result<(), Diagnostic> {
        self.emit(&format!("{label}:"))
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
