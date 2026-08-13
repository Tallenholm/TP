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
            Fn,
            Identifier,
            LParen,
            Identifier,
            Colon,
            Identifier,
            Comma,
            Identifier,
            Colon,
            Identifier,
            RParen,
            Arrow,
            Identifier,
            LBrace,
            Identifier,
            Plus,
            Identifier,
            RBrace,
            Eof,
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
