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

#[test]
fn repeated_identical_import_is_loaded_once() {
    let path = fixture("modules_repeat_main.tp");
    let report = Compiler::new().check_path(&path);
    assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
    let util_sources = report
        .sources
        .iter()
        .filter(|source| source.name().ends_with("util.tp"))
        .count();
    assert_eq!(util_sources, 1);

    let run = Compiler::new()
        .run_path(&path)
        .expect("repeated import should not duplicate declarations");
    assert_eq!(run.value.as_i64(), Some(42));
}

#[test]
fn same_module_can_have_multiple_aliases_without_reloading() {
    let path = fixture("modules_two_aliases_main.tp");
    let report = Compiler::new().check_path(&path);
    assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
    let util_sources = report
        .sources
        .iter()
        .filter(|source| source.name().ends_with("util.tp"))
        .count();
    assert_eq!(util_sources, 1);

    let run = Compiler::new()
        .run_path(&path)
        .expect("aliases should resolve to the same module identity");
    assert_eq!(run.value.as_i64(), Some(42));
}

#[test]
fn duplicate_import_alias_is_rejected_at_import_boundary() {
    let report = Compiler::new().check_path(&fixture("modules_duplicate_alias_main.tp"));
    assert!(
        report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("duplicate import alias `helper`")),
        "{:#?}",
        report.diagnostics
    );
}

#[test]
fn diamond_import_loads_shared_module_once() {
    let path = fixture("diamond_main.tp");
    let report = Compiler::new().check_path(&path);
    assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
    let shared_sources = report
        .sources
        .iter()
        .filter(|source| source.name().ends_with("diamond_shared.tp"))
        .count();
    assert_eq!(shared_sources, 1);

    let run = Compiler::new()
        .run_path(&path)
        .expect("diamond import should execute");
    assert_eq!(run.value.as_i64(), Some(42));
}

#[test]
fn imported_nominal_types_struct_literals_and_enum_patterns_work() {
    let path = fixture("imported_types_main.tp");
    let report = Compiler::new().check_path(&path);
    assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);

    let run = Compiler::new()
        .run_path(&path)
        .expect("qualified imported nominal types should execute");
    assert_eq!(run.value.as_i64(), Some(42));
}
