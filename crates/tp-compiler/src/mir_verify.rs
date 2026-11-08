use std::{collections::HashSet, fmt};

use crate::{
    BlockId, Constant, LocalId, MirFunction, MirModule, MirPattern, MirStatement, Operand, Rvalue,
    Terminator, Type,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirVerificationError {
    pub function: String,
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
    UnknownFunctionReference {
        name: String,
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
            Self::UnknownFunctionReference { name } => {
                write!(f, "MIR references nonexistent callable `{name}`")
            }
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
            Rvalue::Use(operand) | Rvalue::Unary { operand, .. } => {
                Self::check_operand(function, block, operand, errors);
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
            Rvalue::Binary { left, right, .. } => {
                Self::check_operand(function, block, left, errors);
                Self::check_operand(function, block, right, errors);
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

    fn error(
        function: &MirFunction,
        kind: MirVerificationKind,
        errors: &mut Vec<MirVerificationError>,
    ) {
        errors.push(MirVerificationError {
            function: function.name.clone(),
            kind,
        });
    }
}
