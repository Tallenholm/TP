# Contributing to TP

TP is proprietary software. Contributions are not automatically accepted and do not grant any license to TP. Read the root `LICENSE` before submitting anything.

## Mandatory project process

Before making a code or design change, read:

1. `AGENTS.md`
2. `docs/MASTER_GATED_DEVELOPMENT_PLAN.md`
3. `docs/gates/STATUS.md`
4. the active gate report, if present
5. the applicable implementation plan/specification

The current gate status controls what work is allowed. Do not implement work assigned to a LOCKED gate.

## Pull requests

Major work must use a dedicated branch and a draft pull request until its gate permits completion.

Every pull request must:

- identify the gate it belongs to;
- identify the requirement(s) it satisfies;
- include RED -> GREEN evidence for behavior changes/bug fixes;
- include exact test/build/format/lint commands and results;
- list documentation/specification updates;
- disclose new dependencies and licenses;
- disclose unsafe/security-sensitive behavior;
- identify known limitations;
- list unresolved review findings by severity;
- remain unmerged while any Critical or Major finding is unresolved;
- remain unmerged while its required gate is not PASSED.

A green CI check does not override the gate plan.

## Required implementation sequence

For every coding task:

1. Write acceptance criteria.
2. Write a failing test.
3. Confirm the test fails for the correct reason.
4. Implement the smallest correct change.
5. Confirm the targeted test passes.
6. Add negative/boundary/regression coverage.
7. Run subsystem tests.
8. Run the full suite.
9. Run formatting.
10. Run lints with warnings denied.
11. Update docs/specs.
12. Review the diff.
13. Perform adversarial review.
14. Resolve all Critical/Major findings.
15. Run fresh CI.
16. Record evidence in the gate report.

## Specifications and semantics

Do not change TP semantics accidentally while fixing implementation details.

If desired behavior differs from the current specification:

1. write the proposed semantic/specification change;
2. review its impact on existing language laws and conformance;
3. update the controlling specification/ADR;
4. update tests to encode the approved behavior;
5. then implement it.

Never weaken a specification or test solely to make code pass.

## Review severity

- **Critical:** safety, security, data-loss, UB, fundamentally unsound semantics, or release-blocking architecture defect.
- **Major:** incorrect language behavior, material compiler/runtime bug, major architecture debt that blocks later gates, incomplete mandatory requirement, or meaningful security/reproducibility problem.
- **Minor:** non-blocking maintainability, clarity, ergonomics, documentation, or optimization issue.

Critical and Major issues block gate passage and merge.

## Dependency policy

Do not copy third-party source code into TP without explicit review of provenance and license.

New dependencies must be justified, versioned deliberately, and have their licenses recorded. Prefer fewer dependencies in core compiler/safety-critical paths.

## Generated files

Do not commit build output such as `target/`, temporary artifacts, caches, local editor data, or machine-specific files.

## Owner authority

Timothy Holm is the repository/project owner. The owner may change project direction, but the controlling repository documents should be updated first so every subsequent human/agent receives the same instruction.
