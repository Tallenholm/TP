use std::{collections::HashSet, fmt};

use crate::{
    BinaryOp, BlockId, Constant, LocalId, MirFunction, MirModule, MirPattern, MirStatement,
    Operand, Rvalue, Span, Terminator, Type, UnaryOp,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirVerificationError {
    pub function: String,
    pub span: Span,
    pub kind: MirVerificationKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirVerificationKind {
    InvalidEntry {
        entry: BlockId,
    },
    BlockIndexMismatch {
        index: usize,
        actual: BlockId,
    },
    LocalIndexMismatch {
        index: usize,
        actual: LocalId,
    },
    InvalidSuccessor {
        from: BlockId,
        target: BlockId,
    },
    InvalidParameterLocal {
        local: LocalId,
    },
    DuplicateParameterLocal {
        local: LocalId,
    },
    InvalidLocalReference {
        block: BlockId,
        local: LocalId,
    },
    DuplicateFunction {
        name: String,
    },
    InvalidBranchCondition {
        block: BlockId,
        actual: Type,
    },
    InvalidReturnType {
        block: BlockId,
        expected: Type,
        actual: Type,
    },
    InvalidAssignmentType {
        block: BlockId,
        target: LocalId,
        expected: Type,
        actual: Type,
    },
    InvalidUnaryOperandType {
        block: BlockId,
        op: UnaryOp,
        actual: Type,
    },
    InvalidBinaryOperandType {
        block: BlockId,
        op: BinaryOp,
        left: Type,
        right: Type,
    },
    UnknownFunctionReference {
        name: String,
    },
    ReadBeforeAssignment {
        block: BlockId,
        local: LocalId,
    },
}

impl fmt::Display for MirVerificationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEntry { entry } => write!(f, "entry block {} does not exist", entry.0),
            Self::BlockIndexMismatch { index, actual } => write!(
                f,
                "block at index {index} has ID {}, but block IDs must equal their indices",
                actual.0
            ),
            Self::LocalIndexMismatch { index, actual } => write!(
                f,
                "local at index {index} has ID {}, but local IDs must equal their indices",
                actual.0
            ),
            Self::InvalidSuccessor { from, target } => {
                write!(f, "block {} targets nonexistent block {}", from.0, target.0)
            }
            Self::InvalidParameterLocal { local } => {
                write!(f, "parameter refers to nonexistent local {}", local.0)
            }
            Self::DuplicateParameterLocal { local } => {
                write!(f, "parameter local {} appears more than once", local.0)
            }
            Self::InvalidLocalReference { block, local } => {
                write!(
                    f,
                    "block {} refers to nonexistent local {}",
                    block.0, local.0
                )
            }
            Self::DuplicateFunction { name } => {
                write!(f, "function `{name}` is declared more than once in MIR")
            }
            Self::InvalidBranchCondition { block, actual } => {
                write!(f, "block {} branches on non-Bool value {actual}", block.0)
            }
            Self::InvalidReturnType {
                block,
                expected,
                actual,
            } => write!(
                f,
                "block {} returns primitive {actual} but function expects {expected}",
                block.0
            ),
            Self::InvalidAssignmentType {
                block,
                target,
                expected,
                actual,
            } => write!(
                f,
                "block {} assigns primitive {actual} to local {} of type {expected}",
                block.0, target.0
            ),
            Self::InvalidUnaryOperandType { block, op, actual } => write!(
                f,
                "block {} uses unary {op:?} with invalid operand type {actual}",
                block.0
            ),
            Self::InvalidBinaryOperandType {
                block,
                op,
                left,
                right,
            } => write!(
                f,
                "block {} uses binary {op:?} with invalid operand types {left} and {right}",
                block.0
            ),
            Self::UnknownFunctionReference { name } => {
                write!(f, "MIR references nonexistent callable `{name}`")
            }
            Self::ReadBeforeAssignment { block, local } => write!(
                f,
                "block {} reads local {} before it is definitely assigned",
                block.0, local.0
            ),
        }
    }
}

pub struct MirVerifier;

