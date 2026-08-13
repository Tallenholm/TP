use crate::{Diagnostic, Lexer, Parser, SourceFile, TypeChecker};

#[derive(Debug, Default)]
pub struct Compiler;

impl Compiler {
    pub fn new() -> Self {
        Self
    }

    pub fn check_source(&self, name: &str, source: &str) -> CompileReport {
        let source_file = SourceFile::new(name, source);
        let lexed = Lexer::new(&source_file).lex();
        let mut diagnostics = lexed.diagnostics;

        let parsed = Parser::new(&source_file, lexed.tokens).parse_module();
        diagnostics.extend(parsed.diagnostics);

        if diagnostics.is_empty() {
            if let Some(module) = parsed.module {
                diagnostics.extend(TypeChecker::check_module(&module).diagnostics);
            }
        }

        CompileReport { diagnostics }
    }
}

#[derive(Debug, Default)]
pub struct CompileReport {
    pub diagnostics: Vec<Diagnostic>,
}
