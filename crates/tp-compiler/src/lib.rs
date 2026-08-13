//! TP compiler library.

mod diagnostic;
mod pipeline;
mod source;

pub use diagnostic::{render_diagnostic, Diagnostic, Label, Severity};
pub use pipeline::{CompileReport, Compiler};
pub use source::{LineCol, SourceFile, SourceId, Span};
