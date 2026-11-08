# TP

> [!CAUTION]
> **Gate 2 — M1 Semantic Freeze — is VERIFYING. Gate 3+ work is LOCKED.** Before changing code or design, read [`AGENTS.md`](AGENTS.md), [`docs/MASTER_GATED_DEVELOPMENT_PLAN.md`](docs/MASTER_GATED_DEVELOPMENT_PLAN.md), [`docs/gates/STATUS.md`](docs/gates/STATUS.md), and the active [Gate 2 report](docs/gates/GATE-02-M1-semantic-freeze.md). Passing CI does **not** authorize merge or advancement.

TP is the codename for a proprietary experimental general-purpose programming language and toolchain designed for software built jointly by humans and AI agents.

The long-term goal is to combine high-level ergonomics with strong static guarantees, predictable systems behavior, broad interoperability, explicit capabilities, and tooling that exposes enough structured information for both humans and software agents to reason about a program reliably.

> **Maturity:** pre-release research and implementation. Gate 1 adversarial stabilization has passed. The M1 semantic surface has been frozen through Gate 2 feature work and a 144-case implementation-independent conformance corpus; formal Gate 2 closure still requires final exact-head verification and owner authorization. TP is not ready for production use.

> **Proprietary software.** Copyright (c) 2026 Timothy Holm. All Rights Reserved. TP is **not open source**. See [`LICENSE`](LICENSE).

## Why TP exists

Most mainstream languages optimize for only part of the software-development problem:

- Python and TypeScript optimize heavily for approachability and iteration speed.
- Rust emphasizes memory safety and systems-level guarantees.
- Go emphasizes operational simplicity.
- C and C++ expose low-level control and enormous native ecosystems.
- JavaScript owns a huge portion of the web platform.

TP is not intended to clone any of them. The project exists to test whether one language/toolchain can eventually combine the strongest useful properties without inheriting all of their tradeoffs.

TP is worth continuing only if evidence shows that it makes real software easier to build, safer to operate, easier to inspect, and easier for people and agents to maintain.

## Core design laws

The foundational design is governed by ten laws:

1. **Safe code must not invoke undefined behavior.**
2. **Simple programs must stay simple.**
3. **Performance must be predictable and explainable.**
4. **Concurrency must be safe by construction.**
5. **Effects and permissions must be explicit.**
6. **Compiler diagnostics are a public interface.**
7. **Interop is part of the core product.**
8. **One language should span multiple execution environments.**
9. **Tooling is part of the language.**
10. **AI is a first-class programmer of TP.**

See the complete [universal-language design specification](docs/superpowers/specs/2026-08-12-universal-language-design.md).

---

## Current M1 architecture

The active **M1 Executable Core** currently follows this interpreter-backed pipeline:

```text
TP source
   ↓
UTF-8 lexer
   ↓
parser / AST
   ↓
semantic + static type analysis
   ↓
authoritative checked semantic model
   ↓
typed HIR
   ↓
explicit-control-flow MIR
   ↓
MIR interpreter
```

M1 is intentionally interpreter-backed. Native/LLVM output, WebAssembly, ownership checking, effects, concurrency, and the package ecosystem belong to later gates and are not authorized yet.

### What the current architecture is trying to prove

M1 is not about raw performance. It is the semantic oracle for later compiler work.

Before TP grows a native backend or memory model, M1 must prove that the language front end can reliably:

- tokenize and recover from invalid source;
- parse useful programs;
- resolve names consistently;
- type-check expressions and declarations;
- preserve checked semantic facts into HIR;
- lower control flow explicitly into MIR;
- execute MIR deterministically enough to act as a future differential oracle;
- emit stable diagnostic families for invalid programs.

---

## Quick start

### Requirements

The bootstrap compiler is written in Rust. **Rust 1.85.0** is the pinned M1 bootstrap toolchain, including its `rustfmt` and `clippy` components.

The repository carries a committed `Cargo.lock`, and M1 verification uses locked dependency resolution. Using the checked-in `rust-toolchain.toml` is the supported development path.

### Build

```bash
cargo build --workspace --locked
```

### Check a TP program

```bash
cargo run --locked -p tp-cli -- check examples/hello.tp
```

A valid program exits with status `0`. Compile errors exit with status `1`.

### Run a TP program

```bash
cargo run --locked -p tp-cli -- run examples/hello.tp
```

Expected output:

```text
Hello from TP
```

A second runnable example lives at [`examples/fibonacci.tp`](examples/fibonacci.tp) and evaluates to `55`.

---

## CLI

The current development command is `tp`, built by the `tp-cli` crate.

### `tp check`

Parse and statically validate a TP source file without executing it:

```bash
cargo run --locked -p tp-cli -- check path/to/main.tp
```

