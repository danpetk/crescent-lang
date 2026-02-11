use crate::tokens::TokenKind;
use std::fmt;

// Again, I'd like to avoid storing owned strings in the future
#[derive(Debug)]
pub enum DiagnosticKind {
    InvalidToken {
        lexeme: String,
    },
    UnexpectedToken {
        expected: TokenKind,
        found: TokenKind,
    },
    VarRedeclared {
        original_line: i32,
        var_name: String,
    },
    VarUnknown {
        var_name: String,
    },
}

impl fmt::Display for DiagnosticKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidToken { lexeme } => {
                write!(f, "Unexpected token in source file: '{lexeme}'")
            }
            Self::UnexpectedToken { expected, found } => {
                write!(f, "Expected token '{expected}', found '{found}'")
            }
            Self::VarRedeclared {
                original_line,
                var_name,
            } => {
                write!(
                    f,
                    "Variable '{var_name}' redeclared. (Orignally declared on line {original_line})"
                )
            }
            Self::VarUnknown { var_name } => {
                write!(f, "Unknown variable '{var_name}'")
            }
        }
    }
}

#[derive(Debug)]
pub struct Diagnostic {
    pub line: i32,
    pub kind: DiagnosticKind,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ERROR (line {}): {}", self.line, self.kind)
    }
}

#[derive(Debug, Default)]
pub struct Diagnostics {
    diagnostics: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn report(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn has_diagnostics(&self) -> bool {
        self.diagnostics.len() > 0
    }

    pub fn take_diagnostics(&mut self) -> Vec<Diagnostic> {
        std::mem::take(&mut self.diagnostics)
    }
}
