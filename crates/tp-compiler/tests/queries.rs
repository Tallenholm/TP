use std::path::PathBuf;

use tp_compiler::{Compiler, HirExprKind, HirStmt, Type};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn source_snapshot_shares_authoritative_binding_and_source_identity() {
    let query = Compiler::new()
        .analyze_source("main.tp", "fn main() -> i64 { let x = 40; x + 2 }")
        .expect("valid source should produce a checked snapshot");

    assert_eq!(query.module().items.len(), 1);
    let function = &query.hir().functions[0];
    let HirStmt::Let {
        symbol, span, ty, ..
    } = &function.body.statements[0]
    else {
        panic!("expected typed let statement");
    };
    let (checked_id, checked_type) = query
        .semantic()
        .binding(*span)
        .expect("snapshot must retain checked binding identity");
    assert_eq!(*symbol, checked_id);
    assert_eq!(ty, checked_type);

    let HirStmt::Expr { expr, .. } = &function.body.statements[1] else {
        panic!("expected tail expression");
    };
    assert_eq!(query.semantic().expr_type(expr.span), Some(&Type::I64));
    let source = query
        .sources()
        .get(expr.span.source)
        .expect("all HIR spans must resolve to snapshot sources");
    assert_eq!(source.name(), "main.tp");
    assert_eq!(query.mir().functions[0].return_type, Type::I64);
}

#[test]
fn independent_source_snapshots_do_not_reuse_stale_lowering() {
    let compiler = Compiler::new();
    let first = compiler
        .analyze_source("same.tp", "fn main() -> i64 { 1 }")
        .expect("first snapshot");
    let second = compiler
        .analyze_source("same.tp", "fn main() -> i64 { 2 }")
        .expect("second snapshot");
    let HirStmt::Expr {
        expr: first_expr, ..
    } = &first.hir().functions[0].body.statements[0]
    else {
        panic!("expected first expression");
    };
    let HirStmt::Expr {
        expr: second_expr, ..
    } = &second.hir().functions[0].body.statements[0]
    else {
        panic!("expected second expression");
    };
    assert!(matches!(&first_expr.kind, HirExprKind::Integer(1)));
    assert!(matches!(&second_expr.kind, HirExprKind::Integer(2)));
}

#[test]
fn path_snapshot_keeps_diamond_import_sources_unique() {
    let query = Compiler::new()
        .analyze_path(&fixture("diamond_main.tp"))
        .expect("diamond module graph should lower");
    let count = query
        .sources()
        .iter()
        .filter(|source| source.name().ends_with("diamond_shared.tp"))
        .count();
    assert_eq!(count, 1);
    assert!(
        query
            .hir()
            .functions
            .iter()
            .any(|function| function.name == "main")
    );
    assert!(
        query
            .mir()
            .functions
            .iter()
            .any(|function| function.name == "main")
    );
}

#[test]
fn invalid_query_returns_diagnostics_instead_of_partial_ir() {
    let result =
        Compiler::new().analyze_source("invalid.tp", "fn main() -> i64 { nonexistent_value }");
    assert!(
        result.is_err(),
        "invalid source must not expose unchecked IR"
    );
}
