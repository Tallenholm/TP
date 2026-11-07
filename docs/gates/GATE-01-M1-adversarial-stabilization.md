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
| G1-R03 | Correct short-circuit semantics for `&&` and `||` | PASS | Logical operators now lower to MIR branch/RHS/join control flow. RED CI 42356104422 showed eager `Rvalue::Binary::And`; GREEN CI 42356235494 and 42356279096 prove skip/evaluate behavior for both operators. |
| G1-R04 | Validate declared types and generic arity/parameters | PASS | Two-pass nominal type collection now validates declaration existence, exact arity, generic parameters, primitive arity/reserved names, forward refs, and recursive nominal declarations. RED CI 42356411195; GREEN CI 42356689441. |
| G1-R05 | Harden declaration/import/alias collision rules | PASS | Core declaration/scoping policy plus import-alias rules are now tested: exact repeated imports dedupe, one canonical module may have multiple aliases, and conflicting reuse of one alias by different modules diagnoses at the import boundary. |
| G1-R06 | Define/test literal range, float, string, Unicode behavior | PASS | M1 policy is explicit/tested: signed i64 including exact `i64::MIN`; oversized integers diagnose without AST loss; decimal floats must be finite f64; strings are single-line with explicit escapes; Unicode alphabetic/alphanumeric identifiers are accepted as written with no normalization, while combining marks outside that policy diagnose. RED CI 42357118044/42357198323; GREEN CI 42357253254. |
| G1-R07 | Statically validate executable `main` contract | PASS | Declared `main` must take zero parameters; run mode rejects a missing `main` as TP-E0400 compile failure before interpreter execution while `check` remains usable for library/empty sources. RED CI 42356814898; GREEN CI 42356929666. |
| G1-R08 | Source-aware human and rich versioned JSON diagnostics | PASS | `SourceManager` now retains entry/imported sources through checking; CLI human output includes file/line/snippet/caret, Unicode caret width is character-based, secondary labels/help render, and JSON schema 1 includes source ranges, labels, and help. RED CI 42357731005; GREEN CI 42409295937 + rich-boundary CI 42409405310. |
| G1-R09 | Establish one authoritative semantic source of truth | PASS | `SemanticModel` now carries checked function signatures, expression/block types, binding symbols/types, local resolutions, and global categories into HIR; `lower_hir.rs` is structural-only and MIR consumes checked global/binding facts. CI 42355570648 green. |
| G1-R10 | Harden module graph/path/canonicalization semantics | PASS | Canonical-path module identity now deduplicates repeated/diamond/symlink imports; qualified imported nominal types/struct literals/enum constructors/patterns work; deterministic lowering and host-filesystem behavior are tested/documented. |
| G1-R11 | Reproducible locked verification policy | FAIL | Current CI requires hardening/explicit locked policy. |
| G1-R12 | Zero unresolved Critical/Major findings | PASS | All ten Major findings M1-001 through M1-010 are RESOLVED with retained RED -> GREEN evidence; no Critical finding has been identified. Final whole-branch review remains required before Gate 1 PASS. |
| G1-R13 | Full tests/format/lint/fresh CI after all repairs | FAIL | Prior green CI predates required repairs and does not close this gate. |
| G1-R14 | Documentation accurately reflects stabilized M1 behavior | FAIL | Docs now mark gate status, but semantic docs remain subject to Gate 1/2 work. |

## 2. Adversarial Review Ledger

Every finding must remain here until resolved. Do not delete findings after fixes; mark them RESOLVED and retain evidence.

