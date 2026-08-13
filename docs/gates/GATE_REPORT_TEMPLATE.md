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
| R2 | <requirement> | PASS / FAIL | <test/commit/doc> |

## 2. Test-Driven Evidence

For each corrected defect or new behavior, record the RED -> GREEN cycle.

| ID | Test | RED evidence | GREEN evidence | Commit |
|---|---|---|---|---|
| T1 | <test name> | <run/result> | <run/result> | <SHA> |

## 3. Verification Commands

Record exact commands and results; do not write only "tests passed".

```text
<command>
<exit/result summary>
```

Required categories where applicable:

- targeted tests;
- subsystem tests;
- full test suite;
- formatter check;
- linter with warnings denied;
- build;
- conformance suite;
- fuzz/property tests;
- cross-platform checks;
- smoke/real-application checks.

## 4. CI Evidence

| Workflow/run | Commit | Result | Notes |
|---|---|---|---|
| <run ID/link> | <SHA> | PASS / FAIL | <notes> |

## 5. Conformance Evidence

**Cases passed:**  
**Cases failed:**  
**Known exclusions:**

Explain every exclusion explicitly.

## 6. Fuzz / Property / Stress Evidence

**Tool/configuration:**  
**Budget/duration/cases:**  
**Crashes/failures:**  
**Reproducers:**

Write `Not required for this gate` only when the master plan explicitly does not require it.

## 7. Performance Evidence

If this gate can affect performance, record baseline and regression results. Otherwise state why this section is not applicable.

## 8. Review Findings

### Critical

- None / <finding + status>

### Major

- None / <finding + status>

### Minor

- None / <finding + disposition>

A gate with any unresolved Critical or Major finding MUST NOT pass.

## 9. Architecture Review

Record affected invariants, coupling introduced/removed, semantic duplication, unsafe boundaries, and whether later gates can safely build on this result.

## 10. Security Review

Record relevant attack surfaces, trust boundaries, dependency changes, path/file/network behavior, unsafe behavior, and any newly introduced capability.

## 11. Documentation Audit

Confirm that specifications, README/examples, CLI help, and machine-readable interfaces match the implementation.

## 12. Cross-Cutting Language Laws

For each law affected by this gate, record evidence that it remains satisfied:

1. Safe TP code must not invoke undefined behavior.
2. Simple programs must remain simple.
3. Performance must be explainable and predictable.
4. Concurrency must be safe by construction.
5. Effects and permissions must be explicit.
6. Diagnostics are stable interfaces for humans and machines.
7. Interop is first-class.
8. One language should span multiple execution environments.
9. Tooling is part of the language.
10. AI is a first-class programmer.

## 13. Known Limitations

List all known limitations. A limitation that violates an exit criterion is a blocker, not an acceptable limitation.

## 14. Final Decision

**Decision:** PASS / FAIL  
**Reason:**

### Owner authorization

**Next gate authorized to open:** YES / NO  
**Owner:** Timothy Holm  
**Approval record:** <explicit approval reference>

If authorization is NO or absent, dependent gates remain LOCKED even when technical verification is otherwise complete.
