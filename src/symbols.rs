use std::collections::HashMap;
use crate::tokens::Token;
use crate::error::ParserError;

pub struct VarInfo {
    line: i32
}

#[derive(Debug, Clone, Copy)]
pub struct Symbol(usize);

impl Symbol {
    pub fn gg() -> Self {
        Symbol{0:0}
    }
}

#[derive(Default)]
pub struct Symbols {
    scopes: Vec<HashMap<String, Symbol>>, // TODO: Change this to intered id when strings are interned
    variables: Vec<VarInfo>
}

impl Symbols {
    pub fn new() -> Symbols {
        Symbols::default()
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop().expect("pop_scope should always be paired with push_scope");
    }

    pub fn add_local_var(
        &mut self, 
        var_token: &Token, 
        type_token: &Token
    ) -> Result<Symbol, ParserError> {
        println!("{:?}", self.scopes);
        let current_scope = self.scopes.last_mut().expect("can only add local var in a scope");

        if let Some(var) = current_scope.get(&var_token.lexeme) {
            return Err(ParserError::VarRedeclared { line_redec: var_token.line, line_orig: self.variables[var.0].line, var_name: var_token.lexeme.to_owned() })
        }

        let symbol = Symbol(self.variables.len());

        current_scope.insert(var_token.lexeme.to_owned(), symbol);
        self.variables.push(VarInfo { line: var_token.line });

        if type_token.lexeme != "i32" {
            panic!("only i32 suppored rn")
        }

        Ok(symbol)
    }


    pub fn get_local_var(
        &self,
        var_token: &Token, 
        var_ident: &str, 
    ) -> Result<Symbol, ParserError> {
        let symbol = self.scopes.iter().rev().find_map(|map| {
            map.get(var_ident)
        });

        match symbol {
            Some(&s) => Ok(s),
            None => Err(ParserError::VarUnknown { line: var_token.line, var_name: var_ident.to_string() })
        }
    }
}


