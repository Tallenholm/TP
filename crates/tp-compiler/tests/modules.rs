use std::path::PathBuf;

use tp_compiler::{Compiler, Item, Lexer, Parser, RunFailure, SourceFile};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn parses_import_with_alias() {
    let source = SourceFile::new("main.tp", "import util as helpers; fn main() {}");
    let lexed = Lexer::new(&source).lex();
    assert!(lexed.diagnostics.is_empty());
    let result = Parser::new(&source, lexed.tokens).parse_module();
    assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
    let module = result.module.expect("module");
    assert!(matches!(
        &module.items[0],
        Item::Import(import) if import.module == "util" && import.alias.as_deref() == Some("helpers")
    ));
}

#[test]
fn imported_function_can_be_called() {
    let report = Compiler::new()
        .run_path(&fixture("modules_main.tp"))
        .expect("multi-file program should run");
    assert_eq!(report.value.as_i64(), Some(42));
}

#[test]
fn check_path_accepts_valid_multi_file_program() {
    let report = Compiler::new().check_path(&fixture("modules_main.tp"));
    assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
}

#[test]
fn import_cycles_are_rejected() {
    let error = Compiler::new()
        .run_path(&fixture("cycle_a.tp"))
        .expect_err("cycle must be rejected");
    let RunFailure::Compile(diagnostics) = error else {
        panic!("expected compile failure");
    };
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("import cycle")),
        "{:#?}",
        diagnostics
    );
}
