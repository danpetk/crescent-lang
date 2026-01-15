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
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop().expect("pop_scope should always be paired with push_scope");
    }

    pub fn add_local_var(
        &mut self, 
        var_token: &Token, 
        var_ident: &str, 
        type_ident: &str
    ) -> Result<Symbol, ParserError> {
        println!("{:?}", self.scopes);
        let current_scope = self.scopes.last_mut().expect("can only add local var in a scope");

        if let Some(var) = current_scope.get(var_ident) {
            return Err(ParserError::VarRedeclared { line_redec: var_token.line, line_orig: self.variables[var.0].line, var_name: var_ident.to_string() })
        }

        let symbol = Symbol(self.variables.len());

        current_scope.insert(var_ident.to_string(), symbol);
        self.variables.push(VarInfo { line: var_token.line });

        if type_ident != "i32" {
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


#[derive(Debug, Clone, Copy)]
pub struct InternId(usize);

#[derive(Default)]
pub struct Interner {
    interned_map: HashMap<String, InternId>,
    next_id: usize
}

// TODO Storing the string might be suboptimal, look back to fix?
impl Interner {
    pub fn intern_string(&mut self, ident: &str) -> InternId {
        match self.interned_map.get(ident) {
            Some(id) => id.to_owned(),
            None => {
                let id = InternId(self.next_id);
                self.interned_map.insert(ident.to_string(), id);
                self.next_id += 1;
                id
            }

        }
    }
}