impl MirVerifier {
    pub fn verify(module: &MirModule) -> Result<(), Vec<MirVerificationError>> {
        let mut errors = Vec::new();
        let mut unique = HashSet::new();
        for function in &module.functions {
            if !unique.insert(function.name.as_str()) {
                Self::error(
                    function,
                    MirVerificationKind::DuplicateFunction {
                        name: function.name.clone(),
                    },
                    &mut errors,
                );
            }
        }
        for function in &module.functions {
            Self::verify_function(function, &unique, &mut errors);
        }
        // Avoid traversing invalid CFG indices or local IDs. The control-flow
        // proof only runs on structurally verified input.
        if errors.is_empty() {
            for function in &module.functions {
                Self::verify_definite_assignments(function, &mut errors);
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn verify_function(
        function: &MirFunction,
        known_functions: &HashSet<&str>,
        errors: &mut Vec<MirVerificationError>,
    ) {
        for (index, block) in function.blocks.iter().enumerate() {
            if block.id.0 as usize != index {
                Self::error(
                    function,
                    MirVerificationKind::BlockIndexMismatch {
                        index,
                        actual: block.id,
                    },
                    errors,
                );
            }
        }

        for (index, local) in function.locals.iter().enumerate() {
            if local.id.0 as usize != index {
                Self::error(
                    function,
                    MirVerificationKind::LocalIndexMismatch {
                        index,
                        actual: local.id,
                    },
                    errors,
                );
            }
        }

        if !Self::block_exists(function, function.entry) {
            Self::error(
                function,
                MirVerificationKind::InvalidEntry {
                    entry: function.entry,
                },
                errors,
            );
        }

        let mut params = HashSet::new();
        for &param in &function.params {
            if !params.insert(param) {
                Self::error(
                    function,
                    MirVerificationKind::DuplicateParameterLocal { local: param },
                    errors,
                );
            }
            if !Self::local_exists(function, param) {
                Self::error(
                    function,
                    MirVerificationKind::InvalidParameterLocal { local: param },
                    errors,
                );
            }
        }

        for block in &function.blocks {
            for statement in &block.statements {
                let MirStatement::Assign { target, value } = statement;
                Self::check_local(function, block.id, *target, errors);
                Self::check_rvalue(function, known_functions, block.id, value, errors);
                // This deliberately covers only primitive copies. Named and
                // generic values may require contextual typing in frozen M1.
                // Full Rvalue typing belongs to the later Gate 3 type proof.
                if let Rvalue::Use(operand) = value {
                    let expected = function
                        .locals
                        .get(target.0 as usize)
                        .filter(|local| local.id == *target)
                        .map(|local| &local.ty);
                    let actual = Self::operand_type(function, operand);
                    if let (Some(expected), Some(actual)) = (expected, actual) {
                        if Self::is_primitive(expected)
                            && Self::is_primitive(&actual)
                            && *expected != actual
                        {
                            Self::error(
                                function,
                                MirVerificationKind::InvalidAssignmentType {
                                    block: block.id,
                                    target: *target,
                                    expected: expected.clone(),
                                    actual,
                                },
                                errors,
                            );
                        }
                    }
                }
            }
            match &block.terminator {
                Terminator::Unreachable | Terminator::Trap(_) | Terminator::Return(None) => {}
                Terminator::Goto(target) => {
                    Self::check_successor(function, block.id, *target, errors);
                }
                Terminator::Branch {
                    condition,
                    then_block,
                    else_block,
                } => {
                    Self::check_operand(function, block.id, condition, errors);
                    if let Some(actual) = Self::operand_type(function, condition) {
                        if !matches!(actual, Type::Bool | Type::Never) {
                            Self::error(
                                function,
                                MirVerificationKind::InvalidBranchCondition {
                                    block: block.id,
                                    actual,
                                },
                                errors,
                            );
                        }
                    }
                    Self::check_successor(function, block.id, *then_block, errors);
                    Self::check_successor(function, block.id, *else_block, errors);
                }
                Terminator::Match {
                    value,
                    arms,
                    otherwise,
                } => {
                    Self::check_operand(function, block.id, value, errors);
                    for (pattern, target) in arms {
                        Self::check_pattern(function, block.id, pattern, errors);
                        Self::check_successor(function, block.id, *target, errors);
                    }
                    Self::check_successor(function, block.id, *otherwise, errors);
                }
                Terminator::Return(Some(value)) => {
                    Self::check_operand(function, block.id, value, errors);
                    if let Some(actual) = Self::operand_type(function, value) {
                        // Primitive returns have exact types. Generic/named values
                        // can carry context-dependent types in frozen M1 and need
                        // a later, complete type verifier instead of an ad hoc rule.
                        if Self::is_primitive(&actual)
                            && Self::is_primitive(&function.return_type)
                            && actual != function.return_type
                        {
                            Self::error(
                                function,
                                MirVerificationKind::InvalidReturnType {
                                    block: block.id,
                                    expected: function.return_type.clone(),
                                    actual,
                                },
                                errors,
                            );
                        }
                    }
                }
            }
        }
    }

    fn is_primitive(ty: &Type) -> bool {
        matches!(
            ty,
            Type::Unit | Type::Bool | Type::I64 | Type::F64 | Type::String
        )
    }

    fn operand_type(function: &MirFunction, operand: &Operand) -> Option<Type> {
        match operand {
            Operand::Local(local) => function
                .locals
                .get(local.0 as usize)
                .filter(|slot| slot.id == *local)
                .map(|slot| slot.ty.clone()),
            Operand::Constant(value) => Some(match value {
                Constant::Unit => Type::Unit,
                Constant::Bool(_) => Type::Bool,
                Constant::I64(_) => Type::I64,
                Constant::F64(_) => Type::F64,
                Constant::String(_) => Type::String,
            }),
        }
    }

    fn block_exists(function: &MirFunction, block: BlockId) -> bool {
        function
            .blocks
            .get(block.0 as usize)
            .is_some_and(|item| item.id == block)
    }

    fn local_exists(function: &MirFunction, local: LocalId) -> bool {
        function
            .locals
            .get(local.0 as usize)
            .is_some_and(|item| item.id == local)
    }

    fn check_successor(
        function: &MirFunction,
        from: BlockId,
        target: BlockId,
        errors: &mut Vec<MirVerificationError>,
    ) {
        if !Self::block_exists(function, target) {
            Self::error(
                function,
                MirVerificationKind::InvalidSuccessor { from, target },
                errors,
            );
        }
    }

    fn check_local(
        function: &MirFunction,
        block: BlockId,
        local: LocalId,
        errors: &mut Vec<MirVerificationError>,
    ) {
        if !Self::local_exists(function, local) {
            Self::error(
                function,
                MirVerificationKind::InvalidLocalReference { block, local },
                errors,
            );
        }
    }

    fn check_operand(
        function: &MirFunction,
        block: BlockId,
        operand: &Operand,
        errors: &mut Vec<MirVerificationError>,
    ) {
        if let Operand::Local(local) = operand {
            Self::check_local(function, block, *local, errors);
        }
    }

    fn check_rvalue(
        function: &MirFunction,
        known_functions: &HashSet<&str>,
        block: BlockId,
        value: &Rvalue,
        errors: &mut Vec<MirVerificationError>,
    ) {
        match value {
            Rvalue::Use(operand) => {
                Self::check_operand(function, block, operand, errors);
            }
            Rvalue::Unary { op, operand } => {
                Self::check_operand(function, block, operand, errors);
                if let Some(actual) = Self::operand_type(function, operand) {
                    let valid = match op {
                        UnaryOp::Negate => matches!(actual, Type::I64 | Type::F64 | Type::Never),
                        UnaryOp::Not => matches!(actual, Type::Bool | Type::Never),
                    };
                    if !valid && actual != Type::Unknown {
                        Self::error(
                            function,
                            MirVerificationKind::InvalidUnaryOperandType {
                                block,
                                op: *op,
                                actual,
                            },
                            errors,
                        );
                    }
                }
            }
            Rvalue::Function(name) => {
                if name != "print" && !known_functions.contains(name.as_str()) {
                    Self::error(
                        function,
                        MirVerificationKind::UnknownFunctionReference { name: name.clone() },
                        errors,
                    );
                }
            }
            Rvalue::Binary { op, left, right } => {
                Self::check_operand(function, block, left, errors);
                Self::check_operand(function, block, right, errors);
                if let (Some(left_ty), Some(right_ty)) = (
                    Self::operand_type(function, left),
                    Self::operand_type(function, right),
                ) {
                    let valid = match op {
                        BinaryOp::And | BinaryOp::Or => {
                            left_ty == Type::Bool && right_ty == Type::Bool
                        }
                        BinaryOp::Add
                        | BinaryOp::Subtract
                        | BinaryOp::Multiply
                        | BinaryOp::Divide
                        | BinaryOp::Remainder
                        | BinaryOp::Less
                        | BinaryOp::LessEqual
                        | BinaryOp::Greater
                        | BinaryOp::GreaterEqual => {
                            matches!(left_ty, Type::I64 | Type::F64) && left_ty == right_ty
                        }
                        BinaryOp::Equal | BinaryOp::NotEqual => {
                            Self::is_primitive(&left_ty) && left_ty == right_ty
                        }
                    };
                    if !valid
                        && !matches!(left_ty, Type::Never | Type::Unknown)
                        && !matches!(right_ty, Type::Never | Type::Unknown)
                    {
                        Self::error(
                            function,
                            MirVerificationKind::InvalidBinaryOperandType {
                                block,
                                op: *op,
                                left: left_ty,
                                right: right_ty,
                            },
                            errors,
                        );
                    }
                }
            }
            Rvalue::Call { callee, args } => {
                Self::check_operand(function, block, callee, errors);
                for arg in args {
                    Self::check_operand(function, block, arg, errors);
                }
            }
            Rvalue::Struct { fields, .. } => {
                for (_, operand) in fields {
                    Self::check_operand(function, block, operand, errors);
                }
            }
            Rvalue::Field { base, .. } => {
                Self::check_operand(function, block, base, errors);
            }
            Rvalue::Enum { args, .. } => {
                for arg in args {
                    Self::check_operand(function, block, arg, errors);
                }
            }
        }
    }

    fn check_pattern(
        function: &MirFunction,
        block: BlockId,
        pattern: &MirPattern,
        errors: &mut Vec<MirVerificationError>,
    ) {
        match pattern {
            MirPattern::Bind(local) => Self::check_local(function, block, *local, errors),
            MirPattern::Variant { args, .. } => {
                for arg in args {
                    Self::check_pattern(function, block, arg, errors);
                }
            }
            MirPattern::Wildcard
            | MirPattern::Integer(_)
            | MirPattern::Bool(_)
            | MirPattern::String(_) => {}
        }
    }

    fn verify_definite_assignments(function: &MirFunction, errors: &mut Vec<MirVerificationError>) {
        if function.blocks.is_empty() {
            return;
        }

        let entry = function.entry.0 as usize;
        let local_count = function.locals.len();
        let block_count = function.blocks.len();
        let mut reachable = vec![false; block_count];
        let mut predecessors: Vec<Vec<(usize, Vec<LocalId>)>> = vec![Vec::new(); block_count];
        let mut pending = vec![entry];

        // Build the reachable CFG with match-pattern bindings as values
        // assigned on the successful arm edge, not before the match.
        while let Some(index) = pending.pop() {
            if reachable[index] {
                continue;
            }
            reachable[index] = true;
            for (destination, bindings) in Self::successor_edges(&function.blocks[index].terminator)
            {
                let target = destination.0 as usize;
                predecessors[target].push((index, bindings));
                if !reachable[target] {
                    pending.push(target);
                }
            }
        }

        // Must-analysis uses intersection at joins. Initialize non-entry
        // blocks to the top element so loop back-edges converge correctly.
        let mut incoming = vec![vec![true; local_count]; block_count];
        incoming[entry].fill(false);
        for param in &function.params {
            incoming[entry][param.0 as usize] = true;
        }

        loop {
            let mut changed = false;
            for block_index in 0..block_count {
                if !reachable[block_index] || block_index == entry {
                    continue;
                }
                let mut current = vec![true; local_count];
                for (predecessor, bindings) in &predecessors[block_index] {
                    let mut outgoing = incoming[*predecessor].clone();
                    for statement in &function.blocks[*predecessor].statements {
                        let MirStatement::Assign { target, .. } = statement;
                        outgoing[target.0 as usize] = true;
                    }
                    for binding in bindings {
                        outgoing[binding.0 as usize] = true;
                    }
                    for (slot, defined) in current.iter_mut().zip(outgoing) {
                        *slot &= defined;
                    }
                }
                if current != incoming[block_index] {
                    incoming[block_index] = current;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        // Validate every local use at its actual evaluation point, then mark
        // assignment targets initialized only after evaluating the RHS.
        for (index, block) in function.blocks.iter().enumerate() {
            if !reachable[index] {
                continue;
            }
            let mut defined = incoming[index].clone();
            for statement in &block.statements {
                let MirStatement::Assign { target, value } = statement;
                for operand in Self::rvalue_operands(value) {
                    Self::check_definite_read(function, block.id, operand, &defined, errors);
                }
                defined[target.0 as usize] = true;
            }
            for operand in Self::terminator_operands(&block.terminator) {
                Self::check_definite_read(function, block.id, operand, &defined, errors);
            }
        }
    }

    fn successor_edges(terminator: &Terminator) -> Vec<(BlockId, Vec<LocalId>)> {
        match terminator {
            Terminator::Goto(target) => vec![(*target, Vec::new())],
            Terminator::Branch {
                then_block,
                else_block,
                ..
            } => vec![(*then_block, Vec::new()), (*else_block, Vec::new())],
            Terminator::Match {
                arms, otherwise, ..
            } => {
                let mut edges = Vec::with_capacity(arms.len() + 1);
                for (pattern, target) in arms {
                    let mut bindings = Vec::new();
                    Self::pattern_bindings(pattern, &mut bindings);
                    edges.push((*target, bindings));
                }
                edges.push((*otherwise, Vec::new()));
                edges
            }
            Terminator::Unreachable | Terminator::Return(_) | Terminator::Trap(_) => Vec::new(),
        }
    }

    fn pattern_bindings(pattern: &MirPattern, output: &mut Vec<LocalId>) {
        match pattern {
            MirPattern::Bind(local) => output.push(*local),
            MirPattern::Variant { args, .. } => {
                for pattern in args {
                    Self::pattern_bindings(pattern, output);
                }
            }
            MirPattern::Wildcard
            | MirPattern::Integer(_)
            | MirPattern::Bool(_)
            | MirPattern::String(_) => {}
        }
    }

    fn rvalue_operands(value: &Rvalue) -> Vec<&Operand> {
        match value {
            Rvalue::Use(operand) | Rvalue::Unary { operand, .. } => vec![operand],
            Rvalue::Function(_) => Vec::new(),
            Rvalue::Binary { left, right, .. } => vec![left, right],
            Rvalue::Call { callee, args } => std::iter::once(callee).chain(args.iter()).collect(),
            Rvalue::Struct { fields, .. } => fields.iter().map(|(_, operand)| operand).collect(),
            Rvalue::Field { base, .. } => vec![base],
            Rvalue::Enum { args, .. } => args.iter().collect(),
        }
    }

    fn terminator_operands(terminator: &Terminator) -> Vec<&Operand> {
        match terminator {
            Terminator::Branch { condition, .. } => vec![condition],
            Terminator::Match { value, .. } => vec![value],
            Terminator::Return(Some(value)) => vec![value],
            Terminator::Unreachable
            | Terminator::Goto(_)
            | Terminator::Return(None)
            | Terminator::Trap(_) => Vec::new(),
        }
    }

    fn check_definite_read(
        function: &MirFunction,
        block: BlockId,
        operand: &Operand,
        defined: &[bool],
        errors: &mut Vec<MirVerificationError>,
    ) {
        if let Operand::Local(local) = operand {
            if !defined[local.0 as usize] {
                Self::error(
                    function,
                    MirVerificationKind::ReadBeforeAssignment {
                        block,
                        local: *local,
                    },
                    errors,
                );
            }
        }
    }

    fn error(
        function: &MirFunction,
        kind: MirVerificationKind,
        errors: &mut Vec<MirVerificationError>,
    ) {
        errors.push(MirVerificationError {
            function: function.name.clone(),
            span: function.span,
            kind,
        });
    }
}
