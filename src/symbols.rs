use std::collections::HashMap;
use crate::tokens::Token;
use crate::error::ParserError;

pub struct VarInfo {

}
#[derive(Debug)]
pub struct Symbol(usize);

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
        self.scopes.pop().expect("should always be paired with push_scope");
    }

    pub fn add_var(
        &mut self, 
        var_token: &Token, 
        var_ident: &str, 
        type_ident: &str
    ) -> Result<Symbol, ParserError> {
        todo!()
    }
}
