use tp_compiler::{Compiler, HirExprKind, HirGlobalKind, HirStmt, Type};

#[test]
fn hir_resolves_locals_and_attaches_types() {
    let hir = Compiler::new()
        .lower_source("main.tp", "fn main() -> i64 { let x = 40; x + 2 }")
        .expect("valid TP should lower");
    let function = hir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main function");

    let HirStmt::Let { symbol, .. } = &function.body.statements[0] else {
        panic!("expected let statement");
    };
    let HirStmt::Expr { expr, .. } = &function.body.statements[1] else {
        panic!("expected tail expression");
    };

    assert_eq!(expr.ty, Type::I64);
    let HirExprKind::Binary { left, .. } = &expr.kind else {
        panic!("expected binary expression");
    };
    assert!(matches!(&left.kind, HirExprKind::Local(id) if id == symbol));
}

#[test]
fn hir_preserves_checked_global_resolution_kind() {
    let hir = Compiler::new()
        .lower_source(
            "main.tp",
            "enum Choice { Yes } fn helper() -> i64 { 1 } fn main() { helper(); Yes; print(1); }",
        )
        .expect("valid TP should lower");
    let function = hir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main function");

    let HirStmt::Expr {
        expr: helper_call, ..
    } = &function.body.statements[0]
    else {
        panic!("expected helper call");
    };
    let HirExprKind::Call { callee, .. } = &helper_call.kind else {
        panic!("expected call expression");
    };
    assert!(matches!(
        &callee.kind,
        HirExprKind::Global {
            kind: HirGlobalKind::Function,
            ..
        }
    ));

    let HirStmt::Expr { expr: variant, .. } = &function.body.statements[1] else {
        panic!("expected variant expression");
    };
    assert!(matches!(
        &variant.kind,
        HirExprKind::Global {
            kind: HirGlobalKind::Variant,
            ..
        }
    ));

    let HirStmt::Expr {
        expr: print_call, ..
    } = &function.body.statements[2]
    else {
        panic!("expected print call");
    };
    let HirExprKind::Call { callee, .. } = &print_call.kind else {
        panic!("expected print call expression");
    };
    assert!(matches!(
        &callee.kind,
        HirExprKind::Global {
            kind: HirGlobalKind::Builtin,
            ..
        }
    ));
}