| ID | Severity | Subsystem | Finding | Reproducer / evidence | Required resolution | Status |
|---|---|---|---|---|---|---|
| M1-001 | Major | Type/control flow | Function validation previously treated the existence of any explicit return as sufficient and represented returning branches as `Unit`, causing both unsound acceptance and false branch-type rejection. | `missing_return_path_is_rejected`; `returning_branch_unifies_with_tail_branch`; nested/while/unreachable/runtime boundaries. | Removed heuristic; internal `Never` models diverging control flow and propagates through expression typing; block fallthrough is path-sensitive. | RESOLVED |
| M1-002 | Major | MIR/runtime semantics | `&&` / `||` were lowered as ordinary eager binary expressions, so RHS traps occurred when they should have been skipped. | MIR shape test plus runtime trap-skipping/evaluation tests for all four truth-table evaluation cases. | Logical operators now lower through branch/RHS/short-path/join blocks; RHS executes only on the required branch. | RESOLVED |
| M1-003 | Major | Type system | Unknown type names were materialized as nominal types without declaration proof; generic arity/parameter validation and several declaration collisions were missing. | RED matrix: unknown declared type, Option too few/many args, duplicate generic params, duplicate struct fields, struct/enum shared type-name collision, function/type collision. | Two-pass type-header/body collection validates known names/arity while preserving forward/recursive references; declaration/scoping boundary matrix added. | RESOLVED |
| M1-004 | Major | Diagnostics | CLI output discarded source locations/labels/help; imported source retention was insufficient; renderer used byte width and omitted secondary labels. | CLI regressions for human source snippet, JSON source range, imported-file source retention; renderer Unicode/secondary-label test; rich JSON label/help boundary. | Added retained `SourceManager`, source-aware CLI rendering, versioned JSON ranges/labels/help, character-width caret rendering, and imported-source preservation. | RESOLVED |
| M1-005 | Major | Architecture | Type/semantic knowledge was independently reconstructed in TypeChecker and HIR lowering, and pattern-binding types were lost before MIR. | `match_binding_local_keeps_checked_payload_type`; `hir_preserves_checked_global_resolution_kind`; pre-fix `lower_hir.rs` contained independent function/struct/enum/type/generic inference tables. | `SemanticModel` now carries checker-authoritative types/symbols/resolutions; HIR lowering is structural-only; MIR consumes checked global kinds and binding types. | RESOLVED |
| M1-006 | Major | Module system | Recursive namespace-rewrite loader reloaded canonical files as copies, duplicate aliases surfaced late as duplicate declarations, diamond/symlink identity was not deduplicated, and qualified imported nominal types/constructors/patterns were not expressible. | Module identity/alias RED matrix plus qualified imported type/constructor RED program. | Canonical path is now the one module identity per compilation; repeated/diamond/symlink imports reuse it; aliases map to canonical namespaces; conflicting aliases diagnose early; parser/rewriter support qualified imported types, struct literals, enum constructors, and patterns; exact M1 policy documented. | RESOLVED |
| M1-007 | Major | Entry point | `main` shape and presence in executable mode were not fully validated statically; missing/parameterized entry points could reach interpreter runtime errors. | `main_must_not_take_parameters`; `run_requires_main_before_interpreter_execution`. | `main` is zero-parameter; return may be any normal checked M1 value; run mode requires `main` and emits TP-E0400 before interpretation; check mode does not require an entry point. | RESOLVED |
| M1-008 | Major | Literals/parser | Oversized integer parsing could consume a token and return `None` with no diagnostic, silently dropping syntax; `i64::MIN`, non-finite floats, raw-newline strings, escapes, and Unicode identifier boundaries were not all explicit. | Lexer/parser/runtime boundary tests for raw newline, invalid escape, Unicode spelling, oversized integer expression/pattern, huge float, exact `i64::MIN`. | Parser now emits stable TP-E0100 range/finite diagnostics with placeholder AST recovery; exact negative i64 minimum is recognized before positive parsing; lexer rejects raw newlines while retaining existing escape/Unicode behavior. | RESOLVED |
| M1-009 | Major | Pattern matching | Compile-time exhaustiveness was enforced only for enum matches, and refutable payload patterns such as `Some(1)` incorrectly counted as covering the whole variant. Duplicate/post-catch-all arms were not diagnosed. | Bool/open-domain/exhaustive/unreachable/refutable-enum payload regression matrix. | Domain-aware match coverage now requires both Bool values or catch-all; open domains require wildcard/binding; enum variants count as fully covered only with irrefutable payload patterns; duplicate/post-exhaustive arms diagnose as unreachable. | RESOLVED |
| M1-010 | Major | Name resolution / semantic consistency | Global value-name precedence was inconsistent across checker, HIR, and MIR. Function/enum-variant collisions and user declarations named `print` could be accepted yet resolve differently across stages. | Regression tests `function_and_enum_variant_value_names_cannot_collide` and `builtin_print_name_is_reserved`. | Reject ambiguous function/variant value names and reserve builtin `print`; the broader semantic-source refactor in M1-005 will carry checked global resolution into HIR/MIR. | RESOLVED |

