use std::collections::{HashMap, HashSet};

use crate::{
    BasicBlock, BlockId, Constant, HirBlock, HirExpr, HirExprKind, HirFunction, HirModule,
    HirPattern, HirPatternKind, HirStmt, LocalId, MirFunction, MirLocal, MirModule, MirPattern,
    MirStatement, Operand, Rvalue, SymbolId, Terminator, Type,
};

pub struct MirLowerer {
    variants: HashSet<String>,
}

impl MirLowerer {
    pub fn lower(module: &HirModule) -> MirModule {
        let variants = module
            .enums
            .iter()
            .flat_map(|decl| decl.variants.iter().map(|variant| variant.name.clone()))
            .collect();
        let lowerer = Self { variants };
        MirModule {
            functions: module
                .functions
                .iter()
                .map(|function| lowerer.lower_function(function))
                .collect(),
            structs: module.structs.clone(),
            enums: module.enums.clone(),
        }
    }

    fn lower_function(&self, function: &HirFunction) -> MirFunction {
        let mut builder = FunctionBuilder::new(&self.variants);
        let entry = builder.new_block();
        builder.switch_to(entry);

        let mut params = Vec::with_capacity(function.params.len());
        for param in &function.params {
            let local = builder.new_named_local(
                Some(param.name.clone()),
                param.ty.clone(),
                false,
            );
            builder.symbol_locals.insert(param.symbol, local);
            params.push(local);
        }

        let tail = builder.lower_block_value(&function.body);
        if builder.current_is_open() {
            if function.return_type == Type::Unit {
                builder.set_terminator(Terminator::Return(None));
            } else {
                builder.set_terminator(Terminator::Return(tail));
            }
        }

        MirFunction {
            name: function.name.clone(),
            params,
            locals: builder.locals,
            blocks: builder.blocks,
            entry,
            return_type: function.return_type.clone(),
            span: function.span,
        }
    }
}

struct FunctionBuilder<'a> {
    variants: &'a HashSet<String>,
    symbol_locals: HashMap<SymbolId, LocalId>,
    locals: Vec<MirLocal>,
    blocks: Vec<BasicBlock>,
    current: BlockId,
}

impl<'a> FunctionBuilder<'a> {
    fn new(variants: &'a HashSet<String>) -> Self {
        Self {
            variants,
            symbol_locals: HashMap::new(),
            locals: Vec::new(),
            blocks: Vec::new(),
            current: BlockId(0),
        }
    }

    fn new_block(&mut self) -> BlockId {
        let id = BlockId(self.blocks.len() as u32);
        self.blocks.push(BasicBlock {
            id,
            statements: Vec::new(),
            terminator: Terminator::Unreachable,
        });
        id
    }

    fn switch_to(&mut self, block: BlockId) {
        self.current = block;
    }

    fn current_is_open(&self) -> bool {
        matches!(
            self.blocks[self.current.0 as usize].terminator,
            Terminator::Unreachable
        )
    }

    fn set_terminator(&mut self, terminator: Terminator) {
        self.blocks[self.current.0 as usize].terminator = terminator;
    }

    fn emit(&mut self, statement: MirStatement) {
        self.blocks[self.current.0 as usize]
            .statements
            .push(statement);
    }

    fn new_named_local(
        &mut self,
        name: Option<String>,
        ty: Type,
        mutable: bool,
    ) -> LocalId {
        let id = LocalId(self.locals.len() as u32);
        self.locals.push(MirLocal {
            id,
            name,
            ty,
            mutable,
        });
        id
    }

    fn new_temp(&mut self, ty: Type) -> LocalId {
        self.new_named_local(None, ty, true)
    }

    fn assign_rvalue(&mut self, target: LocalId, value: Rvalue) {
        self.emit(MirStatement::Assign { target, value });
    }

    fn temp_rvalue(&mut self, ty: Type, value: Rvalue) -> Operand {
        let local = self.new_temp(ty);
        self.assign_rvalue(local, value);
        Operand::Local(local)
    }

