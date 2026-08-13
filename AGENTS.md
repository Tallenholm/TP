# TP Agent and Contributor Instructions

**This file is mandatory project policy. Read it before making any change.**

TP uses a gated development model. No agent, developer, reviewer, or contributor may advance the project to a later gate merely because code builds, tests pass, or a feature appears complete.

## Required reading order

Before modifying TP, read these files in order:

1. `AGENTS.md` — this file.
2. `docs/MASTER_GATED_DEVELOPMENT_PLAN.md` — controlling roadmap and gate definitions.
3. `docs/gates/STATUS.md` — current gate state and the only phase(s) allowed to be active.
4. The applicable gate report under `docs/gates/`, if one exists.
5. The applicable detailed implementation plan under `docs/superpowers/plans/`.
6. `docs/superpowers/specs/2026-08-12-universal-language-design.md` — language constitution/design baseline.
7. `CONTRIBUTING.md` — human/agent contribution workflow.

If any required document conflicts with another, stop and resolve the conflict in the controlling documents before implementing code.

## Rule Zero: gates are walls

Gate states are:

`LOCKED -> ACTIVE -> VERIFYING -> PASSED` or `FAILED`.

A later gate MUST NOT become `ACTIVE` until every dependency gate is `PASSED`.

The current status file controls what work is allowed. Work belonging to a `LOCKED` gate is prohibited unless Timothy Holm explicitly changes the plan/status first.

Passing CI is necessary but never sufficient to pass a gate.

## Mandatory task micro-gate

Every implementation task follows this sequence without skipping:

1. State the requirement and acceptance criteria.
2. Write the failing test first.
3. Run it and confirm RED for the intended reason.
4. Implement the smallest correct change.
5. Run the targeted test and confirm GREEN.
6. Add negative, boundary, and regression tests.
7. Run the subsystem suite.
8. Run the full suite.
9. Run formatter checks.
10. Run lints with warnings denied.
11. Update documentation/specification if behavior changed.
12. Review the diff for architecture, security, correctness, and accidental scope.
13. Perform adversarial review.
14. Resolve every Critical and Major finding.
15. Obtain fresh CI evidence.
16. Record evidence in the applicable gate report.
17. Only then may the task be called complete.

If RED does not fail for the expected reason, implementation MUST NOT begin.

If any regression appears, the task returns to `FAILED`/incomplete.

## Mandatory major-gate requirements

A gate cannot be marked `PASSED` unless all of the following are true:

- every written gate requirement is implemented;
- every requirement has tests;
- required negative/failure tests exist;
- regression tests demonstrated RED -> GREEN;
- full test suite passes;
- formatter passes;
- linter passes with warnings denied;
- required fuzz/property/conformance tests pass;
- documentation matches behavior;
- there are zero unresolved Critical or Major review findings;
- architecture review has no known blocking issue;
- CI independently reproduces verification;
- a completed gate report records exact evidence;
- Timothy Holm explicitly approves opening the next major gate.

Do not weaken tests, delete acceptance criteria, downgrade severity, or change a specification merely to make a gate pass. If the desired behavior changes, update and review the specification first.

## Going backward is allowed

A later discovery may reopen an earlier gate. Examples:

- real-application testing exposes a bad ownership model -> reopen the ownership gate;
- native code generation exposes semantic ambiguity -> reopen the semantics gate;
- fuzzing exposes parser architecture defects -> reopen the relevant compiler gate.

Milestone numbers are not progress if correctness is compromised.

## Current project restriction

Until `docs/gates/STATUS.md` says otherwise, do not begin M2 ownership/effects/concurrency/native-codegen work. The current priority is governance plus M1 adversarial stabilization and semantic correctness.

## Branch and PR policy

- `main` represents the last gated/stable project state.
- Do not merge a milestone PR while its gate is not `PASSED`.
- Major work should use a dedicated branch and draft PR.
- PRs must identify their gate and include verification evidence.
- Do not commit generated build output such as `target/`.
- Avoid force-pushing reviewed history unless recovery requires it and the reason is documented.

## Intellectual property

TP is proprietary and All Rights Reserved. The root `LICENSE` controls. Do not assume public visibility grants an open-source license or permission to reuse TP in another project.

Do not import third-party code merely because it is publicly visible. Record third-party dependencies and licenses deliberately.

## Authority and changes to this policy

Timothy Holm is the project owner. An explicit owner decision may change this process, but the controlling repository documents should be updated before implementation proceeds so future humans and agents inherit the same decision.

**When uncertain: stop at the current gate, preserve evidence, and do not advance.**
