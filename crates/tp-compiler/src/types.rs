use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Unknown,
    Unit,
    Bool,
    I64,
    F64,
    String,
    Named {
        name: String,
        args: Vec<Type>,
    },
    Function {
        params: Vec<Type>,
        result: Box<Type>,
    },
}

impl Type {
    pub const fn is_numeric(&self) -> bool {
        matches!(self, Self::I64 | Self::F64)
    }

    pub const fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown => f.write_str("<unknown>"),
            Self::Unit => f.write_str("Unit"),
            Self::Bool => f.write_str("Bool"),
            Self::I64 => f.write_str("i64"),
            Self::F64 => f.write_str("f64"),
            Self::String => f.write_str("String"),
            Self::Named { name, args } if args.is_empty() => f.write_str(name),
            Self::Named { name, args } => {
                write!(f, "{name}<")?;
                for (index, arg) in args.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                f.write_str(">")
            }
            Self::Function { params, result } => {
                f.write_str("fn(")?;
                for (index, param) in params.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{param}")?;
                }
                write!(f, ") -> {result}")
            }
        }
    }
}
