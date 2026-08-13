# TP Master Gated Development Plan

**Owner:** Timothy Holm  
**Status:** Controlling project roadmap  
**Policy:** No later gate may begin until all dependency gates are PASSED.  
**Companion files:** `AGENTS.md`, `docs/gates/STATUS.md`, `docs/gates/GATE_REPORT_TEMPLATE.md`, `CONTRIBUTING.md`

---

## 1. Purpose

TP is an ambitious programming-language/toolchain project. The project must optimize for correctness, safety, semantic coherence, reproducibility, and evidence rather than milestone velocity.

This roadmap exists to prevent a common failure mode in compiler/language projects: building the next impressive subsystem on top of an earlier layer that was only superficially complete.

A green build is evidence. A passing test suite is evidence. Neither alone is authorization to advance.

---

## 2. Gate State Machine

Every major gate has exactly one state:

- `LOCKED` — work in this gate is prohibited.
- `ACTIVE` — implementation/research permitted.
- `VERIFYING` — feature work frozen while exit criteria are checked.
- `PASSED` — all exit criteria satisfied and evidence recorded.
- `FAILED` — one or more blocking criteria failed; gate remains open for repair.

Allowed transitions:

```text
LOCKED -> ACTIVE -> VERIFYING -> PASSED
                       |           |
                       v           v
                     FAILED <------+
                       |
                       +-> ACTIVE
```

A previously PASSED gate may be reopened if later evidence invalidates an assumption or reveals a correctness, safety, semantic, security, or architecture defect.

---

## 3. Permanent Micro-Gate for Every Implementation Task

Every coding task must follow this sequence:

1. Define requirement and exact acceptance criteria.
2. Write the failing test first.
3. Run it and verify RED for the intended reason.
4. Implement the smallest correct change.
5. Run the targeted test and verify GREEN.
6. Add boundary, negative, and regression tests.
7. Run the affected subsystem test suite.
8. Run the full project test suite.
9. Run formatter checks.
10. Run lints with warnings denied.
11. Update docs/specs when behavior changes.
12. Inspect the diff for accidental scope and architecture damage.
13. Perform adversarial review.
14. Resolve all Critical and Major findings.
15. Run fresh CI.
16. Record evidence in the active gate report.
17. Commit only after the above is satisfied.

If the failing test does not fail for the expected reason, implementation must stop until the test correctly proves the missing behavior.

---

## 4. Permanent Major-Gate Exit Requirements

Every gate inherits these requirements in addition to its gate-specific requirements:

- every written requirement is implemented;
- every requirement is covered by tests;
- required failure and negative tests exist;
- regressions demonstrate RED -> GREEN;
- full test suite passes;
- formatter passes;
- linter passes with warnings denied;
- required fuzz/property/conformance testing passes;
- documentation and examples match actual behavior;
- zero unresolved Critical or Major review findings;
- architecture review has no known blocking issue;
- fresh CI independently reproduces verification;
- a gate report records exact commands, commits, CI runs, findings, and limitations;
- project owner explicitly approves opening the next major gate.

Tests, specifications, or issue severity must never be weakened solely to make a gate pass.

---

# Gate 0 — Governance, Repository Control, and IP

## Goal

Establish the rules and repository structure that every later person/agent must inherit.

## Required work

- Root `AGENTS.md` with mandatory instructions.
- This master plan.
- Current gate status document.
- Gate-report template.
- Human contribution policy.
- PR template with gate declaration and evidence checklist.
- Proprietary license review.
- Dependency-license inventory process.
- Repository visibility decision (private recommended while proprietary development is sensitive).
- Branch policy: `main` is the last gated/stable state.
- Draft PR required for active milestone work.
- No generated build artifacts.
- CI is read-only verification, not an uncontrolled source-mutating bot.
- Architecture decision record (ADR) convention established.

## Exit criteria

All governance documents exist, agree with each other, and the repository owner has resolved the public/private visibility decision.

---

# Gate 1 — M1 Adversarial Stabilization

## Goal

Prove the current executable-core implementation is correct enough to serve as the semantic baseline. Do not begin M2 work here.

## Required audit areas

- source/span model;
- lexer;
- parser;
- AST;
- scopes/symbols;
- type checker;
- generic inference;
- structs/enums/patterns;
- HIR;
- MIR;
- module loader;
- interpreter;
- diagnostics;
- CLI;
- tests;
- CI;
- docs/examples.

## Known issues already requiring regression tests

