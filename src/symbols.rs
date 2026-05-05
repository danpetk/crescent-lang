use crate::diagnostic::{Diagnostic, DiagnosticKind};
use crate::parser::ParsedType;
use crate::tokens::Token;
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SymbolID(usize);

impl Deref for SymbolID {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// May seem bare-bones or unnecessary now but its future proofing
#[derive(Debug, Clone)]
pub enum GenericType<T> {
    Named(T),
}

type ResolvedType = GenericType<SymbolID>;

#[derive(Debug)]
pub enum TypeDefInfo {
    Primative,
}

#[derive(Debug)]
pub struct VarInfo {
    pub _ty: ResolvedType,
    pub offset: i64,
}

#[derive(Debug)]
pub struct FuncInfo {
    pub _return_ty: ResolvedType,
    pub params: Vec<SymbolID>,
    pub stack_size: usize,
}

#[derive(Debug)]
pub enum SymbolKind {
    Var(VarInfo),
    Func(FuncInfo),
    Type(TypeDefInfo),
}

#[derive(Debug)]
pub struct SymbolInfo {
    line: i32,
    kind: SymbolKind,
}

#[derive(Debug)]
pub struct Symbols {
    scopes: Vec<HashMap<String, SymbolID>>, // TODO: Change this to intered id when strings are interned
    symbols: Vec<SymbolInfo>,
}

impl Symbols {
    pub fn new() -> Self {
        let mut symbols = Self {
            scopes: vec![],
            symbols: vec![],
        };

        symbols.push_scope();
        symbols.register_primative("i64");

        symbols
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes
            .pop()
            .expect("pop_scope should always be paired with push_scope");
    }

    // TODO: Decouple sema and stack offet stuff?
    pub fn register_var(
        &mut self,
        var_token: &Token,
        ty: &ParsedType,
        func_id: SymbolID,
    ) -> Result<SymbolID, Diagnostic> {
        let ParsedType::Named(type_token) = ty;
        let type_id = self.get_type_id(type_token)?;

        let offset = self.increase_stack_size(func_id, 8);
        let symbol = self.add_symbol(
            &var_token,
            SymbolInfo {
                line: var_token.line,
                kind: SymbolKind::Var(VarInfo {
                    _ty: ResolvedType::Named(type_id),
                    offset,
                }),
            },
        )?;

        Ok(symbol)
    }

    pub fn register_param(
        &mut self,
        var_token: &Token,
        ty: &ParsedType,
        func_id: SymbolID,
    ) -> Result<SymbolID, Diagnostic> {
        let ParsedType::Named(type_token) = ty;
        let type_id = self.get_type_id(type_token)?;

        let current_param_num = self.func_info_mut(func_id).params.len();

        let symbol = if current_param_num < 6 {
            self.register_var(var_token, ty, func_id)?
        } else {
            let offset = current_param_num as i64 * 8 + 16;
            self.add_symbol(
                &var_token,
                SymbolInfo {
                    line: var_token.line,
                    kind: SymbolKind::Var(VarInfo {
                        _ty: ResolvedType::Named(type_id),
                        offset,
                    }),
                },
            )?
        };

        self.func_info_mut(func_id).params.push(symbol);
        Ok(symbol)
    }

    pub fn register_func(
        &mut self,
        func_token: &Token,
        ty: &ParsedType,
    ) -> Result<SymbolID, Diagnostic> {
        let ParsedType::Named(type_token) = ty;
        let return_id = self.get_type_id(type_token)?;

        let symbol = self.add_symbol(
            &func_token,
            SymbolInfo {
                line: func_token.line,
                kind: SymbolKind::Func(FuncInfo {
                    _return_ty: ResolvedType::Named(return_id),
                    params: vec![],
                    stack_size: 0,
                }),
            },
        )?;

        return Ok(symbol);
    }

    pub fn get_var_id(&self, var_token: &Token) -> Result<SymbolID, Diagnostic> {
        match self.get_symbol(&var_token.lexeme) {
            Some((id, info)) if matches!(info.kind, SymbolKind::Var(_)) => Ok(id),
            _ => Err(Diagnostic {
                line: var_token.line,
                kind: DiagnosticKind::VarUnknown {
                    var_name: var_token.lexeme.to_owned(),
                },
            }),
        }
    }

