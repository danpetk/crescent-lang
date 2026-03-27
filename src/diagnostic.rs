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
    IdentRedeclared {
        original_line: i32,
        var_name: String,
    },
    VarUnknown {
        var_name: String,
    },
    TypeUnknown {
        type_name: String,
    },
    NumLiteralTooLarge {
        literal: String,
    },
    UnexpectedTokenInExpression {
        found: TokenKind,
    },
    ContinueOutsideLoop,
    BreakOutsideLoop,
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
            Self::IdentRedeclared {
                original_line,
                var_name,
            } => {
                write!(
                    f,
                    "Identifier '{var_name}' redeclared. (Orignally declared on line {original_line})"
                )
            }
            Self::VarUnknown { var_name } => {
                write!(f, "Unknown variable '{var_name}'")
            }
            Self::TypeUnknown { type_name } => {
                write!(f, "Unknown type '{type_name}'")
            }
            Self::NumLiteralTooLarge { literal } => {
                write!(f, "Number literal {literal} too large")
            }
            Self::UnexpectedTokenInExpression { found } => {
                write!(f, "Unexpected token '{found}' found within expression")
            }
            Self::ContinueOutsideLoop => {
                write!(f, "'continue' statement oustide of loop")
            }
            Self::BreakOutsideLoop => {
                write!(f, "'break' statement oustide of loop")
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
