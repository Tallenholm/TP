use std::path::Path;

use crate::{
    Diagnostic, HirLowerer, HirModule, Interpreter, Lexer, MirLowerer, MirModule, Module,
    ModuleLoader, Parser, RuntimeError, SemanticModel, SourceFile, TypeChecker, Value,
};

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

    pub fn check_path(&self, path: &Path) -> CompileReport {
        match self.checked_path_module(path) {
            Ok(_) => CompileReport {
                diagnostics: Vec::new(),
            },
            Err(diagnostics) => CompileReport { diagnostics },
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
        Ok(MirLowerer::lower(&hir))
    }

    pub fn lower_mir_path(&self, path: &Path) -> Result<MirModule, Vec<Diagnostic>> {
        let hir = self.lower_path(path)?;
        Ok(MirLowerer::lower(&hir))
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
