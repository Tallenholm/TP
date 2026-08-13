use std::{error::Error, fmt};

use crate::{
    BinaryOp, Constant, MirFunction, MirModule, MirPattern, MirStatement, Operand, Rvalue,
    Terminator, UnaryOp, Value,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    pub code: &'static str,
    pub message: String,
}

impl RuntimeError {
    fn trap(message: impl Into<String>) -> Self {
        Self {
            code: "TP-E0500",
            message: message.into(),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl Error for RuntimeError {}

pub struct Interpreter<'a> {
    module: &'a MirModule,
    output: String,
}

impl<'a> Interpreter<'a> {
    pub fn new(module: &'a MirModule) -> Self {
        Self {
            module,
            output: String::new(),
        }
    }

    pub fn run_main(&mut self) -> Result<Value, RuntimeError> {
        self.execute_function("main", Vec::new())
    }

    pub fn output(&self) -> &str {
        &self.output
    }

    pub fn into_output(self) -> String {
        self.output
    }

    fn execute_function(&mut self, name: &str, args: Vec<Value>) -> Result<Value, RuntimeError> {
        if name == "print" {
            if args.len() != 1 {
                return Err(RuntimeError::trap(format!(
                    "`print` expected 1 argument, found {}",
                    args.len()
                )));
            }
            self.output.push_str(&args[0].to_string());
            self.output.push('\n');
            return Ok(Value::Unit);
        }

        let function = self
            .module
            .functions
            .iter()
            .find(|function| function.name == name)
            .cloned()
            .ok_or_else(|| RuntimeError::trap(format!("unknown function `{name}` at runtime")))?;

        if args.len() != function.params.len() {
            return Err(RuntimeError::trap(format!(
                "function `{name}` expected {} arguments, found {}",
                function.params.len(),
                args.len()
            )));
        }

        let mut frame = Frame::new(&function);
        for (param, value) in function.params.iter().zip(args) {
            frame.set(*param, value)?;
        }

        self.execute_frame(&function, &mut frame)
    }

    fn execute_frame(
        &mut self,
        function: &MirFunction,
        frame: &mut Frame,
    ) -> Result<Value, RuntimeError> {
        let mut block = function.entry;

        loop {
            let basic_block = function
                .blocks
                .get(block.0 as usize)
                .ok_or_else(|| RuntimeError::trap(format!("invalid MIR block {}", block.0)))?
                .clone();

            for statement in &basic_block.statements {
                self.execute_statement(statement, frame)?;
            }

            match basic_block.terminator {
                Terminator::Unreachable => {
                    return Err(RuntimeError::trap(format!(
                        "reached unfinished MIR block {}",
                        basic_block.id.0
                    )));
                }
                Terminator::Goto(target) => block = target,
                Terminator::Branch {
                    condition,
                    then_block,
                    else_block,
                } => match self.eval_operand(&condition, frame)? {
                    Value::Bool(true) => block = then_block,
                    Value::Bool(false) => block = else_block,
                    other => {
                        return Err(RuntimeError::trap(format!(
                            "branch condition must be Bool, found `{other}`"
                        )));
                    }
                },
                Terminator::Match {
                    value,
                    arms,
                    otherwise,
                } => {
                    let value = self.eval_operand(&value, frame)?;
                    let mut selected = None;
                    for (pattern, target) in arms {
                        let mut bindings = Vec::new();
                        if match_pattern(&pattern, &value, &mut bindings) {
                            for (local, bound) in bindings {
                                frame.set(local, bound)?;
                            }
                            selected = Some(target);
                            break;
                        }
                    }
                    block = selected.unwrap_or(otherwise);
                }
                Terminator::Return(value) => {
                    return value
                        .as_ref()
                        .map(|operand| self.eval_operand(operand, frame))
                        .unwrap_or(Ok(Value::Unit));
                }
                Terminator::Trap(message) => return Err(RuntimeError::trap(message)),
            }
        }
    }

    fn execute_statement(
        &mut self,
        statement: &MirStatement,
        frame: &mut Frame,
    ) -> Result<(), RuntimeError> {
        match statement {
            MirStatement::Assign { target, value } => {
                let value = self.eval_rvalue(value, frame)?;
                frame.set(*target, value)
            }
        }
    }

    fn eval_rvalue(&mut self, rvalue: &Rvalue, frame: &Frame) -> Result<Value, RuntimeError> {
        match rvalue {
            Rvalue::Use(operand) => self.eval_operand(operand, frame),
            Rvalue::Function(name) => Ok(Value::Function(name.clone())),
            Rvalue::Unary { op, operand } => {
                let value = self.eval_operand(operand, frame)?;
                eval_unary(*op, value)
            }
            Rvalue::Binary { op, left, right } => {
                let left = self.eval_operand(left, frame)?;
                let right = self.eval_operand(right, frame)?;
                eval_binary(*op, left, right)
            }
            Rvalue::Call { callee, args } => {
                let callee = self.eval_operand(callee, frame)?;
                let args = args
                    .iter()
                    .map(|operand| self.eval_operand(operand, frame))
                    .collect::<Result<Vec<_>, _>>()?;
                let Value::Function(name) = callee else {
                    return Err(RuntimeError::trap(format!(
                        "attempted to call non-function value `{callee}`"
                    )));
                };
                self.execute_function(&name, args)
            }
            Rvalue::Struct { type_name, fields } => {
                let fields = fields
                    .iter()
                    .map(|(name, operand)| Ok((name.clone(), self.eval_operand(operand, frame)?)))
                    .collect::<Result<Vec<_>, RuntimeError>>()?;
                Ok(Value::Struct {
                    type_name: type_name.clone(),
                    fields,
                })
            }
            Rvalue::Field { base, field } => {
                let base = self.eval_operand(base, frame)?;
                let Value::Struct { fields, .. } = base else {
                    return Err(RuntimeError::trap(format!(
                        "field access requires struct value, found `{base}`"
                    )));
                };
                fields
                    .into_iter()
                    .find(|(name, _)| name == field)
                    .map(|(_, value)| value)
                    .ok_or_else(|| RuntimeError::trap(format!("missing runtime field `{field}`")))
            }
            Rvalue::Enum { variant, args } => {
                let args = args
                    .iter()
                    .map(|operand| self.eval_operand(operand, frame))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Value::Enum {
                    variant: variant.clone(),
                    args,
                })
            }
        }
    }

    fn eval_operand(&self, operand: &Operand, frame: &Frame) -> Result<Value, RuntimeError> {
        match operand {
            Operand::Local(local) => frame.get(*local),
            Operand::Constant(constant) => Ok(match constant {
                Constant::Unit => Value::Unit,
                Constant::Bool(value) => Value::Bool(*value),
                Constant::I64(value) => Value::I64(*value),
                Constant::F64(value) => Value::F64(*value),
                Constant::String(value) => Value::String(value.clone()),
            }),
        }
    }
}

#[derive(Debug)]
struct Frame {
    locals: Vec<Value>,
}

impl Frame {
    fn new(function: &MirFunction) -> Self {
        Self {
            locals: vec![Value::Unit; function.locals.len()],
        }
    }

