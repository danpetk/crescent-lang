use std::error::Error;
use crate::{lexer::Lexer};


pub struct Compiler {

}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler{}
    }

    pub fn compile(&mut self, source: &str) -> Result<(), Vec<Box<dyn Error>>> {
        
        let mut lexer = Lexer::new(&source);
        let mut _token_stream = match lexer.tokenize() {
            Ok(stream) => stream,
            Err(errors) => {
                return Err(errors.into_iter().map(|e| Box::<dyn Error>::from(e)).collect());
            }
        };

        Ok(())    }
}