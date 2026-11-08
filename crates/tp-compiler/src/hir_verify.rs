use std::{
    collections::{HashMap, HashSet},
    fmt,
};

use crate::{
    HirBlock, HirExpr, HirExprKind, HirFunction, HirModule, HirPattern, HirPatternKind, HirStmt,
    Span, SymbolId, Type,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirVerificationError {
    pub function: String,
    pub span: Span,
    pub kind: HirVerificationKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirVerificationKind {
    InvalidSpan {
        start: usize,
        end: usize,
    },
    DuplicateSymbol {
        symbol: SymbolId,
    },
    UnboundLocal {
        symbol: SymbolId,
    },
    LocalTypeMismatch {
        symbol: SymbolId,
        expected: Type,
        actual: Type,
    },
    LiteralTypeMismatch {
        expected: Type,
        actual: Type,
    },
    InvalidConditionType {
        actual: Type,
    },
    ImmutableAssignment {
        symbol: SymbolId,
    },
}

impl fmt::Display for HirVerificationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSpan { start, end } => {
                write!(f, "reversed source span {start}..{end}")
            }
            Self::DuplicateSymbol { symbol } => {
                write!(f, "symbol {} is declared more than once", symbol.raw())
            }
            Self::UnboundLocal { symbol } => {
                write!(f, "local symbol {} is not in scope", symbol.raw())
            }
            Self::LocalTypeMismatch {
                symbol,
                expected,
                actual,
            } => write!(
                f,
                "local symbol {} is typed {actual} but declared {expected}",
                symbol.raw()
            ),
            Self::LiteralTypeMismatch { expected, actual } => {
                write!(f, "literal is typed {actual} but should be {expected}")
            }
            Self::InvalidConditionType { actual } => {
                write!(f, "conditional expression requires Bool, found {actual}")
            }
            Self::ImmutableAssignment { symbol } => {
                write!(f, "assignment targets immutable symbol {}", symbol.raw())
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Binding {
    ty: Type,
    mutable: bool,
}

struct FunctionVerifier {
    function: String,
    errors: Vec<HirVerificationError>,
    seen: HashSet<SymbolId>,
    visible: HashMap<SymbolId, Binding>,
    scopes: Vec<Vec<SymbolId>>,
}

pub struct HirVerifier;

impl HirVerifier {
    pub fn verify(module: &HirModule) -> Result<(), Vec<HirVerificationError>> {
        let mut errors = Vec::new();

        for function in &module.functions {
            let mut verifier = FunctionVerifier::new(&function.name);
            verifier.verify_function(function);
            errors.extend(verifier.errors);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl FunctionVerifier {
    fn new(name: &str) -> Self {
        Self {
            function: name.to_owned(),
            errors: Vec::new(),
            seen: HashSet::new(),
            visible: HashMap::new(),
            scopes: Vec::new(),
        }
    }

    fn error(&mut self, span: Span, kind: HirVerificationKind) {
        self.errors.push(HirVerificationError {
            function: self.function.clone(),
            span,
            kind,
        });
    }

    fn check_span(&mut self, span: Span) {
        if span.start > span.end {
            self.error(
                span,
                HirVerificationKind::InvalidSpan {
                    start: span.start,
                    end: span.end,
                },
            );
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(Vec::new());
    }

    fn end_scope(&mut self) {
        if let Some(ids) = self.scopes.pop() {
            for id in ids {
                self.visible.remove(&id);
            }
        }
    }

    fn declare(&mut self, id: SymbolId, ty: Type, mutable: bool, span: Span) {
        if !self.seen.insert(id) {
            self.error(span, HirVerificationKind::DuplicateSymbol { symbol: id });
            return;
        }
        self.visible.insert(id, Binding { ty, mutable });
        if let Some(scope) = self.scopes.last_mut() {
            scope.push(id);
        }
    }

    fn verify_function(&mut self, function: &HirFunction) {
        self.check_span(function.span);
        self.begin_scope();
        for param in &function.params {
            self.check_span(param.span);
            self.declare(param.symbol, param.ty.clone(), false, param.span);
        }
        self.verify_block(&function.body);
        self.end_scope();
    }

    fn verify_block(&mut self, block: &HirBlock) {
        self.check_span(block.span);
        self.begin_scope();

        for statement in &block.statements {
            match statement {
                HirStmt::Let {
                    symbol,
                    mutable,
                    ty,
                    value,
                    span,
                    ..
                } => {
                    self.check_span(*span);
                    // The initializer runs before its binding becomes visible.
                    // Explicit annotations can be more precise than generic
                    // constructors (for instance: Maybe<i64> initialized by None).
                    self.verify_expr(value);
                    self.declare(*symbol, ty.clone(), *mutable, *span);
                }
                HirStmt::Return { value, span } => {
                    self.check_span(*span);
                    if let Some(value) = value {
                        self.verify_expr(value);
                    }
                }
                HirStmt::While {
                    condition,
                    body,
                    span,
                } => {
                    self.check_span(*span);
                    self.verify_expr(condition);
                    self.check_condition(condition);
                    self.verify_block(body);
                }
                HirStmt::Expr { expr, .. } => self.verify_expr(expr),
            }
        }

        self.end_scope();
    }

    fn verify_expr(&mut self, expr: &HirExpr) {
        self.check_span(expr.span);

        match &expr.kind {
            HirExprKind::Integer(_) => self.check_literal(expr, Type::I64),
            HirExprKind::Float(_) => self.check_literal(expr, Type::F64),
            HirExprKind::Bool(_) => self.check_literal(expr, Type::Bool),
            HirExprKind::String(_) => self.check_literal(expr, Type::String),
            HirExprKind::Local(symbol) => {
                self.check_local(*symbol, &expr.ty, expr.span);
            }
            HirExprKind::Global { .. } => {}
            HirExprKind::Unary { expr: inner, .. } => self.verify_expr(inner),
            HirExprKind::Binary { left, right, .. } => {
                self.verify_expr(left);
                self.verify_expr(right);
            }
            HirExprKind::Assign { symbol, value } => {
                // Assignment RHS is evaluated before mutating the target.
                self.verify_expr(value);
                if let Some(binding) = self.visible.get(symbol).cloned() {
                    if !binding.mutable {
                        self.error(
                            expr.span,
                            HirVerificationKind::ImmutableAssignment { symbol: *symbol },
                        );
                    }
                } else {
                    self.error(
                        expr.span,
                        HirVerificationKind::UnboundLocal { symbol: *symbol },
                    );
                }
            }
            HirExprKind::Call { callee, args } => {
                self.verify_expr(callee);
                for arg in args {
                    self.verify_expr(arg);
                }
            }
            HirExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.verify_expr(condition);
                self.check_condition(condition);
                self.verify_block(then_branch);
                if let Some(else_branch) = else_branch {
                    self.verify_block(else_branch);
                }
            }
            HirExprKind::StructLiteral { fields, .. } => {
                for (_, value) in fields {
                    self.verify_expr(value);
                }
            }
            HirExprKind::Field { base, .. } => self.verify_expr(base),
            HirExprKind::Match { value, arms } => {
                self.verify_expr(value);
                for arm in arms {
                    self.check_span(arm.span);
                    self.begin_scope();
                    self.verify_pattern(&arm.pattern);
                    self.verify_expr(&arm.body);
                    self.end_scope();
                }
            }
        }
    }

    fn verify_pattern(&mut self, pattern: &HirPattern) {
        self.check_span(pattern.span);
        match &pattern.kind {
            HirPatternKind::Bind { symbol, ty, .. } => {
                self.declare(*symbol, ty.clone(), false, pattern.span);
            }
            HirPatternKind::Variant { args, .. } => {
                for arg in args {
                    self.verify_pattern(arg);
                }
            }
            HirPatternKind::Wildcard
            | HirPatternKind::Integer(_)
            | HirPatternKind::Bool(_)
            | HirPatternKind::String(_) => {}
        }
    }

    fn check_literal(&mut self, expr: &HirExpr, expected: Type) {
        if expr.ty != expected {
            self.error(
                expr.span,
                HirVerificationKind::LiteralTypeMismatch {
                    expected,
                    actual: expr.ty.clone(),
                },
            );
        }
    }

    fn check_condition(&mut self, condition: &HirExpr) {
        // A Never-typed condition is unreachable, but valid under frozen M1 rules.
        if condition.ty != Type::Bool && condition.ty != Type::Never {
            self.error(
                condition.span,
                HirVerificationKind::InvalidConditionType {
                    actual: condition.ty.clone(),
                },
            );
        }
    }

    fn check_local(&mut self, id: SymbolId, actual: &Type, span: Span) {
        match self.visible.get(&id).cloned() {
            Some(binding) if binding.ty != *actual => {
                self.error(
                    span,
                    HirVerificationKind::LocalTypeMismatch {
                        symbol: id,
                        expected: binding.ty,
                        actual: actual.clone(),
                    },
                );
            }
            Some(_) => {}
            None => {
                self.error(span, HirVerificationKind::UnboundLocal { symbol: id });
            }
        }
    }
}
