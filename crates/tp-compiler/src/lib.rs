//! TP compiler library.

mod ast;
mod diagnostic;
mod hir;
mod lexer;
mod lower_hir;
mod parser;
mod pipeline;
mod source;
mod symbol;
mod token;
mod typecheck;
mod types;

pub use ast::{
    BinaryOp, Block, EnumDecl, Expr, ExprKind, FieldDecl, FnDecl, ImportDecl, Item, MatchArm,
    Module, Param, Pattern, PatternKind, Stmt, StructDecl, TypeRef, UnaryOp, VariantDecl,
};
pub use diagnostic::{render_diagnostic, Diagnostic, Label, Severity};
pub use hir::{
    HirBlock, HirExpr, HirExprKind, HirFunction, HirMatchArm, HirModule, HirParam, HirPattern,
    HirPatternKind, HirStmt,
};
pub use lexer::{LexResult, Lexer};
pub use lower_hir::HirLowerer;
pub use parser::{ParseResult, Parser};
pub use pipeline::{CompileReport, Compiler};
pub use source::{LineCol, SourceFile, SourceId, Span};
pub use symbol::{Binding, Scopes, SymbolId};
pub use token::{Token, TokenKind};
pub use typecheck::{TypeCheckResult, TypeChecker};
pub use types::Type;
