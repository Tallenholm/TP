# TP Universal Language — v0 Design Specification

**Status:** Foundational design
**Date:** 2026-08-12
**Project codename:** TP
**Public language name:** Intentionally deferred

## 1. Mission

TP is a general-purpose programming language and toolchain designed for software built jointly by humans and AI agents.

The project is justified only if it can combine capabilities that today are fragmented across multiple ecosystems:

- Python/TypeScript-class approachability and iteration speed.
- Rust-class memory safety and compile-time guarantees.
- Predictable native performance without requiring garbage collection for ordinary code.
- Safe structured concurrency.
- First-class compilation to native targets and WebAssembly, with credible paths to web and mobile application targets.
- Practical interoperability with C, JavaScript/TypeScript, Python, and WebAssembly ecosystems.
- Explicit effects/capabilities so code states what resources it may access.
- Compiler and tooling interfaces designed for both humans and machine agents.

TP must make real software easier to build, safer to operate, easier to reason about, and easier to maintain. Novel syntax by itself is not a reason for TP to exist.

## 2. Non-Negotiable Language Laws

1. **Safe TP code must not invoke undefined behavior.** Unsafe operations, when unavoidable for FFI or low-level systems work, must be explicitly isolated and auditable.
2. **Simple programs must stay simple.** Common application code must not require advanced ownership, lifetime, metaprogramming, or build-system knowledge.
3. **Performance must be explainable.** Allocation, copying, asynchronous work, and expensive implicit behavior must be visible or mechanically inspectable.
4. **Concurrency must be safe by construction.** Data races are rejected or structurally prevented in safe code.
5. **Effects and permissions must be explicit.** Filesystem, network, process, environment, device, and other privileged operations must be represented in the program model rather than hidden ambient authority.
6. **Compiler diagnostics are a public interface.** Errors must be structured, stable enough for tooling, machine-readable, and understandable by humans.
7. **Interop is part of the core product.** TP must cooperate with existing ecosystems rather than requiring developers to abandon them.
8. **One language should span multiple execution environments.** Target differences may require explicit adapters, but should not require rewriting core business logic in another language.
9. **Tooling is part of the language.** Compiler, package manager, formatter, test runner, documentation generator, language server, and build system are designed together.
10. **AI is a first-class programmer.** The language, compiler, formatter, project format, and diagnostics must make code generation, refactoring, verification, and repair reliable for software agents.

## 3. What TP Is Not

TP v0 is not trying to be:

- a replacement for every domain-specific language;
- a new operating system;
- a visual/no-code environment;
- a natural-language-only programming system;
- a syntax experiment with no production toolchain;
- a compatibility clone of Python, TypeScript, Rust, Go, C++, or Java;
- an excuse to invent bespoke replacements for mature ecosystems before interop exists.

## 4. Target Users

Primary users:

- application developers who want high-level ergonomics without surrendering static guarantees;
- systems and performance-sensitive developers who want safer defaults and less ceremony;
- teams using AI coding agents heavily;
- library and tool authors who need cross-platform deployment;
- developers building native services, CLIs, desktop utilities, web/WASM modules, and eventually mobile applications.

## 5. Language Philosophy

### 5.1 Syntax

Syntax should be compact, regular, and easy to parse both visually and mechanically.

Principles:

- braces for explicit block boundaries;
- semicolons optional where grammar is unambiguous;
- immutable bindings by default;
- local type inference by default;
- explicit public API types strongly encouraged and eventually enforceable by policy;
- no significant whitespace;
- no implicit truthiness across unrelated types;
- no implicit numeric narrowing;
- minimal hidden coercion;
- pattern matching built in;
- expressions and statements clearly distinguishable by grammar.

Illustrative syntax:

```tp
fn greet(name: String) -> String {
    "Hello, {name}"
}

fn main() {
    let message = greet("world")
    print(message)
}
```

### 5.2 Mutability

Bindings are immutable by default:

```tp
let count = 1
var total = 0
```

Mutation must be visible at the binding and API level.

### 5.3 Nullability

There is no implicit null for ordinary values. Optionality is represented explicitly:

```tp
let user: User?
```

Operations on optional values must use explicit propagation, matching, or unwrapping semantics.

## 6. Type System

TP uses a static type system with strong inference.