### Return-flow soundness

The checker must reason about control-flow paths, not merely whether any explicit return exists somewhere in a function.

Required examples include:

```tp
fn broken(x: Bool) -> i64 {
    if x { return 1; }
}
```

This must fail.

```tp
fn valid(x: Bool) -> i64 {
    if x { return 1; } else { return 2; }
}
```

This must pass.

Cover nested branches, match, loops, diverging paths, tail expressions, and mixed explicit/tail returns.

### Boolean short-circuiting

`&&` and `||` must be control-flow operations.

```tp
false && dangerous()
true || dangerous()
```

The right-hand side must not execute in these cases.

### Type-name validation

Undeclared type names must fail. Also test generic arity, duplicated generic parameters, missing/excess arguments, recursive type policy, and shadowing policy.

### Declaration collision validation

Test duplicate struct fields, duplicate variants, type namespace collisions, function collisions, alias/import collisions, and local shadowing rules.

### Literal correctness

Define/test integer range, negative integer parsing semantics, float overflow/NaN/Infinity policy, Unicode identifiers, newline-in-string policy, and escapes.

### Executable entry-point contract

Define/check `main`, including zero-parameter requirement and accepted return types.

### Diagnostics architecture

Human diagnostics require source path, line, column, highlighted primary span, secondary labels, help, and stable code.

Machine diagnostics require a versioned schema including file and source ranges, labels, help/fixes when present.

A `SourceManager` or equivalent must retain multi-file source identity through checking/rendering.

### Single semantic source of truth

Eliminate duplicated type/semantic reconstruction between validation and HIR lowering. Checked semantics must flow into typed HIR instead of being independently re-derived.

### Module hardening

Test duplicate imports, repeated modules, diamond graphs, aliases, duplicate aliases, nested imports, imported types/constructors, canonicalization/symlinks, deterministic ordering, and cross-platform path behavior.

### Reproducible verification

Use locked dependency resolution and explicit supported toolchain policy. Expand platform CI as soon as practical.

## Exit criteria

- complete issue ledger;
- zero unresolved Critical/Major issues;
- every bug has regression test and RED -> GREEN evidence;
- complete suite/format/lint/CI pass;
- M1 gate report complete;
- PR remains draft until gate passes.

---

# Gate 2 — M1 Semantic Freeze

## Goal

Decide whether the repaired implementation expresses the semantics TP actually wants.

## Freeze/document

- lexical grammar;
- precedence and associativity;
- literals/numerics;
- overflow/traps;
- floating-point semantics;
- equality;
- strings/Unicode;
- scopes/shadowing;
- mutability;
- calls/recursion;
- return/control-flow semantics;
- `if`/`while`;
- structs;
- enums;
- generics;
- patterns/exhaustiveness;
- modules/imports;
- executable entry point;
- runtime trap model.

## Conformance corpus

Create implementation-independent tests under `tests/conformance/` organized by language feature. Every case records source input plus expected compile result, diagnostics, runtime value/output where applicable.

## Exit criteria

100% conformance pass, no unresolved M1 semantic ambiguity, docs/examples match behavior, final M1 review complete. Only then may M1 merge to `main`.

---

# Gate 3 — Compiler Architecture Foundation

## Goal

Establish architecture capable of supporting ownership, effects, native backends, and tooling without semantic duplication.

## Target pipeline

```text
SourceManager
 -> Lexer
 -> Parser
 -> AST
 -> Name Resolution
 -> Semantic/Type Analysis
 -> Typed HIR
 -> Ownership/Effect Analysis
 -> MIR
 -> Interpreter / Backend
```

## Required work

- explicit invariants between stages;
- centralized semantic context;
- stable symbol identities;
- module graph;
- source management;
- compiler query interfaces;
- typed structured diagnostics;
- HIR verifier;
- MIR verifier;
- malformed internal-IR tests.

## Exit criteria

One authoritative semantic model and verifier-protected IR boundaries; malformed internal IR is rejected rather than causing uncontrolled compiler panics.

---

# Gate 4 — Ownership and Memory Model

## Goal

Achieve Rust-class safety with less user-facing ceremony.

## Design before code

Produce a dedicated ownership/memory specification covering owned values, moves/copies, immutable/mutable borrows, aliasing, lifetime inference, drop order, parameter/return ownership, nested values, collections, reference-counted ownership, arenas/regions, raw pointers, and future closure/self-reference implications.

Prototype competing approaches before freezing one.

## Required invalid-program corpus