### `tp run`

Compile through MIR and execute the program with the M1 interpreter:

```bash
cargo run --locked -p tp-cli -- run path/to/main.tp
```

Run mode requires an executable `main` function. Under the current M1 contract, `main` takes zero parameters.

Program output goes to stdout. Compiler/runtime diagnostics go to stderr.

### JSON diagnostics

```bash
cargo run --locked -p tp-cli -- check --diagnostic-format json path/to/main.tp
```

The current JSON diagnostic surface is versioned and includes the diagnostic code, severity, message, source-aware primary range, secondary labels, and help text when present. Human diagnostics retain entry/imported source files and render file/line/snippet/caret context.

---

## Five-minute language tour

### Functions and static types

```tp
fn add(a: i64, b: i64) -> i64 {
    a + b
}

fn main() -> i64 {
    add(20, 22)
}
```

Function parameters and declared return types are statically checked. Local bindings may infer their type.

Named functions and builtins are callable symbols, not first-class M1 values: M1 has no function-value type syntax, closures/lambdas, or higher-order function values.

A function's unterminated tail expression must match its declared result type. A function with no explicit return type defaults to `Unit`; use a semicolon when a non-`Unit` expression is intentionally evaluated and discarded.

### Immutable by default

```tp
fn main() -> i64 {
    let answer = 42;
    answer
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

### Expressions and explicit control flow

`if` / `else` are expressions:

```tp
fn abs(x: i64) -> i64 {
    if x < 0 {
        -x
    } else {
        x
    }
}
```

M1 models diverging `return` paths explicitly so a return on one branch cannot incorrectly hide fallthrough on another. A literal `while true { ... }` is also definitely diverging in M1 because the language has no `break`/`continue` construct yet.

### Short-circuit booleans

```tp
fn allowed(ready: Bool, valid: Bool) -> Bool {
    ready && valid
}
```

`&&` and `||` lower to MIR control-flow branches. Their RHS is evaluated only when the operator requires it.

### Structs

```tp
struct User {
    id: i64,
    name: String
}

fn main() -> String {
    let user = User {
        id: 1,
        name: "TP"
    };

    user.name
}
```

### Generic enums

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

The current type checker validates nominal type declarations and exact generic arity. Generic `Option<T>` / `Result<T, E>`-style types are ordinary enum declarations rather than compiler-only runtime magic.

Optional syntax `T?` is accepted only when an arity-1 `Option<T>` nominal type has actually been declared; the compiler does not synthesize `Option`.

### Pattern matching

M1 currently supports wildcard, binding, literal, and enum-variant patterns.

M1 requires compile-time exhaustive matches: both values of `Bool`, full enum-variant coverage, or a catch-all for open domains. Refutable payload patterns do not count as full variant coverage, and duplicate/post-exhaustive arms are rejected. The Gate 2 corpus freezes these rules.

### Multi-file programs

Sibling modules use `import`.

`util.tp`:

```tp
fn twice(x: i64) -> i64 {
    x * 2
}
```

`main.tp`:

```tp
import util

