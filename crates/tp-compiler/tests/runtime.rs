use tp_compiler::{Compiler, RunFailure};

#[test]
fn executes_arithmetic_program() {
    let report = Compiler::new()
        .run_source("main.tp", "fn main() -> i64 { let x = 6; x * 7 }")
        .expect("program should run");
    assert_eq!(report.value.as_i64(), Some(42));
}

#[test]
fn executes_function_calls_and_loops() {
    let report = Compiler::new()
        .run_source(
            "main.tp",
            "fn twice(x: i64) -> i64 { x * 2 } fn main() -> i64 { var x = 0; while x < 21 { x = x + 1; } twice(x) }",
        )
        .expect("program should run");
    assert_eq!(report.value.as_i64(), Some(42));
}

#[test]
fn executes_struct_construction_and_field_access() {
    let report = Compiler::new()
        .run_source(
            "main.tp",
            "struct User { id: i64, name: String } fn main() -> String { let user = User { id: 1, name: \"TP\" }; user.name }",
        )
        .expect("program should run");
    assert_eq!(report.value.as_string(), Some("TP"));
}

#[test]
fn executes_enum_match_and_binds_payload() {
    let report = Compiler::new()
        .run_source(
            "main.tp",
            "enum Option<T> { Some(T), None } fn main() -> i64 { let value = Some(42); match value { Some(v) => v, None => 0 } }",
        )
        .expect("program should run");
    assert_eq!(report.value.as_i64(), Some(42));
}

#[test]
fn print_writes_to_captured_output() {
    let report = Compiler::new()
        .run_source("main.tp", "fn main() { print(\"hello\"); }")
        .expect("program should run");
    assert_eq!(report.output, "hello\n");
}

#[test]
fn division_by_zero_is_runtime_trap() {
    let error = Compiler::new()
        .run_source("main.tp", "fn main() -> i64 { 10 / 0 }")
        .expect_err("division by zero must trap");
    let RunFailure::Runtime(error) = error else {
        panic!("expected runtime failure");
    };
    assert_eq!(error.code, "TP-E0500");
    assert!(error.message.contains("division by zero"));
}
