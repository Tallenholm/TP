use std::fmt;

use crate::{SourceFile, SourceManager, Span};

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

    pub fn new(severity: Severity, code: &'static str, message: impl Into<String>) -> Self {
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
    let mut sources = SourceManager::default();
    sources.insert(source.clone());
    render_diagnostic_with_sources(&sources, diagnostic)
}

pub fn render_diagnostic_with_sources(
    sources: &SourceManager,
    diagnostic: &Diagnostic,
) -> String {
    let mut out = format!(
        "{}[{}]: {}\n",
        diagnostic.severity, diagnostic.code, diagnostic.message
    );

    if let Some(span) = diagnostic.primary {
        if let Some(source) = sources.get(span.source) {
            render_primary_span(&mut out, source, span);
        }
    }

    for label in &diagnostic.labels {
        let Some(source) = sources.get(label.span.source) else {
            continue;
        };
        let Some(location) = source.line_col(label.span.start) else {
            continue;
        };
        out.push_str(&format!(
            "  = label: {}:{}:{}",
            source.name(),
            location.line,
            location.column
        ));
        if let Some(message) = &label.message {
            out.push_str(&format!(": {message}"));
        }
        out.push('\n');
    }

    if let Some(help) = &diagnostic.help {
        out.push_str(&format!("  = help: {help}\n"));
    }

    out
}

pub fn render_diagnostic_json(sources: &SourceManager, diagnostic: &Diagnostic) -> String {
    let primary = diagnostic
        .primary
        .map(|span| render_json_range(sources, span))
        .unwrap_or_else(|| "null".to_owned());

    let labels = diagnostic
        .labels
        .iter()
        .map(|label| {
            let message = label
                .message
                .as_deref()
                .map(json_string)
                .unwrap_or_else(|| "null".to_owned());
            format!(
                "{{\"range\":{},\"message\":{}}}",
                render_json_range(sources, label.span),
                message
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let help = diagnostic
        .help
        .as_deref()
        .map(json_string)
        .unwrap_or_else(|| "null".to_owned());

    format!(
        "{{\"schema\":1,\"code\":{},\"severity\":{},\"message\":{},\"primary\":{},\"labels\":[{}],\"help\":{}}}",
        json_string(diagnostic.code),
        json_string(&diagnostic.severity.to_string()),
        json_string(&diagnostic.message),
        primary,
        labels,
        help
    )
}

fn render_primary_span(out: &mut String, source: &SourceFile, span: Span) {
    let Some(location) = source.line_col(span.start) else {
        return;
    };
    out.push_str(&format!(
        " --> {}:{}:{}\n",
        source.name(),
        location.line,
        location.column
    ));
    out.push_str("  |\n");

    if let Some(line) = source.line_text(location.line) {
        out.push_str(&format!("{} | {}\n", location.line, line));
        let padding = " ".repeat(location.column.saturating_sub(1));
        let caret_count = first_line_char_width(source, span).max(1);
        out.push_str(&format!("  | {}{}\n", padding, "^".repeat(caret_count)));
    }
}

fn first_line_char_width(source: &SourceFile, span: Span) -> usize {
    let Some(tail) = source.text().get(span.start..) else {
        return 1;
    };
    let line_end = tail
        .find(['\n', '\r'])
        .map(|offset| span.start + offset)
        .unwrap_or(source.text().len());
    let end = span.end.min(line_end);
    source
        .text()
        .get(span.start..end)
        .map(|text| text.chars().count())
        .unwrap_or(1)
}

fn render_json_range(sources: &SourceManager, span: Span) -> String {
    let Some(source) = sources.get(span.source) else {
        return format!(
            "{{\"file\":null,\"source_id\":{},\"start\":{{\"byte\":{},\"line\":null,\"column\":null}},\"end\":{{\"byte\":{},\"line\":null,\"column\":null}}}}",
            span.source.raw(),
            span.start,
            span.end
        );
    };

    let start = source.line_col(span.start);
    let end = source.line_col(span.end);
    format!(
        "{{\"file\":{},\"source_id\":{},\"start\":{{\"byte\":{},\"line\":{},\"column\":{}}},\"end\":{{\"byte\":{},\"line\":{},\"column\":{}}}}}",
        json_string(source.name()),
        span.source.raw(),
        span.start,
        json_usize(start.map(|loc| loc.line)),
        json_usize(start.map(|loc| loc.column)),
        span.end,
        json_usize(end.map(|loc| loc.line)),
        json_usize(end.map(|loc| loc.column))
    )
}

fn json_usize(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_owned())
}

fn json_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => {
                use std::fmt::Write;
                let _ = write!(escaped, "\\u{:04x}", ch as u32);
            }
            ch => escaped.push(ch),
        }
    }
    escaped.push('"');
    escaped
}
