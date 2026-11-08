use tp_compiler::{
    Compiler, HirExprKind, HirGlobalKind, HirPatternKind, HirStmt, HirVerificationKind,
    HirVerifier, SourceFile, Type,
};

fn valid() -> tp_compiler::HirModule {
    Compiler::new()
        .lower_source("main.tp", "fn main() -> i64 { let x = 7; x }")
        .expect("valid M1 source must lower")
}

#[test]
fn valid_hir_with_nested_shadowing_passes() {
    let hir = Compiler::new()
        .lower_source(
            "main.tp",
            "fn main() -> i64 { let x = 1; if true { let x = 2; x } else { x } }",
        )
        .expect("valid M1 source must lower");
    assert!(HirVerifier::verify(&hir).is_ok());
}

#[test]
fn rejects_duplicate_parameter_symbol() {
    let mut hir = Compiler::new()
        .lower_source(
            "main.tp",
            "fn helper(x: i64) -> i64 { x } fn main() -> i64 { helper(1) }",
        )
        .expect("valid M1 source must lower");
    let param = hir.functions[0].params[0].clone();
    hir.functions[0].params.push(param.clone());
    let errors = HirVerifier::verify(&hir).expect_err("reused parameter ID must fail");
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            HirVerificationKind::DuplicateSymbol { symbol } if *symbol == param.symbol
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_unbound_local_reference() {
    let mut hir = valid();
    hir.functions[0].body.statements.remove(0);
    let errors = HirVerifier::verify(&hir).expect_err("missing binding must fail");
    assert!(
        errors
            .iter()
            .any(|error| matches!(error.kind, HirVerificationKind::UnboundLocal { .. })),
        "{errors:#?}"
    );
}

#[test]
fn rejects_wrong_literal_type_metadata() {
    let mut hir = valid();
    let HirStmt::Let { value, .. } = &mut hir.functions[0].body.statements[0] else {
        panic!("expected let");
    };
    value.ty = Type::Bool;
    let errors = HirVerifier::verify(&hir).expect_err("corrupt integer type must fail");
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            HirVerificationKind::LiteralTypeMismatch { expected, actual }
                if *expected == Type::I64 && *actual == Type::Bool
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_wrong_local_expression_type() {
    let mut hir = valid();
    let HirStmt::Expr { expr, .. } = &mut hir.functions[0].body.statements[1] else {
        panic!("expected tail expression");
    };
    expr.ty = Type::Bool;
    let errors = HirVerifier::verify(&hir).expect_err("corrupt local type must fail");
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            HirVerificationKind::LocalTypeMismatch { expected, actual, .. }
                if *expected == Type::I64 && *actual == Type::Bool
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_non_boolean_if_condition_metadata() {
    let mut hir = Compiler::new()
        .lower_source("main.tp", "fn main() -> i64 { if true { 1 } else { 2 } }")
        .expect("valid M1 source must lower");
    let HirStmt::Expr { expr, .. } = &mut hir.functions[0].body.statements[0] else {
        panic!("expected if expression");
    };
    let HirExprKind::If { condition, .. } = &mut expr.kind else {
        panic!("expected if expression kind");
    };
    condition.ty = Type::I64;
    let errors = HirVerifier::verify(&hir).expect_err("invalid condition must fail");
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            HirVerificationKind::InvalidConditionType { actual } if *actual == Type::I64
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_reversed_function_source_span() {
    let mut hir = valid();
    let span = &mut hir.functions[0].span;
    span.start = span.end + 1;
    let errors = HirVerifier::verify(&hir).expect_err("invalid span must fail");
    assert!(
        errors
            .iter()
            .any(|error| matches!(error.kind, HirVerificationKind::InvalidSpan { .. })),
        "{errors:#?}"
    );
}

#[test]
fn rejects_assignment_to_immutable_local() {
    let mut hir = Compiler::new()
        .lower_source(
            "main.tp",
            "fn main() -> i64 { let x = 1; var y = 2; y = 3; x }",
        )
        .expect("valid M1 source must lower");
    let HirStmt::Let {
        symbol: immutable, ..
    } = &hir.functions[0].body.statements[0]
    else {
        panic!("expected let");
    };
    let immutable = *immutable;
    let HirStmt::Expr { expr, .. } = &mut hir.functions[0].body.statements[2] else {
        panic!("expected assignment");
    };
    let HirExprKind::Assign { symbol, .. } = &mut expr.kind else {
        panic!("expected assignment expression");
    };
    *symbol = immutable;
    let errors = HirVerifier::verify(&hir).expect_err("immutable assignment must fail");
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            HirVerificationKind::ImmutableAssignment { symbol } if *symbol == immutable
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_match_arm_binding_escaping_its_arm() {
    let mut hir = Compiler::new()
        .lower_source(
            "main.tp",
            "enum Opt<T> { Some(T), None } fn main() -> i64 { match Some(3) { Some(v) => v, None => 0 } }",
        )
        .expect("valid match must lower");
    let HirStmt::Expr { expr, .. } = &mut hir.functions[0].body.statements[0] else {
        panic!("expected match expression");
    };
    let HirExprKind::Match { arms, .. } = &mut expr.kind else {
        panic!("expected match");
    };
    let HirPatternKind::Variant { args, .. } = &mut arms[0].pattern.kind else {
        panic!("expected variant pattern");
    };
    args[0].kind = HirPatternKind::Wildcard;
    let errors = HirVerifier::verify(&hir).expect_err("orphaned arm body binding must fail");
    assert!(
        errors
            .iter()
            .any(|error| matches!(error.kind, HirVerificationKind::UnboundLocal { .. })),
        "{errors:#?}"
    );
}

#[test]
fn verified_hir_spans_resolve_to_snapshot_sources() {
    let query = Compiler::new()
        .analyze_source("main.tp", "fn main() -> i64 { 42 }")
        .expect("valid source");
    assert!(HirVerifier::verify_with_sources(query.hir(), query.sources()).is_ok());
}

#[test]
fn rejects_span_referencing_a_different_source_manager() {
    let query = Compiler::new()
        .analyze_source("main.tp", "fn main() -> i64 { 42 }")
        .expect("valid source");
    let mut hir = query.hir().clone();
    let foreign = SourceFile::new("foreign.tp", "fn main() {}");
    hir.functions[0].span.source = foreign.id();
    let errors = HirVerifier::verify_with_sources(&hir, query.sources())
        .expect_err("foreign SourceId must not be accepted");
    assert!(
        errors
            .iter()
            .any(|error| matches!(error.kind, HirVerificationKind::UnknownSource { .. })),
        "{errors:#?}"
    );
}

#[test]
fn rejects_source_span_past_file_boundary() {
    let query = Compiler::new()
        .analyze_source("main.tp", "fn main() -> i64 { 42 }")
        .expect("valid source");
    let mut hir = query.hir().clone();
    hir.functions[0].span.end = query
        .sources()
        .iter()
        .next()
        .expect("main source")
        .text()
        .len()
        + 1;
    let errors = HirVerifier::verify_with_sources(&hir, query.sources())
        .expect_err("span past EOF must be rejected");
    assert!(
        errors.iter().any(|error| matches!(
            error.kind,
            HirVerificationKind::InvalidSourceBoundary { .. }
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_span_inside_utf8_character() {
    let query = Compiler::new()
        .analyze_source("main.tp", "fn main() -> String { \"é\" }")
        .expect("valid Unicode source");
    let mut hir = query.hir().clone();
    let HirStmt::Expr { expr, .. } = &mut hir.functions[0].body.statements[0] else {
        panic!("expected string expression");
    };
    expr.span.start += 2;
    let errors = HirVerifier::verify_with_sources(&hir, query.sources())
        .expect_err("non-character-aligned UTF-8 span must be rejected");
    assert!(
        errors.iter().any(|error| matches!(
            error.kind,
            HirVerificationKind::InvalidSourceBoundary { .. }
        )),
        "{errors:#?}"
    );
}

#[test]
fn valid_hir_agrees_with_original_semantic_model() {
    let snapshot = Compiler::new()
        .analyze_source(
            "main.tp",
            "fn helper() -> i64 { 2 } fn main() -> i64 { let x = helper(); x }",
        )
        .expect("valid program");
    assert!(
        HirVerifier::verify_checked(snapshot.hir(), snapshot.semantic(), snapshot.sources(),)
            .is_ok()
    );
}

#[test]
fn rejects_hir_expression_type_that_differs_from_semantic_model() {
    let snapshot = Compiler::new()
        .analyze_source("main.tp", "fn main() -> i64 { 42 }")
        .expect("valid program");
    let mut hir = snapshot.hir().clone();
    let HirStmt::Expr { expr, .. } = &mut hir.functions[0].body.statements[0] else {
        panic!("expected expression");
    };
    expr.ty = Type::Bool;
    let errors = HirVerifier::verify_checked(&hir, snapshot.semantic(), snapshot.sources())
        .expect_err("tampered HIR expression type must fail");
    assert!(
        errors.iter().any(|error| matches!(
            error.kind,
            HirVerificationKind::ExpressionSemanticMismatch { .. }
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_binding_type_that_differs_from_semantic_model() {
    let snapshot = Compiler::new()
        .analyze_source("main.tp", "fn main() -> i64 { let x = 1; x }")
        .expect("valid program");
    let mut hir = snapshot.hir().clone();
    let HirStmt::Let { ty, .. } = &mut hir.functions[0].body.statements[0] else {
        panic!("expected checked binding");
    };
    *ty = Type::Bool;
    let errors = HirVerifier::verify_checked(&hir, snapshot.semantic(), snapshot.sources())
        .expect_err("tampered checked binding type must fail");
    assert!(
        errors.iter().any(|error| matches!(
            error.kind,
            HirVerificationKind::BindingSemanticMismatch { .. }
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_block_type_that_differs_from_semantic_model() {
    let snapshot = Compiler::new()
        .analyze_source("main.tp", "fn main() -> i64 { 42 }")
        .expect("valid program");
    let mut hir = snapshot.hir().clone();
    hir.functions[0].body.ty = Type::Bool;
    let errors = HirVerifier::verify_checked(&hir, snapshot.semantic(), snapshot.sources())
        .expect_err("tampered block result type must fail");
    assert!(
        errors.iter().any(|error| matches!(
            error.kind,
            HirVerificationKind::BlockSemanticMismatch { .. }
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_global_resolution_kind_that_differs_from_semantic_model() {
    let snapshot = Compiler::new()
        .analyze_source(
            "main.tp",
            "fn helper() -> i64 { 2 } fn main() -> i64 { helper() }",
        )
        .expect("valid program");
    let mut hir = snapshot.hir().clone();
    let main = hir
        .functions
        .iter_mut()
        .find(|function| function.name == "main")
        .expect("main function");
    let HirStmt::Expr { expr, .. } = &mut main.body.statements[0] else {
        panic!("expected call");
    };
    let HirExprKind::Call { callee, .. } = &mut expr.kind else {
        panic!("expected call expression");
    };
    let HirExprKind::Global { kind, .. } = &mut callee.kind else {
        panic!("expected global callee");
    };
    *kind = HirGlobalKind::Builtin;
    let errors = HirVerifier::verify_checked(&hir, snapshot.semantic(), snapshot.sources())
        .expect_err("changed global resolution must fail");
    assert!(
        errors.iter().any(|error| matches!(
            error.kind,
            HirVerificationKind::GlobalSemanticMismatch { .. }
        )),
        "{errors:#?}"
    );
}
