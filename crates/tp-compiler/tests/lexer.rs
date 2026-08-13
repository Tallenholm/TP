use tp_compiler::{Lexer, SourceFile, TokenKind};

#[test]
fn lexes_function_and_expression() {
    use TokenKind::*;

    let source = SourceFile::new(
        "main.tp",
        "fn add(a: i64, b: i64) -> i64 { a + b }",
    );
    let result = Lexer::new(&source).lex();
    let kinds: Vec<TokenKind> = result.tokens.into_iter().map(|token| token.kind).collect();

    assert_eq!(
        kinds,
        vec![
            Fn, Identifier, LParen, Identifier, Colon, Identifier, Comma, Identifier,
            Colon, Identifier, RParen, Arrow, Identifier, LBrace, Identifier, Plus,
            Identifier, RBrace, Eof,
        ]
    );
    assert!(result.diagnostics.is_empty());
}

#[test]
fn invalid_character_reports_error_and_recovers() {
    use TokenKind::*;

    let source = SourceFile::new("main.tp", "let x = 1 @ let y = 2");
    let result = Lexer::new(&source).lex();
    let kinds: Vec<TokenKind> = result.tokens.iter().map(|token| token.kind).collect();

    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].code, "TP-E0001");
    assert_eq!(
        kinds,
        vec![Let, Identifier, Equal, Integer, Let, Identifier, Equal, Integer, Eof]
    );
}

#[test]
fn lexes_literals_comments_keywords_and_operators() {
    use TokenKind::*;

    let source = SourceFile::new(
        "main.tp",
        r#"
            // lexer coverage
            var value = 12.5;
            let text = "hello\nworld";
            if true && false || value >= 10.0 {
                return value != 0.0;
            } else {
                while value <= 20.0 { value = value - 1 * 2 / 3 % 2; }
            }
            struct Thing { field: String }
            enum Choice { Yes, No }
            match value { _ => value }
            import util as u
        "#,
    );
    let result = Lexer::new(&source).lex();
    let kinds: Vec<TokenKind> = result.tokens.iter().map(|token| token.kind).collect();

    assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
    assert!(kinds.contains(&Float));
    assert!(kinds.contains(&String));
    assert!(kinds.contains(&Var));
    assert!(kinds.contains(&Let));
    assert!(kinds.contains(&If));
    assert!(kinds.contains(&True));
    assert!(kinds.contains(&False));
    assert!(kinds.contains(&AndAnd));
    assert!(kinds.contains(&OrOr));
    assert!(kinds.contains(&GreaterEqual));
    assert!(kinds.contains(&BangEqual));
    assert!(kinds.contains(&Else));
    assert!(kinds.contains(&While));
    assert!(kinds.contains(&LessEqual));
    assert!(kinds.contains(&Minus));
    assert!(kinds.contains(&Star));
    assert!(kinds.contains(&Slash));
    assert!(kinds.contains(&Percent));
    assert!(kinds.contains(&Struct));
    assert!(kinds.contains(&Enum));
    assert!(kinds.contains(&Match));
    assert!(kinds.contains(&FatArrow));
    assert!(kinds.contains(&Import));
    assert!(kinds.contains(&As));
    assert_eq!(kinds.last(), Some(&Eof));
}
