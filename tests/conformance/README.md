# TP M1 Conformance Corpus

Each conformance case is a directory containing:

- `main.tp` — entry source;
- `expect.txt` — stable observable expectations;
- optional additional sibling `.tp` modules;
- optional `stdout.txt` — exact expected program stdout.

The Gate 2 harness uses only the public TP compiler API. Cases must not depend on AST, HIR, MIR, or private compiler structure.

## `expect.txt`

Supported outcomes:

```text
outcome=check_pass
```

```text
outcome=compile_fail
diagnostics=TP-E0300
```

```text
outcome=run_compile_fail
diagnostics=TP-E0400
```

```text
outcome=run
value=i64:42
```

```text
outcome=runtime_fail
runtime_code=TP-E0500
runtime_message=division by zero
```

`run_compile_fail` runs the executable pipeline and expects compilation/entry-point failure before interpretation. It requires `diagnostics`.

For `run`, `value` supports `unit`, `bool:true|false`, `i64:<n>`, `f64:<n>`, and `string:<text>`. A run case may omit `value` only when `stdout.txt` exists and the case intentionally freezes stdout only.

`diagnostics` is a comma-separated exact ordered list of stable diagnostic codes.

Unknown keys, duplicate keys, invalid outcomes, and missing required conditional keys are harness errors.
