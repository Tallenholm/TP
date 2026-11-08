use std::{
    collections::{HashMap, HashSet},
    fmt,
};

use crate::{
    HirBlock, HirExpr, HirExprKind, HirFunction, HirGlobalKind, HirModule, HirPattern,
    HirPatternKind, HirStmt, SemanticModel, SourceId, SourceManager, Span, SymbolId, Type,
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
    UnknownSource {
        source: SourceId,
    },
    InvalidSourceBoundary {
        start: usize,
        end: usize,
        source_len: usize,
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
    ExpressionSemanticMismatch {
        expected: Option<Type>,
        actual: Type,
    },
    BlockSemanticMismatch {
        expected: Option<Type>,
        actual: Type,
    },
    BindingSemanticMismatch {
        expected: Option<(SymbolId, Type)>,
        actual_symbol: SymbolId,
        actual_type: Type,
    },
    GlobalSemanticMismatch {
        expected: Option<HirGlobalKind>,
        actual: HirGlobalKind,
    },
    FunctionSemanticMismatch {
        expected: Option<Type>,
        actual: Type,
    },
    ParameterSemanticMismatch {
        expected: Option<(SymbolId, Type)>,
        actual_symbol: SymbolId,
        actual_type: Type,
    },
    PatternSemanticMismatch {
        expected: Option<(SymbolId, Type)>,
        actual_symbol: SymbolId,
        actual_type: Type,
    },
    LocalSemanticMismatch {
        expected: Option<SymbolId>,
        actual: SymbolId,
    },
}

impl fmt::Display for HirVerificationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSpan { start, end } => {
                write!(f, "reversed source span {start}..{end}")
            }
            Self::UnknownSource { source } => {
                write!(f, "span refers to unknown source {}", source.raw())
            }
            Self::InvalidSourceBoundary {
                start,
                end,
                source_len,
            } => write!(
                f,
                "span {start}..{end} lies outside UTF-8 source boundaries (length {source_len})"
            ),
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
            Self::ExpressionSemanticMismatch { expected, actual } => {
                write!(
                    f,
                    "HIR expression type {actual} differs from checked type {expected:?}"
                )
            }
            Self::BlockSemanticMismatch { expected, actual } => {
                write!(
                    f,
                    "HIR block type {actual} differs from checked type {expected:?}"
                )
            }
            Self::BindingSemanticMismatch {
                expected,
                actual_symbol,
                actual_type,
            } => write!(
                f,
                "HIR binding {}: {actual_type} differs from checked binding {expected:?}",
                actual_symbol.raw()
            ),
            Self::GlobalSemanticMismatch { expected, actual } => {
                write!(
                    f,
                    "HIR global kind {actual:?} differs from checked {expected:?}"
                )
            }
            Self::FunctionSemanticMismatch { expected, actual } => {
                write!(f, "HIR function result {actual} differs from checked {expected:?}")
            }
            Self::ParameterSemanticMismatch {
                expected,
                actual_symbol,
                actual_type,
            } => write!(
                f,
                "HIR parameter {}: {actual_type} differs from checked {expected:?}",
                actual_symbol.raw()
            ),
            Self::PatternSemanticMismatch {
                expected,
                actual_symbol,
                actual_type,
            } => write!(
                f,
                "HIR pattern {}: {actual_type} differs from checked {expected:?}",
                actual_symbol.raw()
            ),
            Self::LocalSemanticMismatch { expected, actual } => write!(
                f,
                "HIR local resolution {} differs from checked {expected:?}",
                actual.raw()
            ),
        }
    }
}

#[derive(Debug, Clone)]
struct Binding {
    ty: Type,
    mutable: bool,
}

struct FunctionVerifier<'a> {
    function: String,
    sources: Option<&'a SourceManager>,
    semantic: Option<&'a SemanticModel>,
    errors: Vec<HirVerificationError>,
    seen: HashSet<SymbolId>,
    visible: HashMap<SymbolId, Binding>,
    scopes: Vec<Vec<SymbolId>>,
}

pub struct HirVerifier;

impl HirVerifier {
    pub fn verify(module: &HirModule) -> Result<(), Vec<HirVerificationError>> {
        Self::verify_inner(module, None, None)
    }

    pub fn verify_with_sources(
        module: &HirModule,
        sources: &SourceManager,
    ) -> Result<(), Vec<HirVerificationError>> {
        Self::verify_inner(module, Some(sources), None)
    }

    pub fn verify_checked(
        module: &HirModule,
        semantic: &SemanticModel,
        sources: &SourceManager,
    ) -> Result<(), Vec<HirVerificationError>> {
        Self::verify_inner(module, Some(sources), Some(semantic))
    }

