use std::collections::HashMap;

use crate::{
    BinaryOp, Block, Diagnostic, Expr, ExprKind, FnDecl, Item, Module, Scopes, Span, Stmt, Type,
    TypeRef, UnaryOp,
};

#[derive(Debug, Clone)]
struct FunctionSig {
    params: Vec<Type>,
    result: Type,
    span: Span,
}

#[derive(Debug, Default)]
pub struct TypeCheckResult {
    pub diagnostics: Vec<Diagnostic>,
}

pub struct TypeChecker {
    functions: HashMap<String, FunctionSig>,
    scopes: Scopes,
    diagnostics: Vec<Diagnostic>,
    expected_return: Type,
}

impl TypeChecker {
    pub fn check_module(module: &Module) -> TypeCheckResult {
        let mut checker = Self {
            functions: HashMap::new(),
            scopes: Scopes::new(),
            diagnostics: Vec::new(),
            expected_return: Type::Unit,
        };
        checker.collect_functions(module);
        checker.check_functions(module);
        TypeCheckResult {
            diagnostics: checker.diagnostics,
        }
    }

    fn collect_functions(&mut self, module: &Module) {
        for item in &module.items {
            let Item::Function(function) = item else {
                continue;
            };
            let sig = FunctionSig {
                params: function
                    .params
                    .iter()
                    .map(|param| self.resolve_type_ref(&param.ty))
                    .collect(),
                result: function
                    .return_type
                    .as_ref()
                    .map(|ty| self.resolve_type_ref(ty))
                    .unwrap_or(Type::Unit),
                span: function.span,
            };
            if let Some(previous) = self.functions.insert(function.name.clone(), sig) {
                self.diagnostics.push(
                    Diagnostic::error(
                        "TP-E0200",
                        format!("duplicate function `{}`", function.name),
                    )
                    .with_primary(function.span)
                    .with_label(previous.span, "previous declaration is here"),
                );
            }
        }
    }

    fn check_functions(&mut self, module: &Module) {
        for item in &module.items {
            if let Item::Function(function) = item {
                self.check_function(function);
            }
        }
    }

    fn check_function(&mut self, function: &FnDecl) {
        let Some(sig) = self.functions.get(&function.name).cloned() else {
            return;
        };
        self.expected_return = sig.result.clone();
        self.scopes.push();
        for (param, ty) in function.params.iter().zip(sig.params.iter()) {
            if self
                .scopes
                .insert(param.name.clone(), ty.clone(), false, param.span)
                .is_err()
            {
                self.diagnostics.push(
                    Diagnostic::error(
                        "TP-E0200",
                        format!("duplicate parameter `{}`", param.name),
                    )
                    .with_primary(param.span),
                );
            }
        }

        let body_ty = self.check_block(&function.body);
        if sig.result != Type::Unit
            && !contains_explicit_return(&function.body)
            && !types_compatible(&sig.result, &body_ty)
        {
            self.type_mismatch(
                function.body.span,
                &sig.result,
                &body_ty,
                "function body has the wrong result type",
            );
        }
        self.scopes.pop();
        self.expected_return = Type::Unit;
    }

    fn check_block(&mut self, block: &Block) -> Type {
        self.scopes.push();
        let mut result = Type::Unit;
        for statement in &block.statements {
            result = self.check_statement(statement);
        }
        self.scopes.pop();
        result
    }

