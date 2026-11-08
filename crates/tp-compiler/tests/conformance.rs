use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use tp_compiler::{Compiler, RunFailure, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    CheckPass,
    CompileFail,
    Run,
    RuntimeFail,
}

#[derive(Debug, Clone, PartialEq)]
enum ExpectedValue {
    Unit,
    Bool(bool),
    I64(i64),
    F64(f64),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
struct Expectation {
    outcome: Outcome,
    diagnostics: Vec<String>,
    value: Option<ExpectedValue>,
    runtime_code: Option<String>,
    runtime_message: Option<String>,
}

fn parse_expectation_str(input: &str) -> Result<Expectation, String> {
    let mut seen = HashSet::new();
    let mut outcome = None;
    let mut diagnostics = Vec::new();
    let mut value = None;
    let mut runtime_code = None;
    let mut runtime_message = None;

    for (index, raw_line) in input.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((key, raw_value)) = line.split_once('=') else {
            return Err(format!(
                "invalid expectation line {}: expected key=value",
                index + 1
            ));
        };
        let key = key.trim();
        let raw_value = raw_value.trim();

        if !seen.insert(key.to_owned()) {
            return Err(format!("duplicate expectation key `{key}`"));
        }

        match key {
            "outcome" => {
                outcome = Some(match raw_value {
                    "check_pass" => Outcome::CheckPass,
                    "compile_fail" => Outcome::CompileFail,
                    "run" => Outcome::Run,
                    "runtime_fail" => Outcome::RuntimeFail,
                    other => return Err(format!("invalid outcome `{other}`")),
                });
            }
            "diagnostics" => {
                diagnostics = raw_value
                    .split(',')
                    .map(str::trim)
                    .filter(|code| !code.is_empty())
                    .map(str::to_owned)
                    .collect();
            }
            "value" => {
                value = Some(parse_expected_value(raw_value)?);
            }
            "runtime_code" => runtime_code = Some(raw_value.to_owned()),
            "runtime_message" => runtime_message = Some(raw_value.to_owned()),
            other => return Err(format!("unknown expectation key `{other}`")),
        }
    }

    let outcome = outcome.ok_or_else(|| "missing expectation key `outcome`".to_owned())?;
    match outcome {
        Outcome::CompileFail if diagnostics.is_empty() => {
            return Err("compile_fail requires `diagnostics`".to_owned());
        }
        Outcome::RuntimeFail if runtime_code.is_none() => {
            return Err("runtime_fail requires `runtime_code`".to_owned());
        }
        _ => {}
    }

    Ok(Expectation {
        outcome,
        diagnostics,
        value,
        runtime_code,
        runtime_message,
    })
}

fn parse_expected_value(raw: &str) -> Result<ExpectedValue, String> {
    if raw == "unit" {
        return Ok(ExpectedValue::Unit);
    }
    if raw == "bool:true" {
        return Ok(ExpectedValue::Bool(true));
    }
    if raw == "bool:false" {
        return Ok(ExpectedValue::Bool(false));
    }
    if let Some(value) = raw.strip_prefix("i64:") {
        return value
            .parse::<i64>()
            .map(ExpectedValue::I64)
            .map_err(|error| format!("invalid expected i64 `{value}`: {error}"));
    }
    if let Some(value) = raw.strip_prefix("f64:") {
        let parsed = match value {
            "nan" => f64::NAN,
            "inf" => f64::INFINITY,
            "-inf" => f64::NEG_INFINITY,
            _ => value
                .parse::<f64>()
                .map_err(|error| format!("invalid expected f64 `{value}`: {error}"))?,
        };
        return Ok(ExpectedValue::F64(parsed));
    }
    if let Some(value) = raw.strip_prefix("string:") {
        return Ok(ExpectedValue::String(value.to_owned()));
    }

    Err(format!("invalid expected value `{raw}`"))
}

fn discover_cases(root: &Path) -> Result<Vec<PathBuf>, String> {
    if !root.is_dir() {
        return Err(format!(
            "conformance root is not a directory: {}",
            root.display()
        ));
    }

    let mut cases = Vec::new();
    discover_cases_recursive(root, &mut cases)?;
    cases.sort();
    Ok(cases)
}

