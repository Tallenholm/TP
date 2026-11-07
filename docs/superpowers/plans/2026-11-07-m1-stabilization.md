# TP Gate 1 M1 Adversarial Stabilization Plan

> **Execution policy:** Gate 1 only. Gate 2+ remain locked. Every behavior fix follows RED -> GREEN. The Gate 1 report is the durable evidence ledger.

**Goal:** Turn the existing M1 executable core into a trustworthy semantic baseline by completing the adversarial review, removing all Critical/Major findings, hardening reproducibility, and producing fresh verification evidence.

**Authority:** `AGENTS.md`, `docs/MASTER_GATED_DEVELOPMENT_PLAN.md`, `docs/gates/STATUS.md`, `docs/gates/GATE-01-M1-adversarial-stabilization.md`, and `docs/superpowers/specs/2026-08-12-universal-language-design.md`.

**Branch:** `m1-executable-core-clean`

## Global constraints

- No Gate 2+ implementation.
- No production-code fix before a regression test has been observed failing for the intended reason.
- Do not delete findings after repair; mark them RESOLVED with RED/GREEN evidence.
- Do not weaken semantics/tests to make the gate pass.
- Keep PR #1 draft until Gate 1 passes.
- Final Gate 1 verification must use locked dependencies and an explicit toolchain.
- Any newly discovered Critical/Major finding is added to the Gate 1 ledger before repair.

---

## Task 1 — Complete the adversarial review before broad repair

**Purpose:** Finish reviewing the entire M1 surface so the fix plan is based on the complete known defect set, not only the first eight findings.

**Inspect:**
- source/span/diagnostic model;
- lexer/token/literal parsing;
- parser/AST/recovery;
- symbols/scopes/declaration collisions;
- type checker/generics/match/return analysis;
- HIR lowering and semantic duplication;
- MIR lowering/control flow;
- interpreter/runtime traps/evaluation order;
- module loader/path/canonicalization;
- pipeline/source retention;
- CLI human/JSON diagnostics;
- tests/fixtures/examples;
- CI/toolchain/reproducibility.

**Required outputs:**
- add every new Critical/Major/Minor finding to `docs/gates/GATE-01-M1-adversarial-stabilization.md`;
- record semantic-policy decisions that must be fixed before Gate 2;
- identify dependencies between fixes;
- update this plan only if the review changes repair ordering materially.

**Completion:** G1-R01 may become PASS only after every subsystem above has an explicit review result.

---

## Task 2 — Establish a single checked semantic model (M1-005)

**Goal:** Stop HIR lowering from independently rebuilding semantic truth.

**Design target:**
- semantic/type checking produces an authoritative checked module/context;
- function signatures, nominal type definitions, enum variants, generic parameter bindings, symbol identities, and expression types are retained in that checked result;
- HIR lowering consumes that result rather than reconstructing equivalent maps/inference.

**TDD:**
1. Add a regression/invariant test that demonstrates checker/HIR disagreement is possible under the current duplicated path.
2. Observe RED for the intended semantic-drift condition.
3. Introduce the smallest shared semantic representation needed to make the test pass.
4. Migrate HIR lowering to consume it.
5. Add invariants proving every HIR expression/symbol/type comes from checked semantics.
6. Run HIR + typechecker + full workspace tests.

**Completion evidence:** M1-005 RESOLVED with RED/GREEN commits.

---

## Task 3 — Path-sensitive return/control-flow validation (M1-001)

**Required cases:**
- reject `fn broken(x: Bool) -> i64 { if x { return 1; } }`;
- accept if/else where every path returns;
- nested branches;
- match with/without exhaustive returning arms;
- loops that may fall through;
- explicit return mixed with tail expressions;
- Unit-returning functions;
- unreachable statements after unconditional return handled consistently.

**Implementation rule:** use structured control-flow outcome analysis or CFG facts; never restore an “any return exists” heuristic.

**TDD:** one minimal failing regression first, then boundary matrix.

---

## Task 4 — True short-circuit boolean lowering (M1-002)

**Required behavior:**
- `false && rhs()` never evaluates RHS;
- `true || rhs()` never evaluates RHS;
- `true && rhs()` and `false || rhs()` do evaluate RHS;
- RHS side effects and runtime traps prove evaluation/non-evaluation;
- MIR contains branch/join structure, not an eager binary op.

**TDD:** trap/side-effect reproducer must fail before implementation.

---