fn main() -> i64 {
    util.twice(21)
}
```

The M1 loader resolves sibling imports relative to the importing file, canonicalizes module identity, deduplicates repeated/diamond imports, supports aliases and qualified imported types/constructors, and rejects import cycles. The exact stabilized M1 policy is documented in [`docs/M1_MODULE_SEMANTICS.md`](docs/M1_MODULE_SEMANTICS.md).

---

## Current M1 language surface

The following surface is frozen for M1 by the Gate 2 semantic documents and conformance corpus. Formal Gate 2 closure is still pending:

| Area | Current M1 support |
|---|---|
| Functions | parameters, direct named calls, recursion, return values; no first-class function values |
| Bindings | immutable `let`, mutable `var`, local inference, annotations |
| Primitive types | `Unit`, `Bool`, `i64`, `f64`, `String` |
| Operators | checked integer arithmetic, numeric comparison, primitive-only equality, boolean operators |
| Control flow | `if` / `else`, `while`, explicit `return`, short-circuit booleans, defined left-to-right value capture |
| Data types | structs, generic enums, variant constructors |
| Patterns | wildcard, binding, literals, enum variants |
| Modules | sibling imports/aliases, canonical identity/deduplication, qualified imported functions/types/constructors/patterns |
| Source | UTF-8, Unicode alphabetic identifiers, line comments, escaped strings |
| Compiler IR | typed HIR, explicit-control-flow MIR |
| Runtime | MIR interpreter, checked integer arithmetic, runtime traps |
| Builtins | `print(value)` for exactly one primitive `Unit`/`Bool`/`i64`/`f64`/`String` value |
| CLI | `check`, `run`, human/JSON diagnostics |

This table summarizes the Gate 2 M1 contract. The [Gate 2 report](docs/gates/GATE-02-M1-semantic-freeze.md) and the semantic documents below are the authoritative freeze evidence.

---

## Frozen M1 semantic documents

Gate 2 split the M1 contract into inspectable documents backed by the independent `tests/conformance/` corpus:

| Document | Frozen surface |
|---|---|
| [`M1_LEXICAL_AND_SYNTAX_SEMANTICS.md`](docs/M1_LEXICAL_AND_SYNTAX_SEMANTICS.md) | UTF-8 lexical rules, identifiers, strings, semicolons, precedence/associativity |
| [`M1_NUMERIC_SEMANTICS.md`](docs/M1_NUMERIC_SEMANTICS.md) | `i64`/ `f64`, checked arithmetic, overflow/zero traps, IEEE runtime policy |
| [`M1_VALUE_SEMANTICS.md`](docs/M1_VALUE_SEMANTICS.md) | primitive values, equality, strings, Unicode comparison, optional-value policy |
| [`M1_CONTROL_FLOW_SEMANTICS.md`](docs/M1_CONTROL_FLOW_SEMANTICS.md) | scopes, bindings, mutability, calls, recursion, returns, divergence, evaluation order |
| [`M1_DATA_AND_PATTERN_SEMANTICS.md`](docs/M1_DATA_AND_PATTERN_SEMANTICS.md) | structs, enums, generics, constructors, patterns, exhaustiveness |
| [`M1_MODULE_SEMANTICS.md`](docs/M1_MODULE_SEMANTICS.md) | imports, aliases, canonical module identity, qualified imported declarations |
| [`M1_RUNTIME_SEMANTICS.md`](docs/M1_RUNTIME_SEMANTICS.md) | executable `main`, runtime traps, primitive `print`, interpreter-visible behavior |
| [`M1_KNOWN_LIMITATIONS.md`](docs/M1_KNOWN_LIMITATIONS.md) | explicitly accepted M1/bootstrap limitations |

The implementation-independent corpus currently contains **144 cases** across lexical, precedence, values/numerics, equality/Unicode, scopes, control flow, data/generics, patterns, modules, and entry/runtime behavior.

---

## Diagnostic families

Current stable families used by M1:

| Code | Category |
|---|---|
| `TP-E0001` | lexical / token error |
| `TP-E0100` | parse error |
| `TP-E0200` | name, declaration, or module resolution error |
| `TP-E0300` | type / semantic mismatch |
| `TP-E0400` | invalid control flow / executable entry-point contract |
| `TP-E0500` | runtime trap |

The diagnostic system is intended to become a shared public interface for terminal users, editors, CI systems, and AI agents.

### Numeric boundary semantics

M1 uses checked `i64` arithmetic and finite `f64` literals. Floating-point arithmetic otherwise follows IEEE-754 binary64 behavior and may produce Infinity/NaN, while division or remainder by ±0.0 traps. See [`docs/M1_NUMERIC_SEMANTICS.md`](docs/M1_NUMERIC_SEMANTICS.md).

---

## AI-native by design

TP does **not** mean natural-language-only programming.

The goal is a conventional language whose compiler exposes enough structured truth that an agent does not have to guess from text alone.

Planned/ongoing foundations include:

- stable diagnostic codes;
- versioned JSON diagnostics;
- precise source spans and related labels;
- authoritative symbol/type resolution;
- typed HIR;
- explicit MIR control flow;
- deterministic formatting;
- inspectable effects/capabilities;
- source mappings from IR back to user code;
- machine-readable project/dependency state.

An AI agent and a human developer should ultimately be able to inspect the same compiler facts.

---

## Memory, effects, concurrency, and performance

These are central goals of TP, but **not M1 features**.

### Memory model — later gate

The planned direction is ownership-oriented memory management with compiler inference, deterministic destruction, read/mutable borrows, explicit shared ownership where requested, arena/region options, and explicit `unsafe` for operations that cannot be proven safe.

A mandatory tracing GC is not the baseline design.

### Effects / capabilities — later gate

Planned effect families include:

```text
fs.read
fs.write
net
process
env
clock
random
device
unsafe
```

The intent is for privileged behavior to be visible in function/package/runtime contracts rather than hidden ambient authority.

### Structured concurrency — later gate

Spawned work should have an owner/lifetime, cancellation should be part of the structure, and safe TP code should not create data races.

### Performance — evidence required

TP does not currently make systems-language performance claims. Performance claims are explicitly locked until the dedicated performance gate establishes benchmarks and comparison methodology.

---

## Roadmap

The full project is governed by [21 gates](docs/MASTER_GATED_DEVELOPMENT_PLAN.md), not by an informal feature wishlist.

| Phase | Gates | Goal |
|---|---:|---|
| Governance | 0 | Repository, IP, ADR, dependency, contribution, and gate policy |
| Executable core | 1–3 | Stabilize/freeze M1 and establish the durable compiler architecture |
| Safety core | 4–7 | Ownership, unsafe model, effects/capabilities, structured concurrency |
| Ecosystem / interop | 8–14 | Packages/build, C, WASM, native, JS/TS, Python, first-party tooling |
| Hardening | 15–17 | Security/supply chain, performance, real-application trials |
| Bootstrap / release | 18–20 | Self-hosting preparation, self-hosting, v0.1 release candidate |

The current authoritative state is [`docs/gates/STATUS.md`](docs/gates/STATUS.md).

---

## What is deliberately not implemented yet

The following are roadmap work, **not current capabilities**:

- ownership / borrow checking;
- deterministic destruction rules for heap-backed user values;
- an unsafe-boundary model;
- effects and capability checking;
- async / structured concurrency;
- TP package manager / registry / lockfile;
- C ABI interoperability;
- WebAssembly output;
- LLVM/native machine-code output;
- JavaScript/TypeScript interoperability;
- Python interoperability;
- browser, Android, or iOS packaging;
- mature formatter/linter/LSP/doc tooling;
- self-hosting compiler.

Locked-gate work must not be pulled forward merely because it is desirable.

---

## Development workflow

Compiler behavior changes follow a test-first gate:

```text
requirement
  → failing regression test
  → confirm RED for the intended reason
  → minimal implementation
  → targeted GREEN
  → negative / boundary tests
  → subsystem tests
  → full workspace tests
  → formatting
  → lint
  → adversarial review
  → gate evidence
