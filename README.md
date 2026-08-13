# TP

> [!IMPORTANT]
> **TP uses mandatory gated development.** Before changing code or design, read [`AGENTS.md`](AGENTS.md), [`docs/MASTER_GATED_DEVELOPMENT_PLAN.md`](docs/MASTER_GATED_DEVELOPMENT_PLAN.md), and [`docs/gates/STATUS.md`](docs/gates/STATUS.md). A green build or passing CI does **not** authorize advancement to a later gate.

TP is a proprietary experimental general-purpose programming language and toolchain designed for software built jointly by humans and AI agents.

**Copyright (c) 2026 Timothy Holm. All Rights Reserved. TP is not open source.** See [`LICENSE`](LICENSE).

## Current development state

Only work authorized by [`docs/gates/STATUS.md`](docs/gates/STATUS.md) may proceed. Locked-gate work must not begin until the controlling plan/status is explicitly changed and prerequisite gates have passed.

The active M1 implementation is being reviewed under **Gate 1 — M1 Adversarial Stabilization**. Its live gate report is maintained on the active milestone branch and must be PASS before M1 can merge.

## Project controls

- [`AGENTS.md`](AGENTS.md) — mandatory instructions for agents and contributors.
- [`docs/MASTER_GATED_DEVELOPMENT_PLAN.md`](docs/MASTER_GATED_DEVELOPMENT_PLAN.md) — complete Gate 0–20 roadmap and exit criteria.
- [`docs/gates/STATUS.md`](docs/gates/STATUS.md) — authoritative current gate states.
- [`docs/gates/GATE_REPORT_TEMPLATE.md`](docs/gates/GATE_REPORT_TEMPLATE.md) — required evidence package for passing a gate.
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — contribution and review workflow.
- [`docs/superpowers/specs/2026-08-12-universal-language-design.md`](docs/superpowers/specs/2026-08-12-universal-language-design.md) — language design constitution.

Implementation details and runnable examples may live on active milestone branches until their gates pass and they are approved for merge to `main`.
