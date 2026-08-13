//! TP compiler library.

mod ast;
mod diagnostic;
mod hir;
mod interpreter;
mod lexer;
mod lower_hir;
mod lower_mir;
mod mir;
mod module_loader;
mod parser;
mod pipeline;
mod source;
mod symbol;
mod token;
mod typecheck;
mod types;
mod value;

pub use ast::{
    BinaryOp, Block, EnumDecl, Expr, ExprKind, FieldDecl, FnDecl, ImportDecl, Item, MatchArm,
    Module, Param, Pattern, PatternKind, Stmt, StructDecl, TypeRef, UnaryOp, VariantDecl,
};
pub use diagnostic::{Diagnostic, Label, Severity, render_diagnostic};
pub use hir::{
    HirBlock, HirExpr, HirExprKind, HirFunction, HirMatchArm, HirModule, HirParam, HirPattern,
    HirPatternKind, HirStmt,
};
pub use interpreter::{Interpreter, RuntimeError};
pub use lexer::{LexResult, Lexer};
pub use lower_hir::HirLowerer;
pub use lower_mir::MirLowerer;
pub use mir::{
    BasicBlock, BlockId, Constant, LocalId, MirFunction, MirLocal, MirModule, MirPattern,
    MirStatement, Operand, Rvalue, Terminator,
};
pub use module_loader::ModuleLoader;
pub use parser::{ParseResult, Parser};
pub use pipeline::{CompileReport, Compiler, RunFailure, RunReport};
pub use source::{LineCol, SourceFile, SourceId, Span};
pub use symbol::{Binding, Scopes, SymbolId};
pub use token::{Token, TokenKind};
pub use typecheck::{TypeCheckResult, TypeChecker};
pub use types::Type;
pub use value::Value;
