use crate::{Diagnostic, HirLowerer, HirModule, Lexer, Module, Parser, SourceFile, TypeChecker};

#[derive(Debug, Default)]
pub struct Compiler;

impl Compiler {
    pub fn new() -> Self {
        Self
    }

    pub fn check_source(&self, name: &str, source: &str) -> CompileReport {
        match self.checked_module(name, source) {
            Ok(_) => CompileReport {
                diagnostics: Vec::new(),
            },
            Err(diagnostics) => CompileReport { diagnostics },
        }
    }

    pub fn lower_source(&self, name: &str, source: &str) -> Result<HirModule, Vec<Diagnostic>> {
        let module = self.checked_module(name, source)?;
        Ok(HirLowerer::lower(&module))
    }

    fn checked_module(&self, name: &str, source: &str) -> Result<Module, Vec<Diagnostic>> {
        let source_file = SourceFile::new(name, source);
        let lexed = Lexer::new(&source_file).lex();
        let mut diagnostics = lexed.diagnostics;

        let parsed = Parser::new(&source_file, lexed.tokens).parse_module();
        diagnostics.extend(parsed.diagnostics);
        let module = parsed.module.unwrap_or(Module { items: Vec::new() });

        if diagnostics.is_empty() {
            diagnostics.extend(TypeChecker::check_module(&module).diagnostics);
        }

        if diagnostics.is_empty() {
            Ok(module)
        } else {
            Err(diagnostics)
        }
    }
}

#[derive(Debug, Default)]
pub struct CompileReport {
    pub diagnostics: Vec<Diagnostic>,
}
