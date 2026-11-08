# TP Gate Status

**This file is authoritative for what work may currently proceed.**

Last updated: 2026-11-07

| Gate | Name | State | Notes |
|---|---|---|---|
| 0 | Governance, Repository Control, and IP | PASSED | Governance/IP/repository policy complete. Owner-managed `main` without mandatory server-side branch protection is explicitly accepted in ADR-0002. |
| 1 | M1 Adversarial Stabilization | PASSED | G1-R01 through G1-R14 passed with retained RED → GREEN evidence, final whole-branch review, pinned/locked CI, and explicit owner authorization on 2026-11-07. |
| 2 | M1 Semantic Freeze | VERIFYING | 144-case semantic corpus is green, all five Gate 2 semantic decisions are resolved, and documentation/final semantic review are complete. Awaiting fresh exact-head CI and explicit owner authorization before PASS / Gate 3. |
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

Gate 0 and Gate 1 are PASSED. Gate 2 — M1 Semantic Freeze — is VERIFYING. Only Gate 2 verification/evidence/documentation closure work is authorized. Gate 3 and later gates remain LOCKED.

The repository intentionally remains public under `docs/decisions/ADR-0001-public-repository-for-ci.md` so TP can make aggressive use of public-repository GitHub-hosted CI while retaining proprietary licensing. Under `docs/decisions/ADR-0002-owner-managed-main.md`, server-side branch protection is optional defense in depth and is not a prerequisite for continuing.

Examples of allowed work:

- freezing/documenting M1 lexical, syntactic, type, control-flow, runtime, module, and entry-point semantics;
- creating the independent `tests/conformance/` corpus;
- identifying and resolving semantic ambiguities exposed by Gate 2;
- synchronizing README/examples/specification text with the frozen behavior;
- verification and evidence directly required to pass Gate 2.

Examples of prohibited work right now:

- ownership/borrow checker implementation;
- unsafe model implementation;
- effects/capabilities implementation;
- async/concurrency feature development;
- package registry/build ecosystem feature expansion;
- C/WASM/JS/Python interop implementation;
- LLVM/native code generation;
- self-hosting.

If Gate 2 evidence exposes a Gate 1 correctness regression, reopen Gate 1 and automatically lock Gate 2 until the regression is repaired and re-verified.

## Remaining Gate 2 closure work

The complete planned M1 semantic surface is frozen and 144/144 implementation-independent conformance cases are green on the pre-closure head. Final semantic/diff review and documentation audit are complete.

Remaining:

1. receive fresh pinned Rust 1.85.0 / locked-dependency CI on the latest documentation-closure head;
2. obtain explicit owner authorization before Gate 2 may become PASSED / Gate 3 ACTIVE.

M1 remains draft/unmerged until Gate 2 PASSES. Gate 3 remains LOCKED until Gate 2 passes and the owner explicitly authorizes Gate 3.
