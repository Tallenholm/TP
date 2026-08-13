# TP Master Gated Development Plan

**Owner:** Timothy Holm  
**Status:** Controlling project roadmap  
**Policy:** No later gate may begin until all dependency gates are PASSED.  
**Companion files:** `AGENTS.md`, `docs/gates/STATUS.md`, `docs/gates/GATE_REPORT_TEMPLATE.md`, `CONTRIBUTING.md`

## Core rule

TP optimizes for correctness, safety, semantic coherence, reproducibility, and evidence rather than milestone velocity. A green build or passing CI is evidence, but neither alone authorizes advancement.

Every gate is `LOCKED`, `ACTIVE`, `VERIFYING`, `PASSED`, or `FAILED`. A later gate cannot become ACTIVE until every dependency gate is PASSED. A previously PASSED gate may be reopened when later evidence invalidates it.

## Permanent micro-gate for every coding task

1. Define requirement and acceptance criteria.
2. Write failing test first.
3. Confirm RED for intended reason.
4. Implement smallest correct change.
5. Confirm targeted GREEN.
6. Add boundary/negative/regression tests.
7. Run subsystem tests.
8. Run full suite.
9. Run formatter checks.
10. Run lints with warnings denied.
11. Update docs/specs if behavior changed.
12. Review diff for scope/architecture/security/correctness.
13. Perform adversarial review.
14. Resolve all Critical/Major findings.
15. Obtain fresh CI evidence.
16. Record evidence in gate report.
17. Only then call the task complete.

## Permanent major-gate exit requirements

Every gate requires: all written requirements implemented and tested; negative/failure coverage; RED -> GREEN regression evidence; full suite/format/lint success; required fuzz/property/conformance evidence; docs matching behavior; zero unresolved Critical/Major findings; architecture review with no blocker; fresh CI; completed gate report; and explicit Timothy Holm approval to open the next major gate.

Do not weaken tests/specifications or downgrade issues merely to make a gate pass.

---

# Gate 0 — Governance, Repository Control, and IP

Install mandatory agent/human policy, this roadmap, authoritative gate status, gate-report template, PR checklist, contribution policy, proprietary-license/dependency-license process, branch policy, generated-file policy, CI policy, ADR convention, and resolve repository public/private visibility.

**Exit:** governance docs agree and visibility/IP decisions are explicitly resolved.

# Gate 1 — M1 Adversarial Stabilization

Audit source/spans, lexer, parser, AST, scopes/symbols, type checker, generic inference, structs/enums/patterns, HIR, MIR, modules, interpreter, diagnostics, CLI, tests, CI, and documentation.

Known blockers include:

- replace unsound "any explicit return exists" logic with path-correct return/control-flow analysis;
- implement real short-circuit `&&`/`||` semantics via control flow;
- validate type names and generic arity/parameters;
- harden declaration/import/alias collision rules;
- define integer/float/string/Unicode literal behavior and diagnostics;
- statically validate executable `main` contract;
- implement source-aware human and versioned structured diagnostics through multi-file compilation;
- remove duplicated semantic truth between checking and HIR lowering;
- adversarially test module graphs, aliases, duplicate imports, diamonds, canonicalization/symlinks, deterministic ordering, and cross-platform paths;
- use reproducible locked verification.

**Exit:** complete issue ledger; zero unresolved Critical/Major findings; every bug has RED -> GREEN regression evidence; all required suites/format/lint/CI pass; Gate 1 report complete; M1 PR remains draft until this passes.

# Gate 2 — M1 Semantic Freeze

Freeze/document lexical grammar, precedence, literals/numerics, overflow/traps, floating point, equality, strings/Unicode, scopes/shadowing, mutability, calls/recursion, return/control flow, if/while, structs, enums, generics, patterns/exhaustiveness, modules/imports, main, and runtime traps.

Create implementation-independent conformance corpus.

**Exit:** 100% conformance pass, no unresolved M1 semantic ambiguity, docs/examples match behavior, final M1 review complete. Only then may M1 merge to main.

# Gate 3 — Compiler Architecture Foundation

Target pipeline: SourceManager -> Lexer -> Parser -> AST -> Name Resolution -> Semantic/Type Analysis -> Typed HIR -> Ownership/Effect Analysis -> MIR -> Interpreter/Backend.

Add stage invariants, centralized semantic context, stable symbol identity, module graph, compiler queries, structured diagnostics, HIR/MIR verifiers, malformed-IR tests.

**Exit:** one authoritative semantic model; invalid internal IR is rejected rather than causing uncontrolled compiler panics.

# Gate 4 — Ownership and Memory Model

Design before code: owned values, moves/copies, immutable/mutable borrows, aliasing, lifetime inference, deterministic destruction/drop order, parameter/return ownership, nested values, collections, RC ownership, regions/arenas, raw pointers, future closure/self-reference implications. Prototype alternatives.

Invalid-program corpus includes use-after-move/free, double free, dangling references, mutable aliasing, mutation while borrowed, escaped local references, and returns of dead storage.

**Exit:** representative invalid-memory behavior is unrepresentable or rejected at compile time in safe TP.

# Gate 5 — Unsafe Boundary Model

Define exactly what safe TP guarantees and what explicit unsafe may do; unsafe behavior must be visible, auditable, and contract-driven.

**Exit:** evidence supports the rule that safe TP cannot trigger undefined behavior through the core language model.

# Gate 6 — Effects and Capabilities

Implement and specify `fs.read`, `fs.write`, `net`, `process`, `env`, `clock`, `random`, `device`, and `unsafe`, including inference, transitivity, public declarations, polymorphism, package declarations, restriction, and runtime enforcement.

