use std::error::Error;
use crate::{lexer::Lexer, parser::Parser};


pub struct Compiler {

}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler{}
    }

    pub fn compile(&mut self, source: &str) -> Result<(), Vec<Box<dyn Error>>> {
        
        let mut lexer = Lexer::new(&source);
        let token_stream = match lexer.tokenize() {
            Ok(stream) => stream,
            Err(errors) => {
                return Err(errors.into_iter().map(|e| Box::<dyn Error>::from(e)).collect());
            }
        };


        let mut parser = Parser::new(token_stream);
        let a = parser.get_token();
        let _b = parser.get_token();

        

        

        println!("{a:?}");



        Ok(())    }
}