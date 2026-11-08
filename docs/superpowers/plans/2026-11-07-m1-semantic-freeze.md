# M1 Semantic Freeze Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Freeze the complete M1 language semantics and prove them with an implementation-independent conformance corpus that passes 100%.

**Architecture:** External TP case directories under `tests/conformance/` are the semantic oracle. A dependency-free Rust harness consumes case metadata and only the public compiler API; feature-specific freeze tasks add cases and docs, escalating any mismatch into a Gate 2 semantic decision and TDD repair.

**Tech Stack:** Rust 1.85.0, Cargo `--locked`, TP compiler public API, filesystem-based conformance fixtures.

**Spec:** `docs/superpowers/specs/2026-11-07-m1-semantic-freeze-design.md`

## Global Constraints

- Gate 2 only; Gate 3+ work remains locked.
- Rust bootstrap baseline is exactly 1.85.0.
- Cargo verification uses the committed lockfile with `--locked`.
- Existing behavior is evidence, not automatically frozen semantics.
- Any behavior change requires a failing test observed for the intended reason before production-code modification.
- Conformance tests must not inspect AST, SemanticModel, HIR, or MIR.
- Every ambiguity/decision is recorded in `docs/gates/GATE-02-M1-semantic-freeze.md`.
- M1 remains draft/unmerged until Gate 2 PASSES.

## Review Focus

- Conformance metadata malformed or incomplete must fail the harness clearly rather than silently skipping a case.
- Diagnostic expectations must freeze stable codes, not brittle terminal rendering unless wording is deliberately part of semantics.
- Runtime values must distinguish `Unit`, booleans, integers, floats, and strings without relying on Debug formatting.
- Multi-file module cases must execute from the case directory and must not depend on repository working-directory accidents.
- Corpus discovery/order must be deterministic so later backends can compare the same case sequence.

---

### Task 1: Build the implementation-independent conformance harness

**Files:**
- Create: `crates/tp-compiler/tests/conformance.rs`
- Create: `tests/conformance/README.md`
- Create: `tests/conformance/harness/check-pass/main.tp`
- Create: `tests/conformance/harness/check-pass/expect.txt`
- Create: `tests/conformance/harness/compile-fail/main.tp`
- Create: `tests/conformance/harness/compile-fail/expect.txt`
- Create: `tests/conformance/harness/run-value/main.tp`
- Create: `tests/conformance/harness/run-value/expect.txt`
- Create: `tests/conformance/harness/run-stdout/main.tp`
- Create: `tests/conformance/harness/run-stdout/expect.txt`
- Create: `tests/conformance/harness/run-stdout/stdout.txt`

**Interfaces:**
- Consumes: public `Compiler::check_path`, `Compiler::run_path`, `RunFailure`, `Value`.
- Produces: deterministic recursive case discovery and parser for `expect.txt`; later tasks add only fixture directories.

- [ ] **Step 1: Write harness parser/discovery unit tests inside `conformance.rs`**
  - malformed unknown key is rejected;
  - duplicate key is rejected;
  - invalid outcome is rejected;
  - `compile_fail` without `diagnostics` is rejected;
  - `runtime_fail` without `runtime_code` is rejected;
  - discovered case paths are sorted.

- [ ] **Step 2: Run the conformance test target and verify RED**
  - Run: `cargo test --locked -p tp-compiler --test conformance`
  - Expected: FAIL because the parser/discovery/harness functions do not exist yet.

- [ ] **Step 3: Implement the minimal dependency-free metadata parser, discovery, and case runner**
  - Public compiler API only; no AST/HIR/MIR inspection.
  - Runtime value matching supports `unit`, `bool:`, `i64:`, `f64:`, `string:`.
  - Optional `stdout.txt` is exact.

- [ ] **Step 4: Run the target and verify GREEN**
  - Run: `cargo test --locked -p tp-compiler --test conformance`
  - Expected: all harness tests and four seed cases PASS.

- [ ] **Step 5: Run the full workspace suite**
  - Run: `cargo test --workspace --locked`
  - Expected: PASS.

- [ ] **Step 6: Commit**
  - Commit: `test: add M1 semantic conformance harness`

### Task 2: Freeze lexical grammar and precedence

**Files:**
- Create cases under: `tests/conformance/lexical/`, `tests/conformance/precedence/`
- Create: `docs/M1_LEXICAL_AND_SYNTAX_SEMANTICS.md`
- Modify: `docs/gates/GATE-02-M1-semantic-freeze.md`

**Interfaces:**
- Consumes: Task 1 case format/harness.
- Produces: frozen lexical, token, comment, string, identifier, semicolon, precedence and associativity contract.

- [ ] **Step 1: Add minimal conformance cases** for comments, identifier policy, string escapes/raw newline rejection, semicolon/tail-expression distinction, arithmetic/comparison/equality/boolean precedence, unary precedence, assignment associativity/policy, and grouping.
- [ ] **Step 2: Run conformance target**
  - Existing desired behavior may already pass; any mismatch becomes a recorded semantic ambiguity.
- [ ] **Step 3: Resolve each ambiguity**
  - Record decision first.
  - If code must change, add a focused RED regression, verify RED, implement minimal fix, verify GREEN.
- [ ] **Step 4: Write the lexical/syntax freeze document** matching the conformance cases.
- [ ] **Step 5: Run conformance + full workspace suite**
- [ ] **Step 6: Commit**
  - Commit: `docs: freeze M1 lexical and precedence semantics`

### Task 3: Freeze values, literals, numerics, equality, strings, and Unicode

**Files:**
- Create cases under: `tests/conformance/literals-numerics/`, `tests/conformance/equality-strings-unicode/`
- Modify: `docs/M1_NUMERIC_SEMANTICS.md`
- Create: `docs/M1_VALUE_SEMANTICS.md`
- Modify: Gate 2 report.

