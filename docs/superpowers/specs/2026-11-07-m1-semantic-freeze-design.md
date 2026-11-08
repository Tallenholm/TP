# M1 Semantic Freeze Design

**Date:** 2026-11-07  
**Gate:** 2 — M1 Semantic Freeze  
**Status:** approved by Gate 2 activation / owner authorization  
**Binding inputs:** `docs/MASTER_GATED_DEVELOPMENT_PLAN.md`, Gate 1 evidence, current M1 implementation

## Intent

Gate 2 decides whether the repaired M1 implementation expresses the TP semantics we actually want. Existing behavior is evidence, not authority. The output of this gate is a frozen, documented M1 semantic contract plus an implementation-independent conformance corpus that future compiler stages and backends can use as an oracle.

## Goals

- freeze every M1 semantic surface named by the master plan;
- make ambiguities explicit rather than allowing current code to decide silently;
- encode observable behavior in implementation-independent source/expectation cases;
- require 100% conformance before M1 may merge to `main`;
- keep Gate 3+ architecture/features locked.

## Non-goals

- ownership/borrowing;
- unsafe semantics;
- effects/capabilities;
- async/concurrency;
- package system expansion;
- FFI/WASM/native/JS/Python backends;
- performance claims;
- broad fuzz/security campaigns beyond what a semantic ambiguity specifically requires.

## Approaches considered

### 1. Rust-only semantic integration tests

Put all source snippets and assertions directly in Rust test functions.

**Tradeoff:** cheapest to implement, but the semantic corpus becomes coupled to the Rust bootstrap compiler and is harder for future TP/native/WASM backends to consume independently.

### 2. CLI golden-output snapshots

Run `tp check` / `tp run` and freeze complete terminal output.

**Tradeoff:** backend-neutral at the process boundary, but human diagnostic text, paths, line formatting, and presentation details would make semantic cases unnecessarily brittle.

### 3. External source-case corpus + minimal expectation metadata — CHOSEN

Each case lives under:

```text
tests/conformance/<feature>/<case>/
  main.tp
  expect.txt
  [additional .tp modules]
  [stdout.txt]
```

`expect.txt` records only stable observable semantics:

```text
outcome=check_pass
```

or:

```text
outcome=compile_fail
diagnostics=TP-E0300
```

or:

```text
outcome=run
value=i64:42
```

Optional `stdout.txt` stores exact program stdout without inventing string escaping in the metadata format.

**Why chosen:** the corpus is readable without Rust knowledge, source inputs are real TP files, expectations avoid unstable presentation details, module cases are natural directories, and later interpreter/native/WASM runners can consume the same cases.

## Conformance semantics

The harness may use the public Rust compiler API during Gate 2, but it must not inspect AST, SemanticModel, HIR, or MIR. A conformance case judges only observable behavior:

- whether compilation succeeds;
- stable diagnostic code set/order where explicitly frozen;
- whether execution succeeds or traps;
- runtime value;
- stdout;
- module-visible behavior.

A case that exposes an ambiguity becomes a Gate 2 semantic decision. If the desired semantics differ from current behavior, repair follows RED → GREEN and the decision is recorded in the Gate 2 report.

## Corpus organization

Feature groups mirror the Gate 2 freeze surface:

- `lexical/`
- `precedence/`
- `literals-numerics/`
- `equality-strings-unicode/`
- `scopes-mutability/`
- `functions-control-flow/`
- `structs-enums-generics/`
- `patterns/`
- `modules/`
- `entrypoint-runtime/`

Cases should be minimal and normally prove one semantic statement.

## Expectation format

`expect.txt` is deliberately dependency-free and line-oriented.

Required key:

- `outcome=check_pass|compile_fail|run|runtime_fail`

Conditional keys:

- `diagnostics=TP-E0001,TP-E0100` — exact ordered diagnostic codes when diagnostics are part of the frozen contract;
- `value=unit|bool:true|i64:42|f64:1.5|string:hello` — required for successful `run` cases unless the case explicitly freezes only stdout;
- `runtime_code=TP-E0500` — required for `runtime_fail`;
- `runtime_message=<stable substring>` — only when message wording itself is intentionally frozen.

`stdout.txt`, when present, is compared byte-for-byte to interpreter output joined with newline separators exactly as exposed by the public run report.

Unknown keys, duplicate keys, invalid outcomes, and missing required conditional keys are harness errors.

## Freeze documents

Gate 2 will produce or update durable semantic documents for:

- lexical/syntax and precedence;
- values, literals, numerics, equality, strings, Unicode;
- scopes, bindings, mutation, calls, recursion, returns and control flow;
- structs, enums, generics, optional sugar, patterns/exhaustiveness;
- modules/imports;
- executable entry point and runtime traps.

Existing `M1_MODULE_SEMANTICS.md`, `M1_NUMERIC_SEMANTICS.md`, and `M1_KNOWN_LIMITATIONS.md` are inputs and may be revised by Gate 2 decisions.

## Ambiguity policy

Current implementation behavior is never frozen merely because a test already passes. For every surface, Gate 2 asks whether the behavior is desirable and coherent with the ten language laws.

When two reasonable semantics exist:

1. record the ambiguity in the Gate 2 report;
2. choose one explicitly;
3. add conformance cases;
4. if implementation changes, prove RED → GREEN;
5. update the semantic docs.

## Exit condition

Gate 2 may pass only when:

- every G2 requirement is PASS;
- the complete corpus passes 100%;
- no semantic ambiguity remains unresolved;
- docs/examples match the frozen contract;
- no Critical/Major finding remains open;
- final pinned/locked CI is green;
- owner authorizes Gate 3.
