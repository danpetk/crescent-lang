use std::ops::Deref;

use crate::tokens::SourceSpan;

pub struct Source {
    src: String,
}

impl Source {
    pub fn new(src: String) -> Self {
        Source { src }
    }

    pub fn get_spanned(&self, span: &SourceSpan) -> &str {
        &self.src[span.low..span.high]
    }
}

impl Deref for Source {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.src
    }
}