Severity may only be changed with documented reasoning. More findings will be added as review continues.

## 3. Required RED -> GREEN Evidence

No repair is complete until its test has been observed failing for the intended reason before the fix and passing afterward.

| Finding | Regression test(s) | RED | GREEN | Fix commit |
|---|---|---|---|---|
| M1-001 | `missing_return_path_is_rejected`; `returning_branch_unifies_with_tail_branch`; `all_if_branches_may_return_explicitly`; nested/while/unreachable/runtime boundaries | CI 42355699407: missing-return test was accepted with `diagnostics: []`; mixed return/tail test was rejected as `Unit` vs `i64` | CI 42355902092 green for core regressions; CI 42355992729 green for expanded type/runtime boundary matrix | `d0b45f281d0a1ea228b435c457a4e9066e4d6c2d` + integration fixes through `2dc0a9b9811e5d432e481170cdf1a9ed1fe3bf9c`; boundary tests `45b0ba7166a215659067bba9b708e2ce86c0f530` |
| M1-002 | `logical_and_lowers_to_control_flow_not_eager_binary`; `false_and_skips_trapping_rhs`; `true_or_skips_trapping_rhs`; `logical_operators_evaluate_rhs_when_required` | CI 42356104422: MIR test showed a single eager `Rvalue::Binary { op: And }` block | CI 42356235494 green for MIR + skip-RHS runtime tests; CI 42356279096 green for required-RHS evaluation boundaries | `a852470949e9ad17737a083d27a173b946cfb325`, formatting integration `7e137810cce057e67970e39a101aa52625d61d16` |
| M1-003 | `unknown_declared_type_is_rejected`; `nominal_generic_arity_is_checked`; duplicate generic/field/type/function collisions; forward/generic/primitive/scope/recursive boundaries | CI 42356411195: six intended tests failed with no diagnostics; nested-shadowing policy test already passed | CI 42356651208 green for core repairs; CI 42356689441 green for expanded boundary matrix | `5911644db0c8da264c6887340fc4df74f903fb96`, compatibility/cleanup through `1a22cc2329687d980898529e446d5d19eb8ec9a4`; boundary tests `9f7759aa52b1740e7d4b6e70ff680725b19d31c1` |
| M1-004 | human CLI source/snippet; JSON source range; imported-file source retention; Unicode caret/secondary labels; rich JSON labels/help | CI 42357731005: three CLI tests failed—human output had no location/snippet, JSON lacked ranges, imported-file diagnostic lost its source | CI 42409295937 green for source-retention/rendering refactor; CI 42409405310 green for non-empty JSON label/help boundary | source-aware refactor `96b0554d9c35cc468fbae52b971158be4540b089`, formatting integration `bfa82e8649a450bc1c87ef199b11d51917641e2f`, boundary `86d0a389dce12414f35c3c06cde904ab0ad83823` |
| M1-005 | `match_binding_local_keeps_checked_payload_type`; `hir_preserves_checked_global_resolution_kind` | CI 42354924948: MIR binding was `Unknown` vs expected `I64`; CI 42355127048: compile failed because HIR had no checked global-kind representation | CI 42354993528 green after binding-type preservation; CI 42355329658 green after checked global-kind handoff; CI 42355570648 green after full checker-authoritative SemanticModel refactor | `6832ec15764fbe12f131f42b5f70324b21aefdc4`, `7bee810aed2546087aa7d030855e9bdb181a3945`, `9d1b69fc48af8909acfb20fc8e86bd792bed5dd5`, final integration `b012b4e57bc67d2ee4413a06e9f563e713ba05f4` |
| M1-006 | repeated import, two aliases/one module, duplicate alias, diamond import, qualified imported nominal types/constructors/patterns, deterministic lowering, Unix symlink canonical identity | CI 42409537781: all four canonical identity/alias tests failed; CI 42409719942: qualified imported nominal-type program failed with parser diagnostics | CI 42409634171 green for canonical identity/aliases; CI 42409826572 green for qualified imported types/constructors/patterns; CI 42409946529 green for deterministic + symlink path boundaries | canonical loader `2ddbb6cc85e8bbc1caf27dc8dc105f7f6efa4533` + format `99a160df0116de2403d94696b3d7f193cba19cd8`; qualification `9f6c3142ea5391e30038da53759a0c1696ad9875`; path boundaries `f9e94f1863256d7513ed9e419b68f87c56915156` |
| M1-007 | `main_must_not_take_parameters`; `run_requires_main_before_interpreter_execution` | CI 42356814898: missing-main run reached runtime failure instead of compile failure; parameter-shape regression was queued behind that failing suite | CI 42356929666 green for both contract tests plus full suite/smokes | `6752724fd35f641fb7385f182e0b1cb54a77a501`, formatting integration `53c40b4e52034d8475b7ab65731d1cf09b8bfe4c` |
| M1-008 | raw-newline/escape/Unicode lexer boundaries; oversized integer expression/pattern; non-finite float; exact `i64::MIN` runtime | CI 42357118044: raw newline string accepted with `diagnostics: []`; after lexer fix, CI 42357198323: oversized expression/float had no diagnostics and oversized pattern cascaded without the required range diagnostic | CI 42357253254 green for full literal/Unicode matrix plus workspace/smokes | lexer fix `6ccbeee622ed7ba6601b18659b76af61c3e15865`; numeric/parser fix `debb74b0daa0b48d3026ea16e2d59240c7be52cc` |
| M1-009 | `bool_match_must_cover_both_values`; `exhaustive_bool_match_is_allowed`; `open_literal_domain_requires_catch_all`; wildcard/open-domain boundary; unreachable duplicate/post-catch-all arms; refutable enum payload coverage | CI 42357479861: four intended tests failed with empty diagnostics while already-valid exhaustive/wildcard cases passed | CI 42357592503 green for full match matrix + workspace/smokes | `9f1a05034cf414d763b2332d181ad5083d044d18`, formatting integration `47e3430f4377be048fd60de21b5fa18cc05f7c35` |
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

