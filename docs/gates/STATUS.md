# TP Gate Status

**This file is authoritative for what work may currently proceed.**

Last updated: 2026-11-08

| Gate | Name | State | Notes |
|---|---|---|---|
| 0 | Governance, Repository Control, and IP | PASSED | Governance/IP/repository policy complete. Owner-managed `main` without mandatory server-side branch protection is explicitly accepted in ADR-0002. |
| 1 | M1 Adversarial Stabilization | PASSED | G1-R01 through G1-R14 passed with retained RED → GREEN evidence, final whole-branch review, pinned/locked CI, and explicit owner authorization on 2026-11-07. |
| 2 | M1 Semantic Freeze | PASSED | All 17 requirements passed; 144/144 conformance cases green, five Major decisions resolved, exact-head push/PR CI green on `8821c33`, owner authorized on 2026-11-08. |
| 3 | Compiler Architecture Foundation | ACTIVE | Gate 2 PASSED; owner explicitly authorized Gate 3 on 2026-11-08. Start with stage invariants, stable symbol IDs, centralized semantic context, HIR/MIR verifiers, and dedicated Gate 3 plan/report. |
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

Gates 0, 1, and 2 are PASSED. Gate 3 — Compiler Architecture Foundation — is ACTIVE by explicit owner authorization on 2026-11-08. Gate 4 and all subsequent gates remain LOCKED.

The repository remains public with proprietary licensing under `docs/decisions/ADR-0001-public-repository-for-ci.md`; owner-managed `main` without mandatory server-side branch protection is accepted under `docs/decisions/ADR-0002-owner-managed-main.md`.

## Gate 2 closure evidence

- Technical baseline: commit `8821c33ae6b2d38cbad37406d000bf5d2d651823`.
- Exact-head pinned Rust 1.85.0 / locked CI: push run [42440034242](https://github.com/Tallenholm/TP/actions/runs/42440034242) and PR run [42440037141](https://github.com/Tallenholm/TP/actions/runs/42440037141) both PASS.
- Entire 144-case independent semantic conformance corpus PASS; zero unresolved Critical/Major findings after whole-branch review.
- Owner approval (2026-11-08): Asked explicitly whether to authorize `Gate 2 PASS and Gate 3 ACTIVE`, and answered `If its ready`. Technical checks verified ready; the conditional authorization is met.
- Gate 2 closure changes here are governance documentation only. PR #1 must still pass fresh CI and Gate Policy checks on the updated head before merging.

## Gate 3 authorized scope

- Design/document explicit invariants and contracts between source management, lexer, parser, AST, name resolution, type analysis, typed HIR, MIR, and interpreter/backend.
- Introduce and test stable symbol identities, module graph and centralized semantic context without duplicating semantic decisions.
- Develop compiler query interfaces and typed structured diagnostics.
- Build HIR/MIR verifiers with negative tests for malformed internal IR, and record RED → GREEN evidence.
- Work in a new dedicated Gate 3 branch/draft PR with a Gate 3 report after Gate 2 PR #1 is merged into the gated stable `main`.

Do not begin ownership/borrowing, unsafe model, effects/capabilities, concurrency, packaging, FFI, native code generation, or self-hosting until their own gates are authorized.

If Gate 3 evidence exposes a Gate 1/2 correctness or semantic regression, reopen the responsible gate and relock dependent work until repaired and verified.
