//! TP compiler library.

mod pipeline;

pub use pipeline::{CompileReport, Compiler};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
}
