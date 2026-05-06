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
        BinOpInfo, BinOpKind, Expr, ExprKind, FuncCallInfo, IfInfo, ReturnInfo, UnOpInfo, UnOpKind,
        VarDeclInfo, WhileInfo,
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
//
#[derive(Debug)]
struct RegAlloc {
    free: Vec<Register>,
    in_use: VecDeque<Register>,
    spilled: HashMap<Register, Vec<i64>>,
    next_slot: i64,
    spill_activity: Vec<i64>,
}

impl RegAlloc {
    pub fn new(stack_size: usize) -> Self {
        Self {
            free: ALL_REGISTERS.to_vec(),
            in_use: VecDeque::new(),
            spilled: ALL_REGISTERS.iter().map(|&r| (r, vec![])).collect(),
            spill_activity: vec![],
            next_slot: -(stack_size as i64) - 8,
        }
    }

    pub fn alloc_any(&mut self, out: &mut BufWriter<File>) -> Result<Register, Diagnostic> {
        let victim = if self.free.is_empty() {
            self.in_use.front().unwrap()
        } else {
            self.free.last().unwrap()
        };

        Ok(self.alloc_reg(*victim, out)?)
    }

    pub fn alloc_reg(
        &mut self,
        reg: Register,
        out: &mut BufWriter<File>,
    ) -> Result<Register, Diagnostic> {
        if let Some(pos) = self.free.iter().position(|x| *x == reg) {
            self.free.remove(pos);
            self.in_use.push_back(reg);
            return Ok(reg);
        }

        let spill_slot = self.next_slot;
        if spill_slot % 16 == -8 {
            // Make more space for spillage, we dont reuse stack space :0
            self.emit_instr(&format!("subq $16, %rsp"), out)?;
            if let Some(slot) = self.spill_activity.last_mut() {
                *slot += 16;
            }
        }
        self.next_slot -= 8;

        self.shuffle_victim(reg);
        self.spilled.get_mut(&reg).unwrap().push(spill_slot);
        self.emit_instr(&format!("movq {reg}, {spill_slot}(%rbp)"), out)?;

        return Ok(reg);
    }

    pub fn free(&mut self, reg: Register, out: &mut BufWriter<File>) -> Result<(), Diagnostic> {
        if let Some(spill_slot) = self.spilled.get_mut(&reg).unwrap().pop() {
            self.emit_instr(&format!("movq {spill_slot}(%rbp), {reg}"), out)?;

            return Ok(());
        }

        // Here we expect this item to exist in the in_use vec, so we unwrap to enforce this
        // invariant
        let pos = self.in_use.iter().position(|&r| r == reg).unwrap();
        self.in_use.remove(pos);
        self.free.push(reg);
        Ok(())
    }

    pub fn save_registers(
        &mut self,
        dstr: Register,
        saved: &Vec<Register>,
        out: &mut BufWriter<File>,
    ) -> Result<(), Diagnostic> {
        self.spill_activity.push(0);

        for reg in saved {
            let reg_free = self.free.contains(&reg);
            if dstr != *reg && !reg_free {
                self.emit_instr(&format!("pushq {reg}"), out)?;
            }
        }

        Ok(())
    }

    pub fn load_registers(
        &mut self,
        dstr: Register,
        saved: &Vec<Register>,
        out: &mut BufWriter<File>,
    ) -> Result<(), Diagnostic> {
        // Any stack activity from spilling, we must readjust before we pop again
        let spilled = self.spill_activity.pop().unwrap();
        if spilled > 0 {
            self.emit_instr(&format!("addq ${spilled}, %rsp"), out)?
        }

        for reg in saved.into_iter().rev() {
            let reg_free = self.free.contains(&reg);
            if dstr != *reg && !reg_free {
                self.emit_instr(&format!("popq {reg}"), out)?;
            }
        }

        Ok(())
    }

    fn shuffle_victim(&mut self, reg: Register) {
        let pos = self.in_use.iter().position(|x| *x == reg).unwrap();
        self.in_use.remove(pos);
        self.in_use.push_back(reg);
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
            StmtKind::Return(info) => self.gen_return(info),
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
        if stack_size > 0 {
            self.emit_instr(&format!("subq ${stack_size}, %rsp"))?;
        }
        self.emit_blank()?;

        if !func_info.params.is_empty() {
            self.emit_instr("# Move register paramaters onto variable stack slot")?;
        }
        for (index, id) in func_info.params.iter().enumerate().take(6) {
            let reg = self.index_to_param_reg(index);

            let offset = self.symbols().var_info(*id).offset;
            self.emit_instr(&format!("movq {reg}, {offset}(%rbp)"))?;
        }

        self.emit_blank()?;
        self.gen_statement(&decl_info.body)?;

        self.emit_label(&self.epilogue_label(func_id))?;
        self.emit_instr("leave")?;
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
        let store_offset = self.symbols().var_info(var_id).offset;

        self.emit_instr(&format!("movq {cr}, {store_offset}(%rbp)"))?;

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

    fn gen_return(&mut self, info: &ReturnInfo) -> Result<(), Diagnostic> {
        let ReturnInfo { id, expr } = info;
        let cr = self.gen_expr(expr)?;
        self.emit_movq_reg(cr, Register::Rax)?;
        let label = self.epilogue_label(id.unwrap());
        self.emit_instr(&format!("jmp {label}"))?;
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
            ExprKind::Func(info) => self.gen_expr_func(info),
            ExprKind::UnOp(info) => self.gen_expr_unop(info),
            ExprKind::BinOp(info) => self.gen_expr_binop(info),
        }
    }

