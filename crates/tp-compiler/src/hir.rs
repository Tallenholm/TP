use crate::{BinaryOp, EnumDecl, Span, StructDecl, SymbolId, Type, UnaryOp};

#[derive(Debug, Clone, PartialEq)]
pub struct HirModule {
    pub functions: Vec<HirFunction>,
    pub structs: Vec<StructDecl>,
    pub enums: Vec<EnumDecl>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirFunction {
    pub name: String,
    pub params: Vec<HirParam>,
    pub return_type: Type,
    pub body: HirBlock,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirParam {
    pub symbol: SymbolId,
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirBlock {
    pub statements: Vec<HirStmt>,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirStmt {
    Let {
        symbol: SymbolId,
        name: String,
        mutable: bool,
        value: HirExpr,
        span: Span,
    },
    Return {
        value: Option<HirExpr>,
        span: Span,
    },
    While {
        condition: HirExpr,
        body: HirBlock,
        span: Span,
    },
    Expr {
        expr: HirExpr,
        terminated: bool,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirExpr {
    pub ty: Type,
    pub kind: HirExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirExprKind {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Local(SymbolId),
    Global(String),
    Unary {
        op: UnaryOp,
        expr: Box<HirExpr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<HirExpr>,
        right: Box<HirExpr>,
    },
    Assign {
        symbol: SymbolId,
        value: Box<HirExpr>,
    },
    Call {
        callee: Box<HirExpr>,
        args: Vec<HirExpr>,
    },
    If {
        condition: Box<HirExpr>,
        then_branch: HirBlock,
        else_branch: Option<HirBlock>,
    },
    StructLiteral {
        type_name: String,
        fields: Vec<(String, HirExpr)>,
    },
    Field {
        base: Box<HirExpr>,
        field: String,
    },
    Match {
        value: Box<HirExpr>,
        arms: Vec<HirMatchArm>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirMatchArm {
    pub pattern: HirPattern,
    pub body: HirExpr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirPattern {
    pub kind: HirPatternKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirPatternKind {
    Wildcard,
    Bind {
        symbol: SymbolId,
        name: String,
    },
    Integer(i64),
    Bool(bool),
    String(String),
    Variant {
        name: String,
        args: Vec<HirPattern>,
    },
}
