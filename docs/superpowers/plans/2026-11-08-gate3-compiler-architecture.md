# TP Gate 3 — Compiler Architecture Foundation Implementation Plan

**Date:** 2026-11-08  
**State:** ACTIVE (design and staged implementation)  
**Stable baseline:** `main` @ `e4ba609b060dbf824bf4dba3455b47b7cb4b671a` (M1 Gate 2 PASS, 144/144 conformance cases).  
**Governing:** `AGENTS.md`, `docs/MASTER_GATED_DEVELOPMENT_PLAN.md`, `docs/gates/STATUS.md`, `docs/gates/GATE-03-compiler-architecture-foundation.md`.

## Objective

Build an explicit, verifier-protected, implementation-independent compiler architecture that can later support ownership/effects/native backends **without changing the frozen M1 observable semantics**. Do not implement Gate 4+ features.

Target pipeline:
```text
SourceManager -> Lexer -> Parser -> AST -> Name Resolution
 -> Semantic/Type Analysis -> Typed HIR -> [future ownership/effect analysis]
 -> MIR -> Interpreter / [future backend]
```

## Audited starting point

- `pipeline.rs` currently orchestrates check/lower/run and combines lexer, parser, module loader, type checker, HIR, MIR and interpreter.
- `typecheck.rs` produces a `SemanticModel`; `lower_hir.rs` already consumes it instead of re-inferring types.
- `symbol.rs` has a `SymbolId(u32)` and per-`Scopes` counters. Identity consistency beyond one scope collection needs explicit guarantees before a multi-module/public query contract is promised.
- `hir.rs` stores typed expressions and local parameter/binding symbols, but function identity is still represented by names.
- `mir.rs` stores explicit `BlockId`, `LocalId`, entry, statements and terminators. No dedicated structural verifier is present in the public compiler pipeline.
- `source.rs` has `SourceId`, `Span` and `SourceManager`; `diagnostic.rs` has coded, labeled diagnostics and JSON rendering.
- Current design already has a useful semantic single-source-of-truth start. Gate 3 must strengthen it, not replace it with a second inference engine.

## Guardrails

1. All M1 programs and diagnostics must preserve Gate 2 behavior, including evaluation order, traps, function/value distinction and module identity.
2. Never emit MIR or execute an unverified malformed internal IR at newly guarded boundaries.
3. Internal verifier failures must be typed, deterministic compiler errors, not panics or unstable textual guesswork.
4. Do not silently re-typecheck during HIR lowering, MIR lowering or verification. Verifiers check invariants of already-typed IR, not re-infer source semantics.
5. Introduce no Gate 4 ownership, Gate 5 unsafe, Gate 6 effects, async, package runtime, FFI, WASM, LLVM or self-hosting features.
6. For each behavior change: acceptance criteria → failing RED test on CI → minimal implementation → GREEN targeted/full suite, then formatter, Clippy, docs, adversarial review, evidence and fresh CI.

## Deliverables, ordered tasks, and acceptance criteria

### Task G3-T01 — Freeze architecture invariants (documentation-only)

Create `docs/COMPILER_STAGE_CONTRACTS.md` with explicit ownership of data/semantics, stage inputs/outputs, failure modes and verifier contracts. Identify the single authority for symbols, types and module identities. Record unresolved architectural decisions in the Gate 3 report.

**Accept:** every stage in the master roadmap has a written producer/consumer contract; frozen M1 cases are preserved; no executable semantics altered. Documentation-only work does not require artificial RED tests.

### Task G3-T02 — MIR structural verifier, test-first

Introduce a dedicated `MirVerifier` and internal verifier error type; test it against *manually constructed malformed IR* (never depend only on the lowerer emitting invalid IR). At minimum reject: missing entry block, duplicated block IDs, nonexistent terminator targets, duplicated local IDs, nonexistent parameter locals, references to undeclared locals and invalid statement target locals. Ensure valid MIR from current programs passes unchanged.

**Accept:** each malformed category RED before implementation and GREEN afterward; no verifier panic on malformed input; deterministic diagnostics; all M1 conformance cases and smoke examples remain green. Integration must verify MIR before interpreter execution.

### Task G3-T03 — HIR structural verifier, test-first

Add verification for symbol/binding integrity, typed node metadata, valid source/span origin, function/body result invariants and uses of symbols already resolved by the type checker.

**Accept:** adversarial handmade HIR tests reject bad bindings/types without panic; valid HIR passes; no second type inference engine.

### Task G3-T04 — Stable identity and centralized semantic context

Define lifetime/namespace for `SourceId`, `ModuleId` and `SymbolId`, including repeated, diamond and alias imports; decide whether IDs are stable only within a compiler session or across incremental sessions, and make query contracts explicit. Migrate only after decision and RED regression tests.

**Accept:** distinct module-local symbols cannot alias, legitimate repeated imports share canonical module identity, and deterministic diagnostics preserve source location.

### Task G3-T05 — Module graph and compiler query interfaces

Define graph nodes keyed by canonical source/module identity; separate queries for parse, resolve, check, HIR, MIR and diagnostics with stable lifetimes and deterministic invalidation rules. Avoid incremental caching until correctness is proven.

**Accept:** diamond/repeated/cycle/alias cases and source edits have deterministic query results, no re-inference in lowerers, no broken existing module path policy.

### Task G3-T06 — Typed diagnostics and hardening

Expose typed internal verifier failures and map them to stable external diagnostics with source IDs/spans where available. Extend malformed internal IR/graph testing and protect arbitrary-data entry points against panics.

**Accept:** bad IR cannot cause unchecked interpreter execution or crashes; existing machine diagnostics remain compatible; any new codes documented.

### Task G3-T07 — Final architecture audit and exit verification

Audit semantic duplication, IR round-trips, source identities, invalid-program rejection, panics and stage boundaries. Run pinned Rust 1.85.0:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```
Also run all independent 144-case M1 conformance fixtures, Hello/Fibonacci smoke, additional verifier/malformed-IR tests, and required fuzz/property probes where risk warrants.

**Accept:** all Gate 3 requirements PASS, zero unresolved Critical/Major findings, final whole-branch review and exact-head CI green; owner explicitly approves Gate 3 PASS / Gate 4 ACTIVE before any advancement.

## Architectural decisions requiring explicit resolution

- **G3-D01:** Symbol ID scope and stability: module-scoped, program-scoped, session-scoped or persisted identities? Keep source/module identity separate.
- **G3-D02:** Verifier error API: internal result separate from source diagnostics versus directly emitting source diagnostics; prefer explicit structured internal error translated at the pipeline boundary.
- **G3-D03:** SourceManager identity determinism: atomic IDs are convenient within one process, but any public stable query/replay guarantee must define deterministic behavior.
- **G3-D04:** Query invalidation: correctness-first noncached queries initially versus risk of stale cross-file semantic state.

All are OPEN for architectural design; no accidental implementation choice should silently resolve them.

## First concrete coding increment

Start with **G3-T02**, after G3-T01 has established the exact MIR verifier contract. Write a failing `crates/tp-compiler/tests/mir_verifier.rs` test constructing an invalid `MirModule` with an entry block not in `blocks`; run CI RED, implement verifier to return a structured error, and run GREEN. Then expand to all boundary/negative cases, wire the pipeline and finish all micro-gate evidence before claiming completion.
