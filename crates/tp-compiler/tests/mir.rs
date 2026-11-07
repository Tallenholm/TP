use tp_compiler::{BinaryOp, Compiler, MirStatement, Rvalue, Terminator, Type};

#[test]
fn if_expression_creates_branch_and_join_blocks() {
    let mir = Compiler::new()
        .lower_mir_source("main.tp", "fn main() -> i64 { if true { 1 } else { 2 } }")
        .expect("valid TP should lower to MIR");
    let function = mir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main function");

    assert!(
        function
            .blocks
            .iter()
            .any(|block| matches!(block.terminator, Terminator::Branch { .. }))
    );
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

#[test]
fn match_binding_local_keeps_checked_payload_type() {
    let mir = Compiler::new()
        .lower_mir_source(
            "main.tp",
            "enum Option<T> { Some(T), None } fn main() -> i64 { let x = Some(42); match x { Some(v) => v, None => 0 } }",
        )
        .expect("valid TP should lower to MIR");
    let function = mir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main function");
    let binding = function
        .locals
        .iter()
        .find(|local| local.name.as_deref() == Some("v"))
        .expect("match binding local");

    assert_eq!(binding.ty, Type::I64);
}

#[test]
fn logical_and_lowers_to_control_flow_not_eager_binary() {
    let mir = Compiler::new()
        .lower_mir_source("main.tp", "fn main() -> Bool { false && true }")
        .expect("valid TP should lower to MIR");
    let function = mir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main function");

    assert!(
        function
            .blocks
            .iter()
            .any(|block| matches!(block.terminator, Terminator::Branch { .. })),
        "{:#?}",
        function.blocks
    );
    assert!(
        !function.blocks.iter().flat_map(|block| &block.statements).any(
            |statement| matches!(
                statement,
                MirStatement::Assign {
                    value: Rvalue::Binary {
                        op: BinaryOp::And,
                        ..
                    },
                    ..
                }
            )
        ),
        "{:#?}",
        function.blocks
    );
}
