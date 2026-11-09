use tp_compiler::{
    BinaryOp, BlockId, Compiler, Constant, Interpreter, LocalId, MirModule, MirPattern, MirStatement,
    MirVerificationKind, MirVerifier, Operand, Rvalue, Terminator, Type, UnaryOp,
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
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            MirVerificationKind::InvalidEntry { entry } if *entry == BlockId(999)
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_mismatched_block_indices() {
    let mut mir = sample();
    mir.functions[0].blocks[0].id = BlockId(42);
    let errors = MirVerifier::verify(&mir).expect_err("misindexed basic block must be rejected");
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            MirVerificationKind::BlockIndexMismatch { actual, .. } if *actual == BlockId(42)
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_unknown_successor_blocks() {
    let mut mir = sample();
    let function = &mut mir.functions[0];
    let entry = function.entry;
    function.blocks[entry.0 as usize].terminator = Terminator::Goto(BlockId(999));
    let errors = MirVerifier::verify(&mir).expect_err("invalid CFG edge must be rejected");
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            MirVerificationKind::InvalidSuccessor { target, .. } if *target == BlockId(999)
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_mismatched_local_indices() {
    let mut mir = sample();
    assert!(!mir.functions[0].locals.is_empty());
    mir.functions[0].locals[0].id = LocalId(42);
    let errors = MirVerifier::verify(&mir).expect_err("misindexed local must be rejected");
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            MirVerificationKind::LocalIndexMismatch { actual, .. } if *actual == LocalId(42)
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_invalid_local_operands() {
    let mut mir = sample();
    let function = &mut mir.functions[0];
    let entry = function.entry;
    function.blocks[entry.0 as usize]
        .statements
        .push(MirStatement::Assign {
            target: LocalId(0),
            value: Rvalue::Use(Operand::Local(LocalId(999))),
        });
    let errors = MirVerifier::verify(&mir).expect_err("unknown operand local must be rejected");
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            MirVerificationKind::InvalidLocalReference { local, .. } if *local == LocalId(999)
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_invalid_parameter_local() {
    let mut mir = Compiler::new()
        .lower_mir_source(
            "main.tp",
            "fn helper(x: i64) -> i64 { x } fn main() -> i64 { helper(7) }",
        )
        .expect("valid helper function should lower");
    let helper = mir
        .functions
        .iter_mut()
        .find(|f| f.name == "helper")
        .unwrap();
    helper.params[0] = LocalId(999);
    let errors = MirVerifier::verify(&mir).expect_err("invalid parameter local must be rejected");
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            MirVerificationKind::InvalidParameterLocal { local } if *local == LocalId(999)
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_empty_block_table() {
    let mut mir = sample();
    mir.functions[0].blocks.clear();
    let errors = MirVerifier::verify(&mir).expect_err("empty CFG must be rejected");
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            MirVerificationKind::InvalidEntry { entry } if *entry == BlockId(0)
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_duplicate_parameter_local() {
    let mut mir = Compiler::new()
        .lower_mir_source(
            "main.tp",
            "fn helper(x: i64) -> i64 { x } fn main() -> i64 { helper(7) }",
        )
        .expect("valid helper should lower");
    let helper = mir
        .functions
        .iter_mut()
        .find(|f| f.name == "helper")
        .unwrap();
    let parameter = helper.params[0];
    helper.params.push(parameter);
    let errors = MirVerifier::verify(&mir).expect_err("duplicate param must be rejected");
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            MirVerificationKind::DuplicateParameterLocal { local } if *local == parameter
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_invalid_branch_operand_and_successor() {
    let mut mir = sample();
    let function = &mut mir.functions[0];
    let entry = function.entry;
    function.blocks[entry.0 as usize].terminator = Terminator::Branch {
        condition: Operand::Local(LocalId(999)),
        then_block: BlockId(999),
        else_block: entry,
    };
    let errors = MirVerifier::verify(&mir).expect_err("bad branch must be rejected");
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            MirVerificationKind::InvalidLocalReference { local, .. } if *local == LocalId(999)
        )),
        "{errors:#?}"
    );
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            MirVerificationKind::InvalidSuccessor { target, .. } if *target == BlockId(999)
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_nested_match_binding_local() {
    let mut mir = sample();
    let function = &mut mir.functions[0];
    let entry = function.entry;
    function.blocks[entry.0 as usize].terminator = Terminator::Match {
        value: Operand::Constant(Constant::Bool(true)),
        arms: vec![(
            MirPattern::Variant {
                name: "Example".to_owned(),
                args: vec![MirPattern::Bind(LocalId(999))],
            },
            entry,
        )],
        otherwise: entry,
    };
    let errors = MirVerifier::verify(&mir).expect_err("bad match binding must be rejected");
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            MirVerificationKind::InvalidLocalReference { local, .. } if *local == LocalId(999)
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_invalid_return_operand() {
    let mut mir = sample();
    let function = &mut mir.functions[0];
    let entry = function.entry;
    function.blocks[entry.0 as usize].terminator =
        Terminator::Return(Some(Operand::Local(LocalId(999))));
    let errors = MirVerifier::verify(&mir).expect_err("bad return must be rejected");
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            MirVerificationKind::InvalidLocalReference { local, .. } if *local == LocalId(999)
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_undeclared_assignment_target() {
    let mut mir = sample();
    let function = &mut mir.functions[0];
    let entry = function.entry;
    function.blocks[entry.0 as usize]
        .statements
        .push(MirStatement::Assign {
            target: LocalId(999),
            value: Rvalue::Use(Operand::Constant(Constant::Unit)),
        });
    let errors = MirVerifier::verify(&mir).expect_err("unknown target must be rejected");
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            MirVerificationKind::InvalidLocalReference { local, .. } if *local == LocalId(999)
        )),
        "{errors:#?}"
    );
}

#[test]
fn direct_interpreter_rejects_malformed_block_ids() {
    let mut mir = sample();
    mir.functions[0].blocks[0].id = BlockId(999);
    let mut interpreter = Interpreter::new(&mir);
    let error = interpreter
        .run_main()
        .expect_err("raw interpreter must not execute incorrectly indexed MIR");
    assert_eq!(error.code, "TP-E0600");
}

#[test]
fn direct_interpreter_rejects_malformed_local_ids() {
    let mut mir = sample();
    mir.functions[0].locals[0].id = LocalId(999);
    let mut interpreter = Interpreter::new(&mir);
    let error = interpreter
        .run_main()
        .expect_err("raw interpreter must not execute incorrectly indexed locals");
    assert_eq!(error.code, "TP-E0600");
}

#[test]
fn rejects_duplicate_mir_function_names() {
    let mut mir = sample();
    mir.functions.push(mir.functions[0].clone());
    let errors = MirVerifier::verify(&mir).expect_err("duplicate callable names must fail");
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            MirVerificationKind::DuplicateFunction { name } if name == "main"
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_integer_mir_branch_condition() {
    let mut mir = sample();
    let function = &mut mir.functions[0];
    let entry = function.entry;
    function.blocks[entry.0 as usize].terminator = Terminator::Branch {
        condition: Operand::Constant(Constant::I64(1)),
        then_block: entry,
        else_block: entry,
    };
    let errors = MirVerifier::verify(&mir).expect_err("branch must have a boolean condition");
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            MirVerificationKind::InvalidBranchCondition { actual, .. }
                if *actual == Type::I64
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_mir_return_of_wrong_primitive_type() {
    let mut mir = sample();
    let function = &mut mir.functions[0];
    let entry = function.entry;
    function.blocks[entry.0 as usize].terminator =
        Terminator::Return(Some(Operand::Constant(Constant::Bool(true))));
    let errors = MirVerifier::verify(&mir).expect_err("wrong return type must be rejected");
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            MirVerificationKind::InvalidReturnType { expected, actual, .. }
                if *expected == Type::I64 && *actual == Type::Bool
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_unknown_function_reference_in_mir() {
    let mut mir = sample();
    let function = &mut mir.functions[0];
    let entry = function.entry;
    function.blocks[entry.0 as usize]
        .statements
        .push(MirStatement::Assign {
            target: LocalId(0),
            value: Rvalue::Function("ghost".to_owned()),
        });
    let errors = MirVerifier::verify(&mir).expect_err("unknown MIR callable must be rejected");
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            MirVerificationKind::UnknownFunctionReference { name } if name == "ghost"
        )),
        "{errors:#?}"
    );
}

#[test]
fn public_interpreter_rejects_duplicate_function_mir() {
    let mut mir = sample();
    mir.functions.push(mir.functions[0].clone());
    let mut interpreter = Interpreter::new(&mir);
    let error = interpreter
        .run_main()
        .expect_err("direct execution must reject duplicate callable names");
    assert_eq!(error.code, "TP-E0600");
}

#[test]
fn rejects_returning_a_local_before_assignment() {
    let mut mir = sample();
    let function = &mut mir.functions[0];
    let entry = function.entry;
    function.blocks[entry.0 as usize].statements.clear();
    let errors = MirVerifier::verify(&mir).expect_err("local was never assigned");
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            MirVerificationKind::ReadBeforeAssignment { .. }
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_missing_assignment_on_one_branch_of_a_join() {
    let mut mir = Compiler::new()
        .lower_mir_source("main.tp", "fn main() -> i64 { if true { 1 } else { 2 } }")
        .expect("valid if must lower");
    let function = &mut mir.functions[0];
    let branch = function
        .blocks
        .iter()
        .find_map(|block| match &block.terminator {
            Terminator::Branch { then_block, .. } => Some(*then_block),
            _ => None,
        })
        .expect("if contains a then branch");
    let result_local = function
        .blocks
        .iter()
        .find_map(|block| match &block.terminator {
            Terminator::Return(Some(Operand::Local(local))) => Some(*local),
            _ => None,
        })
        .expect("return reads join result");
    let statements = &mut function.blocks[branch.0 as usize].statements;
    let before = statements.len();
    statements.retain(|statement| {
        !matches!(statement, MirStatement::Assign { target, .. } if *target == result_local)
    });
    assert!(
        statements.len() < before,
        "test must remove a join assignment"
    );
    let errors = MirVerifier::verify(&mir).expect_err("branch join reads an unassigned local");
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            MirVerificationKind::ReadBeforeAssignment { local, .. } if *local == result_local
        )),
        "{errors:#?}"
    );
}

#[test]
fn accepts_match_arm_bindings_as_initialized_on_success_edges() {
    let mir = Compiler::new()
        .lower_mir_source(
            "main.tp",
            "enum Opt<T> { Some(T), None } fn main() -> i64 { match Some(3) { Some(v) => v, None => 0 } }",
        )
        .expect("valid enum match must lower");
    assert!(MirVerifier::verify(&mir).is_ok());
}

#[test]
fn accepts_initialized_mutable_locals_through_loops() {
    let mir = Compiler::new()
        .lower_mir_source(
            "main.tp",
            "fn main() -> i64 { var x = 0; while x < 3 { x = x + 1; } x }",
        )
        .expect("valid loop must lower");
    assert!(MirVerifier::verify(&mir).is_ok());
}

#[test]
fn rejects_wrong_primitive_constant_assigned_to_local() {
    let mut mir = sample();
    let function = &mut mir.functions[0];
    let entry = function.entry;
    function.blocks[entry.0 as usize].statements[0] = MirStatement::Assign {
        target: LocalId(0),
        value: Rvalue::Use(Operand::Constant(Constant::Bool(true))),
    };
    let errors = MirVerifier::verify(&mir).expect_err("Bool cannot initialize an i64 local");
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            MirVerificationKind::InvalidAssignmentType { expected, actual, .. }
                if *expected == Type::I64 && *actual == Type::Bool
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_wrong_primitive_local_assigned_to_local() {
    let mut mir = Compiler::new()
        .lower_mir_source("main.tp", "fn main() -> i64 { let a = true; let b = 7; b }")
        .expect("valid source must lower");
    let function = &mut mir.functions[0];
    let bool_local = function
        .locals
        .iter()
        .find(|local| local.ty == Type::Bool)
        .expect("Bool local")
        .id;
    let int_local = function
        .locals
        .iter()
        .find(|local| local.ty == Type::I64)
        .expect("i64 local")
        .id;
    let entry = function.entry;
    function.blocks[entry.0 as usize]
        .statements
        .push(MirStatement::Assign {
            target: int_local,
            value: Rvalue::Use(Operand::Local(bool_local)),
        });
    let errors = MirVerifier::verify(&mir).expect_err("Bool cannot assign into i64 local");
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            MirVerificationKind::InvalidAssignmentType { expected, actual, .. }
                if *expected == Type::I64 && *actual == Type::Bool
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_boolean_not_applied_to_integer() {
    let mut mir = sample();
    let function = &mut mir.functions[0];
    let entry = function.entry;
    function.blocks[entry.0 as usize].statements[0] = MirStatement::Assign {
        target: LocalId(0),
        value: Rvalue::Unary { op: UnaryOp::Not, operand: Operand::Constant(Constant::I64(7)) },
    };
    let errors = MirVerifier::verify(&mir).expect_err("malformed primitive operation must fail");
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            MirVerificationKind::InvalidUnaryOperandType { actual, .. } if *actual == Type::I64
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_numeric_negation_of_boolean() {
    let mut mir = sample();
    let function = &mut mir.functions[0];
    let entry = function.entry;
    function.blocks[entry.0 as usize].statements[0] = MirStatement::Assign {
        target: LocalId(0),
        value: Rvalue::Unary { op: UnaryOp::Negate, operand: Operand::Constant(Constant::Bool(true)) },
    };
    let errors = MirVerifier::verify(&mir).expect_err("malformed primitive operation must fail");
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            MirVerificationKind::InvalidUnaryOperandType { actual, .. } if *actual == Type::Bool
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_boolean_and_of_integers() {
    let mut mir = sample();
    let function = &mut mir.functions[0];
    let entry = function.entry;
    function.blocks[entry.0 as usize].statements[0] = MirStatement::Assign {
        target: LocalId(0),
        value: Rvalue::Binary { op: BinaryOp::And, left: Operand::Constant(Constant::I64(1)), right: Operand::Constant(Constant::I64(2)) },
    };
    let errors = MirVerifier::verify(&mir).expect_err("malformed primitive operation must fail");
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            MirVerificationKind::InvalidBinaryOperandType { .. }
        )),
        "{errors:#?}"
    );
}

#[test]
fn rejects_mixed_numeric_arithmetic() {
    let mut mir = sample();
    let function = &mut mir.functions[0];
    let entry = function.entry;
    function.blocks[entry.0 as usize].statements[0] = MirStatement::Assign {
        target: LocalId(0),
        value: Rvalue::Binary { op: BinaryOp::Add, left: Operand::Constant(Constant::I64(1)), right: Operand::Constant(Constant::Bool(true)) },
    };
    let errors = MirVerifier::verify(&mir).expect_err("malformed primitive operation must fail");
    assert!(
        errors.iter().any(|error| matches!(
            &error.kind,
            MirVerificationKind::InvalidBinaryOperandType { .. }
        )),
        "{errors:#?}"
    );
}