Initial type-system goals:

- algebraic data types;
- structs/records;
- enums/sum types;
- generics;
- traits/protocols/interfaces based on explicit contracts;
- tuples;
- function types;
- explicit option/result types;
- compile-time exhaustiveness checking;
- nominal public types with structural conveniences where they do not weaken API clarity;
- no implicit `any` escape hatch in safe TP.

Example:

```tp
enum Result<T, E> {
    Ok(T),
    Err(E),
}

fn parse_port(text: String) -> Result<u16, ParseError> {
    ...
}
```

## 7. Error Model

Recoverable failures are values, not hidden control flow.

- `Result<T, E>` is the default recoverable error mechanism.
- `?`-style propagation is permitted where the enclosing return type supports it.
- panics/traps are reserved for broken invariants or explicitly unrecoverable states.
- exceptions are not the default application error model.
- compiler diagnostics must distinguish programmer errors, effect/capability errors, type errors, borrow/ownership errors, and target/backend errors.

## 8. Memory Model

### 8.1 Goal

Provide deterministic, high-performance memory management with safe defaults, without forcing ordinary application developers to reason manually about every lifetime.

### 8.2 v0 Direction

TP will use an ownership-oriented model with compiler inference and escape analysis, but its surface model should be less ceremony-heavy than Rust.

Conceptual categories:

- owned values;
- borrowed read-only references;
- explicitly mutable borrows;
- shared reference-counted ownership where requested;
- region/arena allocation where requested;
- unsafe/raw memory only inside explicit `unsafe` boundaries.

The compiler should infer ordinary local lifetimes. Explicit lifetime syntax is not part of the v0 user-facing language unless implementation experience proves it necessary.

### 8.3 Garbage Collection

A mandatory tracing garbage collector is not part of the core v0 runtime model. Managed allocation modes may exist later as explicit libraries/runtime profiles for domains where they are advantageous.

## 9. Effects and Capabilities

Effects are part of function contracts.

Illustrative syntax:

```tp
effect net

effect fs.read

fn load_config(path: Path) -> Result<Config, Error> uses fs.read {
    ...
}

async fn fetch_user(id: UserId) -> Result<User, Error> uses net {
    ...
}
```

Goals:

- make privileged behavior discoverable from signatures;
- allow applications to restrict capabilities at module/package/runtime boundaries;
- let compilers, editors, security tooling, and AI agents reason about side effects;
- enable least-authority deployment profiles;
- avoid viral annotation noise by supporting principled effect inference for private/local code while keeping public boundaries explicit.

Initial capability families:

- `fs.read`
- `fs.write`
- `net`
- `process`
- `env`
- `clock`
- `random`
- `device`
- `unsafe`

The exact hierarchy is subject to implementation validation, but ambient unrestricted I/O is not the design target.

## 10. Concurrency and Async

TP uses structured concurrency.

Goals:

- spawned work has an explicit lifetime/owner;
- cancellation is built into task structure;
- safe code cannot create data races;
- asynchronous code should not infect unrelated synchronous APIs unnecessarily;
- channels, tasks, actors, and shared-state synchronization may coexist through standard-library abstractions.

Illustrative syntax:

```tp
async fn dashboard() -> Dashboard uses net {
    let (user, mail, alerts) = concurrent {
        fetch_user()
        fetch_mail()
        fetch_alerts()
    }

    Dashboard { user, mail, alerts }
}
```

## 11. Modules and Packages

Project metadata should be deterministic and machine-readable.

Tentative project file:

```text
TP.toml
```

Package principles:

- reproducible lockfile;
- content-integrity hashes;
- explicit dependency sources;
- declared required capabilities;
- target-specific dependencies only where necessary;
- package scripts cannot silently acquire arbitrary machine access;
- deterministic builds are a first-class objective.

## 12. Interoperability

Interop is staged by priority.

### Tier 1 — C ABI

Required first. This unlocks operating-system APIs and a huge native library surface.

### Tier 2 — WebAssembly

TP must both produce and consume WASM modules with predictable ABI conventions.

### Tier 3 — JavaScript/TypeScript

Web-target and Node-compatible bindings should permit importing common packages through generated or declared interfaces.

### Tier 4 — Python

