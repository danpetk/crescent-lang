use crate::diagnostic::{Diagnostic, DiagnosticKind};
use crate::parser::ParsedType;
use crate::tokens::Token;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct SymbolID(pub usize);

// May seem bare-bones or unnecessary now but its future proofing
#[derive(Debug, Clone)]
pub enum GenericType<T> {
    Named(T),
}

type ResolvedType = GenericType<SymbolID>;

pub enum TypeDefInfo {
    Primative,
}

pub struct VarInfo {
    _ty: ResolvedType,
}

pub struct FuncInfo {
    _return_ty: ResolvedType,
    _params: Vec<SymbolID>,
}

pub enum SymbolKind {
    Var(VarInfo),
    Func(FuncInfo),
    Type(TypeDefInfo),
}

pub struct SymbolInfo {
    line: i32,
    kind: SymbolKind,
}

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
        symbols.register_primative("i32");

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

    pub fn add_local_var(
        &mut self,
        var_token: &Token,
        ty: &ParsedType,
    ) -> Result<SymbolID, Diagnostic> {
        let ParsedType::Named(type_token) = ty;
        let type_id = self.get_type_id(type_token)?;

        let symbol = self.add_symbol(
            &var_token,
            SymbolInfo {
                line: var_token.line,
                kind: SymbolKind::Var(VarInfo {
                    _ty: ResolvedType::Named(type_id),
                }),
            },
        )?;

        Ok(symbol)
    }

    pub fn add_local_func(
        &mut self,
        func_token: &Token,
        ty: &ParsedType,
        params: Vec<SymbolID>,
    ) -> Result<SymbolID, Diagnostic> {
        let ParsedType::Named(type_token) = ty;
        let return_id = self.get_type_id(type_token)?;

        let symbol = self.add_symbol(
            &func_token,
            SymbolInfo {
                line: func_token.line,
                kind: SymbolKind::Func(FuncInfo {
                    _return_ty: ResolvedType::Named(return_id),
                    _params: params,
                }),
            },
        )?;

        Ok(symbol)
    }

    pub fn get_local_var_id(&self, var_token: &Token) -> Result<SymbolID, Diagnostic> {
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

    fn get_current_scope_mut(&mut self) -> &mut HashMap<String, SymbolID> {
        self.scopes.last_mut().expect("global scope must exist")
    }

    fn get_current_scope(&self) -> &HashMap<String, SymbolID> {
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
        Some((symbol, &self.symbols[symbol.0]))
    }

    fn add_symbol(&mut self, token: &Token, info: SymbolInfo) -> Result<SymbolID, Diagnostic> {
        if let Some(sym) = self.get_current_scope().get(&token.lexeme) {
            return Err(Diagnostic {
                line: token.line,
                kind: DiagnosticKind::IdentRedeclared {
                    original_line: self.symbols[sym.0].line,
                    var_name: token.lexeme.to_owned(),
                },
            });
        }

        let symbol = self.make_symbol_id();
        self.get_current_scope_mut()
            .insert(token.lexeme.to_owned(), symbol);
        self.symbols.push(info);
        Ok(symbol)
    }

    fn make_symbol_id(&self) -> SymbolID {
        SymbolID(self.symbols.len())
    }

    fn register_primative(&mut self, name: &str) {
        let symbol = self.make_symbol_id();
        let current_scope = self.get_current_scope_mut();

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