    fn verify_inner(
        module: &HirModule,
        sources: Option<&SourceManager>,
        semantic: Option<&SemanticModel>,
    ) -> Result<(), Vec<HirVerificationError>> {
        let mut errors = Vec::new();

        for function in &module.functions {
            let mut verifier = FunctionVerifier::new(&function.name, sources, semantic);
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

impl<'a> FunctionVerifier<'a> {
    fn new(
        name: &str,
        sources: Option<&'a SourceManager>,
        semantic: Option<&'a SemanticModel>,
    ) -> Self {
        Self {
            function: name.to_owned(),
            sources,
            semantic,
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

        let Some(sources) = self.sources else {
            return;
        };
        let Some(source) = sources.get(span.source) else {
            self.error(
                span,
                HirVerificationKind::UnknownSource {
                    source: span.source,
                },
            );
            return;
        };
        let text = source.text();
        let source_len = text.len();
        if span.start > source_len
            || span.end > source_len
            || !text.is_char_boundary(span.start)
            || !text.is_char_boundary(span.end)
        {
            self.error(
                span,
                HirVerificationKind::InvalidSourceBoundary {
                    start: span.start,
                    end: span.end,
                    source_len,
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
        if let Some(semantic) = self.semantic {
            let expected = semantic
                .function(&function.name)
                .map(|(_, return_type)| return_type.clone());
            if expected.as_ref() != Some(&function.return_type) {
                self.error(
                    function.span,
                    HirVerificationKind::FunctionSemanticMismatch {
                        expected,
                        actual: function.return_type.clone(),
                    },
                );
            }
        }
        self.begin_scope();
        for param in &function.params {
            self.check_span(param.span);
            if let Some(semantic) = self.semantic {
                let expected = semantic
                    .binding(param.span)
                    .map(|(id, checked_ty)| (id, checked_ty.clone()));
                if expected.as_ref() != Some(&(param.symbol, param.ty.clone())) {
                    self.error(
                        param.span,
                        HirVerificationKind::ParameterSemanticMismatch {
                            expected,
                            actual_symbol: param.symbol,
                            actual_type: param.ty.clone(),
                        },
                    );
                }
            }
            self.declare(param.symbol, param.ty.clone(), false, param.span);
        }
        self.verify_block(&function.body);
        self.end_scope();
    }

    fn verify_block(&mut self, block: &HirBlock) {
        self.check_span(block.span);
        if let Some(semantic) = self.semantic {
            let expected = semantic.block_type(block.span).cloned();
            if expected.as_ref() != Some(&block.ty) {
                self.error(
                    block.span,
                    HirVerificationKind::BlockSemanticMismatch {
                        expected,
                        actual: block.ty.clone(),
                    },
                );
            }
        }
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
                    if let Some(semantic) = self.semantic {
                        let expected = semantic
                            .binding(*span)
                            .map(|(id, checked_ty)| (id, checked_ty.clone()));
                        if expected.as_ref() != Some(&(*symbol, ty.clone())) {
                            self.error(
                                *span,
                                HirVerificationKind::BindingSemanticMismatch {
                                    expected,
                                    actual_symbol: *symbol,
                                    actual_type: ty.clone(),
                                },
                            );
                        }
                    }
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
        if let Some(semantic) = self.semantic {
            let expected = semantic.expr_type(expr.span).cloned();
            if expected.as_ref() != Some(&expr.ty) {
                self.error(
                    expr.span,
                    HirVerificationKind::ExpressionSemanticMismatch {
                        expected,
                        actual: expr.ty.clone(),
                    },
                );
            }
        }

        match &expr.kind {
            HirExprKind::Integer(_) => self.check_literal(expr, Type::I64),
            HirExprKind::Float(_) => self.check_literal(expr, Type::F64),
            HirExprKind::Bool(_) => self.check_literal(expr, Type::Bool),
            HirExprKind::String(_) => self.check_literal(expr, Type::String),
            HirExprKind::Local(symbol) => {
                self.check_local(*symbol, &expr.ty, expr.span);
                self.check_local_resolution(expr.span, *symbol);
            }
            HirExprKind::Global { kind, .. } => {
                if let Some(semantic) = self.semantic {
                    let expected = semantic.global(expr.span);
                    if expected != Some(*kind) {
                        self.error(
                            expr.span,
                            HirVerificationKind::GlobalSemanticMismatch {
                                expected,
                                actual: *kind,
                            },
                        );
                    }
                }
            }
            HirExprKind::Unary { expr: inner, .. } => self.verify_expr(inner),
            HirExprKind::Binary { left, right, .. } => {
                self.verify_expr(left);
                self.verify_expr(right);
            }
            HirExprKind::Assign { symbol, value } => {
                // Assignment RHS is evaluated before mutating the target.
                self.verify_expr(value);
                self.check_local_resolution(expr.span, *symbol);
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
                if let Some(semantic) = self.semantic {
                    let expected = semantic
                        .binding(pattern.span)
                        .map(|(id, checked_ty)| (id, checked_ty.clone()));
                    if expected.as_ref() != Some(&(*symbol, ty.clone())) {
                        self.error(
                            pattern.span,
                            HirVerificationKind::PatternSemanticMismatch {
                                expected,
                                actual_symbol: *symbol,
                                actual_type: ty.clone(),
                            },
                        );
                    }
                }
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

    fn check_local_resolution(&mut self, span: Span, actual: SymbolId) {
        if let Some(semantic) = self.semantic {
            let expected = semantic.local_use(span);
            if expected != Some(actual) {
                self.error(
                    span,
                    HirVerificationKind::LocalSemanticMismatch { expected, actual },
                );
            }
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