Python interoperability should support extension-module generation and calling Python from explicitly managed boundaries. It must not force the entire TP runtime to inherit Python's performance or threading model.

Interop is allowed to require adapter declarations where doing so preserves safety and clarity.

## 13. Compilation Targets

### v0 required

- x86_64 Linux
- x86_64 Windows
- arm64 Linux
- arm64 macOS
- WebAssembly/WASI

### Later target families

- browser WebAssembly + JavaScript glue
- Android
- iOS
- additional embedded targets
- serverless/runtime-specific packaging

A target is not considered supported merely because code generation succeeds. Support requires build, runtime, debugging, packaging, and test stories appropriate to that target.

## 14. Compiler Architecture

The reference compiler should be split into small, independently testable stages:

1. source manager;
2. lexer;
3. parser;
4. concrete/abstract syntax representation;
5. name resolution;
6. type checking/inference;
7. effect checking;
8. ownership/borrow analysis;
9. typed intermediate representation;
10. optimization passes;
11. backend/code generation;
12. linker/packager integration;
13. structured diagnostics.

The front end must not depend directly on a particular machine-code backend.

LLVM is the initial recommended native backend because it lowers the cost of reaching several production targets. WASM code generation may initially go through LLVM or a dedicated backend if testing shows a meaningful benefit.

The compiler implementation language for the bootstrap compiler should be Rust unless a prototype demonstrates a materially better choice. Rust is selected for memory safety, systems tooling maturity, parser/compiler ecosystem support, and the ability to bootstrap TP without first solving TP itself.

## 15. Intermediate Representation

TP should use at least two IR levels:

- **HIR (High-Level IR):** resolved names, desugared syntax, typed constructs, effect information.
- **MIR (Mid-Level IR):** explicit control flow, ownership moves/borrows, drops, async lowering, and optimization-friendly operations.

IR formats should be inspectable via compiler flags and have stable-enough structured forms for testing and AI tooling.

## 16. Toolchain

The first-party command is tentatively `tp` during development.

Expected interface:

```text
tp new
tp init
tp run
tp build
tp test
tp check
tp fmt
tp lint
tp doc
tp add
tp remove
tp update
tp tree
tp explain
tp doctor
```

The toolchain should avoid a collection of unrelated executables where one coherent command can provide the workflow.

## 17. Diagnostics and AI Protocol

Compiler diagnostics are emitted in two coordinated forms:

1. high-quality human-readable terminal/editor output;
2. a versioned structured diagnostic schema.

A diagnostic should be able to include:

- stable diagnostic code;
- severity;
- primary source span;
- related spans;
- explanation;
- machine-applicable fix when safe;
- effect/type/ownership trace where relevant;
- documentation reference;
- compiler version/schema version.

Example conceptual JSON:

```json
{
  "schema": 1,
  "code": "TP-E0214",
  "severity": "error",
  "message": "network capability is not available in this function",
  "span": { "file": "src/main.tp", "start": 142, "end": 160 },
  "required_effect": "net",
  "fixes": [
    { "kind": "signature", "replacement": "uses net" }
  ]
}
```

AI-facing design principles:

- deterministic formatter output;
- parser recovery suitable for partially edited files;
- compiler queries that can inspect symbols/types without full builds;
- structured edits rather than fragile text-only fixes where possible;
- stable symbol identity within a compilation unit;
- source maps from lowered IR back to user code;
- no hidden build state required to understand a project.

## 18. Security Model

Security must be built into the language/toolchain rather than delegated entirely to application discipline.

Required principles:

- memory-safe default language subset;
- explicit unsafe boundaries;
- capability-aware package metadata;
- locked dependencies and integrity verification;
- no arbitrary package install scripts by default;
- compiler/tooling support for dependency provenance;
- reproducible-build goals;
- optional deny-by-default runtime capability manifests for deployed applications.

## 19. Standard Library Strategy

The standard library should be intentionally smaller than batteries-included ecosystems that become difficult to evolve.

Core areas:

- primitives and collections;
- strings/text/Unicode;
- option/result;
- filesystem/path;
- network primitives;
- time;
- concurrency/async;
- serialization interfaces;
- process/environment;
- testing;
- FFI foundations.

Higher-level protocols and frameworks belong in versioned first-party packages when independent evolution is valuable.

