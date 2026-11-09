//! Deterministic adversarial mutations of otherwise-valid MIR.
//! The fixed input corpus avoids flaky randomness and external dependencies.
use tp_compiler::{
    BasicBlock, BlockId, Compiler, Interpreter, LocalId, MirStatement, MirVerificationKind,
    MirVerifier, Operand, Rvalue, Terminator,
};

fn fixture() -> tp_compiler::MirModule {
    Compiler::new()
        .lower_mir_source("mutations.tp", "fn main() -> i64 { let x = 7; x }")
        .expect("baseline fixture must be valid")
}

#[test]
fn corrupted_block_ids_are_rejected_without_panics() {
    let baseline = fixture();
    for bad in [1, 2, 3, 7, 31, 127, 255, 65_535, u32::MAX] {
        let mut mir = baseline.clone();
        mir.functions[0].blocks[0].id = BlockId(bad);
        let errors = MirVerifier::verify(&mir).expect_err("bad block index must fail");
        assert!(
            errors
                .iter()
                .any(|error| matches!(error.kind, MirVerificationKind::BlockIndexMismatch { .. })),
            "block ID {bad}: {errors:#?}"
        );
        let mut interpreter = Interpreter::new(&mir);
        assert!(
            interpreter.run_main().is_err(),
            "direct interpreter accepted invalid block ID {bad}"
        );
    }
}

#[test]
fn corrupted_local_ids_are_rejected_without_panics() {
    let baseline = fixture();
    assert!(!baseline.functions[0].locals.is_empty());
    for bad in [1, 2, 3, 7, 31, 127, 255, 65_535, u32::MAX] {
        let mut mir = baseline.clone();
        mir.functions[0].locals[0].id = LocalId(bad);
        let errors = MirVerifier::verify(&mir).expect_err("bad local index must fail");
        assert!(
            errors
                .iter()
                .any(|error| matches!(error.kind, MirVerificationKind::LocalIndexMismatch { .. })),
            "local ID {bad}: {errors:#?}"
        );
        let mut interpreter = Interpreter::new(&mir);
        assert!(
            interpreter.run_main().is_err(),
            "direct interpreter accepted invalid local ID {bad}"
        );
    }
}

#[test]
fn dangling_cfg_edges_are_rejected_without_panics() {
    let baseline = fixture();
    for bad in [1, 2, 3, 7, 31, 127, 255, 65_535, u32::MAX] {
        let mut mir = baseline.clone();
        mir.functions[0].blocks[0].terminator = Terminator::Goto(BlockId(bad));
        let errors = MirVerifier::verify(&mir).expect_err("dangling edge must fail");
        assert!(
            errors
                .iter()
                .any(|error| matches!(error.kind, MirVerificationKind::InvalidSuccessor { .. })),
            "successor {bad}: {errors:#?}"
        );
    }
}

#[test]
fn dangling_operand_locals_are_rejected_without_panics() {
    let baseline = fixture();
    for bad in [1, 2, 3, 7, 31, 127, 255, 65_535, u32::MAX] {
        let mut mir = baseline.clone();
        mir.functions[0].blocks[0]
            .statements
            .push(MirStatement::Assign {
                target: LocalId(0),
                value: Rvalue::Use(Operand::Local(LocalId(bad))),
            });
        let errors = MirVerifier::verify(&mir).expect_err("dangling operand must fail");
        assert!(
            errors.iter().any(|error| matches!(
                error.kind,
                MirVerificationKind::InvalidLocalReference { .. }
            )),
            "operand local {bad}: {errors:#?}"
        );
    }
}

#[test]
fn valid_mir_verification_is_deterministic_and_side_effect_free() {
    let mir = fixture();
    let unchanged = mir.clone();
    for _ in 0..128 {
        assert_eq!(MirVerifier::verify(&mir), Ok(()));
    }
    assert_eq!(mir, unchanged);
}

#[test]
fn malformed_mir_diagnostics_are_deterministic_and_side_effect_free() {
    let mut mir = fixture();
    mir.functions[0].blocks[0].id = BlockId(u32::MAX);
    let unchanged = mir.clone();
    let first = MirVerifier::verify(&mir).expect_err("corrupt MIR must fail");
    for _ in 0..128 {
        assert_eq!(MirVerifier::verify(&mir), Err(first.clone()));
    }
    assert_eq!(mir, unchanged);
}

#[test]
fn generated_linear_cfg_with_sixty_four_blocks_remains_valid() {
    let mut mir = fixture();
    let function = &mut mir.functions[0];
    assert_eq!(function.blocks.len(), 1, "fixture should have one block");
    function.blocks[0].terminator = Terminator::Goto(BlockId(1));
    for index in 1..64 {
        function.blocks.push(BasicBlock {
            id: BlockId(index),
            statements: Vec::new(),
            terminator: if index == 63 {
                Terminator::Return(Some(Operand::Local(LocalId(0))))
            } else {
                Terminator::Goto(BlockId(index + 1))
            },
        });
    }
    assert_eq!(MirVerifier::verify(&mir), Ok(()));
    let mut interpreter = Interpreter::new(&mir);
    assert_eq!(
        interpreter.run_main().expect("valid generated CFG").as_i64(),
        Some(7)
    );
}

#[test]
fn generated_linear_cfg_rejects_one_corrupted_middle_edge() {
    let mut mir = fixture();
    let function = &mut mir.functions[0];
    assert_eq!(function.blocks.len(), 1, "fixture should have one block");
    function.blocks[0].terminator = Terminator::Goto(BlockId(1));
    for index in 1..64 {
        function.blocks.push(BasicBlock {
            id: BlockId(index),
            statements: Vec::new(),
            terminator: if index == 63 {
                Terminator::Return(Some(Operand::Local(LocalId(0))))
            } else {
                Terminator::Goto(BlockId(index + 1))
            },
        });
    }
    mir.functions[0].blocks[32].terminator = Terminator::Goto(BlockId(u32::MAX));
    let errors = MirVerifier::verify(&mir).expect_err("broken generated CFG must fail");
    assert!(
        errors.iter().any(|error| matches!(
            error.kind,
            MirVerificationKind::InvalidSuccessor {
                from: BlockId(32),
                target: BlockId(u32::MAX)
            }
        )),
        "{errors:#?}"
    );
}