## Task 5 — Declared type, generic, and declaration validation (M1-003 + G1-R05)

**Required coverage:**
- unknown type names;
- generic too few/many arguments;
- duplicate generic parameters;
- duplicate struct fields;
- duplicate enum variants;
- function/type namespace collisions;
- import/alias collisions;
- local shadowing policy;
- recursive nominal type policy;
- prelude/user declaration conflicts.

**Rule:** centralize resolution in the checked semantic model from Task 2.

---

## Task 6 — Static executable entry-point contract (M1-007)

**Define and enforce for M1:**
- exactly one executable `main` after module loading;
- zero parameters;
- accepted result types are explicitly documented and tested;
- invalid shape is a compile-time diagnostic, not an interpreter surprise.

**TDD:** `fn main(x: i64) {}` must fail in checking before runtime.

---

## Task 7 — Literal/parser boundary semantics (M1-008 + G1-R06)

**Decisions/tests required:**
- i64 literal range;
- unary minus interaction with `i64::MIN`;
- stable oversized-integer diagnostic;
- float overflow/non-finite policy;
- string newline policy;
- escape errors;
- Unicode identifier behavior;
- invalid token recovery around literals.

**Rule:** Gate 1 may define only what is needed to make current M1 behavior correct/stable; Gate 2 performs the semantic freeze.

---

## Task 8 — Source-aware diagnostics end to end (M1-004)

**Human diagnostics require:**
- file path;
- line/column;
- highlighted primary span;
- secondary labels where present;
- help text where present;
- stable diagnostic code.

**JSON diagnostics require a versioned schema containing at minimum:**
- schema version;
- code/severity/message;
- file/source identity;
- byte/range positions;
- labels;
- help/fix information when present.

**Architecture requirement:** a SourceManager/equivalent retains all loaded sources through module checking, HIR/MIR preparation, and CLI rendering.

**TDD:** CLI tests must prove source data that is currently discarded becomes visible.

---

## Task 9 — Module graph/path hardening (M1-006)

**Required matrix:**
- duplicate imports;
- repeated import deduplication;
- diamond dependency;
- aliases and duplicate aliases;
- nested imports;
- imported nominal types and constructors;
- canonicalized path identity;
- cycles;
- same file reached by distinct relative paths;
- symlink behavior where supported;
- deterministic ordering;
- cross-directory same-name modules;
- platform path separator/case assumptions documented.

**Security focus:** no accidental path escape behavior is normalized into a different module identity without explicit policy.

---

## Task 10 — Reproducible verification/CI (G1-R11)

**Required changes:**
- pin supported Rust toolchain instead of floating `stable`;
- use `cargo ... --locked` for build/test paths where Cargo supports it;
- keep `Cargo.lock` authoritative;
- CI installs/uses the same pinned toolchain;
- document supported bootstrap version;
- no generated `target/` artifacts committed.

**Verification:**
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

If `cargo clippy --locked` is unsupported by the selected Cargo invocation shape, record the exact supported equivalent rather than silently omitting lock enforcement.

---

## Task 11 — Documentation and complete Gate 1 evidence

Update:
- README claims;
- diagnostic contract docs;
- M1 semantic notes affected by repairs;
- Gate 1 RED/GREEN table;
- finding statuses;
- exact commands/results/CI run IDs;
- known limitations.

No Gate 2 semantic-freeze work is performed here.

---

## Task 12 — Final whole-branch adversarial review and Gate 1 verification

**Review:**
- inspect the complete PR diff against Gate 1 requirements;
- add any newly discovered Critical/Major finding to the ledger;
- fix Critical/Major findings only via RED -> GREEN;
- Minor findings may be deferred only if documented and genuinely non-blocking.

**Final commands:**
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

**Required final state:**
- G1-R01..R14 all PASS;
- zero OPEN Critical/Major findings;
- fresh CI green on the final head;
- Gate 1 report says `Decision: PASS`;
- owner explicitly authorizes Gate 2 before Gate 2 becomes ACTIVE.

---

## Repair ordering rationale

1. Complete review first so the defect set is not moving underneath us.
2. Fix semantic duplication before semantic correctness work so later repairs have one source of truth.
3. Repair control flow/type/entry/literal semantics.
4. Repair diagnostics and modules after the semantic core is authoritative.
5. Harden reproducibility.
6. Run a fresh whole-branch review and verification.

This ordering intentionally trades some short-term speed for avoiding duplicate fixes and semantic drift.
