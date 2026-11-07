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

#[test]
fn function_and_enum_variant_value_names_cannot_collide() {
    let report = check(
        "enum Choice { make(i64) } fn make(x: i64) -> String { \"function\" } fn main() -> Choice { make(1) }",
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("conflicts")),
        "expected a global value-name collision diagnostic, got: {:#?}",
        report.diagnostics
    );
}

#[test]
fn builtin_print_name_is_reserved() {
    let report = check("fn print(x: i64) -> i64 { x } fn main() { print(42); }");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("reserved builtin")),
        "expected a reserved builtin diagnostic, got: {:#?}",
        report.diagnostics
    );
}

#[test]
fn missing_return_path_is_rejected() {
    let report = check("fn broken(x: Bool) -> i64 { if x { return 1; } }");
    assert!(
        report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("function body has the wrong result type")),
        "expected missing return path diagnostic, got: {:#?}",
        report.diagnostics
    );
}

#[test]
fn returning_branch_unifies_with_tail_branch() {
    let report = check("fn choose(x: Bool) -> i64 { if x { return 1; } else { 2 } }");
    assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
}

#[test]
fn all_if_branches_may_return_explicitly() {
    let report = check("fn choose(x: Bool) -> i64 { if x { return 1; } else { return 2; } }");
    assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
}

#[test]
fn nested_missing_return_path_is_rejected() {
    let report = check("fn broken(a: Bool, b: Bool) -> i64 { if a { if b { return 1; } } }");
    assert!(
        report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("function body has the wrong result type")),
        "{:#?}",
        report.diagnostics
    );
}

#[test]
fn return_inside_while_does_not_make_function_total() {
    let report = check("fn broken(x: Bool) -> i64 { while x { return 1; } }");
    assert!(
        report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("function body has the wrong result type")),
        "{:#?}",
        report.diagnostics
    );
}

#[test]
fn unreachable_tail_does_not_restore_fallthrough() {
    let report = check("fn value() -> i64 { return 1; \"unreachable\" }");
    assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
}

#[test]
fn unknown_declared_type_is_rejected() {
    let report = check("fn bad(value: Missing) { }");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("unknown type `Missing`")),
        "{:#?}",
        report.diagnostics
    );
}

#[test]
fn nominal_generic_arity_is_checked() {
    let report = check(
        "enum Option<T> { Some(T), None } fn too_few(value: Option) { } fn too_many(value: Option<i64, String>) { }",
    );
    let arity_errors = report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.message.contains("type `Option` expects 1"))
        .count();
    assert_eq!(arity_errors, 2, "{:#?}", report.diagnostics);
}

#[test]
fn duplicate_generic_parameters_are_rejected() {
    let report = check("struct Pair<T, T> { value: T }");
    assert!(
        report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("duplicate generic parameter `T`")),
        "{:#?}",
        report.diagnostics
    );
}

#[test]
fn duplicate_struct_fields_are_rejected() {
    let report = check("struct User { id: i64, id: String }");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("duplicate struct field `id`")),
        "{:#?}",
        report.diagnostics
    );
}

#[test]
fn struct_and_enum_type_names_cannot_collide() {
    let report = check("struct Thing { value: i64 } enum Thing { Empty }");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("duplicate type name `Thing`")),
        "{:#?}",
        report.diagnostics
    );
}

#[test]
fn function_and_type_names_cannot_collide() {
    let report = check("struct Thing { value: i64 } fn Thing() { }");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("conflicts with type `Thing`")),
        "{:#?}",
        report.diagnostics
    );
}

#[test]
fn nested_local_shadowing_is_allowed() {
    let report = check("fn main() { let x = 1; if true { let x = 2; x; } x; }");
    assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
}
