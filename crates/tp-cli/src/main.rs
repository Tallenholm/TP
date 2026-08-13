use std::{path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand, ValueEnum};
use tp_compiler::{Compiler, Diagnostic, RunFailure, RuntimeError, Severity};

#[derive(Debug, Parser)]
#[command(name = "tp", version, about = "TP programming language toolchain")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Parse and type-check a TP source file.
    Check {
        /// Entry TP source file.
        path: PathBuf,
        /// Diagnostic output format.
        #[arg(long, value_enum, default_value_t = DiagnosticFormat::Human)]
        diagnostic_format: DiagnosticFormat,
    },
    /// Parse, type-check, lower, and execute a TP program.
    Run {
        /// Entry TP source file.
        path: PathBuf,
        /// Diagnostic output format.
        #[arg(long, value_enum, default_value_t = DiagnosticFormat::Human)]
        diagnostic_format: DiagnosticFormat,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum DiagnosticFormat {
    Human,
    Json,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let compiler = Compiler::new();

    match cli.command {
        Command::Check {
            path,
            diagnostic_format,
        } => {
            let report = compiler.check_path(&path);
            if report.diagnostics.is_empty() {
                ExitCode::SUCCESS
            } else {
                emit_diagnostics(&report.diagnostics, diagnostic_format);
                ExitCode::FAILURE
            }
        }
        Command::Run {
            path,
            diagnostic_format,
        } => match compiler.run_path(&path) {
            Ok(report) => {
                print!("{}", report.output);
                ExitCode::SUCCESS
            }
            Err(RunFailure::Compile(diagnostics)) => {
                emit_diagnostics(&diagnostics, diagnostic_format);
                ExitCode::FAILURE
            }
            Err(RunFailure::Runtime(error)) => {
                emit_runtime_error(&error, diagnostic_format);
                ExitCode::FAILURE
            }
        },
    }
}

fn emit_diagnostics(diagnostics: &[Diagnostic], format: DiagnosticFormat) {
    for diagnostic in diagnostics {
        match format {
            DiagnosticFormat::Human => eprintln!(
                "{}[{}]: {}",
                diagnostic.severity, diagnostic.code, diagnostic.message
            ),
            DiagnosticFormat::Json => eprintln!(
                "{{\"schema\":1,\"code\":\"{}\",\"severity\":\"{}\",\"message\":\"{}\"}}",
                json_escape(diagnostic.code),
                diagnostic.severity,
                json_escape(&diagnostic.message)
            ),
        }
    }
}

fn emit_runtime_error(error: &RuntimeError, format: DiagnosticFormat) {
    match format {
        DiagnosticFormat::Human => eprintln!("error[{}]: {}", error.code, error.message),
        DiagnosticFormat::Json => eprintln!(
            "{{\"schema\":1,\"code\":\"{}\",\"severity\":\"{}\",\"message\":\"{}\"}}",
            json_escape(error.code),
            Severity::Error,
            json_escape(&error.message)
        ),
    }
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
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
    escaped
}
