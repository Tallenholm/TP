use crate::{Diagnostic, SourceFile, Span, Token, TokenKind};

#[derive(Debug, Default)]
pub struct LexResult {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
}

pub struct Lexer<'a> {
    source: &'a SourceFile,
    offset: usize,
}

impl<'a> Lexer<'a> {
    pub const fn new(source: &'a SourceFile) -> Self {
        Self { source, offset: 0 }
    }

    pub fn lex(mut self) -> LexResult {
        let mut result = LexResult::default();

        while let Some(ch) = self.current() {
            if ch.is_whitespace() {
                self.advance();
                continue;
            }

            let start = self.offset;
            let kind = if is_identifier_start(ch) {
                self.advance();
                while self.current().is_some_and(is_identifier_continue) {
                    self.advance();
                }
                TokenKind::from_identifier(&self.source.text()[start..self.offset])
            } else if ch.is_ascii_digit() {
                self.advance();
                while self.current().is_some_and(|next| next.is_ascii_digit()) {
                    self.advance();
                }
                TokenKind::Integer
            } else {
                match ch {
                    '(' => self.single(TokenKind::LParen),
                    ')' => self.single(TokenKind::RParen),
                    '{' => self.single(TokenKind::LBrace),
                    '}' => self.single(TokenKind::RBrace),
                    ',' => self.single(TokenKind::Comma),
                    ':' => self.single(TokenKind::Colon),
                    '+' => self.single(TokenKind::Plus),
                    '=' => self.single(TokenKind::Equal),
                    '-' if self.peek() == Some('>') => {
                        self.advance();
                        self.advance();
                        TokenKind::Arrow
                    }
                    _ => {
                        self.advance();
                        let span = Span::new(self.source.id(), start, self.offset);
                        result.diagnostics.push(
                            Diagnostic::error(
                                "TP-E0001",
                                format!("invalid character `{ch}`"),
                            )
                            .with_primary(span),
                        );
                        continue;
                    }
                }
            };

            result.tokens.push(Token::new(
                kind,
                Span::new(self.source.id(), start, self.offset),
            ));
        }

        let end = self.source.text().len();
        result.tokens.push(Token::new(
            TokenKind::Eof,
            Span::new(self.source.id(), end, end),
        ));
        result
    }

    fn current(&self) -> Option<char> {
        self.source.text().get(self.offset..)?.chars().next()
    }

    fn peek(&self) -> Option<char> {
        let mut chars = self.source.text().get(self.offset..)?.chars();
        chars.next()?;
        chars.next()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.current()?;
        self.offset += ch.len_utf8();
        Some(ch)
    }

    fn single(&mut self, kind: TokenKind) -> TokenKind {
        self.advance();
        kind
    }
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch.is_alphabetic()
}

fn is_identifier_continue(ch: char) -> bool {
    ch == '_' || ch.is_alphanumeric()
}
