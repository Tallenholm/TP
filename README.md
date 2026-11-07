# TP

> [!IMPORTANT]
> **TP uses mandatory gated development.** Before changing code or design, read [`AGENTS.md`](AGENTS.md), [`docs/MASTER_GATED_DEVELOPMENT_PLAN.md`](docs/MASTER_GATED_DEVELOPMENT_PLAN.md), and [`docs/gates/STATUS.md`](docs/gates/STATUS.md). Passing CI does **not** authorize advancement to a later gate.

TP is the codename for a proprietary experimental general-purpose programming language and toolchain designed for software built jointly by humans and AI agents.

The long-term goal is deliberately ambitious: combine Python/TypeScript-level approachability with systems-language safety and predictable performance, while making compiler behavior, diagnostics, effects, permissions, intermediate representations, and tooling unusually easy for both people and software agents to inspect.

> **Project maturity:** pre-release research and implementation. TP is not ready for production use and no compatibility guarantee exists yet.

> **Proprietary software.** Copyright (c) 2026 Timothy Holm. All Rights Reserved. TP is **not open source**. Public GitHub visibility does not grant a general right to copy, modify, redistribute, deploy, commercialize, train on, or reuse the project. See [`LICENSE`](LICENSE).

## Why TP exists

Modern software development often forces a tradeoff between fast iteration, strong compile-time guarantees, predictable systems performance, cross-platform reach, and good interoperability with existing ecosystems.

TP is being designed to reduce that tradeoff rather than merely introduce new syntax.

The project is justified only if it can eventually demonstrate all of the following with evidence:

- ordinary application code remains easy to read and write;
- safe code cannot invoke undefined behavior;
- memory and concurrency safety do not require excessive ceremony;
- allocation, copying, async work, and other expensive behavior are explainable;
- effects and privileged capabilities are explicit and inspectable;
- diagnostics work equally well for humans, editors, CI, and AI agents;
- native, WebAssembly, C, JavaScript/TypeScript, and Python interoperability are practical;
- builds and dependency state are reproducible;
- the language can support real applications without constantly escaping into foreign code.

Novelty alone is not a success criterion.

## Core design laws

TP's foundational design is governed by ten non-negotiable laws:

1. **Safe code must not invoke undefined behavior.**
2. **Simple programs must stay simple.**
3. **Performance must be predictable and explainable.**
4. **Concurrency must be safe by construction.**
5. **Effects and permissions must be explicit.**
6. **Compiler diagnostics are a public interface.**
7. **Interop is part of the core product.**
8. **One language should span multiple execution environments.**
9. **Tooling is part of the language.**
10. **AI is a first-class programmer of TP.**

The full design rationale lives in [`docs/superpowers/specs/2026-08-12-universal-language-design.md`](docs/superpowers/specs/2026-08-12-universal-language-design.md).

## Language direction

The planned language model includes:

- static, strong typing with local inference;
- immutable bindings by default with explicit mutation;
- braces instead of significant whitespace;
- no broad truthiness;
- no implicit numeric narrowing;
- explicit optionality rather than implicit null;
- structs, enums, algebraic data types, generics, and pattern matching;
- recoverable errors represented as values;
- ownership-oriented deterministic memory management without a mandatory tracing GC;
- explicit unsafe boundaries for operations that cannot be proven safe;
- explicit effects/capabilities for filesystem, network, process, environment, clock, randomness, devices, and unsafe behavior;
- structured concurrency with explicit task lifetime and cancellation;
- a compiler pipeline with typed HIR and explicit-control-flow MIR;
- human-readable and machine-readable diagnostics as first-class compiler outputs.

Some of these belong to later gates and are **not implemented yet**.

## Intended compiler architecture

The reference implementation is intentionally staged rather than built as one monolithic compiler:

```text
source management
    ↓
UTF-8 lexer
    ↓
parser / AST
    ↓
name resolution
    ↓
semantic + type analysis
    ↓
typed HIR
    ↓
ownership + effect analysis        [later gates]
    ↓
MIR with explicit control flow
    ↓
interpreter / native / WASM backends
```

The bootstrap compiler is written in Rust. The front end is intended to remain backend-neutral so native, WebAssembly, and other targets can share one semantic model.

## AI-native by design

"AI-native" does not mean natural-language-only programming. TP source remains a normal programming language.

Instead, TP is being designed so software agents can reason about the same authoritative information the compiler uses:

