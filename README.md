# TP

> [!CAUTION]
> **Gate 1 is ACTIVE and Gate 2+ work is LOCKED.** Before changing code or design, read [`AGENTS.md`](AGENTS.md), [`docs/MASTER_GATED_DEVELOPMENT_PLAN.md`](docs/MASTER_GATED_DEVELOPMENT_PLAN.md), [`docs/gates/STATUS.md`](docs/gates/STATUS.md), and the active [Gate 1 report](docs/gates/GATE-01-M1-adversarial-stabilization.md). Passing CI does **not** authorize merge or advancement.

TP is the codename for a proprietary experimental general-purpose programming language and toolchain designed for software built jointly by humans and AI agents.

The long-term goal is to combine high-level ergonomics with strong static guarantees, predictable systems behavior, broad interoperability, explicit capabilities, and tooling that exposes enough structured information for both humans and software agents to reason about a program reliably.

> **Maturity:** pre-release research and implementation. The current M1 executable core is runnable, but Gate 1 adversarial stabilization is still active. TP is not ready for production use and its semantics are not frozen.

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

The bootstrap compiler is written in Rust. A Rust toolchain with Cargo is required.

The repository currently carries a `Cargo.lock`. Gate 1 still includes a dedicated reproducibility task to pin the supported bootstrap toolchain and make all CI dependency use explicitly locked.

### Build

```bash
cargo build --workspace
```

### Check a TP program

```bash
cargo run -p tp-cli -- check examples/hello.tp
```

A valid program exits with status `0`. Compile errors exit with status `1`.

### Run a TP program

```bash
cargo run -p tp-cli -- run examples/hello.tp
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
cargo run -p tp-cli -- check path/to/main.tp
```

### `tp run`

Compile through MIR and execute the program with the M1 interpreter:

```bash
cargo run -p tp-cli -- run path/to/main.tp
```

Run mode requires an executable `main` function. Under the current M1 contract, `main` takes zero parameters.

Program output goes to stdout. Compiler/runtime diagnostics go to stderr.

### JSON diagnostics

```bash
cargo run -p tp-cli -- check --diagnostic-format json path/to/main.tp
```

The current JSON surface includes a schema version, diagnostic code, severity, and message. Gate 1 is actively extending the end-to-end diagnostic contract with retained source files, spans, labels, help information, and multi-file context.

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

Gate 1 now models diverging `return` paths explicitly so a return on one branch cannot incorrectly hide fallthrough on another.

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

### Pattern matching

M1 currently supports wildcard, binding, literal, and enum-variant patterns.

Enum exhaustiveness checking exists today. Gate 1 still contains an open hardening item for exhaustive finite-domain behavior outside enums, including booleans/literal matches, so broader exhaustiveness must not yet be considered frozen.

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

Implemented today, but still subject to Gate 1 repair and Gate 2 semantic freeze:

| Area | Current M1 support |
|---|---|
| Functions | parameters, calls, recursion, return values |
| Bindings | immutable `let`, mutable `var`, local inference, annotations |
| Primitive types | `Unit`, `Bool`, `i64`, `f64`, `String` |
| Operators | arithmetic, comparison, equality, boolean operators |
| Control flow | `if` / `else`, `while`, explicit `return` |
| Data types | structs, generic enums, variant constructors |
| Patterns | wildcard, binding, literals, enum variants |
| Modules | sibling imports and aliases |
| Source | UTF-8, Unicode alphabetic identifiers, line comments, escaped strings |
| Compiler IR | typed HIR, explicit-control-flow MIR |
| Runtime | MIR interpreter, checked integer arithmetic, runtime traps |
| Builtins | `print(value)` |
| CLI | `check`, `run`, human/JSON diagnostics |

This table is descriptive, not a stability promise. The [Gate 1 report](docs/gates/GATE-01-M1-adversarial-stabilization.md) is the source of truth for unresolved correctness findings.

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
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Gate 1 also requires reproducibility hardening before it may pass, including a pinned bootstrap toolchain and locked dependency verification.

---

## Repository map

| Path | Purpose |
|---|---|
| [`AGENTS.md`](AGENTS.md) | Mandatory instructions for agents and contributors |
| [`docs/MASTER_GATED_DEVELOPMENT_PLAN.md`](docs/MASTER_GATED_DEVELOPMENT_PLAN.md) | Gate 0–20 roadmap and pass criteria |
| [`docs/gates/STATUS.md`](docs/gates/STATUS.md) | Authoritative current gate states |
| [`docs/gates/GATE-01-M1-adversarial-stabilization.md`](docs/gates/GATE-01-M1-adversarial-stabilization.md) | Live M1 stabilization evidence ledger |
| [`docs/decisions/`](docs/decisions/) | Architecture/governance decision records |
| [`docs/superpowers/specs/2026-08-12-universal-language-design.md`](docs/superpowers/specs/2026-08-12-universal-language-design.md) | Foundational language design |
| [`docs/superpowers/plans/2026-11-07-m1-stabilization.md`](docs/superpowers/plans/2026-11-07-m1-stabilization.md) | Current Gate 1 execution plan |
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