    pub fn get_type_id(&self, type_token: &Token) -> Result<SymbolID, Diagnostic> {
        match self.get_symbol(&type_token.lexeme) {
            Some((id, info)) if matches!(info.kind, SymbolKind::Type(_)) => Ok(id),
            _ => Err(Diagnostic {
                line: type_token.line,
                kind: DiagnosticKind::TypeUnknown {
                    type_name: type_token.lexeme.to_owned(),
                },
            }),
        }
    }

    pub fn get_func_id(&self, func_token: &Token) -> Result<SymbolID, Diagnostic> {
        match self.get_symbol(&func_token.lexeme) {
            Some((id, info)) if matches!(info.kind, SymbolKind::Func(_)) => Ok(id),
            _ => Err(Diagnostic {
                line: func_token.line,
                kind: DiagnosticKind::FuncUnknown {
                    func_name: func_token.lexeme.to_owned(),
                },
            }),
        }
    }

    pub fn get_main_id(&self) -> Option<SymbolID> {
        match self.get_symbol_id("main") {
            Some(id) => match &self.symbols[*id].kind {
                SymbolKind::Func(_) => Some(id),
                _ => None,
            },
            None => None,
        }
    }

    pub fn increase_stack_size(&mut self, id: SymbolID, size: usize) -> i64 {
        let stack_size = &mut self.func_info_mut(id).stack_size;
        *stack_size += size;
        -(*stack_size as i64)
    }

    pub fn func_info(&self, id: SymbolID) -> &FuncInfo {
        match &self.symbols[*id].kind {
            SymbolKind::Func(info) => info,
            _ => panic!("expected symbol to be function"),
        }
    }

    pub fn var_info(&self, id: SymbolID) -> &VarInfo {
        match &self.symbols[*id].kind {
            SymbolKind::Var(info) => info,
            _ => panic!("expected symbol to be var"),
        }
    }

    fn func_info_mut(&mut self, id: SymbolID) -> &mut FuncInfo {
        match &mut self.symbols[*id].kind {
            SymbolKind::Func(info) => info,
            _ => panic!("expected symbol to be function"),
        }
    }

    fn current_scope_mut(&mut self) -> &mut HashMap<String, SymbolID> {
        self.scopes.last_mut().expect("global scope must exist")
    }

    fn current_scope(&self) -> &HashMap<String, SymbolID> {
        self.scopes.last().expect("global scope must exist")
    }

    fn get_symbol_id(&self, name: &str) -> Option<SymbolID> {
        self.scopes
            .iter()
            .rev()
            .find_map(|map| map.get(name))
            .copied()
    }

    fn get_symbol(&self, name: &str) -> Option<(SymbolID, &SymbolInfo)> {
        let symbol = self.get_symbol_id(name)?;
        Some((symbol, &self.symbols[*symbol]))
    }

    fn add_symbol(&mut self, token: &Token, info: SymbolInfo) -> Result<SymbolID, Diagnostic> {
        if let Some(sym) = self.current_scope().get(&token.lexeme) {
            return Err(Diagnostic {
                line: token.line,
                kind: DiagnosticKind::IdentRedeclared {
                    original_line: self.symbols[**sym].line,
                    var_name: token.lexeme.to_owned(),
                },
            });
        }

        let symbol = self.make_symbol_id();
        self.current_scope_mut()
            .insert(token.lexeme.to_owned(), symbol);
        self.symbols.push(info);
        Ok(symbol)
    }

    fn make_symbol_id(&self) -> SymbolID {
        SymbolID(self.symbols.len())
    }

    fn register_primative(&mut self, name: &str) {
        let symbol = self.make_symbol_id();
        let current_scope = self.current_scope_mut();

        if let Some(_) = current_scope.get(name) {
            panic!("registering duplicate primatives")
        }

        current_scope.insert(name.to_owned(), symbol);
        self.symbols.push(SymbolInfo {
            line: -1,
            kind: SymbolKind::Type(TypeDefInfo::Primative),
        });
    }
}
