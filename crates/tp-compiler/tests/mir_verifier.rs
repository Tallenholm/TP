use tp_compiler::{
    BlockId, Compiler, LocalId, MirModule, MirStatement, MirVerificationKind, MirVerifier, Operand,
    Rvalue, Terminator,
};

fn sample() -> MirModule {
    Compiler::new()
        .lower_mir_source("main.tp", "fn main() -> i64 { let x = 7; x }")
        .expect("valid M1 source should lower")
}

#[test]
fn valid_m1_mir_passes_structural_verification() {
    let mir = sample();
    assert!(MirVerifier::verify(&mir).is_ok());
}

#[test]
fn rejects_missing_entry_block() {
    let mut mir = sample();
    let function = &mut mir.functions[0];
    function.entry = BlockId(999);
    let errors = MirVerifier::verify(&mir).expect_err("nonexistent entry must be rejected");
    assert!(errors.iter().any(|error| matches!(
        &error.kind,
        MirVerificationKind::InvalidEntry { entry } if *entry == BlockId(999)
    )), "{errors:#?}");
}

#[test]
fn rejects_mismatched_block_indices() {
    let mut mir = sample();
    mir.functions[0].blocks[0].id = BlockId(42);
    let errors = MirVerifier::verify(&mir).expect_err("misindexed basic block must be rejected");
    assert!(errors.iter().any(|error| matches!(
        &error.kind,
        MirVerificationKind::BlockIndexMismatch { actual, .. } if *actual == BlockId(42)
    )), "{errors:#?}");
}

#[test]
fn rejects_unknown_successor_blocks() {
    let mut mir = sample();
    let function = &mut mir.functions[0];
    let entry = function.entry;
    function.blocks[entry.0 as usize].terminator = Terminator::Goto(BlockId(999));
    let errors = MirVerifier::verify(&mir).expect_err("invalid CFG edge must be rejected");
    assert!(errors.iter().any(|error| matches!(
        &error.kind,
        MirVerificationKind::InvalidSuccessor { target, .. } if *target == BlockId(999)
    )), "{errors:#?}");
}

#[test]
fn rejects_mismatched_local_indices() {
    let mut mir = sample();
    assert!(!mir.functions[0].locals.is_empty());
    mir.functions[0].locals[0].id = LocalId(42);
    let errors = MirVerifier::verify(&mir).expect_err("misindexed local must be rejected");
    assert!(errors.iter().any(|error| matches!(
        &error.kind,
        MirVerificationKind::LocalIndexMismatch { actual, .. } if *actual == LocalId(42)
    )), "{errors:#?}");
}

#[test]
fn rejects_invalid_local_operands() {
    let mut mir = sample();
    let function = &mut mir.functions[0];
    let entry = function.entry;
    function.blocks[entry.0 as usize].statements.push(MirStatement::Assign {
        target: LocalId(0),
        value: Rvalue::Use(Operand::Local(LocalId(999))),
    });
    let errors = MirVerifier::verify(&mir).expect_err("unknown operand local must be rejected");
    assert!(errors.iter().any(|error| matches!(
        &error.kind,
        MirVerificationKind::InvalidLocalReference { local, .. } if *local == LocalId(999)
    )), "{errors:#?}");
}

#[test]
fn rejects_invalid_parameter_local() {
    let mut mir = Compiler::new()
        .lower_mir_source(
            "main.tp",
            "fn helper(x: i64) -> i64 { x } fn main() -> i64 { helper(7) }",
        )
        .expect("valid helper function should lower");
    let helper = mir.functions.iter_mut().find(|f| f.name == "helper").unwrap();
    helper.params[0] = LocalId(999);
    let errors = MirVerifier::verify(&mir).expect_err("invalid parameter local must be rejected");
    assert!(errors.iter().any(|error| matches!(
        &error.kind,
        MirVerificationKind::InvalidParameterLocal { local } if *local == LocalId(999)
    )), "{errors:#?}");
}