- use after move;
- use after free;
- double free;
- dangling reference;
- mutable aliasing;
- mutation while borrowed;
- escaped local reference;
- returning references to dead storage.

## Exit criteria

Representative invalid-memory behavior is unrepresentable or compile-time rejected in safe TP.

---

# Gate 5 — Unsafe Boundary Model

## Goal

Define exactly what safe TP guarantees and what explicit `unsafe` may do.

## Requirements

Unsafe operations must be lexically visible, have documented safety contracts, remain auditable, and not silently contaminate safe code.

## Exit criteria

The project can state and support with tests: **safe TP cannot trigger undefined behavior through the core language model.**

---

# Gate 6 — Effects and Capabilities

## Goal

Make privileged effects part of the program model.

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

Define inference, public-boundary declaration, transitivity, polymorphism, package declarations, restriction, and runtime enforcement.

## Exit criteria

Code/package authority is mechanically enforced; undeclared filesystem/network authority cannot be silently acquired.

---

# Gate 7 — Structured Concurrency and Async

## Goal

Provide async/concurrency whose lifetimes and safety are structurally controlled.

## Design areas

Task ownership, cancellation, joining, child lifetime, channels, shared state, synchronization, sendability, async I/O, scheduler/runtime boundaries, and optional actor abstractions.

## Exit criteria

Safe TP cannot produce a data race in the supported concurrency model; stress/cancellation/ownership/effect suites pass together.

---

# Gate 8 — Package and Build System

## Goal

Create deterministic project/dependency management.

Target files:

```text
TP.toml
TP.lock
```

Implement package identity, versions, dependency graph, integrity hashes, capability declarations, target dependencies, reproducible lockfile, provenance, build graph, caching, and offline mode. Arbitrary install scripts are denied by default.

## Exit criteria

A clean environment plus lockfile resolves identical build inputs/dependency graph.

---

# Gate 9 — C ABI Interoperability

## Goal

Unlock the native ecosystem through controlled FFI.

Define primitive/struct ABI mapping, pointers, strings, ownership transfer, callbacks, errors, unsafe requirements, and header import/generation.

## Exit criteria

Representative C libraries are usable and unsafe behavior remains confined to explicit FFI boundaries.

---

# Gate 10 — WebAssembly

## Goal

Support both TP -> WASM and WASM -> TP, beginning with WASI and then browser-hosted WASM.

Test numeric calls, strings, memory, imports/exports, errors, and deterministic output.

## Exit criteria

A nontrivial TP library executes with conformance parity under at least two independent WASM runtimes.

---

# Gate 11 — Native Backend

## Goal

Compile MIR to native executables through a backend-neutral lowering and LLVM initially.

Initial target families:

- x86_64 Linux;
- x86_64 Windows;
- arm64 Linux;
- arm64 macOS.

The interpreter remains a semantic oracle. Differential tests require identical observable behavior between interpreter and native backend for supported programs.

## Exit criteria

Conformance parity across interpreter/native backends and required target families available to CI.

---

# Gate 12 — JavaScript / TypeScript Interop

## Goal

Support controlled Node/browser interoperability, declarations, Promise mapping, conversions, errors, and package adapters.

## Exit criteria

Representative Node/browser applications consume TP without hidden safety/semantic escapes.

---

# Gate 13 — Python Interop

## Goal

Support Python extension generation plus Python -> TP and TP -> Python calls.

Define conversions, error mapping, ownership, GIL/threading behavior, and performance boundaries.

## Exit criteria

Representative Python/NumPy integration works without forcing ordinary TP code to inherit Python runtime semantics.

---

# Gate 14 — First-Party Developer Tooling

## Goal

Make the language usable without direct compiler-internal manipulation.

Target commands:

```text
tp fmt
tp lint
tp test
tp doc
tp add
tp remove
tp tree
tp explain
tp doctor
```

Develop LSP completion, definition, rename, references, diagnostics, semantic highlighting, and signature help.

AI tooling requires stable structured diagnostics, semantic queries, symbol identity, machine-applicable fixes, inspectable AST/HIR, and deterministic formatting.

## Exit criteria

A normal TP project can be built/tested/debugged/documented through first-party tooling.

---

# Gate 15 — Security and Supply-Chain Hardening

## Goal

Threat-model and attack-test the compiler/toolchain/ecosystem.

Cover malicious source, dependencies, parser/compiler DoS, crashes, path traversal, symlink escapes, package substitution, integrity, capability escalation, FFI, malformed WASM, and generated unsafe behavior.

