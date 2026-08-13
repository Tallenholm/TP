use std::collections::HashMap;

use crate::{
    BinaryOp, Block, EnumDecl, Expr, ExprKind, FnDecl, HirBlock, HirExpr, HirExprKind, HirFunction,
    HirMatchArm, HirModule, HirParam, HirPattern, HirPatternKind, HirStmt, Item, Module, Pattern,
    PatternKind, Scopes, Span, Stmt, StructDecl, SymbolId, Type, TypeRef, UnaryOp,
};

#[derive(Debug, Clone)]
struct FunctionSig {
    params: Vec<Type>,
    result: Type,
}

#[derive(Debug, Clone)]
struct StructInfo {
    type_params: Vec<String>,
    fields: HashMap<String, Type>,
}

#[derive(Debug, Clone)]
struct VariantInfo {
    enum_name: String,
    type_params: Vec<String>,
    payload: Vec<Type>,
}

#[derive(Debug, Clone)]
struct EnumInfo {
    type_params: Vec<String>,
    variants: HashMap<String, VariantInfo>,
}

pub struct HirLowerer {
    functions: HashMap<String, FunctionSig>,
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
    variants: HashMap<String, VariantInfo>,
    scopes: Scopes,
}

impl HirLowerer {
    pub fn lower(module: &Module) -> HirModule {
        let mut lowerer = Self {
            functions: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            variants: HashMap::new(),
            scopes: Scopes::new(),
        };
        lowerer.collect_types(module);
        lowerer.collect_functions(module);

        let functions = module
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Function(function) => Some(lowerer.lower_function(function)),
                _ => None,
            })
            .collect();
        let structs = module
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Struct(decl) => Some(decl.clone()),
                _ => None,
            })
            .collect();
        let enums = module
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Enum(decl) => Some(decl.clone()),
                _ => None,
            })
            .collect();

        HirModule {
            functions,
            structs,
            enums,
        }
    }

    fn collect_types(&mut self, module: &Module) {
        for item in &module.items {
            match item {
                Item::Struct(decl) => {
                    self.structs.insert(
                        decl.name.clone(),
                        StructInfo {
                            type_params: decl.type_params.clone(),
                            fields: decl
                                .fields
                                .iter()
                                .map(|field| (field.name.clone(), resolve_type_ref(&field.ty)))
                                .collect(),
                        },
                    );
                }
                Item::Enum(decl) => self.collect_enum(decl),
                _ => {}
            }
        }
    }

    fn collect_enum(&mut self, decl: &EnumDecl) {
        let mut variants = HashMap::new();
        for variant in &decl.variants {
            let info = VariantInfo {
                enum_name: decl.name.clone(),
                type_params: decl.type_params.clone(),
                payload: variant.payload.iter().map(resolve_type_ref).collect(),
            };
            variants.insert(variant.name.clone(), info.clone());
            self.variants.insert(variant.name.clone(), info);
        }
        self.enums.insert(
            decl.name.clone(),
            EnumInfo {
                type_params: decl.type_params.clone(),
                variants,
            },
        );
    }

    fn collect_functions(&mut self, module: &Module) {
        for item in &module.items {
            if let Item::Function(function) = item {
                self.functions.insert(
                    function.name.clone(),
                    FunctionSig {
                        params: function
                            .params
                            .iter()
                            .map(|param| resolve_type_ref(&param.ty))
                            .collect(),
                        result: function
                            .return_type
                            .as_ref()
                            .map(resolve_type_ref)
                            .unwrap_or(Type::Unit),
                    },
                );
            }
        }
    }

    fn lower_function(&mut self, function: &FnDecl) -> HirFunction {
        let signature = self
            .functions
            .get(&function.name)
            .cloned()
            .expect("checked function signature exists");
        self.scopes.push();
        let params = function
            .params
            .iter()
            .zip(signature.params.iter())
            .map(|(param, ty)| {
                let symbol = self.insert_symbol(&param.name, ty.clone(), false, param.span);
                HirParam {
                    symbol,
                    name: param.name.clone(),
                    ty: ty.clone(),
                    span: param.span,
                }
            })
            .collect();
        let body = self.lower_block(&function.body);
        self.scopes.pop();

        HirFunction {
            name: function.name.clone(),
            params,
            return_type: signature.result,
            body,
            span: function.span,
        }
    }

    fn lower_block(&mut self, block: &Block) -> HirBlock {
        self.scopes.push();
        let mut statements = Vec::with_capacity(block.statements.len());
        let mut block_ty = Type::Unit;
        for statement in &block.statements {
            let lowered = self.lower_statement(statement);
            block_ty = match &lowered {
                HirStmt::Expr { expr, terminated } if !terminated => expr.ty.clone(),
                _ => Type::Unit,
            };
            statements.push(lowered);
        }
        self.scopes.pop();
        HirBlock {
            statements,
            ty: block_ty,
            span: block.span,
        }
    }

    fn lower_statement(&mut self, statement: &Stmt) -> HirStmt {
        match statement {
            Stmt::Let {
                mutable,
                name,
                type_annotation,
                value,
                span,
            } => {
                let value = self.lower_expr(value);
                let binding_ty = type_annotation
                    .as_ref()
                    .map(resolve_type_ref)
                    .unwrap_or_else(|| value.ty.clone());
                let symbol = self.insert_symbol(name, binding_ty, *mutable, *span);
                HirStmt::Let {
                    symbol,
                    name: name.clone(),
                    mutable: *mutable,
                    value,
                    span: *span,
                }
            }
            Stmt::Return { value, span } => HirStmt::Return {
                value: value.as_ref().map(|expr| self.lower_expr(expr)),
                span: *span,
            },
            Stmt::While {
                condition,
                body,
                span,
            } => HirStmt::While {
                condition: self.lower_expr(condition),
                body: self.lower_block(body),
                span: *span,
            },
            Stmt::Expr { expr, terminated } => HirStmt::Expr {
                expr: self.lower_expr(expr),
                terminated: *terminated,
            },
        }
    }

    fn lower_expr(&mut self, expr: &Expr) -> HirExpr {
        match &expr.kind {
            ExprKind::Integer(value) => self.hir(expr.span, Type::I64, HirExprKind::Integer(*value)),
            ExprKind::Float(value) => self.hir(expr.span, Type::F64, HirExprKind::Float(*value)),
            ExprKind::String(value) => {
                self.hir(expr.span, Type::String, HirExprKind::String(value.clone()))
            }
            ExprKind::Bool(value) => self.hir(expr.span, Type::Bool, HirExprKind::Bool(*value)),
            ExprKind::Name(name) => {
                if let Some(binding) = self.scopes.get(name) {
                    return self.hir(expr.span, binding.ty.clone(), HirExprKind::Local(binding.id));
                }
                let ty = if let Some(function) = self.functions.get(name) {
                    Type::Function {
                        params: function.params.clone(),
                        result: Box::new(function.result.clone()),
                    }
                } else if let Some(variant) = self.variants.get(name) {
                    enum_instance(variant, &HashMap::new())
                } else if name == "print" {
                    Type::Function {
                        params: vec![Type::Unknown],
                        result: Box::new(Type::Unit),
                    }
                } else {
                    Type::Unknown
                };
                self.hir(expr.span, ty, HirExprKind::Global(name.clone()))
            }
            ExprKind::Unary { op, expr: inner } => {
                let inner = self.lower_expr(inner);
                let ty = match op {
                    UnaryOp::Negate => inner.ty.clone(),
                    UnaryOp::Not => Type::Bool,
                };
                self.hir(
                    expr.span,
                    ty,
                    HirExprKind::Unary {
                        op: *op,
                        expr: Box::new(inner),
                    },
                )
            }
            ExprKind::Binary { op, left, right } => {
                let left = self.lower_expr(left);
                let right = self.lower_expr(right);
                let ty = match op {
                    BinaryOp::Add
                    | BinaryOp::Subtract
                    | BinaryOp::Multiply
                    | BinaryOp::Divide
                    | BinaryOp::Remainder => more_specific_type(left.ty.clone(), right.ty.clone()),
                    BinaryOp::Equal
                    | BinaryOp::NotEqual
                    | BinaryOp::Less
                    | BinaryOp::LessEqual
                    | BinaryOp::Greater
                    | BinaryOp::GreaterEqual
                    | BinaryOp::And
                    | BinaryOp::Or => Type::Bool,
                };
                self.hir(
                    expr.span,
                    ty,
                    HirExprKind::Binary {
                        op: *op,
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                )
            }
            ExprKind::Assign { target, value } => {
                let binding = self
                    .scopes
                    .get(target)
                    .cloned()
                    .expect("checked assignment target exists");
                let value = self.lower_expr(value);
                self.hir(
                    expr.span,
                    binding.ty,
                    HirExprKind::Assign {
                        symbol: binding.id,
                        value: Box::new(value),
                    },
                )
            }
            ExprKind::Call { callee, args } => {
                let callee_name = match &callee.kind {
                    ExprKind::Name(name) => Some(name.clone()),
                    _ => None,
                };
                let callee = self.lower_expr(callee);
                let args: Vec<_> = args.iter().map(|arg| self.lower_expr(arg)).collect();
                let ty = if let Some(name) = callee_name {
                    if name == "print" {
                        Type::Unit
                    } else if let Some(function) = self.functions.get(&name) {
                        function.result.clone()
                    } else if let Some(variant) = self.variants.get(&name) {
                        let mut substitution = HashMap::new();
                        for (template, actual) in variant.payload.iter().zip(args.iter()) {
                            infer_type_params(
                                template,
                                &actual.ty,
                                &variant.type_params,
                                &mut substitution,
                            );
                        }
                        enum_instance(variant, &substitution)
                    } else {
                        result_type(&callee.ty)
                    }
                } else {
                    result_type(&callee.ty)
                };
                self.hir(
                    expr.span,
                    ty,
                    HirExprKind::Call {
                        callee: Box::new(callee),
                        args,
                    },
                )
            }
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = self.lower_expr(condition);
                let then_branch = self.lower_block(then_branch);
                let else_branch = else_branch.as_ref().map(|block| self.lower_block(block));
                let ty = else_branch
                    .as_ref()
                    .map(|other| more_specific_type(then_branch.ty.clone(), other.ty.clone()))
                    .unwrap_or(Type::Unit);
                self.hir(
                    expr.span,
                    ty,
                    HirExprKind::If {
                        condition: Box::new(condition),
                        then_branch,
                        else_branch,
                    },
                )
            }
            ExprKind::StructLiteral { type_name, fields } => {
                let fields: Vec<_> = fields
                    .iter()
                    .map(|(name, value)| (name.clone(), self.lower_expr(value)))
                    .collect();
                let ty = self.struct_instance_type(type_name, &fields);
                self.hir(
                    expr.span,
                    ty,
                    HirExprKind::StructLiteral {
                        type_name: type_name.clone(),
                        fields,
                    },
                )
            }
            ExprKind::Field { base, field } => {
                let base = self.lower_expr(base);
                let ty = self.field_type(&base.ty, field);
                self.hir(
                    expr.span,
                    ty,
                    HirExprKind::Field {
                        base: Box::new(base),
                        field: field.clone(),
                    },
                )
            }
            ExprKind::Match { value, arms } => {
                let value = self.lower_expr(value);
                let target_ty = value.ty.clone();
                let mut lowered_arms = Vec::with_capacity(arms.len());
                let mut match_ty = Type::Unknown;
                for arm in arms {
                    self.scopes.push();
                    let pattern = self.lower_pattern(&arm.pattern, &target_ty);
                    let body = self.lower_expr(&arm.body);
                    self.scopes.pop();
                    match_ty = more_specific_type(match_ty, body.ty.clone());
                    lowered_arms.push(HirMatchArm {
                        pattern,
                        body,
                        span: arm.span,
                    });
                }
                self.hir(
                    expr.span,
                    match_ty,
                    HirExprKind::Match {
                        value: Box::new(value),
                        arms: lowered_arms,
                    },
                )
            }
        }
    }

    fn lower_pattern(&mut self, pattern: &Pattern, target: &Type) -> HirPattern {
        let kind = match &pattern.kind {
            PatternKind::Wildcard => HirPatternKind::Wildcard,
            PatternKind::Name(name) => {
                if self.is_nullary_variant_for_target(name, target) {
                    HirPatternKind::Variant {
                        name: name.clone(),
                        args: Vec::new(),
                    }
                } else {
                    let symbol = self.insert_symbol(name, target.clone(), false, pattern.span);
                    HirPatternKind::Bind {
                        symbol,
                        name: name.clone(),
                    }
                }
            }
            PatternKind::Integer(value) => HirPatternKind::Integer(*value),
            PatternKind::Bool(value) => HirPatternKind::Bool(*value),
            PatternKind::String(value) => HirPatternKind::String(value.clone()),
            PatternKind::Variant { name, args } => {
                let payload_types = self.variant_payload_for_target(name, target);
                let args = args
                    .iter()
                    .enumerate()
                    .map(|(index, child)| {
                        let child_ty = payload_types.get(index).cloned().unwrap_or(Type::Unknown);
                        self.lower_pattern(child, &child_ty)
                    })
                    .collect();
                HirPatternKind::Variant {
                    name: name.clone(),
                    args,
                }
            }
        };
        HirPattern {
            kind,
            span: pattern.span,
        }
    }

    fn is_nullary_variant_for_target(&self, name: &str, target: &Type) -> bool {
        let Type::Named { name: enum_name, .. } = target else {
            return false;
        };
        self.enums
            .get(enum_name)
            .and_then(|info| info.variants.get(name))
            .is_some_and(|variant| variant.payload.is_empty())
    }

    fn variant_payload_for_target(&self, variant_name: &str, target: &Type) -> Vec<Type> {
        let Type::Named {
            name: enum_name,
            args: enum_args,
        } = target
        else {
            return Vec::new();
        };
        let Some(enum_info) = self.enums.get(enum_name) else {
            return Vec::new();
        };
        let Some(variant) = enum_info.variants.get(variant_name) else {
            return Vec::new();
        };
        let substitution: HashMap<String, Type> = enum_info
            .type_params
            .iter()
            .cloned()
            .zip(enum_args.iter().cloned())
            .collect();
        variant
            .payload
            .iter()
            .map(|ty| substitute_type(ty, &substitution))
            .collect()
    }

    fn struct_instance_type(&self, name: &str, fields: &[(String, HirExpr)]) -> Type {
        let Some(info) = self.structs.get(name) else {
            return Type::Unknown;
        };
        let mut substitution = HashMap::new();
        for (field_name, value) in fields {
            if let Some(template) = info.fields.get(field_name) {
                infer_type_params(
                    template,
                    &value.ty,
                    &info.type_params,
                    &mut substitution,
                );
            }
        }
        Type::Named {
            name: name.to_owned(),
            args: info
                .type_params
                .iter()
                .map(|param| substitution.get(param).cloned().unwrap_or(Type::Unknown))
                .collect(),
        }
    }

    fn field_type(&self, base: &Type, field: &str) -> Type {
        let Type::Named { name, args } = base else {
            return Type::Unknown;
        };
        let Some(info) = self.structs.get(name) else {
            return Type::Unknown;
        };
        let Some(template) = info.fields.get(field) else {
            return Type::Unknown;
        };
        let substitution: HashMap<String, Type> = info
            .type_params
            .iter()
            .cloned()
            .zip(args.iter().cloned())
            .collect();
        substitute_type(template, &substitution)
    }

    fn insert_symbol(&mut self, name: &str, ty: Type, mutable: bool, span: Span) -> SymbolId {
        match self.scopes.insert(name.to_owned(), ty, mutable, span) {
            Ok(id) => id,
            Err(existing) => existing.id,
        }
    }

    fn hir(&self, span: Span, ty: Type, kind: HirExprKind) -> HirExpr {
        HirExpr { ty, kind, span }
    }
}

fn resolve_type_ref(type_ref: &TypeRef) -> Type {
    let base = match type_ref.name.as_str() {
        "Unit" => Type::Unit,
        "Bool" => Type::Bool,
        "i64" => Type::I64,
        "f64" => Type::F64,
        "String" => Type::String,
        other => Type::Named {
            name: other.to_owned(),
            args: type_ref.args.iter().map(resolve_type_ref).collect(),
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

fn result_type(ty: &Type) -> Type {
    match ty {
        Type::Function { result, .. } => (**result).clone(),
        _ => Type::Unknown,
    }
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
            substitution.entry(name.clone()).or_insert_with(|| actual.clone());
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

fn more_specific_type(left: Type, right: Type) -> Type {
    match (&left, &right) {
        (Type::Unknown, _) => right,
        (_, Type::Unknown) => left,
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
