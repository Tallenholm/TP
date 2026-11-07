use std::collections::{HashMap, HashSet};

use crate::{
    BinaryOp, Block, Diagnostic, Expr, ExprKind, FnDecl, HirGlobalKind, Item, MatchArm, Module,
    Pattern, PatternKind, Scopes, SemanticModel, Span, Stmt, Type, TypeRef, UnaryOp,
};

#[derive(Debug, Clone)]
struct FunctionSig {
    params: Vec<Type>,
    result: Type,
    span: Span,
}

#[derive(Debug, Clone)]
struct TypeDeclInfo {
    arity: usize,
    span: Span,
}

#[derive(Debug, Clone)]
struct StructInfo {
    type_params: Vec<String>,
    fields: HashMap<String, (Type, Span)>,
}

#[derive(Debug, Clone)]
struct VariantInfo {
    enum_name: String,
    type_params: Vec<String>,
    name: String,
    payload: Vec<Type>,
    span: Span,
}

#[derive(Debug, Clone)]
struct EnumInfo {
    type_params: Vec<String>,
    variants: HashMap<String, VariantInfo>,
}

#[derive(Debug, Default)]
pub struct TypeCheckResult {
    pub diagnostics: Vec<Diagnostic>,
    pub semantic: SemanticModel,
}

pub struct TypeChecker {
    functions: HashMap<String, FunctionSig>,
    type_decls: HashMap<String, TypeDeclInfo>,
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
    variants: HashMap<String, VariantInfo>,
    scopes: Scopes,
    diagnostics: Vec<Diagnostic>,
    semantic: SemanticModel,
    expected_return: Type,
}

impl TypeChecker {
    pub fn check_module(module: &Module) -> TypeCheckResult {
        let mut checker = Self {
            functions: HashMap::new(),
            type_decls: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            variants: HashMap::new(),
            scopes: Scopes::new(),
            diagnostics: Vec::new(),
            semantic: SemanticModel::default(),
            expected_return: Type::Unit,
        };
        checker.collect_type_headers(module);
        checker.collect_types(module);
        checker.collect_functions(module);
        checker.check_functions(module);
        TypeCheckResult {
            diagnostics: checker.diagnostics,
            semantic: checker.semantic,
        }
    }

