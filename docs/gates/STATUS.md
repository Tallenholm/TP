# TP Gate Status

**This file is authoritative for what work may currently proceed.**

Last updated: 2026-11-07

| Gate | Name | State | Notes |
|---|---|---|---|
| 0 | Governance, Repository Control, and IP | PASSED | Governance/IP/repository policy complete. Owner-managed `main` without mandatory server-side branch protection is explicitly accepted in ADR-0002. |
| 1 | M1 Adversarial Stabilization | ACTIVE | All known Critical/Major findings are resolved. Remaining work is stress/conformance evidence, documentation audit, branch reconciliation/final adversarial review, and fresh final locked CI. |
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

Gate 0 is PASSED. Only Gate 1 implementation/review work is currently authorized; Gate 2 and later gates remain LOCKED.

The repository intentionally remains public under `docs/decisions/ADR-0001-public-repository-for-ci.md` so TP can make aggressive use of public-repository GitHub-hosted CI while retaining proprietary licensing. Under `docs/decisions/ADR-0002-owner-managed-main.md`, server-side branch protection is optional defense in depth and is not a prerequisite for continuing.

Examples of allowed work:

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

## Remaining Gate 1 closure work

All ten known Major findings in the Gate 1 adversarial ledger are currently **RESOLVED** with retained RED -> GREEN evidence. No Critical finding is currently open.

Gate 1 remains ACTIVE because the proof/closure work is not yet complete:

1. record deterministic stress/recovery evidence appropriate to Gate 1;
2. finish the M1 documentation audit so README/spec-facing claims match implemented behavior;
3. reconcile the active branch with the current `main` governance/docs state;
4. perform a fresh whole-branch adversarial architecture/correctness review;
5. run the complete final verification on pinned Rust 1.85.0 with locked dependencies;
6. record final CI/evidence in the Gate 1 report.

Gate 2 remains LOCKED until Gate 1 is formally PASS and the owner explicitly authorizes Gate 2.