    fn get(&self, local: crate::LocalId) -> Result<Value, RuntimeError> {
        self.locals
            .get(local.0 as usize)
            .cloned()
            .ok_or_else(|| RuntimeError::trap(format!("invalid local {}", local.0)))
    }

    fn set(&mut self, local: crate::LocalId, value: Value) -> Result<(), RuntimeError> {
        let slot = self
            .locals
            .get_mut(local.0 as usize)
            .ok_or_else(|| RuntimeError::trap(format!("invalid local {}", local.0)))?;
        *slot = value;
        Ok(())
    }
}

fn match_pattern(
    pattern: &MirPattern,
    value: &Value,
    bindings: &mut Vec<(crate::LocalId, Value)>,
) -> bool {
    match pattern {
        MirPattern::Wildcard => true,
        MirPattern::Bind(local) => {
            bindings.push((*local, value.clone()));
            true
        }
        MirPattern::Integer(expected) => matches!(value, Value::I64(actual) if actual == expected),
        MirPattern::Bool(expected) => matches!(value, Value::Bool(actual) if actual == expected),
        MirPattern::String(expected) => {
            matches!(value, Value::String(actual) if actual == expected)
        }
        MirPattern::Variant { name, args } => {
            let Value::Enum {
                variant,
                args: values,
            } = value
            else {
                return false;
            };
            if variant != name || args.len() != values.len() {
                return false;
            }
            let binding_start = bindings.len();
            for (pattern, value) in args.iter().zip(values) {
                if !match_pattern(pattern, value, bindings) {
                    bindings.truncate(binding_start);
                    return false;
                }
            }
            true
        }
    }
}

fn eval_unary(op: UnaryOp, value: Value) -> Result<Value, RuntimeError> {
    match (op, value) {
        (UnaryOp::Negate, Value::I64(value)) => value
            .checked_neg()
            .map(Value::I64)
            .ok_or_else(|| RuntimeError::trap("integer overflow in unary negation")),
        (UnaryOp::Negate, Value::F64(value)) => Ok(Value::F64(-value)),
        (UnaryOp::Not, Value::Bool(value)) => Ok(Value::Bool(!value)),
        (_, value) => Err(RuntimeError::trap(format!(
            "invalid unary operation on `{value}`"
        ))),
    }
}

fn eval_binary(op: BinaryOp, left: Value, right: Value) -> Result<Value, RuntimeError> {
    use BinaryOp::*;

    match op {
        Equal => return Ok(Value::Bool(left == right)),
        NotEqual => return Ok(Value::Bool(left != right)),
        And | Or => {
            let (Value::Bool(left), Value::Bool(right)) = (left, right) else {
                return Err(RuntimeError::trap(
                    "logical operator requires Bool operands",
                ));
            };
            return Ok(Value::Bool(if op == And {
                left && right
            } else {
                left || right
            }));
        }
        _ => {}
    }

    match (left, right) {
        (Value::I64(left), Value::I64(right)) => eval_i64(op, left, right),
        (Value::F64(left), Value::F64(right)) => eval_f64(op, left, right),
        (left, right) => Err(RuntimeError::trap(format!(
            "invalid operands `{left}` and `{right}`"
        ))),
    }
}

fn eval_i64(op: BinaryOp, left: i64, right: i64) -> Result<Value, RuntimeError> {
    use BinaryOp::*;
    let checked = |value: Option<i64>, operation: &str| {
        value
            .map(Value::I64)
            .ok_or_else(|| RuntimeError::trap(format!("integer overflow in {operation}")))
    };

    match op {
        Add => checked(left.checked_add(right), "addition"),
        Subtract => checked(left.checked_sub(right), "subtraction"),
        Multiply => checked(left.checked_mul(right), "multiplication"),
        Divide => {
            if right == 0 {
                Err(RuntimeError::trap("division by zero"))
            } else {
                checked(left.checked_div(right), "division")
            }
        }
        Remainder => {
            if right == 0 {
                Err(RuntimeError::trap("remainder by zero"))
            } else {
                checked(left.checked_rem(right), "remainder")
            }
        }
        Less => Ok(Value::Bool(left < right)),
        LessEqual => Ok(Value::Bool(left <= right)),
        Greater => Ok(Value::Bool(left > right)),
        GreaterEqual => Ok(Value::Bool(left >= right)),
        _ => Err(RuntimeError::trap("invalid integer operator")),
    }
}

fn eval_f64(op: BinaryOp, left: f64, right: f64) -> Result<Value, RuntimeError> {
    use BinaryOp::*;
    match op {
        Add => Ok(Value::F64(left + right)),
        Subtract => Ok(Value::F64(left - right)),
        Multiply => Ok(Value::F64(left * right)),
        Divide if right == 0.0 => Err(RuntimeError::trap("division by zero")),
        Divide => Ok(Value::F64(left / right)),
        Remainder if right == 0.0 => Err(RuntimeError::trap("remainder by zero")),
        Remainder => Ok(Value::F64(left % right)),
        Less => Ok(Value::Bool(left < right)),
        LessEqual => Ok(Value::Bool(left <= right)),
        Greater => Ok(Value::Bool(left > right)),
        GreaterEqual => Ok(Value::Bool(left >= right)),
        _ => Err(RuntimeError::trap("invalid floating-point operator")),
    }
}
