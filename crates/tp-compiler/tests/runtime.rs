use tp_compiler::{Compiler, RunFailure, Value};

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

#[test]
fn mixed_return_and_tail_branch_executes_both_paths() {
    let report = Compiler::new()
        .run_source(
            "main.tp",
            "fn choose(x: Bool) -> i64 { if x { return 1; } else { 2 } } fn main() -> i64 { choose(true) + choose(false) }",
        )
        .expect("program should run");
    assert_eq!(report.value.as_i64(), Some(3));
}

#[test]
fn false_and_skips_trapping_rhs() {
    let report = Compiler::new()
        .run_source("main.tp", "fn main() -> Bool { false && (10 / 0 == 1) }")
        .expect("short-circuit should skip RHS");
    assert_eq!(report.value, Value::Bool(false));
}

#[test]
fn true_or_skips_trapping_rhs() {
    let report = Compiler::new()
        .run_source("main.tp", "fn main() -> Bool { true || (10 / 0 == 1) }")
        .expect("short-circuit should skip RHS");
    assert_eq!(report.value, Value::Bool(true));
}

#[test]
fn logical_operators_evaluate_rhs_when_required() {
    for source in [
        "fn main() -> Bool { true && (10 / 0 == 1) }",
        "fn main() -> Bool { false || (10 / 0 == 1) }",
    ] {
        let error = Compiler::new()
            .run_source("main.tp", source)
            .expect_err("RHS must be evaluated");
        let RunFailure::Runtime(error) = error else {
            panic!("expected runtime failure");
        };
        assert_eq!(error.code, "TP-E0500");
        assert!(error.message.contains("division by zero"));
    }
}

#[test]
fn run_requires_main_before_interpreter_execution() {
    let error = Compiler::new()
        .run_source("library.tp", "fn helper() -> i64 { 42 }")
        .expect_err("run mode must require an entry point");
    let RunFailure::Compile(diagnostics) = error else {
        panic!("missing main must be a compile failure");
    };
    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("missing executable entry point `main`")),
        "{diagnostics:#?}"
    );
}