## 20. Testing Strategy

The language implementation requires multiple test layers:

- lexer/parser golden tests;
- type-system compile-pass/compile-fail suites;
- effect-system tests;
- ownership/memory-safety tests;
- diagnostic snapshot tests;
- IR lowering tests;
- backend code-generation tests;
- executable integration tests;
- cross-target conformance tests;
- fuzzing for parser/typechecker/compiler boundaries;
- differential/property testing where semantics permit;
- performance regression benchmarks.

A compiler feature is not complete until both success and failure behavior are tested.

## 21. Bootstrap Strategy

### Stage 0

Reference compiler written in Rust.

### Stage 1

TP becomes capable of compiling a deliberately restricted subset of itself.

### Stage 2

A TP-written compiler frontend or selected components are built and checked against the Rust implementation.

### Stage 3

Self-hosting becomes a release goal only after semantic parity, reproducibility, and bootstrap trust are understood.

Self-hosting is not an early vanity milestone.

## 22. v0 Milestones

### M0 — Constitution

- language laws;
- syntax principles;
- ownership direction;
- type/effect/error models;
- compiler architecture;
- interop priorities;
- toolchain philosophy.

### M1 — Executable Core

The compiler can parse, type-check, lower, and execute/build programs containing:

- functions;
- primitive values;
- immutable/mutable locals;
- arithmetic and comparisons;
- conditionals;
- loops;
- structs;
- enums;
- basic pattern matching;
- strings;
- results/options;
- basic modules;
- useful diagnostics.

### M2 — Safety Core

- ownership/borrowing;
- deterministic destruction;
- safe references;
- unsafe boundary model;
- effect checking;
- core concurrency semantics.

### M3 — Ecosystem Core

- package manager;
- lockfile;
- formatter;
- language server;
- test runner;
- C FFI;
- WASM target;
- documentation generation.

### M4 — Real Application Trial

Build at least three nontrivial applications from different categories using TP, document every point of friction, and change the language where evidence warrants it.

Suggested validation applications:

1. CLI/data-processing tool;
2. network service with persistence;
3. WASM/browser-facing application or library.

## 23. Success Criteria

TP v0 is successful enough to continue only if evidence demonstrates all of the following:

- newcomers can understand ordinary TP code without learning the compiler internals;
- safe code prevents representative memory-safety and data-race bugs;
- typical application code requires materially less ownership ceremony than equivalent Rust while retaining strong guarantees;
- native performance is within a credible systems-language range for representative workloads;
- diagnostics are actionable for both humans and automated agents;
- at least C and WASM interop are practical rather than theoretical;
- builds are reproducible enough to debug and automate reliably;
- the language can support real applications without immediately escaping into foreign code for basic functionality.

If TP cannot meet these criteria, the design should be revised rather than protecting earlier decisions for prestige.

## 24. Intellectual Property and Repository Policy

TP is proprietary. It is not open-source software.

The repository is governed by the root `LICENSE`, which reserves rights to Timothy Holm and grants no open-source usage rights. Public GitHub visibility, if retained, must not be interpreted as permission to use, modify, redistribute, commercialize, or build derivative products beyond the limited rights that may arise from GitHub's own Terms of Service.

The project should avoid accepting outside contributions unless contributor ownership and licensing terms are intentionally established first.

## 25. Decisions Intentionally Deferred

The following are not fixed in v0 because implementation evidence should decide them:

- final public language name;
- final file extension if `.tp` conflicts with branding or ecosystem constraints;
- exact borrow-checking algorithm;
- whether the native backend remains LLVM permanently;
- exact generics implementation strategy (monomorphization, dictionary passing, or hybrid);
- exact ABI stabilization policy;
- actor framework semantics;
- tracing-GC optional runtime profile;
- macro/metaprogramming system;
- package registry governance and hosting;
- mobile UI framework strategy.

These are deferred decisions, not missing requirements.

## 26. First Implementation Boundary

The first implementation plan must target only **M1 — Executable Core** plus the compiler infrastructure necessary to support it.

M2–M4 must not be pulled into the first coding phase except for interfaces that M1 must deliberately leave room for. This prevents the language from becoming an untestable collection of ambitions before its syntax, parser, type checker, diagnostics, IR, and executable pipeline are proven.
