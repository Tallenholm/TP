use tp_compiler::{BinaryOp, ExprKind, Item, Lexer, Parser, SourceFile, Stmt};

fn parse(source_text: &str) -> tp_compiler::ParseResult {
    let source = SourceFile::new("main.tp", source_text);
    let lexed = Lexer::new(&source).lex();
    assert!(
        lexed.diagnostics.is_empty(),
        "lexer diagnostics: {:#?}",
        lexed.diagnostics
    );
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
    let ExprKind::Binary {
        op: BinaryOp::Add,
        right,
        ..
    } = &value.kind
    else {
        panic!("expected addition at root: {:#?}", value.kind);
    };
    assert!(matches!(
        right.kind,
        ExprKind::Binary {
            op: BinaryOp::Multiply,
            ..
        }
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
    assert!(
        function
            .body
            .statements
            .iter()
            .any(|stmt| matches!(stmt, Stmt::Let { name, .. } if name == "y"))
    );
}

#[test]
fn parses_generic_enum_and_match() {
    let result = parse(
        "enum Option<T> { Some(T), None } fn value(x: Option<i64>) -> i64 { match x { Some(v) => v, None => 0 } }",
    );
    assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
    let module = result.module.expect("module");
    assert_eq!(module.items.len(), 2);
    assert!(
        matches!(&module.items[0], Item::Enum(decl) if decl.name == "Option" && decl.type_params == vec!["T"])
    );
    let Item::Function(function) = &module.items[1] else {
        panic!("expected function");
    };
    let Stmt::Expr { expr, .. } = &function.body.statements[0] else {
        panic!("expected match expression");
    };
    assert!(matches!(&expr.kind, ExprKind::Match { arms, .. } if arms.len() == 2));
}

#[test]
fn parses_struct_construction_and_field_access() {
    let result = parse(
        "struct User { id: i64, name: String } fn main() { let user = User { id: 1, name: \"A\" }; user.name; }",
    );
    assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
    let module = result.module.expect("module");
    assert!(matches!(&module.items[0], Item::Struct(decl) if decl.fields.len() == 2));
    let Item::Function(function) = &module.items[1] else {
        panic!("expected function");
    };
    let Stmt::Let { value, .. } = &function.body.statements[0] else {
        panic!("expected let");
    };
    assert!(
        matches!(&value.kind, ExprKind::StructLiteral { type_name, fields } if type_name == "User" && fields.len() == 2)
    );
    let Stmt::Expr { expr, .. } = &function.body.statements[1] else {
        panic!("expected field expression");
    };
    assert!(matches!(&expr.kind, ExprKind::Field { field, .. } if field == "name"));
}
