//! TP compiler library.

mod diagnostic;
mod lexer;
mod pipeline;
mod source;
mod token;

pub use diagnostic::{render_diagnostic, Diagnostic, Label, Severity};
pub use lexer::{LexResult, Lexer};
pub use pipeline::{CompileReport, Compiler};
pub use source::{LineCol, SourceFile, SourceId, Span};
pub use token::{Token, TokenKind};