```

The current baseline verification commands are:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

M1 reproducibility is pinned to Rust 1.85.0 and locked Cargo dependencies. Gate 1 has passed; Gate 2 is in final verification after semantic freeze, conformance, and whole-branch semantic review.

---

## Repository map

| Path | Purpose |
|---|---|
| [`AGENTS.md`](AGENTS.md) | Mandatory instructions for agents and contributors |
| [`docs/MASTER_GATED_DEVELOPMENT_PLAN.md`](docs/MASTER_GATED_DEVELOPMENT_PLAN.md) | Gate 0–20 roadmap and pass criteria |
| [`docs/gates/STATUS.md`](docs/gates/STATUS.md) | Authoritative current gate states |
| [`docs/gates/GATE-01-M1-adversarial-stabilization.md`](docs/gates/GATE-01-M1-adversarial-stabilization.md) | Passed Gate 1 stabilization evidence ledger |
| [`docs/gates/GATE-02-M1-semantic-freeze.md`](docs/gates/GATE-02-M1-semantic-freeze.md) | Active Gate 2 semantic-freeze report and decisions |
| [`tests/conformance/`](tests/conformance/) | Implementation-independent M1 semantic oracle |
| [`docs/decisions/`](docs/decisions/) | Architecture/governance decision records |
| [`docs/superpowers/specs/2026-08-12-universal-language-design.md`](docs/superpowers/specs/2026-08-12-universal-language-design.md) | Foundational language design |
| [`docs/superpowers/plans/2026-11-07-m1-stabilization.md`](docs/superpowers/plans/2026-11-07-m1-stabilization.md) | Historical Gate 1 execution plan |
| [`docs/superpowers/plans/2026-11-07-m1-semantic-freeze.md`](docs/superpowers/plans/2026-11-07-m1-semantic-freeze.md) | Gate 2 semantic-freeze execution plan |
| [`crates/tp-compiler/`](crates/tp-compiler/) | Lexer, parser, checker, semantic model, HIR, MIR, interpreter |
| [`crates/tp-cli/`](crates/tp-cli/) | Development CLI |
| [`examples/`](examples/) | Runnable TP examples |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Contribution/review policy |
| [`THIRD_PARTY_LICENSES.md`](THIRD_PARTY_LICENSES.md) | Third-party dependency/license ledger |
| [`LICENSE`](LICENSE) | Proprietary license |

---

## Gated development

The gate status model is:

```text
LOCKED → ACTIVE → VERIFYING → PASSED
                     ↘ FAILED
```

A later gate cannot become active until its dependencies pass. A previously passed gate can be reopened if later work exposes a foundational problem.

A green build is necessary, but **CI alone never proves a gate is complete**.

---

## Project name

`TP` is the project codename. The final public language name is intentionally deferred until the language has earned a more permanent identity through implementation evidence.


Known M1 limitations: [`docs/M1_KNOWN_LIMITATIONS.md`](docs/M1_KNOWN_LIMITATIONS.md).
