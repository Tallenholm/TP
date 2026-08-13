use crate::{BinaryOp, EnumDecl, Span, StructDecl, Type, UnaryOp};

#[derive(Debug, Clone, PartialEq)]
pub struct MirModule {
    pub functions: Vec<MirFunction>,
    pub structs: Vec<StructDecl>,
    pub enums: Vec<EnumDecl>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirFunction {
    pub name: String,
    pub params: Vec<LocalId>,
    pub locals: Vec<MirLocal>,
    pub blocks: Vec<BasicBlock>,
    pub entry: BlockId,
    pub return_type: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirLocal {
    pub id: LocalId,
    pub name: Option<String>,
    pub ty: Type,
    pub mutable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LocalId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockId(pub u32);

#[derive(Debug, Clone, PartialEq)]
pub struct BasicBlock {
    pub id: BlockId,
    pub statements: Vec<MirStatement>,
    pub terminator: Terminator,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirStatement {
    Assign {
        target: LocalId,
        value: Rvalue,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Rvalue {
    Use(Operand),
    Unary {
        op: UnaryOp,
        operand: Operand,
    },
    Binary {
        op: BinaryOp,
        left: Operand,
        right: Operand,
    },
    Call {
        callee: String,
        args: Vec<Operand>,
    },
    Struct {
        type_name: String,
        fields: Vec<(String, Operand)>,
    },
    Field {
        base: Operand,
        field: String,
    },
    Enum {
        variant: String,
        args: Vec<Operand>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Local(LocalId),
    Constant(Constant),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Unit,
    Bool(bool),
    I64(i64),
    F64(f64),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Terminator {
    Unreachable,
    Goto(BlockId),
    Branch {
        condition: Operand,
        then_block: BlockId,
        else_block: BlockId,
    },
    Match {
        value: Operand,
        arms: Vec<(MirPattern, BlockId)>,
        otherwise: BlockId,
    },
    Return(Option<Operand>),
    Trap(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirPattern {
    Wildcard,
    Bind(LocalId),
    Integer(i64),
    Bool(bool),
    String(String),
    Variant {
        name: String,
        args: Vec<MirPattern>,
    },
}
