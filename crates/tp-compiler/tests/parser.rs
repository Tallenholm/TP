use tp_compiler::{BinaryOp, ExprKind, Item, Lexer, Parser, SourceFile, Stmt};

fn parse(source_text: &str) -> tp_compiler::ParseResult {
    let source = SourceFile::new("main.tp", source_text);
    let lexed = Lexer::new(&source).lex();
    assert!(lexed.diagnostics.is_empty(), "lexer diagnostics: {:#?}", lexed.diagnostics);
    Parser::new(&source, lexed.tokens).parse_module()
}

#[test]
fn multiplication_binds_tighter_than_addition() {
    let result = parse("fn main() { let x = 1 + 2 * 3; }");
    assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
    let module = result.module.expect("module");
    let Item::Function(function) = &module.items[0] else {
        panic!("expected function");
    };
    let Stmt::Let { value, .. } = &function.body.statements[0] else {
        panic!("expected let");
    };
    let ExprKind::Binary { op: BinaryOp::Add, right, .. } = &value.kind else {
        panic!("expected addition at root: {:#?}", value.kind);
    };
    assert!(matches!(
        right.kind,
        ExprKind::Binary { op: BinaryOp::Multiply, .. }
    ));
}

#[test]
fn parser_recovers_at_statement_boundary() {
    let result = parse("fn main() { let x = ; let y = 2; }");
    assert_eq!(result.diagnostics.len(), 1, "{:#?}", result.diagnostics);
    assert_eq!(result.diagnostics[0].code, "TP-E0100");
    let module = result.module.expect("recovered module");
    let Item::Function(function) = &module.items[0] else {
        panic!("expected function");
    };
    assert!(function
        .body
        .statements
        .iter()
        .any(|stmt| matches!(stmt, Stmt::Let { name, .. } if name == "y")));
}
