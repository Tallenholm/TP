# Gate 01 — M1 Adversarial Stabilization Report

**Gate:** 01 — M1 Adversarial Stabilization  
**State:** ACTIVE  
**Specification revision:** `docs/superpowers/specs/2026-08-12-universal-language-design.md`  
**Implementation PR:** #1  
**Head commit at report creation:** see PR head  
**Report date:** 2026-11-07

> This gate is NOT PASSED. PR #1 must remain draft/unmerged until this report satisfies every mandatory gate requirement and Timothy Holm approves advancement.

## 1. Requirements

| ID | Requirement | Status | Evidence / blocker |
|---|---|---|---|
| G1-R01 | Complete adversarial review of all M1 subsystems | PASS | Full source/lexer/parser/AST/symbol/type/HIR/MIR/interpreter/module/pipeline/CLI/tests/docs/CI review completed 2026-11-07; findings retained below. |
| G1-R02 | Correct path-sensitive return/control-flow validation | PASS | Old `contains_explicit_return` heuristic removed; internal `Never`/divergence typing now distinguishes total return paths from fallthrough. RED CI 42355699407; GREEN CI 42355992729 with nested/loop/unreachable/runtime boundaries. |
| G1-R03 | Correct short-circuit semantics for `&&` and `||` | FAIL | MIR currently lowers both operands eagerly. |
| G1-R04 | Validate declared types and generic arity/parameters | FAIL | Additional validation required. |
| G1-R05 | Harden declaration/import/alias collision rules | FAIL | Adversarial matrix incomplete. |
| G1-R06 | Define/test literal range, float, string, Unicode behavior | FAIL | Boundary policy/tests incomplete. |
| G1-R07 | Statically validate executable `main` contract | FAIL | Entry-point contract not fully enforced before runtime. |
| G1-R08 | Source-aware human and rich versioned JSON diagnostics | FAIL | CLI currently discards portions of structured source rendering/context. |
| G1-R09 | Establish one authoritative semantic source of truth | PASS | `SemanticModel` now carries checked function signatures, expression/block types, binding symbols/types, local resolutions, and global categories into HIR; `lower_hir.rs` is structural-only and MIR consumes checked global/binding facts. CI 42355570648 green. |
| G1-R10 | Harden module graph/path/canonicalization semantics | FAIL | Broader duplicate/diamond/symlink/platform matrix required. |
| G1-R11 | Reproducible locked verification policy | FAIL | Current CI requires hardening/explicit locked policy. |
| G1-R12 | Zero unresolved Critical/Major findings | FAIL | Review remains open. |
| G1-R13 | Full tests/format/lint/fresh CI after all repairs | FAIL | Prior green CI predates required repairs and does not close this gate. |
| G1-R14 | Documentation accurately reflects stabilized M1 behavior | FAIL | Docs now mark gate status, but semantic docs remain subject to Gate 1/2 work. |

## 2. Adversarial Review Ledger

Every finding must remain here until resolved. Do not delete findings after fixes; mark them RESOLVED and retain evidence.