fn discover_cases_recursive(directory: &Path, cases: &mut Vec<PathBuf>) -> Result<(), String> {
    if directory.join("main.tp").is_file() && directory.join("expect.txt").is_file() {
        cases.push(directory.to_path_buf());
        return Ok(());
    }

    let mut children = fs::read_dir(directory)
        .map_err(|error| format!("unable to read {}: {error}", directory.display()))?
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .map_err(|error| format!("unable to read {} entry: {error}", directory.display()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    children.sort();

    for child in children {
        if child.is_dir() {
            discover_cases_recursive(&child, cases)?;
        }
    }

    Ok(())
}

fn run_case(case: &Path) -> Result<(), String> {
    let expectation_path = case.join("expect.txt");
    let expectation_text = fs::read_to_string(&expectation_path)
        .map_err(|error| format!("unable to read {}: {error}", expectation_path.display()))?;
    let expectation = parse_expectation_str(&expectation_text)?;
    let main_path = case.join("main.tp");
    if !main_path.is_file() {
        return Err("case is missing main.tp".to_owned());
    }

    let stdout_path = case.join("stdout.txt");
    let expected_stdout = if stdout_path.is_file() {
        Some(
            fs::read_to_string(&stdout_path)
                .map_err(|error| format!("unable to read {}: {error}", stdout_path.display()))?,
        )
    } else {
        None
    };

    let compiler = Compiler::new();
    match expectation.outcome {
        Outcome::CheckPass => {
            let report = compiler.check_path(&main_path);
            if report.diagnostics.is_empty() {
                Ok(())
            } else {
                Err(format!(
                    "expected check_pass, got diagnostics {:?}",
                    diagnostic_codes(&report.diagnostics)
                ))
            }
        }
        Outcome::CompileFail => {
            let report = compiler.check_path(&main_path);
            let actual = diagnostic_codes(&report.diagnostics);
            if actual == expectation.diagnostics {
                Ok(())
            } else {
                Err(format!(
                    "expected diagnostics {:?}, got {actual:?}",
                    expectation.diagnostics
                ))
            }
        }
        Outcome::Run => {
            let report = compiler
                .run_path(&main_path)
                .map_err(|failure| match failure {
                    RunFailure::Compile(diagnostics) => format!(
                        "expected run success, got compile diagnostics {:?}",
                        diagnostic_codes(&diagnostics)
                    ),
                    RunFailure::Runtime(error) => {
                        format!("expected run success, got runtime error {error}")
                    }
                })?;

            if let Some(expected) = &expectation.value {
                if !value_matches(expected, &report.value) {
                    return Err(format!(
                        "expected runtime value {expected:?}, got {:?}",
                        report.value
                    ));
                }
            } else if expected_stdout.is_none() {
                return Err("run case requires `value` or stdout.txt".to_owned());
            }

            if let Some(expected) = expected_stdout {
                if report.output != expected {
                    return Err(format!(
                        "expected stdout {expected:?}, got {:?}",
                        report.output
                    ));
                }
            }

            Ok(())
        }
        Outcome::RuntimeFail => match compiler.run_path(&main_path) {
            Err(RunFailure::Runtime(error)) => {
                let expected_code = expectation
                    .runtime_code
                    .as_deref()
                    .expect("runtime_code validated by parser");
                if error.code != expected_code {
                    return Err(format!(
                        "expected runtime code {expected_code}, got {}",
                        error.code
                    ));
                }
                if let Some(expected_message) = &expectation.runtime_message {
                    if !error.message.contains(expected_message) {
                        return Err(format!(
                            "expected runtime message containing {expected_message:?}, got {:?}",
                            error.message
                        ));
                    }
                }
                Ok(())
            }
            Err(RunFailure::Compile(diagnostics)) => Err(format!(
                "expected runtime failure, got compile diagnostics {:?}",
                diagnostic_codes(&diagnostics)
            )),
            Ok(report) => Err(format!(
                "expected runtime failure, got value {:?}",
                report.value
            )),
        },
    }
}

fn diagnostic_codes(diagnostics: &[tp_compiler::Diagnostic]) -> Vec<String> {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.to_owned())
        .collect()
}

fn value_matches(expected: &ExpectedValue, actual: &Value) -> bool {
    match (expected, actual) {
        (ExpectedValue::Unit, Value::Unit) => true,
        (ExpectedValue::Bool(expected), Value::Bool(actual)) => expected == actual,
        (ExpectedValue::I64(expected), Value::I64(actual)) => expected == actual,
        (ExpectedValue::F64(expected), Value::F64(actual)) if expected.is_nan() => actual.is_nan(),
        (ExpectedValue::F64(expected), Value::F64(actual)) => expected == actual,
        (ExpectedValue::String(expected), Value::String(actual)) => expected == actual,
        _ => false,
    }
}

#[test]
fn expectation_rejects_unknown_key() {
    let error = parse_expectation_str("outcome=check_pass\nmystery=yes\n")
        .expect_err("unknown metadata key must fail");
    assert!(
        error.contains("unknown expectation key `mystery`"),
        "{error}"
    );
}

#[test]
fn expectation_rejects_duplicate_key() {
    let error = parse_expectation_str("outcome=check_pass\noutcome=run\n")
        .expect_err("duplicate metadata key must fail");
    assert!(
        error.contains("duplicate expectation key `outcome`"),
        "{error}"
    );
}

#[test]
fn expectation_rejects_invalid_outcome() {
    let error = parse_expectation_str("outcome=maybe\n").expect_err("invalid outcome must fail");
    assert!(error.contains("invalid outcome `maybe`"), "{error}");
}

#[test]
fn compile_fail_requires_diagnostics() {
    let error = parse_expectation_str("outcome=compile_fail\n")
        .expect_err("compile_fail without diagnostics must fail");
    assert!(
        error.contains("compile_fail requires `diagnostics`"),
        "{error}"
    );
}

#[test]
fn runtime_fail_requires_runtime_code() {
    let error = parse_expectation_str("outcome=runtime_fail\n")
        .expect_err("runtime_fail without runtime_code must fail");
    assert!(
        error.contains("runtime_fail requires `runtime_code`"),
        "{error}"
    );
}

#[test]
fn conformance_case_discovery_is_sorted() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/conformance/harness");
    let cases = discover_cases(&root).expect("discover seed conformance cases");
    let names: Vec<_> = cases
        .iter()
        .map(|path| {
            path.file_name()
                .expect("case directory name")
                .to_string_lossy()
                .into_owned()
        })
        .collect();

    assert_eq!(
        names,
        vec!["check-pass", "compile-fail", "run-stdout", "run-value",]
    );
}

#[test]
fn seed_conformance_cases_match_expectations() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/conformance/harness");
    let cases = discover_cases(&root).expect("discover seed conformance cases");
    for case in cases {
        run_case(&case).unwrap_or_else(|error| {
            panic!("{}: {error}", case.display());
        });
    }
}
