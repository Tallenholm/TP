use tp_compiler::Compiler;

fn check(source: &str) -> tp_compiler::CompileReport {
    Compiler::new().check_source("main.tp", source)
}

fn has_code(report: &tp_compiler::CompileReport, code: &str) -> bool {
    report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == code)
}

#[test]
fn reports_unknown_name() {
    let report = check("fn main() { print(missing); }");
    assert!(has_code(&report, "TP-E0200"), "{:#?}", report.diagnostics);
}

#[test]
fn assignment_to_let_is_rejected() {
    let report = check("fn main() { let x = 1; x = 2; }");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("immutable")),
        "{:#?}",
        report.diagnostics
    );
}

#[test]
fn boolean_condition_is_required() {
    let report = check("fn main() { if 1 { } }");
    assert!(has_code(&report, "TP-E0300"), "{:#?}", report.diagnostics);
}

#[test]
fn valid_primitive_program_type_checks() {
    let report = check(
        "fn add(a: i64, b: i64) -> i64 { a + b } fn main() { var x = add(20, 22); if x == 42 { x = x + 1; } }",
    );
    assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
}

#[test]
fn function_call_argument_types_are_checked() {
    let report = check("fn add(a: i64, b: i64) -> i64 { a + b } fn main() { add(1, true); }");
    assert!(has_code(&report, "TP-E0300"), "{:#?}", report.diagnostics);
}

#[test]
fn struct_fields_are_type_checked() {
    let report = check(
        "struct User { id: i64, name: String } fn main() { let user = User { id: true, name: \"A\" }; user.name; }",
    );
    assert!(has_code(&report, "TP-E0300"), "{:#?}", report.diagnostics);
}

#[test]
fn valid_struct_field_access_has_the_declared_type() {
    let report = check(
        "struct User { id: i64, name: String } fn main() -> String { let user = User { id: 1, name: \"A\" }; user.name }",
    );
    assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
}

#[test]
fn non_exhaustive_option_match_is_rejected() {
    let report = check(
        "enum Option<T> { Some(T), None } fn unwrap_or_zero(x: Option<i64>) -> i64 { match x { Some(v) => v } }",
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("non-exhaustive")),
        "{:#?}",
        report.diagnostics
    );
}

#[test]
fn option_constructor_and_exhaustive_match_type_check() {
    let report = check(
        "enum Option<T> { Some(T), None } fn make() -> Option<i64> { Some(42) } fn unwrap(x: Option<i64>) -> i64 { match x { Some(v) => v, None => 0 } }",
    );
    assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
}

#[test]
fn result_constructor_infers_generic_payloads() {
    let report = check(
        "enum Result<T, E> { Ok(T), Err(E) } fn success() -> Result<i64, String> { Ok(42) } fn failure() -> Result<i64, String> { Err(\"nope\") }",
    );
    assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
}
