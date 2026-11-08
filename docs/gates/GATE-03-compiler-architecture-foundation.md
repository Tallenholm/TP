# Gate 03 — Compiler Architecture Foundation Report

**Gate:** 03 — Compiler Architecture Foundation  
**State:** ACTIVE  
**Specification revision:** `docs/superpowers/specs/2026-08-12-universal-language-design.md` and `docs/MASTER_GATED_DEVELOPMENT_PLAN.md`  
**Implementation PR:** [#2 — Gate 3 architecture (draft)](https://github.com/Tallenholm/TP/pull/2)  
**Stable baseline:** `e4ba609b060dbf824bf4dba3455b47b7cb4b671a`  
**Report date:** 2026-11-08

> Gate 2 PASSED on 2026-11-08; M1 merged to `main` via PR #1. Gate 3 is owner-authorized ACTIVE; work is scoped to compiler architecture, typed analysis and verifier-protected IR boundaries. Gate 4+ remain LOCKED.

## 1. Requirements

| ID | Requirement | Status | Evidence / required closure |
|---|---|---|---|
| G3-R01 | Explicit invariant contracts between compiler stages | IN PROGRESS | `docs/COMPILER_STAGE_CONTRACTS.md` covers source/loader/typechecker/HIR/MIR/interpreter boundaries, verification, identity and query snapshot rules; final whole-branch contract audit pending. |
| G3-R02 | One centralized authoritative semantic context | IN PROGRESS | Checked binding type flows from `SemanticModel` to HIR/MIR; `HirVerifier::verify_checked` compares expression/block/binding/global/parameter/pattern/local IDs and function returns against original semantics. 23/23 HIR tests passed CI 42506018205. Full semantic provenance audit pending. |
| G3-R03 | Stable symbol identities with scoped lifetime policy | IN PROGRESS | `SymbolId` unique within each checked function; ID is not globally persistent across compiles. Canonical `ModuleId` path is stable within the same filesystem; scope contract recorded in ADR-0003. Future cross-session rehydration/identity validation pending. |
| G3-R04 | Authoritative canonical module graph | IN PROGRESS | `ModuleGraph` exposes canonical nodes/import edges via file-backed snapshot. Diamond/alias identities green on CI 42505002345; existing symlink/cycle tests green. Further adversarial graph and identity tests pending. |
| G3-R05 | Source management and cross-file identity invariants | IN PROGRESS | HIR verifier rejects foreign SourceIds, spans beyond EOF and invalid UTF-8 boundaries; 13/13 HIR tests green CI 42505287570. SourceId remains process-allocated/ephemeral; complete replay policy pending. |
| G3-R06 | Compiler query API contracts | IN PROGRESS | Noncached `CompilationSnapshot` exposes SourceManager, AST, checked semantic model, verified HIR/MIR, optional canonical ModuleGraph; six query tests green CI 42505002345. Incremental caching/invalidation deliberately deferred in ADR-0003. |
| G3-R07 | Typed structured internal and user diagnostics | IN PROGRESS | Typed HIR/MIR error enums and stable `TP-E0601` / `TP-E0600` outward codes. Internal source-span-to-primary-diagnostic mapping and complete machine-structured provenance remain open. |
| G3-R08 | HIR verifier | IN PROGRESS | Scoped symbols, checked expression/block/function/parameter/global/pattern/local metadata and source UTF-8 provenance verified. 23/23 dedicated tests passed pinned Rust 1.85 CI 42506018205 and 42506366279; deeper type/flow completeness pending. |
| G3-R09 | MIR verifier and guarded execution | IN PROGRESS | MIR index/CFG/local/parameter/callable structural checks and primitive branch/return type checks; public `Interpreter::run_main` validates MIR. 20/20 dedicated tests passed CI 42506366279. Full definite-assignment/type proof pending. |
| G3-R10 | Negative/adversarial malformed-IR corpus | IN PROGRESS | 23 HIR + 20 MIR malformed/positive verifier tests and six query tests pass CI 42506366279, alongside frozen 144-case M1 oracle. More graph/property/fuzz/IR-dominance tests required. |
| G3-R11 | Full regression / conformance / CI / final architecture review | IN PROGRESS | Exact-head pinned/locked Rust CI 42506366279 PASS (fmt, Clippy, workspace, 144 M1 conformance, Hello/Fibonacci). Final review, fuzz/property budget and Gate 3 approval still pending. |

## 2. Test-Driven Evidence

**G3-T02 initial RED → GREEN evidence:**

- RED: [CI 42498929751](https://github.com/Tallenholm/TP/actions/runs/42498929751), commit `c5c1d9c90c2e72aa4b51419d6ab0221e6817876d`, formatting PASS, Rust compile/lint failed with E0432 unresolved imports `tp_compiler::MirVerificationKind` and `tp_compiler::MirVerifier` as expected.
- GREEN: [CI 42499191663](https://github.com/Tallenholm/TP/actions/runs/42499191663), commit `5ff892d963a0b602e8f88d86ff3b45e132c577b3`, formatting PASS, Clippy with warnings denied PASS, full locked workspace tests PASS, all seven MIR verifier tests PASS, independent conformance harness and Hello/Fibonacci smoke PASS.
- A prior intermediate CI run 42498832214 failed formatting before it could demonstrate the missing API; this formatting-only failure is **not** treated as TDD RED evidence. A later implementation commit needed only rustfmt adjustment (CI 42499120226), resolved before GREEN.
- Additional malformed MIR boundary tests passed **13/13** on [CI 42499410650](https://github.com/Tallenholm/TP/actions/runs/42499410650), exact head `b71a0c76fa2918912a8cfcd1360a0b34b896422c`, with formatter, Clippy, workspace tests, M1 conformance, Hello/Fibonacci all green. No new compiler semantic changes were required for these boundary tests. Gate 3 overall remains ACTIVE, not PASSED.

**G3-T03 HIR verifier and semantic binding provenance RED → GREEN:**

- Intended RED: [CI 42503096400](https://github.com/Tallenholm/TP/actions/runs/42503096400), commit `5ae1026a553a2bd198b98c6cd7df1553307ee265`: formatter passed; Rust E0432 on missing `HirVerifier`/`HirVerificationKind` as expected.
- First implementation: commit `286d6f42909127c213d21dce0b074b35d891d850`; [CI 42503348927](https://github.com/Tallenholm/TP/actions/runs/42503348927) exposed a **real conformance regression** on `structs-enums-generics/enum-zero-payload-context` because HIR verification incorrectly inferred a binding's type from `None` instead of preserving its explicit `Maybe<i64>` annotation.
- Repair: commit `02d061d61f26ec796e1ad9ed7110bf5a3ca8fa1e` introduces `ty` in `HirStmt::Let`, populated from the authoritative `SemanticModel`, and propagates it into MIR local metadata. This fixes the semantic provenance issue instead of weakening the conformance expectation.
- Test-fixture correction: [CI 42503567969](https://github.com/Tallenholm/TP/actions/runs/42503567969) failed solely because the duplicated-parameter test used an invalid `main(x)` fixture. Updated to a valid `helper(x)` declaration; did not change language semantics.
- GREEN: [CI 42503647514](https://github.com/Tallenholm/TP/actions/runs/42503647514), commit `6e9be965786135dafa9902b45bf684ea6f0df89c`: Rust 1.85.0 formatting/Clippy/full locked workspace, 9/9 HIR adversarial tests, 13/13 MIR verifier tests, the 144-case frozen M1 corpus, and Hello/Fibonacci smoke checks all PASS.

**G3-T02 public Interpreter boundary RED → GREEN:**

- RED: [CI 42503940577](https://github.com/Tallenholm/TP/actions/runs/42503940577) on `fbac4aade080ecfcf98f719edad7b09eceb58fa6`, formatter/Clippy PASS but both hand-built malformed MIR direct `Interpreter::new(&mir).run_main()` tests failed because execution silently accepted wrong block/local metadata.
- GREEN: [CI 42504027480](https://github.com/Tallenholm/TP/actions/runs/42504027480) on `d2f465d51fd9ce0d0fd80b9f8c0265f5a4faa997`, all 15 MIR tests PASS; public interpreter `run_main` now runs `MirVerifier` and returns typed `RuntimeError` code `TP-E0600` for structurally invalid external MIR. Full locked workspace, frozen M1 corpus, Rust fmt, strict Clippy and Hello/Fibonacci PASS.

### Additional Gate 3 RED → GREEN verification (2026-11-08)

| Change | RED evidence | GREEN evidence |
|---|---|---|
| Single-pass checked source/path snapshots and stale-source isolation | [42504389383](https://github.com/Tallenholm/TP/actions/runs/42504389383) — no `analyze_source/path` methods (fmt PASS) | [42504573620](https://github.com/Tallenholm/TP/actions/runs/42504573620) — four query tests, M1/workspace/smokes PASS |
| Canonical module graph with diamond and alias identity | [42504819970](https://github.com/Tallenholm/TP/actions/runs/42504819970) — `CompilationSnapshot::module_graph` absent (fmt PASS) | [42505002345](https://github.com/Tallenholm/TP/actions/runs/42505002345) — six query tests, canonical diamond/aliases PASS |
| Source-aware HIR UTF-8 spans | [42505182879](https://github.com/Tallenholm/TP/actions/runs/42505182879) — no `verify_with_sources`/boundary error variants (fmt PASS) | [42505287570](https://github.com/Tallenholm/TP/actions/runs/42505287570) — 13 HIR tests, M1/workspace/smokes PASS |
| Authoritative semantic HIR metadata: expression/block/binding/global | [42505533281](https://github.com/Tallenholm/TP/actions/runs/42505533281) — no `verify_checked`/semantic variants (fmt PASS) | [42505775012](https://github.com/Tallenholm/TP/actions/runs/42505775012) — 18 HIR tests, M1/workspace/smokes PASS |
| Function/parameter/pattern/local symbol semantic provenance | [42505895761](https://github.com/Tallenholm/TP/actions/runs/42505895761) — no signature/pattern/local variants (fmt PASS) | [42506018205](https://github.com/Tallenholm/TP/actions/runs/42506018205) — 23 HIR tests, frozen M1 all green |
| MIR duplicate callables, unknown functions, primitive branches and returns | [42506212013](https://github.com/Tallenholm/TP/actions/runs/42506212013) — absent MIR error variants (fmt PASS) | [42506366279](https://github.com/Tallenholm/TP/actions/runs/42506366279) — 20 MIR + 23 HIR + six query tests, full frozen M1 suite/format/Clippy/smokes PASS |

The intermediate formatting-only failures do **not** count as successful RED demonstrations. Final whole-branch security/architecture review is not complete.

## 3. Verification Commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

These are required for each completed code increment and at exit. Gate 3 work will also run verifier-negative cases, the full 144-case M1 conformance corpus, and Hello/Fibonacci smoke tests. Initial Gate 3 T02 verification PASS on exact head `5ff892d963a0b602e8f88d86ff3b45e132c577b3` via CI 42499191663, including 7/7 verifier tests and full frozen M1 regression. Current independently verified implementation head `7f5c654ccb50457af26b7d2958facf943f1d5461` passed exact-head CI **42506366279**: 20/20 MIR, 23/23 HIR, six query tests, all frozen 144 M1 conformance cases, workspace, formatter, Clippy and Hello/Fibonacci. This evidence-closure commit must also pass fresh CI before a completion claim. Final Gate 3 exit verification has **not** been claimed. Final full Gate 3 exit verification is not yet claimed.

## 4. CI Evidence

| Run | Commit | Result | Scope |
|---|---|---|---|
| [Gate 2 PR CI 42498290061](https://github.com/Tallenholm/TP/actions/runs/42498290061) | `adb3c692` | PASS | Stable M1 predecessor, not Gate 3 evidence |
| [Gate Policy 42498347731](https://github.com/Tallenholm/TP/actions/runs/42498347731) | `adb3c692` | PASS | Authoritative Gate 2 closure, not Gate 3 evidence |
| [G3-T02 RED 42498929751](https://github.com/Tallenholm/TP/actions/runs/42498929751) | `c5c1d9c` | RED (expected) | Missing verifier API, E0432; formatting passed |
| [G3-T02 GREEN 42499191663](https://github.com/Tallenholm/TP/actions/runs/42499191663) | `5ff892d` | PASS | 7/7 verifier tests, M1 conformance, Rust fmt/Clippy/workspace/smokes |
| [G3-T02 expanded GREEN 42499410650](https://github.com/Tallenholm/TP/actions/runs/42499410650) | `b71a0c7` | PASS | 13/13 MIR verifier tests, M1 conformance, Rust fmt/Clippy/workspace/smokes |
| [G3-T03 intended RED 42503096400](https://github.com/Tallenholm/TP/actions/runs/42503096400) | `5ae1026` | RED (expected) | E0432 missing HIR verifier API; formatting passed |
| [G3-T03 regression 42503348927](https://github.com/Tallenholm/TP/actions/runs/42503348927) | `286d6f4` | FAILED | Frozen enum-generic explicit annotation conformance case exposed HIR type provenance issue |
| [G3-T03 GREEN 42503647514](https://github.com/Tallenholm/TP/actions/runs/42503647514) | `6e9be96` | PASS | 9/9 HIR + 13/13 MIR, M1 corpus, full Rust fmt/Clippy/workspace/smokes |
| [G3 direct-interpreter RED 42503940577](https://github.com/Tallenholm/TP/actions/runs/42503940577) | `fbac4aa` | RED (expected) | Raw Interpreter accepted invalid block/local metadata |
| [G3 direct-interpreter GREEN 42504027480](https://github.com/Tallenholm/TP/actions/runs/42504027480) | `d2f465d` | PASS | 15/15 MIR, 9/9 HIR, frozen corpus, full Rust fmt/Clippy/workspace/smokes |

| [Gate 3 latest feature CI 42506366279](https://github.com/Tallenholm/TP/actions/runs/42506366279) | `7f5c654` | PASS | 20 MIR, 23 HIR, six query, 144 M1 conformance, fmt/lint/workspace/smokes |

## 5. Conformance Evidence

**Gate 3 cases passed:** 20/20 MIR + 23/23 HIR and six query tests on CI 42506366279.  
**Gate 3 cases failed:** none on verified implementation head `7f5c654ccb50457af26b7d2958facf943f1d5461`.  
**Frozen baseline:** M1 Gate 2 144/144 PASS.  
**Exclusions:** none approved.

## 6. Fuzz / Property / Stress Evidence

Pending design and verifier implementation; no Gate 3 fuzz evidence claimed.

## 7. Performance Evidence

Gate 3 has no permission to make performance claims; watch for material regressions while preserving semantics.

## 8. Review Findings

### Critical

No new critical finding identified at activation; review incomplete.

### Major / design decisions

- **G3-D01 RESOLVED for current compiler scope:** canonical path-based `ModuleId`; per-function ephemeral `SymbolId`, never an unqualified persisted ID. ADR-0003 plus HIR/query tests; future persistent identity designs require a new reviewed decision.
- **G3-D02 PARTIALLY RESOLVED:** typed internal errors mapped to stable `TP-E0600`/`TP-E0601`; attaching HIR source spans to public diagnostic primary labels and complete machine error provenance are still OPEN.
- **G3-D03 RESOLVED for current scope:** `SourceId` values are process-allocated and only valid within a snapshot's `SourceManager`. Deterministic replay uses file/path identity, not raw IDs; document limitations under ADR-0003.
- **G3-D04 RESOLVED for current scope:** compiler snapshots are noncached, recreated on each request; incremental cache/invalidation is forbidden pending a separate correctness-first design. Six query tests show no stale source state.

### Minor

None recorded yet; no final review has occurred.

## 9. Architecture Review

Current `SemanticModel` already bridges type checking and HIR. G3-T02 adds a dedicated MIR structural verifier and guards both `Compiler::lower_mir_source` and `Compiler::lower_mir_path` before interpreter execution, without introducing a parallel type system. `BlockId` and `LocalId` must match vector indices, because the current interpreter indexes them directly. G3-T03 adds a scoped HIR verifier, checks literal/type metadata and checked symbol resolution, and preserves authoritative checked Let type through MIR. Further Gate 3 additions check verified HIR metadata against the exact original `SemanticModel` (23 tests total), verify UTF-8 spans against the owned `SourceManager`, expose one-pass typed query snapshots and canonical import graphs, and require direct public interpreter MIR validation. MIR now rejects duplicate function names, unknown callees, non-Bool primitive branch conditions, and invalid primitive return types. These checks are **not** a complete IR type/definite-assignment proof; such work and final adversarial review remain OPEN.

## 10. Security Review

Malformed internal IR, cross-module identity collisions, source path mismatches and unchecked interpreter assumptions are in scope. Initial MIR verification rejects malformed CFG indices/edges and invalid local references at public `Compiler` lowering boundaries with coded `TP-E0600` diagnostics. HIR verification now guards `Compiler::lower_source`/`lower_path` using scoped binding metadata and coded `TP-E0601` diagnostics. These are compiler-internal failures, not new source-language errors. Direct public `Interpreter::new` still accepts borrowed MIR by design, but `run_main` now validates it before execution and rejects malformed structural MIR with `TP-E0600`; this guard passed adversarial RED→GREEN tests. Deeper MIR type/definite-assignment and runtime recursion safety are not yet proven. Dedicated Gate 15 supply-chain hardening remains out of scope.

## 11. Documentation Audit

Gate 3 implementation plan and `docs/COMPILER_STAGE_CONTRACTS.md` were added, recording the interpreter's index-based MIR invariants and the initial verifier contract. HIR and MIR contracts and the typed diagnostic code mapping are now documented. Stage contracts, README active gate notice, verified HIR/MIR, UTF-8 source boundary rules, module/query identity and an identity/cache ADR are reconciled at this milestone. Final complete implementation/doc reconciliation remains pending.

## 12. Cross-Cutting Language Laws

Affected laws: safety/defined behavior (1), semantic simplicity (2), stable diagnostics (6), tooling (9), and AI-readable compiler interfaces (10). Must prove all remain satisfied at exit.

## 13. Known Limitations

Gate 2's frozen `docs/M1_KNOWN_LIMITATIONS.md` remains authoritative. Initial MIR verification covers structural integrity at both compiler and direct interpreter run boundaries, not all type/definite-assignment invariants. Full compiler architecture, stable symbol/module IDs, source/query model, full HIR/MIR type and definite-assignment proofs, and Gate 3 exit reviews remain unfinished; do not claim production quality or Gate 4 safety features.

## 14. Final Decision

**Decision:** FAIL / ACTIVE  
**Reason:** Gate 3 is ACTIVE. Multiple requirements have independently passing test evidence (23 HIR, 20 MIR, six query, 144 M1 cases), but final HIR/MIR type and definite-assignment verification, complete diagnostics, fuzz/property coverage, final architecture/security review and owner authorization for Gate 4 remain outstanding. No merge permitted.

### Owner authorization

**Next gate authorized to open:** NO  
**Owner:** Timothy Holm  
**Approval record:** On 2026-11-08, the owner conditionally authorized Gate 2 PASS and Gate 3 ACTIVE after final checks. This is **not** approval for Gate 3 PASS or Gate 4 ACTIVE.
