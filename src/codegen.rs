use std::{fs::File, io::BufWriter};

use crate::{
    ast::Program,
    compiler::Context,
    diagnostic::{Diagnostic, DiagnosticKind},
};

pub struct Codegen<'ctx> {
    _ctx: &'ctx Context,
    _out: BufWriter<File>,
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
            _ctx: ctx,
            _out: out,
        })
    }

    pub fn generate_output(&mut self, _ast: &Program) {}
}
