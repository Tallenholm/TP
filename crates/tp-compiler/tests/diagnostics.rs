use tp_compiler::{Diagnostic, SourceFile, Span, render_diagnostic};

#[test]
fn byte_span_maps_to_line_and_column() {
    let src = SourceFile::new("main.tp", "let x = 1\nprint(x)\n");
    let span = Span::new(src.id(), 10, 15);
    let loc = src.line_col(span.start).unwrap();
    assert_eq!((loc.line, loc.column), (2, 1));
}

#[test]
fn renders_stable_human_diagnostic() {
    let src = SourceFile::new("main.tp", "let x = 1\nprint(x)\n");
    let diagnostic = Diagnostic::error("TP-E0300", "type mismatch")
        .with_primary(Span::new(src.id(), 10, 15))
        .with_help("expected i64");

    assert_eq!(
        render_diagnostic(&src, &diagnostic),
        "error[TP-E0300]: type mismatch\n --> main.tp:2:1\n  |\n2 | print(x)\n  | ^^^^^\n  = help: expected i64\n"
    );
}