No Critical finding was identified in this pass. All Major findings M1-001 through M1-010 are RESOLVED with retained RED -> GREEN evidence. A fresh whole-branch adversarial review remains mandatory after all repairs. A fresh whole-branch adversarial review remains mandatory after all repairs.

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

OPEN: none.  
RESOLVED with retained RED -> GREEN evidence: M1-001 through M1-010.

### Minor

Not yet fully catalogued.

## 10. Architecture Review

The M1 semantic-duplication blocker is resolved: TypeChecker is the semantic authority, `SemanticModel` retains checked facts, HIR lowering consumes those facts without maintaining its own function/type/enum/generic inference engine, and MIR consumes explicit checked global categories and pattern-binding types. A final whole-branch architecture review is still mandatory before Gate 1 PASS.

## 11. Security Review

M1 module identity is canonical-path based and Unix symlink deduplication is regression-tested. M1 import grammar accepts sibling identifiers rather than filesystem path syntax, reducing path-traversal surface. Host filesystem case behavior and sibling-only limitations are explicitly documented in `docs/M1_MODULE_SEMANTICS.md`.

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

No Critical/Major finding is currently open. Remaining Gate 1 work is reproducibility, final verification/conformance/stress evidence, documentation audit, and whole-branch adversarial review. M1 module limitations accepted for this gate are documented in `docs/M1_MODULE_SEMANTICS.md` and are subject to Gate 2 semantic freeze.

## 15. Final Decision

**Decision:** FAIL / ACTIVE  
**Reason:** All known Critical/Major correctness findings are resolved, but Gate 1 still requires reproducible locked verification, final conformance/stress evidence, documentation audit, fresh final CI, and whole-branch adversarial review.

### Owner authorization

**Next gate authorized to open:** NO  
**Owner:** Timothy Holm  
**Approval record:** Owner explicitly required a complete gated plan before further advancement; Gate 2 remains locked until Gate 1 is formally passed and owner authorization is recorded.
