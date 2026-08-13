use tp_compiler::{Item, Lexer, Parser, SourceFile};

#[test]
fn parses_import_with_alias() {
    let source = SourceFile::new("main.tp", "import util as helpers; fn main() {}");
    let lexed = Lexer::new(&source).lex();
    assert!(lexed.diagnostics.is_empty());
    let result = Parser::new(&source, lexed.tokens).parse_module();
    assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
    let module = result.module.expect("module");
    assert!(matches!(
        &module.items[0],
        Item::Import(import) if import.module == "util" && import.alias.as_deref() == Some("helpers")
    ));
}
