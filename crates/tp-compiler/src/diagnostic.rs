use std::fmt;

use crate::{SourceFile, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Note,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Note => "note",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub span: Span,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub severity: Severity,
    pub message: String,
    pub primary: Option<Span>,
    pub labels: Vec<Label>,
    pub help: Option<String>,
}

impl Diagnostic {
    pub fn error(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(Severity::Error, code, message)
    }

    pub fn warning(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(Severity::Warning, code, message)
    }

    pub fn note(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(Severity::Note, code, message)
    }

    pub fn new(
        severity: Severity,
        code: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity,
            message: message.into(),
            primary: None,
            labels: Vec::new(),
            help: None,
        }
    }

    pub fn with_primary(mut self, span: Span) -> Self {
        self.primary = Some(span);
        self
    }

    pub fn with_label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.push(Label {
            span,
            message: Some(message.into()),
        });
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}

pub fn render_diagnostic(source: &SourceFile, diagnostic: &Diagnostic) -> String {
    let mut out = format!(
        "{}[{}]: {}\n",
        diagnostic.severity, diagnostic.code, diagnostic.message
    );

    if let Some(span) = diagnostic.primary.filter(|span| span.source == source.id()) {
        if let Some(location) = source.line_col(span.start) {
            out.push_str(&format!(
                " --> {}:{}:{}\n",
                source.name(), location.line, location.column
            ));
            out.push_str("  |\n");

            if let Some(line) = source.line_text(location.line) {
                out.push_str(&format!("{} | {}\n", location.line, line));
                let padding = " ".repeat(location.column.saturating_sub(1));
                let caret_count = span.len().max(1);
                out.push_str(&format!("  | {}{}\n", padding, "^".repeat(caret_count)));
            }
        }
    }

    if let Some(help) = &diagnostic.help {
        out.push_str(&format!("  = help: {help}\n"));
    }

    out
}
