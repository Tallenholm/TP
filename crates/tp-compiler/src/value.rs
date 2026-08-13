use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Unit,
    Bool(bool),
    I64(i64),
    F64(f64),
    String(String),
    Function(String),
    Struct {
        type_name: String,
        fields: Vec<(String, Value)>,
    },
    Enum {
        variant: String,
        args: Vec<Value>,
    },
}

impl Value {
    pub const fn as_i64(&self) -> Option<i64> {
        match self {
            Self::I64(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unit => f.write_str("()"),
            Self::Bool(value) => write!(f, "{value}"),
            Self::I64(value) => write!(f, "{value}"),
            Self::F64(value) => write!(f, "{value}"),
            Self::String(value) => f.write_str(value),
            Self::Function(name) => write!(f, "<fn {name}>"),
            Self::Struct { type_name, fields } => {
                write!(f, "{type_name} {{ ")?;
                for (index, (name, value)) in fields.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{name}: {value}")?;
                }
                f.write_str(" }")
            }
            Self::Enum { variant, args } if args.is_empty() => f.write_str(variant),
            Self::Enum { variant, args } => {
                write!(f, "{variant}(")?;
                for (index, value) in args.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{value}")?;
                }
                f.write_str(")")
            }
        }
    }
}