    fn gen_expr_literal(&mut self, val: i64) -> Result<Register, Diagnostic> {
        let r = self.ra.alloc_any(&mut self.out)?;
        self.emit_instr(&format!("movq ${val}, {r}"))?;
        Ok(r)
    }

    fn gen_expr_var(&mut self, id: SymbolID) -> Result<Register, Diagnostic> {
        let load_offset = self.symbols().var_info(id).offset;
        let r = self.ra.alloc_any(&mut self.out)?;
        self.emit_instr(&format!("movq {load_offset}(%rbp), {r}"))?;
        Ok(r)
    }

    // Do you still think you shouldnt use a real IR?
    fn gen_expr_func(&mut self, info: &FuncCallInfo) -> Result<Register, Diagnostic> {
        let FuncCallInfo { id, args } = info;
        let id = id.unwrap();

        self.emit_blank()?;
        self.emit_instr(&format!("# Calling function {}", self.mangle(id)))?;

        let r = self.ra.alloc_any(&mut self.out)?;

        let caller_saved = {
            use Register::*;
            vec![Rax, Rcx, Rdx, Rsi, Rdi, R8, R9, R10, R11]
        };
        println!("{:?}", self.ra);
        self.ra.save_registers(r, &caller_saved, &mut self.out)?;

        // left to right
        let mut arg_regs = vec![];
        for expr in args {
            arg_regs.push(self.gen_expr(expr)?)
        }

        const MAX_REGISTER_PARAMS: usize = 6;
        let mid = arg_regs.len().min(MAX_REGISTER_PARAMS);
        let (register_params, stack_params) = arg_regs.split_at(mid);

        let total_param_offset = if stack_params.len() % 2 == 0 {
            stack_params.len() * 8
        } else {
            // odd number of params we must pad
            self.emit_instr("subq $8, %rsp")?;
            stack_params.len() * 8 + 8
        };

        // now we handle the allocated registers in reverse
        for reg in stack_params.iter().rev() {
            self.emit_instr(&format!("pushq {reg}"))?;
            self.ra.free(*reg, &mut self.out)?;
        }

        // TODO: Having to push and pop like this really sucks but i dont have enough
        // trust in my allocater to break the cycles with a temporary reliably
        for (index, reg) in register_params.iter().enumerate().rev() {
            if *reg != self.index_to_param_reg(index) {
                self.emit_instr(&format!("pushq {reg}"))?;
            }
            self.ra.free(*reg, &mut self.out)?;
        }

        for i in 0..register_params.len() {
            let param_reg = self.index_to_param_reg(i);
            if register_params[i] != param_reg {
                self.emit_instr(&format!("popq {param_reg}"))?;
            }
        }

        self.emit_instr(&format!("call {}", self.mangle(id)))?;
        self.emit_movq_reg(Register::Rax, r)?;

        // clear all the stack params that we pushed
        if total_param_offset > 0 {
            self.emit_instr(&format!("addq ${total_param_offset}, %rsp"))?;
        }

        println!("{:?}", self.ra);
        self.ra.load_registers(r, &caller_saved, &mut self.out)?;

        self.emit_instr(&format!("# Done calling function {}", self.mangle(id)))?;
        self.emit_blank()?;

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

            let store_offset = self.symbols().var_info(id.unwrap()).offset;
            let cr = self.gen_expr(rhs)?;

            self.emit_instr(&format!("movq {cr}, {store_offset}(%rbp)"))?;
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
                self.ra
                    .save_registers(lhsr, &vec![Register::Rax, Register::Rdx], &mut self.out)?;

                self.emit_movq_reg(lhsr, Register::Rax)?;
                //self.emit_instr(&format!("movq {lhsr}, %rax"))?;
                self.emit_instr("cqto")?;
                self.emit_instr(&format!("idivq {rhsr}"))?;
                //self.emit_instr(&format!("movq %rax, {lhsr}"))?;
                self.emit_movq_reg(Register::Rax, lhsr)?;

                self.ra
                    .load_registers(lhsr, &vec![Register::Rax, Register::Rdx], &mut self.out)?;
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

    fn epilogue_label(&self, id: SymbolID) -> String {
        format!(".L{}_epilogue", self.mangle(id))
    }

    fn index_to_param_reg(&self, index: usize) -> Register {
        match index {
            0 => Register::Rdi,
            1 => Register::Rsi,
            2 => Register::Rdx,
            3 => Register::Rcx,
            4 => Register::R8,
            5 => Register::R9,
            _ => unreachable!(),
        }
    }

    fn emit_movq_reg(&mut self, src: Register, dst: Register) -> Result<(), Diagnostic> {
        if src != dst {
            self.emit_instr(&format!("movq {src}, {dst}"))?;
        }
        Ok(())
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
