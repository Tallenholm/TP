use crate::{
    BinaryOp, Block, Diagnostic, Expr, ExprKind, FnDecl, Item, Module, Param, SourceFile, Span,
    Stmt, Token, TokenKind, TypeRef, UnaryOp,
};

#[derive(Debug, Default)]
pub struct ParseResult {
    pub module: Option<Module>,
    pub diagnostics: Vec<Diagnostic>,
}

pub struct Parser<'a> {
    source: &'a SourceFile,
    tokens: Vec<Token>,
    current: usize,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a SourceFile, tokens: Vec<Token>) -> Self {
        Self {
            source,
            tokens,
            current: 0,
            diagnostics: Vec::new(),
        }
    }

    pub fn parse_module(mut self) -> ParseResult {
        let mut items = Vec::new();
        while !self.at(TokenKind::Eof) {
            if self.at(TokenKind::Fn) {
                if let Some(function) = self.parse_function() {
                    items.push(Item::Function(function));
                } else {
                    self.synchronize_item();
                }
            } else {
                self.error_current("expected a top-level declaration");
                self.synchronize_item();
            }
        }

        ParseResult {
            module: Some(Module { items }),
            diagnostics: self.diagnostics,
        }
    }

    fn parse_function(&mut self) -> Option<FnDecl> {
        let start = self.expect(TokenKind::Fn, "expected `fn`")?.span.start;
        let name_token = self.expect(TokenKind::Identifier, "expected function name")?;
        let name = self.text(&name_token).to_owned();
        self.expect(TokenKind::LParen, "expected `(` after function name")?;

        let mut params = Vec::new();
        if !self.at(TokenKind::RParen) {
            loop {
                let param_start = self.current_span().start;
                let param_name_token =
                    self.expect(TokenKind::Identifier, "expected parameter name")?;
                let param_name = self.text(&param_name_token).to_owned();
                self.expect(TokenKind::Colon, "expected `:` after parameter name")?;
                let ty = self.parse_type_ref()?;
                let span = Span::new(self.source.id(), param_start, ty.span.end);
                params.push(Param {
                    name: param_name,
                    ty,
                    span,
                });
                if !self.consume(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(TokenKind::RParen, "expected `)` after parameters")?;

        let return_type = if self.consume(TokenKind::Arrow) {
            Some(self.parse_type_ref()?)
        } else {
            None
        };

        let body = self.parse_block()?;
        let span = Span::new(self.source.id(), start, body.span.end);
        Some(FnDecl {
            name,
            params,
            return_type,
            body,
            span,
        })
    }

    fn parse_type_ref(&mut self) -> Option<TypeRef> {
        let name_token = self.expect(TokenKind::Identifier, "expected type name")?;
        let start = name_token.span.start;
        let name = self.text(&name_token).to_owned();
        let mut args = Vec::new();

        if self.consume(TokenKind::Less) {
            if !self.at(TokenKind::Greater) {
                loop {
                    args.push(self.parse_type_ref()?);
                    if !self.consume(TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.expect(TokenKind::Greater, "expected `>` after type arguments")?;
        }

        let optional = self.consume(TokenKind::Question);
        let end = self.previous_span().end;
        Some(TypeRef {
            name,
            args,
            optional,
            span: Span::new(self.source.id(), start, end),
        })
    }

    fn parse_block(&mut self) -> Option<Block> {
        let open = self.expect(TokenKind::LBrace, "expected `{`")?;
        let mut statements = Vec::new();

        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            let before = self.current;
            if let Some(statement) = self.parse_statement() {
                statements.push(statement);
            } else {
                self.synchronize_statement();
            }
            if self.current == before {
                self.advance();
            }
        }

        let close = self.expect(TokenKind::RBrace, "expected `}` after block")?;
        Some(Block {
            statements,
            span: Span::new(self.source.id(), open.span.start, close.span.end),
        })
    }

    fn parse_statement(&mut self) -> Option<Stmt> {
        match self.current_kind() {
            TokenKind::Let => self.parse_binding(false),
            TokenKind::Var => self.parse_binding(true),
            TokenKind::Return => self.parse_return(),
            TokenKind::While => self.parse_while(),
            _ => {
                let expr = self.parse_expression()?;
                let terminated = self.consume(TokenKind::Semicolon);
                Some(Stmt::Expr { expr, terminated })
            }
        }
    }

    fn parse_binding(&mut self, mutable: bool) -> Option<Stmt> {
        let start = self.advance().span.start;
        let name_token = self.expect(TokenKind::Identifier, "expected binding name")?;
        let name = self.text(&name_token).to_owned();
        let type_annotation = if self.consume(TokenKind::Colon) {
            Some(self.parse_type_ref()?)
        } else {
            None
        };
        self.expect(TokenKind::Equal, "expected `=` in binding")?;
        let value = self.parse_expression()?;
        self.consume(TokenKind::Semicolon);
        let end = self.previous_span().end.max(value.span.end);
        Some(Stmt::Let {
            mutable,
            name,
            type_annotation,
            value,
            span: Span::new(self.source.id(), start, end),
        })
    }

    fn parse_return(&mut self) -> Option<Stmt> {
        let start = self.advance().span.start;
        let value = if self.at(TokenKind::Semicolon) || self.at(TokenKind::RBrace) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.consume(TokenKind::Semicolon);
        Some(Stmt::Return {
            value,
            span: Span::new(self.source.id(), start, self.previous_span().end),
        })
    }

    fn parse_while(&mut self) -> Option<Stmt> {
        let start = self.advance().span.start;
        let condition = self.parse_expression()?;
        let body = self.parse_block()?;
        let end = body.span.end;
        Some(Stmt::While {
            condition,
            body,
            span: Span::new(self.source.id(), start, end),
        })
    }

    fn parse_expression(&mut self) -> Option<Expr> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Option<Expr> {
        let left = self.parse_binary(1)?;
        if !self.consume(TokenKind::Equal) {
            return Some(left);
        }

        let start = left.span.start;
        let target = match left.kind {
            ExprKind::Name(name) => name,
            _ => {
                self.diagnostics.push(
                    Diagnostic::error("TP-E0100", "invalid assignment target")
                        .with_primary(left.span),
                );
                return None;
            }
        };
        let value = self.parse_assignment()?;
        let end = value.span.end;
        Some(Expr {
            kind: ExprKind::Assign {
                target,
                value: Box::new(value),
            },
            span: Span::new(self.source.id(), start, end),
        })
    }

    fn parse_binary(&mut self, min_precedence: u8) -> Option<Expr> {
        let mut left = self.parse_unary()?;

        loop {
            let Some((precedence, op)) = binary_operator(self.current_kind()) else {
                break;
            };
            if precedence < min_precedence {
                break;
            }
            self.advance();
            let right = self.parse_binary(precedence + 1)?;
            let span = Span::new(self.source.id(), left.span.start, right.span.end);
            left = Expr {
                kind: ExprKind::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
            };
        }

        Some(left)
    }

    fn parse_unary(&mut self) -> Option<Expr> {
        let op = match self.current_kind() {
            TokenKind::Minus => Some(UnaryOp::Negate),
            TokenKind::Bang => Some(UnaryOp::Not),
            _ => None,
        };
        if let Some(op) = op {
            let start = self.advance().span.start;
            let expr = self.parse_unary()?;
            let end = expr.span.end;
            return Some(Expr {
                kind: ExprKind::Unary {
                    op,
                    expr: Box::new(expr),
                },
                span: Span::new(self.source.id(), start, end),
            });
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Option<Expr> {
        let mut expr = self.parse_primary()?;
        while self.consume(TokenKind::LParen) {
            let mut args = Vec::new();
            if !self.at(TokenKind::RParen) {
                loop {
                    args.push(self.parse_expression()?);
                    if !self.consume(TokenKind::Comma) {
                        break;
                    }
                }
            }
            let close = self.expect(TokenKind::RParen, "expected `)` after arguments")?;
            let start = expr.span.start;
            expr = Expr {
                kind: ExprKind::Call {
                    callee: Box::new(expr),
                    args,
                },
                span: Span::new(self.source.id(), start, close.span.end),
            };
        }
        Some(expr)
    }

    fn parse_primary(&mut self) -> Option<Expr> {
        let token = self.current_token().clone();
        match token.kind {
            TokenKind::Integer => {
                self.advance();
                let value = self.text(&token).parse::<i64>().ok()?;
                Some(Expr {
                    kind: ExprKind::Integer(value),
                    span: token.span,
                })
            }
            TokenKind::Float => {
                self.advance();
                let value = self.text(&token).parse::<f64>().ok()?;
                Some(Expr {
                    kind: ExprKind::Float(value),
                    span: token.span,
                })
            }
            TokenKind::String => {
                self.advance();
                Some(Expr {
                    kind: ExprKind::String(decode_string(self.text(&token))),
                    span: token.span,
                })
            }
            TokenKind::True | TokenKind::False => {
                self.advance();
                Some(Expr {
                    kind: ExprKind::Bool(token.kind == TokenKind::True),
                    span: token.span,
                })
            }
            TokenKind::Identifier => {
                self.advance();
                Some(Expr {
                    kind: ExprKind::Name(self.text(&token).to_owned()),
                    span: token.span,
                })
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RParen, "expected `)` after expression")?;
                Some(expr)
            }
            TokenKind::If => self.parse_if_expression(),
            _ => {
                self.error_current("expected expression");
                None
            }
        }
    }

    fn parse_if_expression(&mut self) -> Option<Expr> {
        let start = self.advance().span.start;
        let condition = self.parse_expression()?;
        let then_branch = self.parse_block()?;
        let else_branch = if self.consume(TokenKind::Else) {
            Some(self.parse_block()?)
        } else {
            None
        };
        let end = else_branch
            .as_ref()
            .map(|block| block.span.end)
            .unwrap_or(then_branch.span.end);
        Some(Expr {
            kind: ExprKind::If {
                condition: Box::new(condition),
                then_branch,
                else_branch,
            },
            span: Span::new(self.source.id(), start, end),
        })
    }

    fn synchronize_statement(&mut self) {
        while !self.at(TokenKind::Eof) && !self.at(TokenKind::RBrace) {
            if self.consume(TokenKind::Semicolon) {
                return;
            }
            if matches!(
                self.current_kind(),
                TokenKind::Let | TokenKind::Var | TokenKind::Return | TokenKind::While
            ) {
                return;
            }
            self.advance();
        }
    }

    fn synchronize_item(&mut self) {
        while !self.at(TokenKind::Eof) {
            if self.at(TokenKind::Fn) {
                return;
            }
            self.advance();
        }
    }

    fn expect(&mut self, kind: TokenKind, message: &str) -> Option<Token> {
        if self.at(kind) {
            Some(self.advance())
        } else {
            self.error_current(message);
            None
        }
    }

    fn consume(&mut self, kind: TokenKind) -> bool {
        if self.at(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.current_kind() == kind
    }

    fn current_kind(&self) -> TokenKind {
        self.current_token().kind
    }

    fn current_token(&self) -> &Token {
        &self.tokens[self.current.min(self.tokens.len().saturating_sub(1))]
    }

    fn current_span(&self) -> Span {
        self.current_token().span
    }

    fn previous_span(&self) -> Span {
        if self.current == 0 {
            self.current_span()
        } else {
            self.tokens[self.current - 1].span
        }
    }

    fn advance(&mut self) -> Token {
        let token = self.current_token().clone();
        if token.kind != TokenKind::Eof {
            self.current += 1;
        }
        token
    }

    fn text<'b>(&self, token: &'b Token) -> &'a str {
        token.text(self.source).unwrap_or("")
    }

    fn error_current(&mut self, message: impl Into<String>) {
        self.diagnostics.push(
            Diagnostic::error("TP-E0100", message).with_primary(self.current_span()),
        );
    }
}

fn binary_operator(kind: TokenKind) -> Option<(u8, BinaryOp)> {
    Some(match kind {
        TokenKind::OrOr => (1, BinaryOp::Or),
        TokenKind::AndAnd => (2, BinaryOp::And),
        TokenKind::EqualEqual => (3, BinaryOp::Equal),
        TokenKind::BangEqual => (3, BinaryOp::NotEqual),
        TokenKind::Less => (4, BinaryOp::Less),
        TokenKind::LessEqual => (4, BinaryOp::LessEqual),
        TokenKind::Greater => (4, BinaryOp::Greater),
        TokenKind::GreaterEqual => (4, BinaryOp::GreaterEqual),
        TokenKind::Plus => (5, BinaryOp::Add),
        TokenKind::Minus => (5, BinaryOp::Subtract),
        TokenKind::Star => (6, BinaryOp::Multiply),
        TokenKind::Slash => (6, BinaryOp::Divide),
        TokenKind::Percent => (6, BinaryOp::Remainder),
        _ => return None,
    })
}

fn decode_string(token_text: &str) -> String {
    let inner = token_text
        .strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
        .unwrap_or(token_text);
    let mut out = String::new();
    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('"') => out.push('"'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}