    fn lower_block_value(&mut self, block: &HirBlock) -> Option<Operand> {
        let mut tail = None;
        let last_index = block.statements.len().checked_sub(1);
        for (index, statement) in block.statements.iter().enumerate() {
            if !self.current_is_open() {
                break;
            }
            match statement {
                HirStmt::Expr { expr, terminated }
                    if Some(index) == last_index && !terminated =>
                {
                    tail = Some(self.lower_expr(expr));
                }
                _ => self.lower_statement(statement),
            }
        }
        tail
    }

    fn lower_statement(&mut self, statement: &HirStmt) {
        match statement {
            HirStmt::Let {
                symbol,
                name,
                mutable,
                value,
                ..
            } => {
                let value_operand = self.lower_expr(value);
                let local = self.new_named_local(
                    Some(name.clone()),
                    value.ty.clone(),
                    *mutable,
                );
                self.symbol_locals.insert(*symbol, local);
                self.assign_rvalue(local, Rvalue::Use(value_operand));
            }
            HirStmt::Return { value, .. } => {
                let value = value.as_ref().map(|expr| self.lower_expr(expr));
                self.set_terminator(Terminator::Return(value));
            }
            HirStmt::While {
                condition, body, ..
            } => self.lower_while(condition, body),
            HirStmt::Expr { expr, .. } => {
                self.lower_expr(expr);
            }
        }
    }

    fn lower_while(&mut self, condition: &HirExpr, body: &HirBlock) {
        let header = self.new_block();
        let body_block = self.new_block();
        let exit = self.new_block();

        self.set_terminator(Terminator::Goto(header));
        self.switch_to(header);
        let condition = self.lower_expr(condition);
        self.set_terminator(Terminator::Branch {
            condition,
            then_block: body_block,
            else_block: exit,
        });

        self.switch_to(body_block);
        self.lower_block_value(body);
        if self.current_is_open() {
            self.set_terminator(Terminator::Goto(header));
        }

        self.switch_to(exit);
    }

    fn lower_expr(&mut self, expr: &HirExpr) -> Operand {
        match &expr.kind {
            HirExprKind::Integer(value) => Operand::Constant(Constant::I64(*value)),
            HirExprKind::Float(value) => Operand::Constant(Constant::F64(*value)),
            HirExprKind::String(value) => Operand::Constant(Constant::String(value.clone())),
            HirExprKind::Bool(value) => Operand::Constant(Constant::Bool(*value)),
            HirExprKind::Local(symbol) => Operand::Local(
                *self
                    .symbol_locals
                    .get(symbol)
                    .expect("HIR local is mapped before use"),
            ),
            HirExprKind::Global(name) if self.variants.contains(name) => self.temp_rvalue(
                expr.ty.clone(),
                Rvalue::Enum {
                    variant: name.clone(),
                    args: Vec::new(),
                },
            ),
            HirExprKind::Global(name) => self.temp_rvalue(
                expr.ty.clone(),
                Rvalue::Function(name.clone()),
            ),
            HirExprKind::Unary { op, expr: inner } => {
                let operand = self.lower_expr(inner);
                self.temp_rvalue(
                    expr.ty.clone(),
                    Rvalue::Unary {
                        op: *op,
                        operand,
                    },
                )
            }
            HirExprKind::Binary { op, left, right } => {
                let left = self.lower_expr(left);
                let right = self.lower_expr(right);
                self.temp_rvalue(
                    expr.ty.clone(),
                    Rvalue::Binary {
                        op: *op,
                        left,
                        right,
                    },
                )
            }
            HirExprKind::Assign { symbol, value } => {
                let target = *self
                    .symbol_locals
                    .get(symbol)
                    .expect("HIR assignment target is mapped");
                let value = self.lower_expr(value);
                self.assign_rvalue(target, Rvalue::Use(value));
                Operand::Local(target)
            }
            HirExprKind::Call { callee, args } => {
                if let HirExprKind::Global(name) = &callee.kind {
                    if self.variants.contains(name) {
                        let args = args.iter().map(|arg| self.lower_expr(arg)).collect();
                        return self.temp_rvalue(
                            expr.ty.clone(),
                            Rvalue::Enum {
                                variant: name.clone(),
                                args,
                            },
                        );
                    }
                }
                let callee = self.lower_expr(callee);
                let args = args.iter().map(|arg| self.lower_expr(arg)).collect();
                self.temp_rvalue(expr.ty.clone(), Rvalue::Call { callee, args })
            }
            HirExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.lower_if(expr, condition, then_branch, else_branch.as_ref()),
            HirExprKind::StructLiteral { type_name, fields } => {
                let fields = fields
                    .iter()
                    .map(|(name, value)| (name.clone(), self.lower_expr(value)))
                    .collect();
                self.temp_rvalue(
                    expr.ty.clone(),
                    Rvalue::Struct {
                        type_name: type_name.clone(),
                        fields,
                    },
                )
            }
            HirExprKind::Field { base, field } => {
                let base = self.lower_expr(base);
                self.temp_rvalue(
                    expr.ty.clone(),
                    Rvalue::Field {
                        base,
                        field: field.clone(),
                    },
                )
            }
            HirExprKind::Match { value, arms } => self.lower_match(expr, value, arms),
        }
    }

