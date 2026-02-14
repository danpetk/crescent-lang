use crate::diagnostic::{Diagnostic, DiagnosticKind};
use crate::tokens::Token;
use std::collections::HashMap;

// May seem bare-bones or unnecessary now but its future proofing
pub enum Type {
    Named(SymbolID),
}

pub enum TypeDef {
    Primative,
}

pub struct VarInfo {
    _var_type: Type,
}

pub enum SymbolKind {
    Var(VarInfo),
    Type(TypeDef),
}

pub struct SymbolInfo {
    line: i32,
    kind: SymbolKind,
}

#[derive(Debug, Clone, Copy)]
pub struct SymbolID(usize);

pub struct Symbols {
    scopes: Vec<HashMap<String, SymbolID>>, // TODO: Change this to intered id when strings are interned
    variables: Vec<SymbolInfo>,
}

impl Symbols {
    pub fn new() -> Self {
        let mut symbols = Self {
            scopes: vec![],
            variables: vec![],
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
        type_token: &Token,
    ) -> Result<SymbolID, Diagnostic> {
        if let Some(sym) = self.get_current_scope().get(&var_token.lexeme) {
            return Err(Diagnostic {
                line: var_token.line,
                kind: DiagnosticKind::VarRedeclared {
                    original_line: self.variables[sym.0].line,
                    var_name: var_token.lexeme.to_owned(),
                },
            });
        }

        let type_id = self.get_type_id(type_token)?;

        let symbol = self.add_symbol(
            &var_token.lexeme,
            SymbolInfo {
                line: var_token.line,
                kind: SymbolKind::Var(VarInfo {
                    _var_type: Type::Named(type_id),
                }),
            },
        );

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
        Some((symbol, &self.variables[symbol.0]))
    }

    fn add_symbol(&mut self, name: &str, info: SymbolInfo) -> SymbolID {
        let symbol = self.make_symbol_id();
        self.get_current_scope_mut().insert(name.to_owned(), symbol);
        self.variables.push(info);
        symbol
    }

    fn make_symbol_id(&self) -> SymbolID {
        SymbolID(self.variables.len())
    }

    fn register_primative(&mut self, name: &str) {
        let symbol = self.make_symbol_id();
        let current_scope = self.get_current_scope_mut();

        if let Some(_) = current_scope.get(name) {
            panic!("registering duplicate primatives")
        }

        current_scope.insert(name.to_owned(), symbol);
        self.variables.push(SymbolInfo {
            line: -1,
            kind: SymbolKind::Type(TypeDef::Primative),
        });
    }
}
