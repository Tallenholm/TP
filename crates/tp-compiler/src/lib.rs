//! TP compiler library.

mod ast;
mod diagnostic;
mod lexer;
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
pub use lexer::{LexResult, Lexer};
pub use parser::{ParseResult, Parser};
pub use pipeline::{CompileReport, Compiler};
pub use source::{LineCol, SourceFile, SourceId, Span};
pub use symbol::{Binding, Scopes, SymbolId};
pub use token::{Token, TokenKind};
pub use typecheck::{TypeCheckResult, TypeChecker};
pub use types::Type;
