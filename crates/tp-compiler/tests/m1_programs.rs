use std::path::PathBuf;

use tp_compiler::{Compiler, RunFailure};

#[test]
fn hello_program_prints() {
    let report = Compiler::new()
        .run_source("hello.tp", "fn main() { print(\"Hello from TP\"); }")
        .expect("hello program should run");
    assert_eq!(report.output, "Hello from TP\n");
}

#[test]
fn arithmetic_functions_mutation_if_and_while_work_together() {
    let source = r#"
        fn step(x: i64) -> i64 { x + 1 }
        fn main() -> i64 {
            var x = 0;
            while x < 40 {
                x = step(x);
            }
            if x == 40 { x + 2 } else { 0 }
        }
    "#;
    let report = Compiler::new()
        .run_source("control.tp", source)
        .expect("control-flow program should run");
    assert_eq!(report.value.as_i64(), Some(42));
}

#[test]
fn strings_structs_enums_match_and_option_work_together() {
    let source = r#"
        struct User { id: i64, name: String }
        enum Option<T> { Some(T), None }

        fn name_or_default(value: Option<User>) -> String {
            match value {
                Some(user) => user.name,
                None => "missing"
            }
        }

        fn main() -> String {
            let user = User { id: 42, name: "Tim" };
            name_or_default(Some(user))
        }
    "#;
    let report = Compiler::new()
        .run_source("data.tp", source)
        .expect("data program should run");
    assert_eq!(report.value.as_string(), Some("Tim"));
}

#[test]
fn result_generic_type_checks() {
    let source = r#"
        enum Result<T, E> { Ok(T), Err(E) }
        fn success() -> Result<i64, String> { Ok(42) }
        fn failure() -> Result<i64, String> { Err("failed") }
        fn main() { success(); failure(); }
    "#;
    let report = Compiler::new().check_source("result.tp", source);
    assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
}

#[test]
fn sibling_imports_are_part_of_m1_conformance() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("modules_main.tp");
    let report = Compiler::new()
        .run_path(&path)
        .expect("module program should run");
    assert_eq!(report.value.as_i64(), Some(42));
}

#[test]
fn lexical_parse_type_and_runtime_failures_are_reported() {
    let compiler = Compiler::new();

    let lexical = compiler.check_source("lexical.tp", "fn main() { @ }");
    assert!(lexical.diagnostics.iter().any(|d| d.code == "TP-E0001"));

    let parse = compiler.check_source("parse.tp", "fn main( { }");
    assert!(parse.diagnostics.iter().any(|d| d.code == "TP-E0100"));

    let type_error = compiler.check_source("type.tp", "fn main() { if 1 { } }");
    assert!(type_error.diagnostics.iter().any(|d| d.code == "TP-E0300"));

    let runtime = compiler
        .run_source("runtime.tp", "fn main() -> i64 { 1 / 0 }")
        .expect_err("division by zero must trap");
    assert!(matches!(
        runtime,
        RunFailure::Runtime(error) if error.code == "TP-E0500"
    ));
}
