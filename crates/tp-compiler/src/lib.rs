//! TP compiler library.

mod ast;
mod diagnostic;
mod lexer;
mod parser;
mod pipeline;
mod source;
mod token;

pub use ast::{
    BinaryOp, Block, Expr, ExprKind, FnDecl, Item, Module, Param, Stmt, TypeRef, UnaryOp,
};
pub use diagnostic::{render_diagnostic, Diagnostic, Label, Severity};
pub use lexer::{LexResult, Lexer};
pub use parser::{ParseResult, Parser};
pub use pipeline::{CompileReport, Compiler};
pub use source::{LineCol, SourceFile, SourceId, Span};
pub use token::{Token, TokenKind};