    fn lower_if(
        &mut self,
        expr: &HirExpr,
        condition: &HirExpr,
        then_branch: &HirBlock,
        else_branch: Option<&HirBlock>,
    ) -> Operand {
        let condition = self.lower_expr(condition);
        let then_block = self.new_block();
        let else_block = self.new_block();
        let join = self.new_block();
        let result = self.new_temp(expr.ty.clone());

        self.set_terminator(Terminator::Branch {
            condition,
            then_block,
            else_block,
        });

        self.switch_to(then_block);
        let then_value = self.lower_block_value(then_branch);
        if self.current_is_open() {
            self.assign_rvalue(
                result,
                Rvalue::Use(then_value.unwrap_or(Operand::Constant(Constant::Unit))),
            );
            self.set_terminator(Terminator::Goto(join));
        }

        self.switch_to(else_block);
        let else_value = else_branch
            .and_then(|block| self.lower_block_value(block))
            .unwrap_or(Operand::Constant(Constant::Unit));
        if self.current_is_open() {
            self.assign_rvalue(result, Rvalue::Use(else_value));
            self.set_terminator(Terminator::Goto(join));
        }

        self.switch_to(join);
        Operand::Local(result)
    }

    fn lower_match(
        &mut self,
        expr: &HirExpr,
        value: &HirExpr,
        arms: &[crate::HirMatchArm],
    ) -> Operand {
        let value = self.lower_expr(value);
        let join = self.new_block();
        let otherwise = self.new_block();
        let result = self.new_temp(expr.ty.clone());

        let mut arm_blocks = Vec::with_capacity(arms.len());
        for arm in arms {
            let block = self.new_block();
            let pattern = self.lower_pattern(&arm.pattern);
            arm_blocks.push((pattern, block));
        }

        self.set_terminator(Terminator::Match {
            value,
            arms: arm_blocks.clone(),
            otherwise,
        });

        for (arm, (_, block)) in arms.iter().zip(arm_blocks.iter()) {
            self.switch_to(*block);
            let body = self.lower_expr(&arm.body);
            if self.current_is_open() {
                self.assign_rvalue(result, Rvalue::Use(body));
                self.set_terminator(Terminator::Goto(join));
            }
        }

        self.switch_to(otherwise);
        self.set_terminator(Terminator::Trap("non-exhaustive match reached at runtime".into()));
        self.switch_to(join);
        Operand::Local(result)
    }

    fn lower_pattern(&mut self, pattern: &HirPattern) -> MirPattern {
        match &pattern.kind {
            HirPatternKind::Wildcard => MirPattern::Wildcard,
            HirPatternKind::Bind { symbol, name } => {
                let local = self.new_named_local(Some(name.clone()), Type::Unknown, false);
                self.symbol_locals.insert(*symbol, local);
                MirPattern::Bind(local)
            }
            HirPatternKind::Integer(value) => MirPattern::Integer(*value),
            HirPatternKind::Bool(value) => MirPattern::Bool(*value),
            HirPatternKind::String(value) => MirPattern::String(value.clone()),
            HirPatternKind::Variant { name, args } => MirPattern::Variant {
                name: name.clone(),
                args: args.iter().map(|arg| self.lower_pattern(arg)).collect(),
            },
        }
    }
}
