use crate::{
    Block, Expr, ExprKind, FnDecl, HirBlock, HirExpr, HirExprKind, HirFunction, HirMatchArm,
    HirModule, HirParam, HirPattern, HirPatternKind, HirStmt, Item, Module, Pattern, PatternKind,
    SemanticModel, Stmt, Type,
};

pub struct HirLowerer<'a> {
    semantic: &'a SemanticModel,
}

impl<'a> HirLowerer<'a> {
    pub fn lower(module: &Module, semantic: &'a SemanticModel) -> HirModule {
        let mut lowerer = Self { semantic };
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

    fn lower_function(&mut self, function: &FnDecl) -> HirFunction {
        let (param_types, result) = self
            .semantic
            .function(&function.name)
            .expect("checked function signature exists");
        let param_types = param_types.to_vec();
        let result = result.clone();

        let params = function
            .params
            .iter()
            .zip(param_types)
            .map(|(param, ty)| {
                let (symbol, checked_ty) = self
                    .semantic
                    .binding(param.span)
                    .expect("checked parameter binding exists");
                debug_assert_eq!(&ty, checked_ty);
                HirParam {
                    symbol,
                    name: param.name.clone(),
                    ty,
                    span: param.span,
                }
            })
            .collect();
        let body = self.lower_block(&function.body);

        HirFunction {
            name: function.name.clone(),
            params,
            return_type: result,
            body,
            span: function.span,
        }
    }

    fn lower_block(&mut self, block: &Block) -> HirBlock {
        let statements = block
            .statements
            .iter()
            .map(|statement| self.lower_statement(statement))
            .collect();
        let ty = self
            .semantic
            .block_type(block.span)
            .cloned()
            .expect("checked block type exists");
        HirBlock {
            statements,
            ty,
            span: block.span,
        }
    }

    fn lower_statement(&mut self, statement: &Stmt) -> HirStmt {
        match statement {
            Stmt::Let {
                mutable,
                name,
                value,
                span,
                ..
            } => {
                let (symbol, _) = self
                    .semantic
                    .binding(*span)
                    .expect("checked binding exists");
                HirStmt::Let {
                    symbol,
                    name: name.clone(),
                    mutable: *mutable,
                    value: self.lower_expr(value),
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
        let ty = self
            .semantic
            .expr_type(expr.span)
            .cloned()
            .expect("checked expression type exists");
        let kind = match &expr.kind {
            ExprKind::Integer(value) => HirExprKind::Integer(*value),
            ExprKind::Float(value) => HirExprKind::Float(*value),
            ExprKind::String(value) => HirExprKind::String(value.clone()),
            ExprKind::Bool(value) => HirExprKind::Bool(*value),
            ExprKind::Name(name) => {
                if let Some(symbol) = self.semantic.local_use(expr.span) {
                    HirExprKind::Local(symbol)
                } else {
                    let kind = self
                        .semantic
                        .global(expr.span)
                        .expect("checked global resolution exists");
                    HirExprKind::Global {
                        name: name.clone(),
                        kind,
                    }
                }
            }
            ExprKind::Unary { op, expr: inner } => HirExprKind::Unary {
                op: *op,
                expr: Box::new(self.lower_expr(inner)),
            },
            ExprKind::Binary { op, left, right } => HirExprKind::Binary {
                op: *op,
                left: Box::new(self.lower_expr(left)),
                right: Box::new(self.lower_expr(right)),
            },
            ExprKind::Assign { value, .. } => {
                let symbol = self
                    .semantic
                    .local_use(expr.span)
                    .expect("checked assignment target exists");
                HirExprKind::Assign {
                    symbol,
                    value: Box::new(self.lower_expr(value)),
                }
            }
            ExprKind::Call { callee, args } => HirExprKind::Call {
                callee: Box::new(self.lower_expr(callee)),
                args: args.iter().map(|arg| self.lower_expr(arg)).collect(),
            },
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => HirExprKind::If {
                condition: Box::new(self.lower_expr(condition)),
                then_branch: self.lower_block(then_branch),
                else_branch: else_branch.as_ref().map(|block| self.lower_block(block)),
            },
            ExprKind::StructLiteral { type_name, fields } => HirExprKind::StructLiteral {
                type_name: type_name.clone(),
                fields: fields
                    .iter()
                    .map(|(name, value)| (name.clone(), self.lower_expr(value)))
                    .collect(),
            },
            ExprKind::Field { base, field } => HirExprKind::Field {
                base: Box::new(self.lower_expr(base)),
                field: field.clone(),
            },
            ExprKind::Match { value, arms } => HirExprKind::Match {
                value: Box::new(self.lower_expr(value)),
                arms: arms
                    .iter()
                    .map(|arm| HirMatchArm {
                        pattern: self.lower_pattern(&arm.pattern),
                        body: self.lower_expr(&arm.body),
                        span: arm.span,
                    })
                    .collect(),
            },
        };

        HirExpr {
            ty,
            kind,
            span: expr.span,
        }
    }

    fn lower_pattern(&mut self, pattern: &Pattern) -> HirPattern {
        let kind = match &pattern.kind {
            PatternKind::Wildcard => HirPatternKind::Wildcard,
            PatternKind::Name(name) => {
                if let Some((symbol, ty)) = self.semantic.binding(pattern.span) {
                    HirPatternKind::Bind {
                        symbol,
                        name: name.clone(),
                        ty: ty.clone(),
                    }
                } else {
                    HirPatternKind::Variant {
                        name: name.clone(),
                        args: Vec::new(),
                    }
                }
            }
            PatternKind::Integer(value) => HirPatternKind::Integer(*value),
            PatternKind::Bool(value) => HirPatternKind::Bool(*value),
            PatternKind::String(value) => HirPatternKind::String(value.clone()),
            PatternKind::Variant { name, args } => HirPatternKind::Variant {
                name: name.clone(),
                args: args.iter().map(|child| self.lower_pattern(child)).collect(),
            },
        };

        HirPattern {
            kind,
            span: pattern.span,
        }
    }
}
