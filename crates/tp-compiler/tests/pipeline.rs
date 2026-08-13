use tp_compiler::Compiler;

#[test]
fn empty_source_can_be_checked() {
    let compiler = Compiler::new();
    let report = compiler.check_source("empty.tp", "");
    assert!(report.diagnostics.is_empty());
}
