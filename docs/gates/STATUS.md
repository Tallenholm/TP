# TP Gate Status

**This file is authoritative for what work may currently proceed.**

Last updated: 2026-08-12

| Gate | Name | State | Notes |
|---|---|---|---|
| 0 | Governance, Repository Control, and IP | ACTIVE | Public-repository decision, proprietary policy, ADR process, dependency-license audit, and automated Gate Policy are complete. Only verified server-side protection of `main` remains. |
| 1 | M1 Adversarial Stabilization | ACTIVE | Current M1 implementation is under adversarial review. Known semantic defects must be repaired before this gate can pass. |
| 2 | M1 Semantic Freeze | LOCKED | Depends on Gate 1 PASSED. |
| 3 | Compiler Architecture Foundation | LOCKED | Depends on Gate 2 PASSED. |
| 4 | Ownership and Memory Model | LOCKED | Depends on Gate 3 PASSED. |
| 5 | Unsafe Boundary Model | LOCKED | Depends on Gate 4 PASSED. |
| 6 | Effects and Capabilities | LOCKED | Depends on Gate 5 PASSED. |
| 7 | Structured Concurrency and Async | LOCKED | Depends on Gates 4-6 PASSED. |
| 8 | Package and Build System | LOCKED | Depends on prior semantic/safety foundations. |
| 9 | C ABI Interoperability | LOCKED | Depends on safety/unsafe model and package/build foundations. |
| 10 | WebAssembly | LOCKED | Depends on stable semantics and compiler architecture. |
| 11 | Native Backend | LOCKED | Depends on stable MIR/semantics and conformance oracle. |
| 12 | JavaScript / TypeScript Interop | LOCKED | Depends on stable WASM/native semantics. |
| 13 | Python Interop | LOCKED | Depends on stable FFI/runtime semantics. |
| 14 | First-Party Developer Tooling | LOCKED | Tooling architecture may be prepared earlier only when explicitly required by an active gate; feature completion belongs here. |
| 15 | Security and Supply-Chain Hardening | LOCKED | Security checks exist throughout, but the dedicated hardening gate is locked. |
| 16 | Performance and Predictability | LOCKED | No performance claims until this gate passes. |
| 17 | Real-Application Validation | LOCKED | Depends on usable toolchain/ecosystem. |
| 18 | Self-Hosting Preparation | LOCKED | Depends on mature semantics and tooling. |
| 19 | Full Self-Hosting | LOCKED | Depends on Gate 18 PASSED. |
| 20 | v0.1 Release Candidate | LOCKED | Depends on all release-critical prior gates PASSED. |

## Current allowed work

Only work necessary to complete Gate 0 or Gate 1 is authorized.

### Remaining Gate 0 work

The sole unresolved Gate 0 requirement is **G0-R13: server-side protection of `main`**. See:

- `docs/gates/GATE-00-governance.md`
- `docs/gates/GATE-00-MANUAL-BRANCH-PROTECTION.md`

The repository intentionally remains public under `docs/decisions/ADR-0001-public-repository-for-ci.md` so TP can make aggressive use of public-repository GitHub-hosted CI while retaining proprietary licensing.

Examples of allowed work:

- completing/validating `main` protection;
- completing the adversarial M1 review;
- writing regression tests for M1 defects;
- repairing M1 correctness/semantic/diagnostic/module/architecture issues required by Gate 1;
- verification and documentation directly required to pass Gate 0/1.

Examples of prohibited work right now:

- ownership/borrow checker implementation;
- unsafe model implementation;
- effects/capabilities implementation;
- async/concurrency feature development;
- package registry/build ecosystem feature expansion;
- C/WASM/JS/Python interop implementation;
- LLVM/native code generation;
- self-hosting.

If a task appears to belong to a locked gate, stop and update the controlling plan/status only after an explicit owner decision.

## Known Gate 1 blockers

At minimum, the following issues are already known and must be tracked/resolved before Gate 1 can pass:

1. Return-flow analysis is unsound because the presence of any explicit return can suppress missing-return validation on other paths.
2. `&&` / `||` currently use eager operand lowering and therefore lack correct short-circuit semantics.
3. Type-name/generic validation needs hardening.
4. Diagnostics need end-to-end source-aware rendering and richer machine-readable spans.
5. Semantic logic is duplicated between checking and HIR lowering and must be reviewed/refactored toward a single source of truth.
6. Module loading requires a broader adversarial/conformance test matrix.

This list is not exhaustive; the Gate 1 adversarial review ledger controls the complete finding set.
