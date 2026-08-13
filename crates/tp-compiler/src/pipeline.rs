use std::path::Path;

use crate::{
    Diagnostic, HirLowerer, HirModule, Interpreter, Lexer, MirLowerer, MirModule, Module,
    ModuleLoader, Parser, RuntimeError, SourceFile, TypeChecker, Value,
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
        let module = self.checked_module(name, source)?;
        Ok(HirLowerer::lower(&module))
    }

    pub fn lower_path(&self, path: &Path) -> Result<HirModule, Vec<Diagnostic>> {
        let module = self.checked_path_module(path)?;
        Ok(HirLowerer::lower(&module))
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
        let mut interpreter = Interpreter::new(mir);
        let value = interpreter.run_main().map_err(RunFailure::Runtime)?;
        Ok(RunReport {
            value,
            output: interpreter.into_output(),
        })
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

    fn checked_path_module(&self, path: &Path) -> Result<Module, Vec<Diagnostic>> {
        let module = ModuleLoader::load(path)?;
        let diagnostics = TypeChecker::check_module(&module).diagnostics;
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