**Interfaces:**
- Consumes: conformance harness.
- Produces: frozen primitive/value semantics used by later control-flow/data cases.

- [ ] **Step 1: Add conformance cases** for integer boundaries/overflow traps, float finite literals and runtime Infinity/NaN policy, divide/remainder zero traps, numeric operator typing, equality compatibility, strings/escapes/equality, Unicode identifier policy, optional sugar declaration requirement.
- [ ] **Step 2: Run corpus; ledger ambiguities.**
- [ ] **Step 3: TDD-repair any behavior intentionally changed.**
- [ ] **Step 4: Update numeric/value docs.**
- [ ] **Step 5: Run conformance + full workspace.**
- [ ] **Step 6: Commit**
  - Commit: `docs: freeze M1 value and numeric semantics`

### Task 4: Freeze scopes, mutability, functions, recursion, returns, if, and while

**Files:**
- Create cases under: `tests/conformance/scopes-mutability/`, `tests/conformance/functions-control-flow/`
- Create: `docs/M1_CONTROL_FLOW_SEMANTICS.md`
- Modify: Gate 2 report.

**Interfaces:**
- Consumes: primitive/value semantics from Task 3.
- Produces: frozen binding/call/control-flow contract.

- [ ] **Step 1: Add conformance cases** for same-scope duplicates, nested shadowing, immutable/mutable assignment, annotation/inference, arity/types, recursion, Unit/tail behavior, explicit/tail returns, diverging branches, `if` value typing, `while` condition/body/fallthrough, short-circuit behavior.
- [ ] **Step 2: Run corpus; record ambiguities.**
- [ ] **Step 3: TDD-repair intentional changes.**
- [ ] **Step 4: Write control-flow freeze doc.**
- [ ] **Step 5: Run conformance + full workspace.**
- [ ] **Step 6: Commit**
  - Commit: `docs: freeze M1 binding and control flow semantics`

### Task 5: Freeze structs, enums, generics, optional sugar, patterns, and exhaustiveness

**Files:**
- Create cases under: `tests/conformance/structs-enums-generics/`, `tests/conformance/patterns/`
- Create: `docs/M1_DATA_AND_PATTERN_SEMANTICS.md`
- Modify: Gate 2 report.

**Interfaces:**
- Consumes: type/value/control-flow rules.
- Produces: frozen nominal/generic/pattern contract.

- [ ] **Step 1: Add conformance cases** for struct construction/field access/duplicates, enum constructors, generic arity and inference, recursive nominal declarations, Option sugar, bool/enum/open-domain exhaustiveness, wildcard/binding, refutable payloads, duplicate/unreachable arms.
- [ ] **Step 2: Run corpus; record ambiguities.**
- [ ] **Step 3: TDD-repair intentional changes.**
- [ ] **Step 4: Write data/pattern freeze doc.**
- [ ] **Step 5: Run conformance + full workspace.**
- [ ] **Step 6: Commit**
  - Commit: `docs: freeze M1 data and pattern semantics`

### Task 6: Freeze modules, entry point, runtime traps, and known limitations

**Files:**
- Create cases under: `tests/conformance/modules/`, `tests/conformance/entrypoint-runtime/`
- Modify: `docs/M1_MODULE_SEMANTICS.md`
- Create: `docs/M1_RUNTIME_SEMANTICS.md`
- Modify: `docs/M1_KNOWN_LIMITATIONS.md`
- Modify: Gate 2 report.

**Interfaces:**
- Consumes: all earlier semantic rules and harness multi-file support.
- Produces: final executable M1 observable contract.

- [ ] **Step 1: Add module cases** for sibling import, alias, repeated/diamond canonical identity, qualified imported types/constructors/patterns, cycle failure, duplicate alias.
- [ ] **Step 2: Add entry/runtime cases** for missing/parameterized `main`, accepted return values, checked integer traps, divide/remainder zero, non-exhaustive safety invariant, recursion boundary as documented limitation.
- [ ] **Step 3: Run corpus; record ambiguities and TDD-repair intentional changes.**
- [ ] **Step 4: Update module/runtime/limitations docs.**
- [ ] **Step 5: Run conformance + full workspace.**
- [ ] **Step 6: Commit**
  - Commit: `docs: freeze M1 module and runtime semantics`

### Task 7: Final semantic audit and Gate 2 closure evidence

**Files:**
- Modify: `README.md`
- Modify: `docs/gates/GATE-02-M1-semantic-freeze.md`
- Modify: `docs/gates/STATUS.md`
- Review: all `examples/*.tp`, all M1 semantic docs, complete corpus.

**Interfaces:**
- Consumes: Tasks 1–6.
- Produces: Gate 2 verification package; does not authorize Gate 3 without owner approval.

- [ ] **Step 1: Audit corpus coverage against every G2-R01..R17 requirement and every semantic bullet in the master plan.**
- [ ] **Step 2: Add any missing case before changing docs; run it.**
- [ ] **Step 3: Review README/examples against frozen semantics and correct documentation drift.**
- [ ] **Step 4: Run final verification**
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets --locked -- -D warnings`
  - `cargo test --workspace --locked`
  - Expected: 100% conformance and full suite PASS.
- [ ] **Step 5: Perform whole-branch semantic/adversarial review; resolve all Critical/Major findings with TDD.**
- [ ] **Step 6: Record exact CI run IDs, case totals, exclusions, decisions, known limitations, and review findings in Gate 2 report.**
- [ ] **Step 7: Move Gate 2 to VERIFYING, not PASSED, unless explicit owner authorization for Gate 3 is present.**
- [ ] **Step 8: Commit**
  - Commit: `docs: move Gate 2 to final verification`
