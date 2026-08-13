use crate::Diagnostic;

#[derive(Debug, Default)]
pub struct Compiler;

impl Compiler {
    pub fn new() -> Self {
        Self
    }

    pub fn check_source(&self, _name: &str, _source: &str) -> CompileReport {
        CompileReport {
            diagnostics: Vec::new(),
        }
    }
}

#[derive(Debug, Default)]
pub struct CompileReport {
    pub diagnostics: Vec<Diagnostic>,
}