    fn check_statement(&mut self, statement: &Stmt) -> Type {
        match statement {
            Stmt::Let {
                mutable,
                name,
                type_annotation,
                value,
                span,
            } => {
                let value_ty = self.check_expr(value);
                let binding_ty = if let Some(annotation) = type_annotation {
                    let annotated = self.resolve_type_ref(annotation);
                    if !types_compatible(&annotated, &value_ty) {
                        self.type_mismatch(
                            value.span,
                            &annotated,
                            &value_ty,
                            "binding initializer has the wrong type",
                        );
                    }
                    annotated
                } else {
                    value_ty
                };
                if self
                    .scopes
                    .insert(name.clone(), binding_ty, *mutable, *span)
                    .is_err()
                {
                    self.diagnostics.push(
                        Diagnostic::error("TP-E0200", format!("duplicate binding `{name}`"))
                            .with_primary(*span),
                    );
                }
                Type::Unit
            }
            Stmt::Return { value, span } => {
                let actual = value
                    .as_ref()
                    .map(|expr| self.check_expr(expr))
                    .unwrap_or(Type::Unit);
                let expected = self.expected_return.clone();
                if !types_compatible(&expected, &actual) {
                    self.type_mismatch(*span, &expected, &actual, "return type mismatch");
                }
                Type::Unit
            }
            Stmt::While {
                condition, body, ..
            } => {
                let condition_ty = self.check_expr(condition);
                self.require_bool(condition.span, &condition_ty, "while condition");
                self.check_block(body);
                Type::Unit
            }
            Stmt::Expr { expr, terminated } => {
                let ty = self.check_expr(expr);
                if *terminated { Type::Unit } else { ty }
            }
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Type {
        match &expr.kind {
            ExprKind::Integer(_) => Type::I64,
            ExprKind::Float(_) => Type::F64,
            ExprKind::String(_) => Type::String,
            ExprKind::Bool(_) => Type::Bool,
            ExprKind::Name(name) => {
                if let Some(binding) = self.scopes.get(name) {
                    return binding.ty.clone();
                }
                if let Some(function) = self.functions.get(name) {
                    return Type::Function {
                        params: function.params.clone(),
                        result: Box::new(function.result.clone()),
                    };
                }
                if name == "print" {
                    return Type::Function {
                        params: vec![Type::Unknown],
                        result: Box::new(Type::Unit),
                    };
                }
                self.unknown_name(expr.span, name);
                Type::Unknown
            }
            ExprKind::Unary { op, expr: inner } => {
                let inner_ty = self.check_expr(inner);
                match op {
                    UnaryOp::Negate => {
                        if !inner_ty.is_unknown() && !inner_ty.is_numeric() {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "TP-E0300",
                                    format!("unary `-` requires a numeric value, found `{inner_ty}`"),
                                )
                                .with_primary(expr.span),
                            );
                            Type::Unknown
                        } else {
                            inner_ty
                        }
                    }
                    UnaryOp::Not => {
                        self.require_bool(inner.span, &inner_ty, "logical negation");
                        Type::Bool
                    }
                }
            }
            ExprKind::Binary { op, left, right } => {
                let left_ty = self.check_expr(left);
                let right_ty = self.check_expr(right);
                self.check_binary(*op, expr.span, left_ty, right_ty)
            }
            ExprKind::Assign { target, value } => {
                let value_ty = self.check_expr(value);
                let Some(binding) = self.scopes.get(target).cloned() else {
                    self.unknown_name(expr.span, target);
                    return Type::Unknown;
                };
                if !binding.mutable {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "TP-E0300",
                            format!("cannot assign to immutable binding `{target}`"),
                        )
                        .with_primary(expr.span)
                        .with_label(binding.span, "binding declared immutable here"),
                    );
                }
                if !types_compatible(&binding.ty, &value_ty) {
                    self.type_mismatch(
                        value.span,
                        &binding.ty,
                        &value_ty,
                        "assignment type mismatch",
                    );
                }
                binding.ty
            }
            ExprKind::Call { callee, args } => self.check_call(expr.span, callee, args),
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition_ty = self.check_expr(condition);
                self.require_bool(condition.span, &condition_ty, "if condition");
                let then_ty = self.check_block(then_branch);
                let Some(else_branch) = else_branch else {
                    return Type::Unit;
                };
                let else_ty = self.check_block(else_branch);
                if types_compatible(&then_ty, &else_ty) {
                    if then_ty.is_unknown() { else_ty } else { then_ty }
                } else {
                    self.type_mismatch(
                        expr.span,
                        &then_ty,
                        &else_ty,
                        "if branches have incompatible types",
                    );
                    Type::Unknown
                }
            }
            ExprKind::StructLiteral { .. }
            | ExprKind::Field { .. }
            | ExprKind::Match { .. } => Type::Unknown,
        }
    }

    fn check_call(&mut self, span: Span, callee: &Expr, args: &[Expr]) -> Type {
        if let ExprKind::Name(name) = &callee.kind {
            if name == "print" {
                for arg in args {
                    self.check_expr(arg);
                }
                if args.len() != 1 {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "TP-E0300",
                            format!("`print` expects 1 argument, found {}", args.len()),
                        )
                        .with_primary(span),
                    );
                }
                return Type::Unit;
            }
            if let Some(sig) = self.functions.get(name).cloned() {
                if args.len() != sig.params.len() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "TP-E0300",
                            format!(
                                "function `{name}` expects {} arguments, found {}",
                                sig.params.len(),
                                args.len()
                            ),
                        )
                        .with_primary(span),
                    );
                }
                for (index, arg) in args.iter().enumerate() {
                    let actual = self.check_expr(arg);
                    if let Some(expected) = sig.params.get(index) {
                        if !types_compatible(expected, &actual) {
                            self.type_mismatch(
                                arg.span,
                                expected,
                                &actual,
                                format!("argument {} to `{name}` has the wrong type", index + 1),
                            );
                        }
                    }
                }
                return sig.result;
            }
        }

        let callee_ty = self.check_expr(callee);
        match callee_ty {
            Type::Function { params, result } => {
                if args.len() != params.len() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "TP-E0300",
                            format!("call expects {} arguments, found {}", params.len(), args.len()),
                        )
                        .with_primary(span),
                    );
                }
                for (index, arg) in args.iter().enumerate() {
                    let actual = self.check_expr(arg);
                    if let Some(expected) = params.get(index) {
                        if !types_compatible(expected, &actual) {
                            self.type_mismatch(
                                arg.span,
                                expected,
                                &actual,
                                format!("argument {} has the wrong type", index + 1),
                            );
                        }
                    }
                }
                *result
            }
            Type::Unknown => {
                for arg in args {
                    self.check_expr(arg);
                }
                Type::Unknown
            }
            other => {
                for arg in args {
                    self.check_expr(arg);
                }
                self.diagnostics.push(
                    Diagnostic::error("TP-E0300", format!("value of type `{other}` is not callable"))
                        .with_primary(callee.span),
                );
                Type::Unknown
            }
        }
    }

    fn check_binary(&mut self, op: BinaryOp, span: Span, left: Type, right: Type) -> Type {
        use BinaryOp::*;
        match op {
            Add | Subtract | Multiply | Divide | Remainder => {
                if left.is_unknown() || right.is_unknown() {
                    return Type::Unknown;
                }
                if left.is_numeric() && left == right {
                    left
                } else {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "TP-E0300",
                            format!("numeric operator requires matching numeric operands, found `{left}` and `{right}`"),
                        )
                        .with_primary(span),
                    );
                    Type::Unknown
                }
            }
            Less | LessEqual | Greater | GreaterEqual => {
                if !left.is_unknown()
                    && !right.is_unknown()
                    && !(left.is_numeric() && left == right)
                {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "TP-E0300",
                            format!("comparison requires matching numeric operands, found `{left}` and `{right}`"),
                        )
                        .with_primary(span),
                    );
                }
                Type::Bool
            }
            Equal | NotEqual => {
                if !types_compatible(&left, &right) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "TP-E0300",
                            format!("cannot compare `{left}` with `{right}`"),
                        )
                        .with_primary(span),
                    );
                }
                Type::Bool
            }
            And | Or => {
                self.require_bool(span, &left, "logical operator left operand");
                self.require_bool(span, &right, "logical operator right operand");
                Type::Bool
            }
        }
    }

    fn resolve_type_ref(&self, type_ref: &TypeRef) -> Type {
        let base = match type_ref.name.as_str() {
            "Unit" => Type::Unit,
            "Bool" => Type::Bool,
            "i64" => Type::I64,
            "f64" => Type::F64,
            "String" => Type::String,
            other => Type::Named {
                name: other.to_owned(),
                args: type_ref
                    .args
                    .iter()
                    .map(|arg| self.resolve_type_ref(arg))
                    .collect(),
            },
        };
        if type_ref.optional {
            Type::Named {
                name: "Option".to_owned(),
                args: vec![base],
            }
        } else {
            base
        }
    }

    fn require_bool(&mut self, span: Span, ty: &Type, context: &str) {
        if !ty.is_unknown() && *ty != Type::Bool {
            self.diagnostics.push(
                Diagnostic::error(
                    "TP-E0300",
                    format!("{context} must be `Bool`, found `{ty}`"),
                )
                .with_primary(span),
            );
        }
    }

    fn unknown_name(&mut self, span: Span, name: &str) {
        self.diagnostics.push(
            Diagnostic::error("TP-E0200", format!("unknown name `{name}`")).with_primary(span),
        );
    }

    fn type_mismatch(
        &mut self,
        span: Span,
        expected: &Type,
        actual: &Type,
        context: impl Into<String>,
    ) {
        if expected.is_unknown() || actual.is_unknown() {
            return;
        }
        self.diagnostics.push(
            Diagnostic::error(
                "TP-E0300",
                format!("{}: expected `{expected}`, found `{actual}`", context.into()),
            )
            .with_primary(span),
        );
    }
}

fn types_compatible(expected: &Type, actual: &Type) -> bool {
    expected.is_unknown() || actual.is_unknown() || expected == actual
}

fn contains_explicit_return(block: &Block) -> bool {
    block.statements.iter().any(|statement| match statement {
        Stmt::Return { .. } => true,
        Stmt::While { body, .. } => contains_explicit_return(body),
        Stmt::Expr { expr, .. } => expr_contains_return(expr),
        Stmt::Let { value, .. } => expr_contains_return(value),
    })
}

fn expr_contains_return(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::If {
            then_branch,
            else_branch,
            ..
        } => {
            contains_explicit_return(then_branch)
                || else_branch.as_ref().is_some_and(contains_explicit_return)
        }
        _ => false,
    }
}