| ID | Severity | Subsystem | Finding | Reproducer / evidence | Required resolution | Status |
|---|---|---|---|---|---|---|
| M1-001 | Major | Type/control flow | Function validation previously treated the existence of any explicit return as sufficient and represented returning branches as `Unit`, causing both unsound acceptance and false branch-type rejection. | `missing_return_path_is_rejected`; `returning_branch_unifies_with_tail_branch`; nested/while/unreachable/runtime boundaries. | Removed heuristic; internal `Never` models diverging control flow and propagates through expression typing; block fallthrough is path-sensitive. | RESOLVED |
| M1-002 | Major | MIR/runtime semantics | `&&` / `||` are lowered as ordinary eager binary expressions, so RHS side effects/traps occur when they should be skipped. | `false && dangerous()`; `true || dangerous()` | Lower through branches/join blocks and add side-effect/trap regressions. | OPEN |
| M1-003 | Major | Type system | Unknown type names are materialized as named types without proving declarations exist; generic arity/parameter validation requires hardening. | Undeclared type signatures/generic edge cases. | Central type-name resolution and generic-arity diagnostics with tests. | OPEN |
| M1-004 | Major | Diagnostics | CLI output discards source locations/labels/help; multi-file source retention is insufficient. The renderer itself ignores secondary labels and uses byte length for caret width, producing incorrect Unicode/multi-line highlighting. | Compare `render_diagnostic` with CLI emission; inspect secondary-label and Unicode spans. | SourceManager/end-to-end source-aware human + JSON diagnostics, including correct label/span rendering. | OPEN |
| M1-005 | Major | Architecture | Type/semantic knowledge was independently reconstructed in TypeChecker and HIR lowering, and pattern-binding types were lost before MIR. | `match_binding_local_keeps_checked_payload_type`; `hir_preserves_checked_global_resolution_kind`; pre-fix `lower_hir.rs` contained independent function/struct/enum/type/generic inference tables. | `SemanticModel` now carries checker-authoritative types/symbols/resolutions; HIR lowering is structural-only; MIR consumes checked global kinds and binding types. | RESOLVED |
| M1-006 | Major | Module system | Recursive namespace-rewrite loader needs broader duplicate/diamond/alias/canonicalization/symlink/platform coverage. Canonical paths are used only for cycle detection, not global module identity/deduplication, and imported nominal types/struct constructors are not expressible through current qualification rewriting. | Current loader recursively re-loads modules; `util.twice` works, but qualified imported type/constructor paths do not. | Define module identity/qualification semantics and add the full conformance/regression matrix. | OPEN |
| M1-007 | Major | Entry point | `main` shape is not fully validated statically; invalid parameter shape can survive until interpreter execution. | `fn main(x: i64) {}` | Define executable entry-point contract and enforce in semantic analysis. | OPEN |
| M1-008 | Major | Literals/parser | Oversized integer parsing uses `.parse::<i64>().ok()?` after consuming the token, so a range failure can produce no diagnostic and the parser can recover by silently dropping the statement. `i64::MIN` unary-minus handling, float non-finite policy, raw-newline strings, escapes, and Unicode identifier policy also need hardening. | oversized integer statement; `-9223372036854775808`; huge float; raw newline string; Unicode boundary cases. | Define M1 boundary semantics and stable diagnostics; never silently discard an invalid literal. | OPEN |
| M1-009 | Major | Pattern matching | Compile-time exhaustiveness is enforced only for enum matches. Non-exhaustive Bool/literal matches can pass checking and lower to an `otherwise` runtime trap, contradicting the design requirement for compile-time exhaustiveness checking. | `fn main(x: Bool) -> i64 { match x { true => 1 } }` can reach MIR runtime trap for `false`. | Require exhaustive finite-domain coverage where provable and require wildcard/default coverage for open/infinite literal domains; add unreachable/duplicate-arm policy tests. | OPEN |
| M1-010 | Major | Name resolution / semantic consistency | Global value-name precedence was inconsistent across checker, HIR, and MIR. Function/enum-variant collisions and user declarations named `print` could be accepted yet resolve differently across stages. | Regression tests `function_and_enum_variant_value_names_cannot_collide` and `builtin_print_name_is_reserved`. | Reject ambiguous function/variant value names and reserve builtin `print`; the broader semantic-source refactor in M1-005 will carry checked global resolution into HIR/MIR. | RESOLVED |

Severity may only be changed with documented reasoning. More findings will be added as review continues.

## 3. Required RED -> GREEN Evidence

No repair is complete until its test has been observed failing for the intended reason before the fix and passing afterward.

| Finding | Regression test(s) | RED | GREEN | Fix commit |
|---|---|---|---|---|
| M1-001 | `missing_return_path_is_rejected`; `returning_branch_unifies_with_tail_branch`; `all_if_branches_may_return_explicitly`; nested/while/unreachable/runtime boundaries | CI 42355699407: missing-return test was accepted with `diagnostics: []`; mixed return/tail test was rejected as `Unit` vs `i64` | CI 42355902092 green for core regressions; CI 42355992729 green for expanded type/runtime boundary matrix | `d0b45f281d0a1ea228b435c457a4e9066e4d6c2d` + integration fixes through `2dc0a9b9811e5d432e481170cdf1a9ed1fe3bf9c`; boundary tests `45b0ba7166a215659067bba9b708e2ce86c0f530` |
| M1-002 | pending | NOT RECORDED | NOT RECORDED | — |
| M1-003 | pending | NOT RECORDED | NOT RECORDED | — |
| M1-004 | pending | NOT RECORDED | NOT RECORDED | — |
| M1-005 | `match_binding_local_keeps_checked_payload_type`; `hir_preserves_checked_global_resolution_kind` | CI 42354924948: MIR binding was `Unknown` vs expected `I64`; CI 42355127048: compile failed because HIR had no checked global-kind representation | CI 42354993528 green after binding-type preservation; CI 42355329658 green after checked global-kind handoff; CI 42355570648 green after full checker-authoritative SemanticModel refactor | `6832ec15764fbe12f131f42b5f70324b21aefdc4`, `7bee810aed2546087aa7d030855e9bdb181a3945`, `9d1b69fc48af8909acfb20fc8e86bd792bed5dd5`, final integration `b012b4e57bc67d2ee4413a06e9f563e713ba05f4` |
| M1-006 | pending | NOT RECORDED | NOT RECORDED | — |
| M1-007 | pending | NOT RECORDED | NOT RECORDED | — |
| M1-008 | pending | NOT RECORDED | NOT RECORDED | — |
| M1-009 | pending | NOT RECORDED | NOT RECORDED | — |
| M1-010 | `function_and_enum_variant_value_names_cannot_collide`; `builtin_print_name_is_reserved` | CI 42354716838: collision test failed with `diagnostics: []`; CI 42354802869: builtin test failed with `diagnostics: []` | CI 42354765665 green after function/variant rejection; CI 42354843025 green after builtin reservation | `d31237b5466a90504eeb342defd9d10b34e87904`, `fd6faf4c252054ca7739610a9f1452259f05c0fb` |

