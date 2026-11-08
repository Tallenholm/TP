use std::path::Path;

use crate::{
    Diagnostic, HirLowerer, HirModule, Interpreter, Lexer, MirLowerer, MirModule, MirVerifier,
    Module, ModuleLoader, Parser, RuntimeError, SemanticModel, SourceFile, SourceManager,
    TypeChecker, Value,
};

#[derive(Debug, Default)]
pub struct Compiler;

impl Compiler {
    pub fn new() -> Self {
        Self
    }

    pub fn check_source(&self, name: &str, source: &str) -> CompileReport {
        let source_file = SourceFile::new(name, source);
        let mut sources = SourceManager::default();
        sources.insert(source_file.clone());

        let lexed = Lexer::new(&source_file).lex();
        let mut diagnostics = lexed.diagnostics;
        let parsed = Parser::new(&source_file, lexed.tokens).parse_module();
        diagnostics.extend(parsed.diagnostics);
        let module = parsed.module.unwrap_or(Module { items: Vec::new() });

        if diagnostics.is_empty() {
            diagnostics.extend(TypeChecker::check_module(&module).diagnostics);
        }

        CompileReport {
            diagnostics,
            sources,
        }
    }

    pub fn check_path(&self, path: &Path) -> CompileReport {
        let loaded = ModuleLoader::load_with_sources(path);
        let mut diagnostics = loaded.diagnostics;
        if diagnostics.is_empty() {
            if let Some(module) = loaded.module {
                diagnostics.extend(TypeChecker::check_module(&module).diagnostics);
            }
        }

        CompileReport {
            diagnostics,
            sources: loaded.sources,
        }
    }

    pub fn lower_source(&self, name: &str, source: &str) -> Result<HirModule, Vec<Diagnostic>> {
        let checked = self.checked_module(name, source)?;
        Ok(HirLowerer::lower(&checked.module, &checked.semantic))
    }

    pub fn lower_path(&self, path: &Path) -> Result<HirModule, Vec<Diagnostic>> {
        let checked = self.checked_path_module(path)?;
        Ok(HirLowerer::lower(&checked.module, &checked.semantic))
    }

    pub fn lower_mir_source(&self, name: &str, source: &str) -> Result<MirModule, Vec<Diagnostic>> {
        let hir = self.lower_source(name, source)?;
        let mir = MirLowerer::lower(&hir);
        MirVerifier::verify(&mir).map_err(mir_verification_diagnostics)?;
        Ok(mir)
    }

    pub fn lower_mir_path(&self, path: &Path) -> Result<MirModule, Vec<Diagnostic>> {
        let hir = self.lower_path(path)?;
        let mir = MirLowerer::lower(&hir);
        MirVerifier::verify(&mir).map_err(mir_verification_diagnostics)?;
        Ok(mir)
    }

    pub fn run_source(&self, name: &str, source: &str) -> Result<RunReport, RunFailure> {
        let mir = self
            .lower_mir_source(name, source)
            .map_err(RunFailure::Compile)?;
        self.run_mir(&mir)
    }

    pub fn run_path(&self, path: &Path) -> Result<RunReport, RunFailure> {
        let mir = self.lower_mir_path(path).map_err(RunFailure::Compile)?;
        self.run_mir(&mir)
    }

    fn run_mir(&self, mir: &MirModule) -> Result<RunReport, RunFailure> {
        let Some(main) = mir
            .functions
            .iter()
            .find(|function| function.name == "main")
        else {
            return Err(RunFailure::Compile(vec![Diagnostic::error(
                "TP-E0400",
                "missing executable entry point `main`",
            )]));
        };
        if !main.params.is_empty() {
            return Err(RunFailure::Compile(vec![Diagnostic::error(
                "TP-E0400",
                "entry point `main` must not take parameters",
            )]));
        }

        let mut interpreter = Interpreter::new(mir);
        let value = interpreter.run_main().map_err(RunFailure::Runtime)?;
        Ok(RunReport {
            value,
            output: interpreter.into_output(),
        })
    }

    fn checked_module(&self, name: &str, source: &str) -> Result<CheckedModule, Vec<Diagnostic>> {
        let source_file = SourceFile::new(name, source);
        let lexed = Lexer::new(&source_file).lex();
        let mut diagnostics = lexed.diagnostics;

        let parsed = Parser::new(&source_file, lexed.tokens).parse_module();
        diagnostics.extend(parsed.diagnostics);
        let module = parsed.module.unwrap_or(Module { items: Vec::new() });

        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }

        let checked = TypeChecker::check_module(&module);
        if checked.diagnostics.is_empty() {
            Ok(CheckedModule {
                module,
                semantic: checked.semantic,
            })
        } else {
            Err(checked.diagnostics)
        }
    }

    fn checked_path_module(&self, path: &Path) -> Result<CheckedModule, Vec<Diagnostic>> {
        let module = ModuleLoader::load(path)?;
        let checked = TypeChecker::check_module(&module);
        if checked.diagnostics.is_empty() {
            Ok(CheckedModule {
                module,
                semantic: checked.semantic,
            })
        } else {
            Err(checked.diagnostics)
        }
    }
}

struct CheckedModule {
    module: Module,
    semantic: SemanticModel,
}

#[derive(Debug, Default)]
pub struct CompileReport {
    pub diagnostics: Vec<Diagnostic>,
    pub sources: SourceManager,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunReport {
    pub value: Value,
    pub output: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RunFailure {
    Compile(Vec<Diagnostic>),
    Runtime(RuntimeError),
}

fn mir_verification_diagnostics(errors: Vec<crate::MirVerificationError>) -> Vec<Diagnostic> {
    errors
        .into_iter()
        .map(|error| {
            Diagnostic::error(
                "TP-E0600",
                format!(
                    "internal MIR verification failed in function `{}`: {}",
                    error.function, error.kind
                ),
            )
        })
        .collect()
}
