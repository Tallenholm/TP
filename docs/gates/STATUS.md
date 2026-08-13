# TP Gate Status

**This file is authoritative for what work may currently proceed.**

Last updated: 2026-08-12

| Gate | Name | State | Notes |
|---|---|---|---|
| 0 | Governance, Repository Control, and IP | ACTIVE | Public-repository decision, proprietary policy, ADR process, dependency-license audit, and Gate Policy are complete. Only verified server-side protection of `main` remains. |
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
| 14 | First-Party Developer Tooling | LOCKED | Feature completion belongs here; only work directly required by an active gate may occur earlier. |
| 15 | Security and Supply-Chain Hardening | LOCKED | Security review occurs throughout, but dedicated hardening remains locked. |
| 16 | Performance and Predictability | LOCKED | No performance claims until this gate passes. |
| 17 | Real-Application Validation | LOCKED | Depends on usable toolchain/ecosystem. |
| 18 | Self-Hosting Preparation | LOCKED | Depends on mature semantics and tooling. |
| 19 | Full Self-Hosting | LOCKED | Depends on Gate 18 PASSED. |
| 20 | v0.1 Release Candidate | LOCKED | Depends on all release-critical prior gates PASSED. |

## Current allowed work

Only work necessary to complete Gate 0 or Gate 1 is authorized.

The sole unresolved Gate 0 requirement is server-side protection of `main`. The repository intentionally remains public under `docs/decisions/ADR-0001-public-repository-for-ci.md` so TP can use public-repository GitHub-hosted CI while retaining proprietary licensing.

Do not begin ownership/borrow checking, unsafe semantics, effects/capabilities, async/concurrency, package ecosystem expansion, C/WASM/JS/Python interop, LLVM/native generation, or self-hosting.

## Known Gate 1 blockers

1. Return-flow analysis is unsound because the presence of any explicit return can suppress missing-return validation on other paths.
2. `&&` / `||` currently use eager operand lowering and therefore lack correct short-circuit semantics.
3. Type-name/generic validation requires hardening.
4. Diagnostics need end-to-end source-aware rendering and richer machine-readable spans.
5. Semantic logic is duplicated between checking and HIR lowering and must move toward a single source of truth.
6. Module loading requires a broader adversarial/conformance matrix.

This list is not exhaustive. The Gate 1 review ledger controls the complete finding set.
