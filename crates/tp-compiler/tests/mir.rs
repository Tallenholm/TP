use tp_compiler::{Compiler, Terminator};

#[test]
fn if_expression_creates_branch_and_join_blocks() {
    let mir = Compiler::new()
        .lower_mir_source(
            "main.tp",
            "fn main() -> i64 { if true { 1 } else { 2 } }",
        )
        .expect("valid TP should lower to MIR");
    let function = mir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main function");

    assert!(function
        .blocks
        .iter()
        .any(|block| matches!(block.terminator, Terminator::Branch { .. })));
    assert!(function.blocks.len() >= 4, "{:#?}", function.blocks);
}

#[test]
fn while_creates_back_edge() {
    let mir = Compiler::new()
        .lower_mir_source(
            "main.tp",
            "fn main() { var x = 0; while x < 3 { x = x + 1; } }",
        )
        .expect("valid TP should lower to MIR");
    let function = mir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main function");

    assert!(function.blocks.iter().any(|block| {
        matches!(block.terminator, Terminator::Goto(target) if target.0 <= block.id.0)
    }));
}
