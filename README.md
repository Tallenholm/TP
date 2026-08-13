# TP

TP is an experimental general-purpose programming language and toolchain designed for software built jointly by humans and AI agents.

> **Proprietary software.** Copyright (c) 2026 Timothy Holm. All Rights Reserved. TP is **not** open source. The root [`LICENSE`](LICENSE) grants no general right to copy, modify, redistribute, deploy, commercialize, or reuse this project.

## M1 status

The current **M1 Executable Core** implements a complete interpreter-backed path from TP source code to execution:

```text
source
  -> UTF-8 lexer
  -> parser / AST
  -> name resolution + static type checking
  -> typed HIR
  -> explicit-control-flow MIR
  -> MIR interpreter
```

M1 is deliberately interpreter-backed. Native/LLVM code generation is not implemented yet; stabilizing language semantics comes first.

## Build the bootstrap toolchain

TP's bootstrap compiler is written in Rust.

```bash
cargo build --workspace
```

Run the compiler CLI through Cargo:

```bash
cargo run -p tp-cli -- check examples/hello.tp
cargo run -p tp-cli -- run examples/hello.tp
```

The second command prints:

```text
Hello from TP
```

Another executable example is available at `examples/fibonacci.tp`.

## CLI

### Check a program

```bash
cargo run -p tp-cli -- check path/to/main.tp
```

A valid program exits with status `0`. Compile errors exit with status `1`.

### Run a program

```bash
cargo run -p tp-cli -- run path/to/main.tp
```

Program output is written to stdout. Compiler and runtime diagnostics are written to stderr.

### Machine-readable diagnostics

```bash
cargo run -p tp-cli -- check --diagnostic-format json path/to/main.tp
```

M1 emits one JSON object per diagnostic with a versioned schema, stable diagnostic code, severity, and message.

## Language examples

### Functions, inference, and immutable-by-default bindings

```tp
fn add(a: i64, b: i64) -> i64 {
    a + b
}

fn main() {
    let answer = add(20, 22);
    print(answer);
}
```

Use `var` when mutation is required:

```tp
fn main() -> i64 {
    var x = 0;
    while x < 42 {
        x = x + 1;
    }
    x
}
```

### Structs

```tp
struct User {
    id: i64,
    name: String
}

fn main() -> String {
    let user = User { id: 1, name: "TP" };
    user.name
}
```

### Generic enums and exhaustive match

```tp
enum Option<T> {
    Some(T),
    None
}

fn unwrap_or_zero(value: Option<i64>) -> i64 {
    match value {
        Some(v) => v,
        None => 0
    }
}
```

`Result<T, E>` can be declared with the same general enum machinery; `Option` and `Result` are not compiler-only runtime magic.

### Multi-file programs

Sibling modules use `import`:

```tp
// util.tp
fn twice(x: i64) -> i64 {
    x * 2
}
```

```tp
// main.tp
import util

fn main() -> i64 {
    util.twice(21)
}
```

M1 resolves imports relative to the importing file and rejects import cycles.

## M1 language surface

Implemented and tested:

- functions, parameters, calls, recursion, and return types;
- `let` immutable bindings and `var` mutable bindings;
- local type inference and explicit annotations;
- `Unit`, `Bool`, `i64`, `f64`, and `String`;
- arithmetic, comparison, equality, and boolean operators;
- `if` / `else` expressions;
- `while` loops;
- structs, construction, and field access;
- generic enums and variant constructors;
- `match` expressions with enum exhaustiveness checking;
- wildcard, binding, literal, and variant patterns;
- generic `Option<T>` / `Result<T, E>`-style definitions;
- sibling-file imports and import aliases;
- UTF-8 source handling and Unicode identifiers;
- line comments and escaped string literals;
- stable compiler diagnostic families;
- typed HIR and control-flow MIR;
- interpreter execution with checked integer arithmetic;
- `print(value)` as the first builtin;
- `tp check` and `tp run`;
- human-readable and JSON diagnostic output.

## Diagnostic codes

Current diagnostic families include:

- `TP-E0001` — lexical/token error
- `TP-E0100` — parse error
- `TP-E0200` — name/module resolution error
- `TP-E0300` — type/semantic error
- `TP-E0500` — runtime trap

The diagnostic model is intentionally structured so editors and AI agents can consume the same compiler facts shown to humans.

## Not implemented yet

These are intentionally outside M1 rather than silently implied to work:

- ownership and borrow checking;
- deterministic destruction rules for heap-backed user values;
- effects and capability checking;
- async / structured concurrency;
- package registry, package manager, or TP lockfile;
- C ABI interoperability;
- WebAssembly interoperability/code generation;
- JavaScript/TypeScript or Python interoperability;
- LLVM/native machine-code output;
- browser, Android, or iOS packaging;
- language server;
- formatter and linter commands;
- generated API documentation;
- macros/metaprogramming;
- self-hosting compiler.

Those belong to later milestones after the M1 semantics are proven.

## Design documents

The foundational language design and executable-core implementation plan live under:

```text
docs/superpowers/specs/2026-08-12-universal-language-design.md
docs/superpowers/plans/2026-08-12-m1-executable-core.md
```

## Development verification

The final M1 gate is intended to pass all of:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Compiler features are expected to have both successful-program tests and failure/recovery tests.
