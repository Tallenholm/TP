//! TP compiler library.

mod pipeline;
mod source;

pub use pipeline::{CompileReport, Compiler};
pub use source::{LineCol, SourceFile, SourceId, Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
}