Fuzz at minimum lexer, parser, type/semantic analysis, module loader, HIR verifier, MIR verifier, and interpreter.

## Exit criteria

Zero unresolved Critical/High security findings and defined fuzz budgets complete without reproducible untriaged crashes.

---

# Gate 16 — Performance and Predictability

## Goal

Prove performance claims with measured evidence.

Track compile time, incremental builds, binary size, startup, allocations, peak memory, throughput, latency, async overhead, FFI overhead, and WASM performance.

Use relevant Rust/Go/C++/Java/Python/TypeScript baselines where comparisons are meaningful.

## Exit criteria

Internal baselines and regression thresholds exist; no performance claim is made without supporting measurements.

---

# Gate 17 — Real-Application Validation

## Goal

Prove TP works outside compiler tests.

Required applications:

1. CLI/data-processing application.
2. Network service with persistence.
3. WASM/browser-facing application/library.

Preferred additional trials: native desktop utility and later mobile proof of concept.

Maintain a friction ledger containing problem, frequency, severity, workaround, proposed language fix, and decision.

## Exit criteria

Real applications do not require routine escape to another language for ordinary functionality. If they do, reopen the responsible earlier gate.

---

# Gate 18 — Self-Hosting Preparation

## Goal

Port compiler components only after semantics/toolchain are mature enough to compare independent implementations.

Suggested progression: lexer -> parser -> diagnostics -> semantic structures -> analyzer -> HIR -> MIR -> backend orchestration.

## Exit criteria

Rust bootstrap and TP implementations agree on the conformance corpus.

---

# Gate 19 — Full Self-Hosting

## Goal

Build the TP compiler with TP while retaining a reproducible bootstrap chain.

Verify compiler A builds B, B builds C, and B/C behavior/artifacts are compared. Address bootstrap/trusting-trust concerns explicitly.

## Exit criteria

Self-hosted compiler passes the same conformance suite as the Rust bootstrap compiler; the bootstrap compiler remains archived and reproducibly buildable.

---

# Gate 20 — v0.1 Release Candidate

## Goal

Freeze features and prove release readiness.

Required documentation includes language reference, grammar, compiler reference, installation, tutorials, standard library, security model, FFI, packages, AI/tooling protocol, version/migration policy, supported platforms, and known limitations.

Verification includes clean builds, dependency resolution, cross-platform CI, fuzzing, safety validation, security review, performance suite, real-app regressions, and bootstrap verification.

## Exit criteria

Only after Gate 20 passes may a build be called **TP v0.1**.

---

## 5. Cross-Cutting Language Laws

Every gate must preserve these laws from the language constitution:

1. Safe TP code must not invoke undefined behavior.
2. Simple programs must remain simple.
3. Performance must be explainable and predictable.
4. Concurrency must be safe by construction.
5. Effects and permissions must be explicit.
6. Diagnostics are stable interfaces for humans and machines.
7. Interop is a first-class product requirement.
8. One language should span multiple execution environments.
9. Tooling is part of the language, not an afterthought.
10. AI is a first-class programmer in the ecosystem.

Each gate report must identify which laws are affected and show why the gate does not violate them.

---

## 6. Gate Report Requirement

Every gate produces `docs/gates/GATE-XX-<name>.md` based on `docs/gates/GATE_REPORT_TEMPLATE.md` and records:

- specification revision;
- implementation PR/head commit;
- requirement-by-requirement PASS/FAIL state;
- exact verification commands/results;
- CI run identifiers;
- conformance/fuzz/property results;
- performance evidence where applicable;
- Critical/Major/Minor findings;
- known limitations;
- documentation audit;
- final PASS/FAIL decision;
- owner approval for opening the next gate.

No chat statement, PR comment, or informal conclusion replaces the gate report.

---

## 7. Backtracking Policy

Going backward is allowed and required when evidence demands it.

Examples:

- Gate 17 exposes unusable ownership ergonomics -> reopen Gate 4.
- Gate 11 exposes semantic ambiguity -> reopen Gate 2.
- Gate 15 fuzzing exposes parser architecture problems -> reopen Gate 1/3 as appropriate.

A reopened gate automatically locks dependent later gates until the regression is resolved and evidence refreshed.

---

## 8. Current Direction

The current project priority is **Gate 0 + Gate 1**. Gate 2 and everything after it remains LOCKED until the status document and gate reports prove otherwise.