**Exit:** undeclared filesystem/network/privileged authority cannot be silently acquired.

# Gate 7 — Structured Concurrency and Async

Design task ownership, cancellation, joining, child lifetime, channels, shared state, synchronization, sendability, async I/O, scheduler/runtime, and actor options.

**Exit:** safe TP cannot produce a data race; stress/cancellation/ownership/effect suites pass together.

# Gate 8 — Package and Build System

Create deterministic `TP.toml` / `TP.lock`, package identity, versions, dependency graph, integrity hashes, capabilities, target dependencies, provenance, build graph, caching, offline mode; arbitrary install scripts denied by default.

**Exit:** clean environment + lockfile yields identical build inputs/dependency graph.

# Gate 9 — C ABI Interoperability

Define primitives/structs ABI, pointers, strings, ownership transfer, callbacks, errors, unsafe requirements, header import/generation.

**Exit:** representative C libraries work while unsafety remains confined to explicit FFI boundaries.

# Gate 10 — WebAssembly

Support TP -> WASM and WASM -> TP, starting with WASI then browser WASM. Test numbers, strings, memory, imports/exports, errors, deterministic output.

**Exit:** nontrivial TP library runs with conformance parity under at least two independent WASM runtimes.

# Gate 11 — Native Backend

Compile MIR through backend-neutral lowering to LLVM initially for x86_64 Linux/Windows and arm64 Linux/macOS. Interpreter remains semantic oracle; differential tests compare observable behavior.

**Exit:** conformance parity between interpreter/native across required target families available to CI.

# Gate 12 — JavaScript / TypeScript Interop

Controlled Node/browser interoperability, declarations, Promise mapping, conversions, errors, package adapters.

**Exit:** representative Node/browser apps consume TP without hidden safety/semantic escapes.

# Gate 13 — Python Interop

Python extensions plus Python -> TP and TP -> Python; conversions, error mapping, ownership, GIL/threading rules, performance boundaries.

**Exit:** representative Python/NumPy integration works without imposing Python runtime semantics on ordinary TP code.

# Gate 14 — First-Party Developer Tooling

Implement `tp fmt`, `lint`, `test`, `doc`, `add`, `remove`, `tree`, `explain`, `doctor`; LSP completion/definition/rename/references/diagnostics/highlighting/signature help; AI structured diagnostics, semantic queries, stable symbols, machine-applicable fixes, inspectable AST/HIR, deterministic formatting.

**Exit:** normal TP projects can be developed through first-party tooling.

# Gate 15 — Security and Supply-Chain Hardening

Threat model malicious source/dependencies, parser/compiler DoS, crashes, path traversal, symlink escapes, package substitution, integrity, capability escalation, FFI, malformed WASM, generated unsafe behavior. Fuzz lexer, parser, semantic analysis, modules, HIR/MIR verifiers, interpreter.

**Exit:** zero unresolved Critical/High security findings and defined fuzz budgets complete without untriaged reproducible crashes.

# Gate 16 — Performance and Predictability

Measure compile/incremental time, binary size, startup, allocations, peak memory, throughput, latency, async/FFI/WASM overhead; compare meaningfully against relevant languages.

**Exit:** internal baselines and regression thresholds exist; no performance claim lacks evidence.

# Gate 17 — Real-Application Validation

Build at least a CLI/data tool, network service with persistence, and WASM/browser application/library; preferably desktop and later mobile trials. Maintain a friction ledger.

**Exit:** ordinary functionality does not routinely require escaping TP; otherwise reopen the responsible earlier gate.

# Gate 18 — Self-Hosting Preparation

Port components progressively (lexer, parser, diagnostics, semantic structures/analyzer, HIR, MIR, backend orchestration) and compare against Rust bootstrap.

**Exit:** independent Rust/TP implementations agree on conformance corpus.

# Gate 19 — Full Self-Hosting

TP builds TP. Verify compiler A -> B -> C and compare B/C; preserve reproducible bootstrap and address trusting-trust concerns.

**Exit:** self-hosted compiler passes the same conformance suite as Rust bootstrap; bootstrap remains reproducibly buildable.

# Gate 20 — v0.1 Release Candidate

Freeze features; complete language/grammar/compiler/install/tutorial/stdlib/security/FFI/package/AI-tooling/version/platform/limitations docs; run clean builds, cross-platform CI, fuzzing, safety/security/performance/real-app/bootstrap verification.

**Exit:** only after Gate 20 passes may a build be called TP v0.1.

---

## Cross-cutting language laws

Every gate preserves:

1. Safe TP code must not invoke undefined behavior.
2. Simple programs remain simple.
3. Performance is explainable/predictable.
4. Concurrency is safe by construction.
5. Effects/permissions are explicit.
6. Diagnostics are stable human/machine interfaces.
7. Interop is first-class.
8. One language spans multiple execution environments.
9. Tooling is part of the language.
10. AI is a first-class programmer.

## Gate report and backtracking

Every gate produces `docs/gates/GATE-XX-<name>.md` from `GATE_REPORT_TEMPLATE.md`, recording exact requirements, RED/GREEN evidence, commands/results, CI, conformance/fuzz/performance evidence, review findings, architecture/security/docs audit, limitations, final decision, and owner authorization.

No chat statement or informal PR conclusion replaces the gate report.

Going backward is allowed and required. Reopening an earlier gate automatically locks dependent later gates until evidence is refreshed.

## Current direction

Only Gate 0 and Gate 1 work is authorized. Gate 2 and everything later remain LOCKED until `docs/gates/STATUS.md` and completed gate reports prove otherwise.