    fn collect_type_headers(&mut self, module: &Module) {
        for item in &module.items {
            let (name, type_params, span) = match item {
                Item::Struct(decl) => (&decl.name, &decl.type_params, decl.span),
                Item::Enum(decl) => (&decl.name, &decl.type_params, decl.span),
                _ => continue,
            };

            if matches!(name.as_str(), "Unit" | "Bool" | "i64" | "f64" | "String") {
                self.diagnostics.push(
                    Diagnostic::error("TP-E0200", format!("type name `{name}` is reserved"))
                        .with_primary(span),
                );
                continue;
            }

            let mut seen_params = HashSet::new();
            for parameter in type_params {
                if !seen_params.insert(parameter.clone()) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "TP-E0200",
                            format!("duplicate generic parameter `{parameter}`"),
                        )
                        .with_primary(span),
                    );
                }
            }

            let info = TypeDeclInfo {
                arity: type_params.len(),
                span,
            };
            if let Some(previous) = self.type_decls.get(name) {
                self.diagnostics.push(
                    Diagnostic::error("TP-E0200", format!("duplicate type name `{name}`"))
                        .with_primary(span)
                        .with_label(previous.span, "previous type declaration is here"),
                );
            } else {
                self.type_decls.insert(name.clone(), info);
            }
        }
    }

    fn collect_types(&mut self, module: &Module) {
        for item in &module.items {
            match item {
                Item::Struct(decl) => {
                    if !matches!(
                        self.type_decls.get(&decl.name),
                        Some(info) if info.span == decl.span
                    ) {
                        continue;
                    }

                    let type_params: HashSet<String> = decl.type_params.iter().cloned().collect();
                    let mut fields = HashMap::new();
                    for field in &decl.fields {
                        let ty = self.resolve_type_ref_with_generics(&field.ty, &type_params);
                        if let Some((_, previous_span)) =
                            fields.insert(field.name.clone(), (ty, field.span))
                        {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "TP-E0200",
                                    format!("duplicate struct field `{}`", field.name),
                                )
                                .with_primary(field.span)
                                .with_label(previous_span, "previous field is here"),
                            );
                        }
                    }
                    self.structs.insert(
                        decl.name.clone(),
                        StructInfo {
                            type_params: decl.type_params.clone(),
                            fields,
                        },
                    );
                }
                Item::Enum(decl) => {
                    if !matches!(
                        self.type_decls.get(&decl.name),
                        Some(info) if info.span == decl.span
                    ) {
                        continue;
                    }

                    let type_params: HashSet<String> = decl.type_params.iter().cloned().collect();
                    let mut variants = HashMap::new();
                    for variant in &decl.variants {
                        let info = VariantInfo {
                            enum_name: decl.name.clone(),
                            type_params: decl.type_params.clone(),
                            name: variant.name.clone(),
                            payload: variant
                                .payload
                                .iter()
                                .map(|ty| self.resolve_type_ref_with_generics(ty, &type_params))
                                .collect(),
                            span: variant.span,
                        };
                        if let Some(previous) = variants.insert(variant.name.clone(), info.clone())
                        {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "TP-E0200",
                                    format!("duplicate enum variant `{}`", variant.name),
                                )
                                .with_primary(variant.span)
                                .with_label(previous.span, "previous variant is here"),
                            );
                        }
                        if let Some(previous) = self.variants.insert(variant.name.clone(), info) {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "TP-E0200",
                                    format!(
                                        "enum variant `{}` is ambiguous between `{}` and `{}`",
                                        variant.name, previous.enum_name, decl.name
                                    ),
                                )
                                .with_primary(variant.span)
                                .with_label(previous.span, "other variant is here"),
                            );
                        }
                    }
                    self.enums.insert(
                        decl.name.clone(),
                        EnumInfo {
                            type_params: decl.type_params.clone(),
                            variants,
                        },
                    );
                }
                _ => {}
            }
        }
    }

    fn collect_functions(&mut self, module: &Module) {
        for item in &module.items {
            let Item::Function(function) = item else {
                continue;
            };
            if function.name == "print" {
                self.diagnostics.push(
                    Diagnostic::error("TP-E0200", "`print` is a reserved builtin name")
                        .with_primary(function.span),
                );
            }

            if function.name == "main" && !function.params.is_empty() {
                self.diagnostics.push(
                    Diagnostic::error("TP-E0400", "entry point `main` must not take parameters")
                        .with_primary(function.span),
                );
            }

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

            self.semantic.record_function(
                function.name.clone(),
                sig.params.clone(),
                sig.result.clone(),
            );

            if let Some(type_decl) = self.type_decls.get(&function.name) {
                self.diagnostics.push(
                    Diagnostic::error(
                        "TP-E0200",
                        format!(
                            "function `{}` conflicts with type `{}`",
                            function.name, function.name
                        ),
                    )
                    .with_primary(function.span)
                    .with_label(type_decl.span, "type is declared here"),
                );
            }

            if let Some(variant) = self.variants.get(&function.name) {
                self.diagnostics.push(
                    Diagnostic::error(
                        "TP-E0200",
                        format!(
                            "function `{}` conflicts with enum variant `{}`",
                            function.name, function.name
                        ),
                    )
                    .with_primary(function.span)
                    .with_label(variant.span, "enum variant is declared here"),
                );
            }
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
            match self
                .scopes
                .insert(param.name.clone(), ty.clone(), false, param.span)
            {
                Ok(symbol) => {
                    self.semantic.record_binding(param.span, symbol, ty.clone());
                }
                Err(_) => {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "TP-E0200",
                            format!("duplicate parameter `{}`", param.name),
                        )
                        .with_primary(param.span),
                    );
                }
            }
        }

        let body_ty = self.check_block(&function.body);
        if sig.result != Type::Unit && !types_compatible(&sig.result, &body_ty) {
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
        let mut diverged = false;
        for statement in &block.statements {
            let statement_ty = self.check_statement(statement);
            if !diverged {
                diverged = statement_ty.is_never();
                result = statement_ty;
            }
        }
        self.scopes.pop();
        self.semantic.record_block_type(block.span, result.clone());
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
                let diverges = value_ty.is_never();
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
                match self
                    .scopes
                    .insert(name.clone(), binding_ty.clone(), *mutable, *span)
                {
                    Ok(symbol) => {
                        self.semantic.record_binding(*span, symbol, binding_ty);
                    }
                    Err(_) => {
                        self.diagnostics.push(
                            Diagnostic::error("TP-E0200", format!("duplicate binding `{name}`"))
                                .with_primary(*span),
                        );
                    }
                }
                if diverges { Type::Never } else { Type::Unit }
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
                Type::Never
            }
            Stmt::While {
                condition, body, ..
            } => {
                let condition_ty = self.check_expr(condition);
                self.require_bool(condition.span, &condition_ty, "while condition");
                self.check_block(body);
                if condition_ty.is_never() {
                    Type::Never
                } else {
                    Type::Unit
                }
            }
            Stmt::Expr { expr, terminated } => {
                let ty = self.check_expr(expr);
                if ty.is_never() {
                    Type::Never
                } else if *terminated {
                    Type::Unit
                } else {
                    ty
                }
            }
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Type {
        let ty = self.check_expr_inner(expr);
        self.semantic.record_expr_type(expr.span, ty.clone());
        ty
    }

    fn check_expr_inner(&mut self, expr: &Expr) -> Type {
        match &expr.kind {
            ExprKind::Integer(_) => Type::I64,
            ExprKind::Float(_) => Type::F64,
            ExprKind::String(_) => Type::String,
            ExprKind::Bool(_) => Type::Bool,
            ExprKind::Name(name) => {
                if let Some(binding) = self.scopes.get(name).cloned() {
                    self.semantic.record_local_use(expr.span, binding.id);
                    return binding.ty;
                }
                if let Some(function) = self.functions.get(name).cloned() {
                    self.semantic
                        .record_global(expr.span, HirGlobalKind::Function);
                    return Type::Function {
                        params: function.params,
                        result: Box::new(function.result),
                    };
                }
                if let Some(variant) = self.variants.get(name).cloned() {
                    if variant.payload.is_empty() {
                        self.semantic
                            .record_global(expr.span, HirGlobalKind::Variant);
                        return enum_instance(&variant, &HashMap::new());
                    }
                }
                if name == "print" {
                    self.semantic
                        .record_global(expr.span, HirGlobalKind::Builtin);
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
                if inner_ty.is_never() {
                    return Type::Never;
                }
                match op {
                    UnaryOp::Negate => {
                        if !inner_ty.is_unknown() && !inner_ty.is_numeric() {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "TP-E0300",
                                    format!(
                                        "unary `-` requires a numeric value, found `{inner_ty}`"
                                    ),
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
                self.semantic.record_local_use(expr.span, binding.id);
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
                if value_ty.is_never() {
                    Type::Never
                } else {
                    binding.ty
                }
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
                let else_ty = else_branch.as_ref().map(|branch| self.check_block(branch));
                if condition_ty.is_never() {
                    return Type::Never;
                }
                let Some(else_ty) = else_ty else {
                    return Type::Unit;
                };
                self.unify_branch_types(expr.span, then_ty, else_ty, "if branches")
            }
            ExprKind::StructLiteral { type_name, fields } => {
                self.check_struct_literal(expr.span, type_name, fields)
            }
            ExprKind::Field { base, field } => self.check_field(expr.span, base, field),
            ExprKind::Match { value, arms } => self.check_match(expr.span, value, arms),
        }
    }

    fn check_struct_literal(
        &mut self,
        span: Span,
        type_name: &str,
        fields: &[(String, Expr)],
    ) -> Type {
        let Some(info) = self.structs.get(type_name).cloned() else {
            self.unknown_name(span, type_name);
            for (_, value) in fields {
                self.check_expr(value);
            }
            return Type::Unknown;
        };

        let mut substitution = HashMap::new();
        let mut seen = HashSet::new();
        for (field_name, value) in fields {
            let value_ty = self.check_expr(value);
            if !seen.insert(field_name.clone()) {
                self.diagnostics.push(
                    Diagnostic::error(
                        "TP-E0300",
                        format!("duplicate field `{field_name}` in `{type_name}` literal"),
                    )
                    .with_primary(value.span),
                );
                continue;
            }
            let Some((template, _field_span)) = info.fields.get(field_name) else {
                self.diagnostics.push(
                    Diagnostic::error(
                        "TP-E0300",
                        format!("struct `{type_name}` has no field `{field_name}`"),
                    )
                    .with_primary(value.span),
                );
                continue;
            };
            infer_type_params(template, &value_ty, &info.type_params, &mut substitution);
            let expected = substitute_type(template, &substitution);
            if !types_compatible(&expected, &value_ty) {
                self.type_mismatch(
                    value.span,
                    &expected,
                    &value_ty,
                    format!("field `{field_name}` has the wrong type"),
                );
            }
        }

        for field_name in info.fields.keys() {
            if !seen.contains(field_name) {
                self.diagnostics.push(
                    Diagnostic::error(
                        "TP-E0300",
                        format!("missing field `{field_name}` in `{type_name}` literal"),
                    )
                    .with_primary(span),
                );
            }
        }

        Type::Named {
            name: type_name.to_owned(),
            args: info
                .type_params
                .iter()
                .map(|name| substitution.get(name).cloned().unwrap_or(Type::Unknown))
                .collect(),
        }
    }

    fn check_field(&mut self, span: Span, base: &Expr, field: &str) -> Type {
        let base_ty = self.check_expr(base);
        if base_ty.is_never() {
            return Type::Never;
        }
        let Type::Named { name, args } = &base_ty else {
            if !base_ty.is_unknown() {
                self.diagnostics.push(
                    Diagnostic::error("TP-E0300", format!("type `{base_ty}` does not have fields"))
                        .with_primary(span),
                );
            }
            return Type::Unknown;
        };
        let Some(info) = self.structs.get(name).cloned() else {
            self.diagnostics.push(
                Diagnostic::error("TP-E0300", format!("type `{base_ty}` does not have fields"))
                    .with_primary(span),
            );
            return Type::Unknown;
        };
        let Some((field_ty, _)) = info.fields.get(field) else {
            self.diagnostics.push(
                Diagnostic::error(
                    "TP-E0300",
                    format!("struct `{name}` has no field `{field}`"),
                )
                .with_primary(span),
            );
            return Type::Unknown;
        };
        let substitution: HashMap<String, Type> = info
            .type_params
            .iter()
            .cloned()
            .zip(args.iter().cloned())
            .collect();
        substitute_type(field_ty, &substitution)
    }

    fn check_match(&mut self, span: Span, value: &Expr, arms: &[MatchArm]) -> Type {
        let value_ty = self.check_expr(value);
        let value_diverges = value_ty.is_never();
        let enum_info = match &value_ty {
            Type::Named { name, .. } => self.enums.get(name).cloned(),
            _ => None,
        };
        let mut covered = HashSet::new();
        let mut wildcard = false;
        let mut result: Option<Type> = None;

        for arm in arms {
            self.scopes.push();
            let coverage = self.check_pattern(&arm.pattern, &value_ty);
            match coverage {
                PatternCoverage::Variant(name) => {
                    covered.insert(name);
                }
                PatternCoverage::Wildcard => wildcard = true,
                PatternCoverage::Other => {}
            }
            let arm_ty = self.check_expr(&arm.body);
            self.scopes.pop();
            result = Some(match result {
                None => arm_ty,
                Some(previous) => self.unify_branch_types(arm.span, previous, arm_ty, "match arms"),
            });
        }

        if let Some(info) = enum_info {
            if !wildcard {
                let mut missing: Vec<_> = info
                    .variants
                    .keys()
                    .filter(|name| !covered.contains(*name))
                    .cloned()
                    .collect();
                missing.sort();
                if !missing.is_empty() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "TP-E0300",
                            format!(
                                "non-exhaustive match; missing {}",
                                missing
                                    .iter()
                                    .map(|name| format!("`{name}`"))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                        )
                        .with_primary(span),
                    );
                }
            }
        }

        if value_diverges {
            Type::Never
        } else {
            result.unwrap_or(Type::Unit)
        }
    }

    fn check_pattern(&mut self, pattern: &Pattern, target: &Type) -> PatternCoverage {
        match &pattern.kind {
            PatternKind::Wildcard => PatternCoverage::Wildcard,
            PatternKind::Name(name) => {
                if let Type::Named {
                    name: enum_name,
                    args,
                } = target
                {
                    if let Some(enum_info) = self.enums.get(enum_name).cloned() {
                        if let Some(variant) = enum_info.variants.get(name).cloned() {
                            if variant.payload.is_empty() {
                                return PatternCoverage::Variant(name.clone());
                            }
                        }
                    }
                    let _ = args;
                }
                match self
                    .scopes
                    .insert(name.clone(), target.clone(), false, pattern.span)
                {
                    Ok(symbol) => {
                        self.semantic
                            .record_binding(pattern.span, symbol, target.clone());
                    }
                    Err(_) => {
                        self.diagnostics.push(
                            Diagnostic::error(
                                "TP-E0200",
                                format!("duplicate pattern binding `{name}`"),
                            )
                            .with_primary(pattern.span),
                        );
                    }
                }
                PatternCoverage::Other
            }
            PatternKind::Variant { name, args } => {
                let Type::Named {
                    name: enum_name,
                    args: enum_args,
                } = target
                else {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "TP-E0300",
                            format!("variant pattern `{name}` requires an enum value"),
                        )
                        .with_primary(pattern.span),
                    );
                    return PatternCoverage::Other;
                };
                let Some(enum_info) = self.enums.get(enum_name).cloned() else {
                    return PatternCoverage::Other;
                };
                let Some(variant) = enum_info.variants.get(name).cloned() else {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "TP-E0300",
                            format!("enum `{enum_name}` has no variant `{name}`"),
                        )
                        .with_primary(pattern.span),
                    );
                    return PatternCoverage::Other;
                };
                if args.len() != variant.payload.len() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "TP-E0300",
                            format!(
                                "variant `{name}` expects {} pattern values, found {}",
                                variant.payload.len(),
                                args.len()
                            ),
                        )
                        .with_primary(pattern.span),
                    );
                }
                let substitution: HashMap<String, Type> = enum_info
                    .type_params
                    .iter()
                    .cloned()
                    .zip(enum_args.iter().cloned())
                    .collect();
                for (index, child) in args.iter().enumerate() {
                    let child_ty = variant
                        .payload
                        .get(index)
                        .map(|ty| substitute_type(ty, &substitution))
                        .unwrap_or(Type::Unknown);
                    self.check_pattern(child, &child_ty);
                }
                PatternCoverage::Variant(name.clone())
            }
            PatternKind::Integer(_) => {
                self.require_pattern_type(pattern.span, &Type::I64, target);
                PatternCoverage::Other
            }
            PatternKind::Bool(_) => {
                self.require_pattern_type(pattern.span, &Type::Bool, target);
                PatternCoverage::Other
            }
            PatternKind::String(_) => {
                self.require_pattern_type(pattern.span, &Type::String, target);
                PatternCoverage::Other
            }
        }
    }

    fn require_pattern_type(&mut self, span: Span, expected: &Type, actual: &Type) {
        if !types_compatible(expected, actual) {
            self.type_mismatch(span, expected, actual, "pattern type mismatch");
        }
    }

    fn check_call(&mut self, span: Span, callee: &Expr, args: &[Expr]) -> Type {
        if let ExprKind::Name(name) = &callee.kind {
            if name == "print" {
                self.semantic
                    .record_global(callee.span, HirGlobalKind::Builtin);
                self.semantic.record_expr_type(
                    callee.span,
                    Type::Function {
                        params: vec![Type::Unknown],
                        result: Box::new(Type::Unit),
                    },
                );
                let mut diverges = false;
                for arg in args {
                    diverges |= self.check_expr(arg).is_never();
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
                return if diverges { Type::Never } else { Type::Unit };
            }
            if let Some(variant) = self.variants.get(name).cloned() {
                self.semantic
                    .record_global(callee.span, HirGlobalKind::Variant);
                self.semantic.record_expr_type(
                    callee.span,
                    Type::Function {
                        params: variant.payload.clone(),
                        result: Box::new(enum_instance(&variant, &HashMap::new())),
                    },
                );
                return self.check_variant_constructor(span, &variant, args);
            }
            if let Some(sig) = self.functions.get(name).cloned() {
                self.semantic
                    .record_global(callee.span, HirGlobalKind::Function);
                self.semantic.record_expr_type(
                    callee.span,
                    Type::Function {
                        params: sig.params.clone(),
                        result: Box::new(sig.result.clone()),
                    },
                );
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
                let mut diverges = false;
                for (index, arg) in args.iter().enumerate() {
                    let actual = self.check_expr(arg);
                    diverges |= actual.is_never();
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
                return if diverges { Type::Never } else { sig.result };
            }
        }

        let callee_ty = self.check_expr(callee);
        let callee_diverges = callee_ty.is_never();
        match callee_ty {
            Type::Function { params, result } => {
                if args.len() != params.len() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "TP-E0300",
                            format!(
                                "call expects {} arguments, found {}",
                                params.len(),
                                args.len()
                            ),
                        )
                        .with_primary(span),
                    );
                }
                let mut diverges = callee_diverges;
                for (index, arg) in args.iter().enumerate() {
                    let actual = self.check_expr(arg);
                    diverges |= actual.is_never();
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
                if diverges { Type::Never } else { *result }
            }
            Type::Never => {
                for arg in args {
                    self.check_expr(arg);
                }
                Type::Never
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
                    Diagnostic::error(
                        "TP-E0300",
                        format!("value of type `{other}` is not callable"),
                    )
                    .with_primary(callee.span),
                );
                Type::Unknown
            }
        }
    }

    fn check_variant_constructor(
        &mut self,
        span: Span,
        variant: &VariantInfo,
        args: &[Expr],
    ) -> Type {
        if args.len() != variant.payload.len() {
            self.diagnostics.push(
                Diagnostic::error(
                    "TP-E0300",
                    format!(
                        "variant `{}` expects {} arguments, found {}",
                        variant.name,
                        variant.payload.len(),
                        args.len()
                    ),
                )
                .with_primary(span),
            );
        }
        let mut substitution = HashMap::new();
        let mut diverges = false;
        for (index, arg) in args.iter().enumerate() {
            let actual = self.check_expr(arg);
            diverges |= actual.is_never();
            if let Some(template) = variant.payload.get(index) {
                infer_type_params(template, &actual, &variant.type_params, &mut substitution);
                let expected = substitute_type(template, &substitution);
                if !types_compatible(&expected, &actual) {
                    self.type_mismatch(
                        arg.span,
                        &expected,
                        &actual,
                        format!("variant `{}` payload has the wrong type", variant.name),
                    );
                }
            }
        }
        if diverges {
            Type::Never
        } else {
            enum_instance(variant, &substitution)
        }
    }

    fn check_binary(&mut self, op: BinaryOp, span: Span, left: Type, right: Type) -> Type {
        use BinaryOp::*;
        if left.is_never() {
            return Type::Never;
        }
        if right.is_never() && !matches!(op, BinaryOp::And | BinaryOp::Or) {
            return Type::Never;
        }
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

    fn resolve_type_ref(&mut self, type_ref: &TypeRef) -> Type {
        self.resolve_type_ref_with_generics(type_ref, &HashSet::new())
    }

    fn resolve_type_ref_with_generics(
        &mut self,
        type_ref: &TypeRef,
        generic_params: &HashSet<String>,
    ) -> Type {
        let args: Vec<Type> = type_ref
            .args
            .iter()
            .map(|arg| self.resolve_type_ref_with_generics(arg, generic_params))
            .collect();

        let base = match type_ref.name.as_str() {
            "Unit" | "Bool" | "i64" | "f64" | "String" => {
                if !args.is_empty() {
                    self.report_type_arity(type_ref, 0, args.len());
                }
                match type_ref.name.as_str() {
                    "Unit" => Type::Unit,
                    "Bool" => Type::Bool,
                    "i64" => Type::I64,
                    "f64" => Type::F64,
                    "String" => Type::String,
                    _ => unreachable!(),
                }
            }
            name if generic_params.contains(name) => {
                if !args.is_empty() {
                    self.report_type_arity(type_ref, 0, args.len());
                }
                Type::Named {
                    name: name.to_owned(),
                    args: Vec::new(),
                }
            }
            name => {
                if let Some(info) = self.type_decls.get(name).cloned() {
                    if args.len() != info.arity {
                        self.report_type_arity(type_ref, info.arity, args.len());
                    }
                    Type::Named {
                        name: name.to_owned(),
                        args,
                    }
                } else {
                    self.diagnostics.push(
                        Diagnostic::error("TP-E0200", format!("unknown type `{name}`"))
                            .with_primary(type_ref.span),
                    );
                    Type::Unknown
                }
            }
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

    fn report_type_arity(&mut self, type_ref: &TypeRef, expected: usize, actual: usize) {
        let noun = if expected == 1 {
            "type argument"
        } else {
            "type arguments"
        };
        self.diagnostics.push(
            Diagnostic::error(
                "TP-E0200",
                format!(
                    "type `{}` expects {expected} {noun}, found {actual}",
                    type_ref.name
                ),
            )
            .with_primary(type_ref.span),
        );
    }

    fn unify_branch_types(&mut self, span: Span, left: Type, right: Type, context: &str) -> Type {
        if types_compatible(&left, &right) {
            more_specific_type(left, right)
        } else {
            self.type_mismatch(
                span,
                &left,
                &right,
                format!("{context} have incompatible types"),
            );
            Type::Unknown
        }
    }

    fn require_bool(&mut self, span: Span, ty: &Type, context: &str) {
        if !ty.is_unknown() && !ty.is_never() && *ty != Type::Bool {
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
        if types_compatible(expected, actual) {
            return;
        }
        self.diagnostics.push(
            Diagnostic::error(
                "TP-E0300",
                format!(
                    "{}: expected `{expected}`, found `{actual}`",
                    context.into()
                ),
            )
            .with_primary(span),
        );
    }
}

#[derive(Debug)]
enum PatternCoverage {
    Variant(String),
    Wildcard,
    Other,
}

fn enum_instance(variant: &VariantInfo, substitution: &HashMap<String, Type>) -> Type {
    Type::Named {
        name: variant.enum_name.clone(),
        args: variant
            .type_params
            .iter()
            .map(|name| substitution.get(name).cloned().unwrap_or(Type::Unknown))
            .collect(),
    }
}

fn infer_type_params(
    template: &Type,
    actual: &Type,
    type_params: &[String],
    substitution: &mut HashMap<String, Type>,
) {
    if let Type::Named { name, args } = template {
        if args.is_empty() && type_params.iter().any(|param| param == name) {
            match substitution.get(name) {
                None => {
                    substitution.insert(name.clone(), actual.clone());
                }
                Some(existing) if types_compatible(existing, actual) => {}
                Some(_) => {}
            }
            return;
        }
    }

    if let (
        Type::Named {
            name: template_name,
            args: template_args,
        },
        Type::Named {
            name: actual_name,
            args: actual_args,
        },
    ) = (template, actual)
    {
        if template_name == actual_name && template_args.len() == actual_args.len() {
            for (template_arg, actual_arg) in template_args.iter().zip(actual_args) {
                infer_type_params(template_arg, actual_arg, type_params, substitution);
            }
        }
    }
}

fn substitute_type(template: &Type, substitution: &HashMap<String, Type>) -> Type {
    match template {
        Type::Named { name, args } if args.is_empty() => substitution
            .get(name)
            .cloned()
            .unwrap_or_else(|| template.clone()),
        Type::Named { name, args } => Type::Named {
            name: name.clone(),
            args: args
                .iter()
                .map(|arg| substitute_type(arg, substitution))
                .collect(),
        },
        Type::Function { params, result } => Type::Function {
            params: params
                .iter()
                .map(|param| substitute_type(param, substitution))
                .collect(),
            result: Box::new(substitute_type(result, substitution)),
        },
        other => other.clone(),
    }
}

fn types_compatible(expected: &Type, actual: &Type) -> bool {
    match (expected, actual) {
        (Type::Unknown, _) | (_, Type::Unknown) | (Type::Never, _) | (_, Type::Never) => true,
        (
            Type::Named {
                name: expected_name,
                args: expected_args,
            },
            Type::Named {
                name: actual_name,
                args: actual_args,
            },
        ) => {
            expected_name == actual_name
                && expected_args.len() == actual_args.len()
                && expected_args
                    .iter()
                    .zip(actual_args)
                    .all(|(expected, actual)| types_compatible(expected, actual))
        }
        (
            Type::Function {
                params: expected_params,
                result: expected_result,
            },
            Type::Function {
                params: actual_params,
                result: actual_result,
            },
        ) => {
            expected_params.len() == actual_params.len()
                && expected_params
                    .iter()
                    .zip(actual_params)
                    .all(|(expected, actual)| types_compatible(expected, actual))
                && types_compatible(expected_result, actual_result)
        }
        _ => expected == actual,
    }
}

fn more_specific_type(left: Type, right: Type) -> Type {
    match (&left, &right) {
        (Type::Unknown, _) | (Type::Never, _) => right,
        (_, Type::Unknown) | (_, Type::Never) => left,
        (
            Type::Named {
                name: left_name,
                args: left_args,
            },
            Type::Named {
                name: right_name,
                args: right_args,
            },
        ) if left_name == right_name && left_args.len() == right_args.len() => Type::Named {
            name: left_name.clone(),
            args: left_args
                .iter()
                .cloned()
                .zip(right_args.iter().cloned())
                .map(|(left, right)| more_specific_type(left, right))
                .collect(),
        },
        _ => left,
    }
}
