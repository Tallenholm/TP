use std::path::Path;

#[test]
fn expectation_rejects_unknown_key() {
    let error = parse_expectation_str("outcome=check_pass\nmystery=yes\n")
        .expect_err("unknown metadata key must fail");
    assert!(error.contains("unknown expectation key `mystery`"), "{error}");
}

#[test]
fn expectation_rejects_duplicate_key() {
    let error = parse_expectation_str("outcome=check_pass\noutcome=run\n")
        .expect_err("duplicate metadata key must fail");
    assert!(error.contains("duplicate expectation key `outcome`"), "{error}");
}

#[test]
fn expectation_rejects_invalid_outcome() {
    let error = parse_expectation_str("outcome=maybe\n")
        .expect_err("invalid outcome must fail");
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
        vec![
            "check-pass",
            "compile-fail",
            "run-stdout",
            "run-value",
        ]
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
