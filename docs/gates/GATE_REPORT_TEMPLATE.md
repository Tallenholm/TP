# Gate XX — <Name> Report

**Gate:** XX — <Name>  
**State:** ACTIVE / VERIFYING / PASSED / FAILED  
**Specification revision:** <commit/tag/path>  
**Implementation PR:** <PR number/link>  
**Head commit:** <SHA>  
**Report date:** YYYY-MM-DD

> A gate cannot be marked PASSED unless every required PASS item below is supported by evidence and there are zero unresolved Critical or Major findings.

## 1. Requirements

| ID | Requirement | Status | Evidence |
|---|---|---|---|
| R1 | <requirement> | PASS / FAIL | <test/commit/doc> |

## 2. Test-Driven Evidence

| ID | Test | RED evidence | GREEN evidence | Commit |
|---|---|---|---|---|
| T1 | <test name> | <run/result> | <run/result> | <SHA> |

## 3. Verification Commands

Record exact commands and results for targeted tests, subsystem tests, full suite, formatter, linter with warnings denied, build, conformance, fuzz/property/stress, cross-platform, and smoke/real-app checks as applicable.

## 4. CI Evidence

| Workflow/run | Commit | Result | Notes |
|---|---|---|---|
| <run ID/link> | <SHA> | PASS / FAIL | <notes> |

## 5. Conformance Evidence

**Cases passed:**  
**Cases failed:**  
**Known exclusions:**

## 6. Fuzz / Property / Stress Evidence

**Tool/configuration:**  
**Budget/duration/cases:**  
**Crashes/failures:**  
**Reproducers:**

## 7. Performance Evidence

Record baseline/regression evidence where relevant.

## 8. Review Findings

### Critical
- None / <finding + status>

### Major
- None / <finding + status>

### Minor
- None / <finding + disposition>

A gate with any unresolved Critical or Major finding MUST NOT pass.

## 9. Architecture Review

Record affected invariants, coupling, semantic duplication, unsafe boundaries, and whether later gates can safely build on this result.

## 10. Security Review

Record relevant attack surfaces, trust boundaries, dependency changes, path/file/network behavior, unsafe behavior, and new capabilities.

## 11. Documentation Audit

Confirm specifications, README/examples, CLI help, and machine-readable interfaces match implementation.

## 12. Cross-Cutting Language Laws

Record evidence for affected laws: safe code/UB, simplicity, predictable performance, concurrency safety, explicit effects, stable diagnostics, interop, multi-target reach, first-party tooling, and AI-first programmability.

## 13. Known Limitations

A limitation violating an exit criterion is a blocker, not an acceptable limitation.

## 14. Final Decision

**Decision:** PASS / FAIL  
**Reason:**

### Owner authorization

**Next gate authorized to open:** YES / NO  
**Owner:** Timothy Holm  
**Approval record:** <explicit approval reference>

If authorization is NO or absent, dependent gates remain LOCKED.
