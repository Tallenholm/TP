use tp_compiler::{SourceFile, Span};

#[test]
fn byte_span_maps_to_line_and_column() {
    let src = SourceFile::new("main.tp", "let x = 1\nprint(x)\n");
    let span = Span::new(src.id(), 10, 15);
    let loc = src.line_col(span.start).unwrap();
    assert_eq!((loc.line, loc.column), (2, 1));
}
