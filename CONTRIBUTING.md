# Contributing to TP

TP is proprietary software. Contributions are not automatically accepted and do not grant any license to TP. Read the root `LICENSE` before submitting anything.

Before changing code or design, read `AGENTS.md`, `docs/MASTER_GATED_DEVELOPMENT_PLAN.md`, `docs/gates/STATUS.md`, the active gate report, and the applicable implementation plan/specification.

The current gate status controls what work is allowed. Do not implement work assigned to a LOCKED gate.

Every pull request must identify its gate/requirements, include RED -> GREEN evidence, exact verification commands/results, documentation/spec updates, dependency/license changes, unsafe/security-sensitive behavior, known limitations, and unresolved review findings by severity. Critical/Major findings and an unpassed gate block merge.

Every coding task follows: acceptance criteria -> failing test -> confirm RED -> minimal correct implementation -> targeted GREEN -> negative/boundary/regression coverage -> subsystem suite -> full suite -> format -> lint with warnings denied -> docs/spec update -> diff/adversarial review -> resolve Critical/Major -> fresh CI -> gate-report evidence.

Do not change semantics accidentally. If desired behavior changes, update/review the controlling specification or ADR first, then update tests, then implementation. Never weaken tests/specifications solely to make a gate pass.

Do not copy third-party source without provenance/license review. New dependencies must be justified and their licenses recorded. Do not commit generated build output such as `target/`.

Timothy Holm is the project owner. Owner decisions that change process/direction should be reflected in the controlling repository documents before implementation proceeds.