- structured diagnostics with stable codes and source spans;
- deterministic formatting and project state;
- stable symbol identity within a compilation unit;
- typed intermediate representations;
- explicit effects, permissions, and unsafe boundaries;
- inspectable compiler queries rather than hidden global state;
- source mappings from lowered IR back to user code;
- machine-readable build and dependency metadata.

The intent is to make automated code generation, repair, refactoring, and verification less fragile than text-only workflows.

## Current development state

The authoritative state is [`docs/gates/STATUS.md`](docs/gates/STATUS.md).

As of the current repository state:

| Gate | State | Meaning |
|---|---|---|
| Gate 0 — Governance / IP / repository control | **PASSED** | Project policy, IP model, ADR process, dependency ledger, and repository governance are established. |
| Gate 1 — M1 adversarial stabilization | **ACTIVE** | The executable-core implementation is being aggressively reviewed and repaired. |
| Gate 2 — M1 semantic freeze | **LOCKED** | M1 semantics cannot be declared stable until Gate 1 passes. |
| Gates 3–20 | **LOCKED** | Architecture, ownership, effects, concurrency, package/build, interop, native/WASM, tooling, security, performance, real-app validation, self-hosting, and release work remain gated. |

The active M1 implementation is maintained on `m1-executable-core-clean` and reviewed through PR #1. It must not be treated as a passed milestone until its gate report says PASS and the owner authorizes advancement.

## Roadmap

The 21-gate roadmap is intentionally stricter than a normal feature checklist.

A compressed view:

| Phase | Gates | Goal |
|---|---:|---|
| Governance | 0 | Repository, IP, contribution, ADR, dependency, and gate policy |
| Executable core | 1–3 | Stabilize M1 semantics, freeze behavior, establish compiler architecture |
| Safety core | 4–7 | Ownership, unsafe model, effects/capabilities, structured concurrency |
| Ecosystem / interop | 8–14 | Packages/builds, C, WASM, native backend, JS/TS, Python, first-party tooling |
| Hardening | 15–17 | Security, supply chain, performance, real-application validation |
| Bootstrap / release | 18–20 | Self-hosting preparation, full self-hosting, v0.1 release candidate |

Every gate has explicit exit criteria and evidence requirements. See [`docs/MASTER_GATED_DEVELOPMENT_PLAN.md`](docs/MASTER_GATED_DEVELOPMENT_PLAN.md).

## Development model

TP uses a RED → GREEN workflow for compiler behavior changes:

```text
requirement
  → failing regression test
  → confirm the failure is for the intended reason
  → minimal implementation
  → targeted passing test
  → boundary / negative tests
  → full workspace verification
  → formatter + lint
  → adversarial review
  → gate evidence
```

A green CI run is necessary but never sufficient to pass a major gate.

## Repository map

| Path | Purpose |
|---|---|
| [`AGENTS.md`](AGENTS.md) | Mandatory project rules for agents and contributors |
| [`docs/MASTER_GATED_DEVELOPMENT_PLAN.md`](docs/MASTER_GATED_DEVELOPMENT_PLAN.md) | Gate 0–20 roadmap and pass criteria |
| [`docs/gates/STATUS.md`](docs/gates/STATUS.md) | Authoritative current gate states |
| [`docs/gates/`](docs/gates/) | Gate reports and verification evidence |
| [`docs/decisions/`](docs/decisions/) | Architecture / governance decision records |
| [`docs/superpowers/specs/2026-08-12-universal-language-design.md`](docs/superpowers/specs/2026-08-12-universal-language-design.md) | Foundational language design |
| [`docs/superpowers/plans/`](docs/superpowers/plans/) | Executable implementation/stabilization plans |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Human contribution and review policy |
| [`THIRD_PARTY_LICENSES.md`](THIRD_PARTY_LICENSES.md) | Third-party dependency/license ledger |
| [`LICENSE`](LICENSE) | Proprietary license |

## What is intentionally not claimed yet

TP currently makes **no production-readiness, performance, memory-safety, concurrency-safety, native-code, WebAssembly, package-ecosystem, or cross-platform support claim** beyond what a passed gate explicitly proves.

Those capabilities are roadmap targets, not marketing claims.

Implementation details, runnable examples, and CLI documentation for the active executable-core work live on the active milestone branch until the relevant gates pass.

## Project name

`TP` is the current project codename. The final public language name is intentionally deferred until the implementation and design are mature enough to justify branding it permanently.
