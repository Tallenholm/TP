use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use tp_compiler::Compiler;

#[test]
fn large_generated_program_executes_deterministically() {
    let mut source = String::from("fn main() -> i64 {\n    var x = 0;\n");
    for _ in 0..512 {
        source.push_str("    x = x + 1;\n");
    }
    source.push_str("    x\n}\n");

    let compiler = Compiler::new();
    for _ in 0..3 {
        let report = compiler
            .run_source("generated.tp", &source)
            .expect("large generated M1 program should execute");
        assert_eq!(report.value.as_i64(), Some(512));
    }
}

#[test]
fn malformed_source_recovery_corpus_reports_diagnostics_without_panicking() {
    let corpus = [
        "fn",
        "fn main(",
        "fn main() { let x = ; }",
        "fn main() { if true { 1 } else }",
        "fn main() { match true { true => 1 false => 0 } }",
        "struct User { id i64 }",
        "enum Option<T> { Some(T), None",
        "fn main() { \"unterminated }",
        "fn main() { @@@ }",
        "fn main() -> Missing { 1 }",
    ];

    let compiler = Compiler::new();
    for (index, source) in corpus.iter().enumerate() {
        let report = compiler.check_source(&format!("malformed-{index}.tp"), source);
        assert!(
            !report.diagnostics.is_empty(),
            "malformed corpus entry {index} unexpectedly produced no diagnostics: {source:?}"
        );
    }
}

#[test]
fn generated_module_chain_loads_and_executes_deterministically() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("tp-module-chain-{}-{nonce}", std::process::id()));
    fs::create_dir_all(&dir).expect("create temp module-chain directory");

    const MODULE_COUNT: usize = 16;
    for index in 0..MODULE_COUNT {
        let path = dir.join(format!("m{index}.tp"));
        let source = if index + 1 == MODULE_COUNT {
            "fn value() -> i64 { 1 }\n".to_owned()
        } else {
            format!(
                "import m{next}\nfn value() -> i64 {{ m{next}.value() + 1 }}\n",
                next = index + 1
            )
        };
        fs::write(path, source).expect("write generated module");
    }

    let main = dir.join("main.tp");
    fs::write(&main, "import m0\nfn main() -> i64 { m0.value() }\n")
        .expect("write generated entry module");

    let compiler = Compiler::new();
    let checked = compiler.check_path(&main);
    assert!(checked.diagnostics.is_empty(), "{:#?}", checked.diagnostics);
    assert_eq!(checked.sources.iter().count(), MODULE_COUNT + 1);

    for _ in 0..3 {
        let report = compiler
            .run_path(&main)
            .expect("generated module chain should execute");
        assert_eq!(report.value.as_i64(), Some(MODULE_COUNT as i64));
    }

    fs::remove_dir_all(&dir).expect("remove temp module-chain directory");
}
