use crate::{SourceFile, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Identifier,
    Integer,
    Float,
    String,

    Fn,
    Let,
    Var,
    If,
    Else,
    While,
    Return,
    Struct,
    Enum,
    Match,
    True,
    False,
    Import,
    As,

    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Dot,
    Colon,
    Semicolon,
    Question,

    Arrow,
    FatArrow,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    EqualEqual,
    Bang,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    AndAnd,
    OrOr,

    Eof,
}

impl TokenKind {
    pub(crate) fn from_identifier(text: &str) -> Self {
        match text {
            "fn" => Self::Fn,
            "let" => Self::Let,
            "var" => Self::Var,
            "if" => Self::If,
            "else" => Self::Else,
            "while" => Self::While,
            "return" => Self::Return,
            "struct" => Self::Struct,
            "enum" => Self::Enum,
            "match" => Self::Match,
            "true" => Self::True,
            "false" => Self::False,
            "import" => Self::Import,
            "as" => Self::As,
            _ => Self::Identifier,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub const fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub fn text<'a>(&self, source: &'a SourceFile) -> Option<&'a str> {
        if self.span.source != source.id() {
            return None;
        }
        source.text().get(self.span.start..self.span.end)
    }
}
