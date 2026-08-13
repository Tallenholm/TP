use tp_compiler::{Compiler, HirExprKind, HirStmt, Type};

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