## 3.1 Completed adversarial review scope

The 2026-11-07 review explicitly inspected:

- source IDs/spans and diagnostic rendering;
- lexer/token behavior and literal boundaries;
- parser/AST/recovery;
- scopes/symbols/declaration collisions;
- type checking, generics, match coverage, and return analysis;
- HIR construction and checked-semantic duplication;
- MIR lowering/control flow and local typing;
- interpreter/runtime traps/evaluation order;
- module loading, namespace rewriting, canonicalization, aliases, and source retention;
- compiler pipeline orchestration;
- CLI human/JSON diagnostics;
- all current test suites/fixtures/examples;
- Rust manifests, lockfile policy, toolchain file, and CI workflow;
- README capability claims.

No Critical finding was identified in this pass. Major findings M1-002 through M1-004 and M1-006 through M1-009 remain OPEN; M1-001, M1-005, and M1-010 are RESOLVED with RED -> GREEN evidence. A fresh whole-branch adversarial review remains mandatory after all repairs.

## 3.2 Execution rulings

- **Ruling (repair order):** M1-010 was repaired immediately after discovery, before the larger M1-005 semantic-model refactor, because it was a narrow declaration-boundary guardrail that prevented known checker/HIR/MIR disagreement while shared semantic state is being redesigned. Cost if wrong: some of this rejection logic may later move into the centralized semantic collector, but its behavior and regression tests remain required.

## 4. Verification Commands

Final verification must include at least:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked
```

Additional Gate 1 conformance/adversarial commands must be recorded here once created.

**Current final verification state:** NOT RUN AFTER REQUIRED REPAIRS.

## 5. CI Evidence

Prior CI run(s) proved the pre-review implementation built and passed its then-current tests. They are historical evidence only and do not pass Gate 1 because the adversarial review exposed uncovered defects.

Fresh CI evidence will be recorded after all Gate 1 blockers are resolved.

## 6. Conformance Evidence

Gate 1 requires regression/adversarial coverage. Gate 2 will create the full M1 semantic-freeze conformance corpus.

**State:** INCOMPLETE.

## 7. Fuzz / Property / Stress Evidence

Determine the Gate 1 minimum campaign during detailed stabilization planning. Parser/module/control-flow property tests should be considered where they materially improve confidence.

**State:** NOT YET SATISFIED.

## 8. Performance Evidence

Not a Gate 1 exit focus unless a correctness repair causes a material regression. No performance claims may be inferred from this gate.

## 9. Review Findings Summary

### Critical

None identified yet. Initial full M1 review completed 2026-11-07. Final whole-branch review is still required after repairs.

### Major

OPEN: M1-002, M1-003, M1-004, M1-006, M1-007, M1-008, M1-009.  
RESOLVED with retained RED -> GREEN evidence: M1-001, M1-005, M1-010.

### Minor

Not yet fully catalogued.

## 10. Architecture Review

The M1 semantic-duplication blocker is resolved: TypeChecker is the semantic authority, `SemanticModel` retains checked facts, HIR lowering consumes those facts without maintaining its own function/type/enum/generic inference engine, and MIR consumes explicit checked global categories and pattern-binding types. A final whole-branch architecture review is still mandatory before Gate 1 PASS.

## 11. Security Review

Module path/canonicalization behavior and future source-file handling are security-relevant surfaces. Gate 1 must avoid introducing path traversal/symlink ambiguity that later package tooling would inherit.

## 12. Documentation Audit

Repository governance now explicitly marks Gate 1 as active and M2+ as locked. Language behavior documentation still requires review after correctness repairs and before Gate 2 semantic freeze.

## 13. Cross-Cutting Language Laws Affected

- Law 1 (safe/defined behavior): control-flow/runtime correctness foundations.
- Law 2 (simple programs remain simple): fixes must not add unnecessary syntax ceremony.
- Law 3 (performance explainable): short-circuit semantics must match visible control flow.
- Law 6 (diagnostics): source-aware stable diagnostics are an explicit Gate 1 blocker.
- Law 9 (tooling): CLI/diagnostics must expose compiler truth consistently.
- Law 10 (AI-first): machine-readable diagnostics and single semantic truth are prerequisites.

## 14. Known Limitations

All OPEN findings above are blockers, not accepted limitations.

## 15. Final Decision

**Decision:** FAIL / ACTIVE  
**Reason:** Gate 1 still has unresolved Major correctness findings and incomplete final verification; M1-005/M1-010 are resolved, but Gate 1 is not closeable yet.

### Owner authorization

**Next gate authorized to open:** NO  
**Owner:** Timothy Holm  
**Approval record:** Owner explicitly required a complete gated plan before further advancement; Gate 2 remains locked until Gate 1 is formally passed and owner authorization is recorded.
