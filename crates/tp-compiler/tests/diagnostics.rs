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

#[test]
fn human_diagnostic_uses_character_width_and_renders_secondary_labels() {
    let src = SourceFile::new("unicode.tp", "αβ\n");
    let diagnostic = Diagnostic::error("TP-E0200", "collision")
        .with_primary(Span::new(src.id(), 0, 2))
        .with_label(Span::new(src.id(), 2, 4), "other declaration");

    let rendered = render_diagnostic(&src, &diagnostic);
    assert!(rendered.contains("1 | αβ\n  | ^\n"), "{rendered}");
    assert!(
        rendered.contains("label: unicode.tp:1:2: other declaration"),
        "{rendered}"
    );
}
