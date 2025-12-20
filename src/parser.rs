use crate::tokens::TokenStream;

pub struct Parser<'a> {
    token_stream: TokenStream<'a>
}

impl<'a> Parser<'a> {
    pub fn new(token_stream: TokenStream<'a>) -> Parser<'a> {
        Parser {
            token_stream
        }
    }

    pub fn get_token(&mut self) -> crate::tokens::Token<'a> {
        return self.token_stream.advance();
    }
}