# ADR-0003 — Gate 3 Identity and Query Lifetime Policy

**Date:** 2026-11-08  
**Status:** ACCEPTED for Gate 3's current noncached compiler query boundary.  
**Gate:** 3 — Compiler Architecture Foundation (ACTIVE).  
**Scope:** Source IDs, symbol IDs, canonical modules, query snapshots. This ADR does not authorize Gate 4 or Gate 3 merging.

## Context

TP's frozen M1 type checker records expression types, resolved bindings, local uses and global symbol kinds in a `SemanticModel` indexed by `Span`. `SourceFile::new` allocates numeric `SourceId` using a process-wide atomic counter. `Scopes::new` allocates numeric `SymbolId` within a checked function/scope collection. Module loading canonicalizes imported sibling paths and deduplicates aliases/diamonds.

Raw numeric identities cannot safely serve as persisted, portable or global identifiers. Tooling needs one coherent context containing these ephemeral IDs and a separately defined module identity that is meaningful between compilations.

## Decisions

1. **Canonical module identity:** `ModuleId` is the filesystem-canonical `PathBuf` of a module. Equal canonical paths mean the same module within the same filesystem/mount state. `ModuleGraph` records canonical nodes and import edges; nodes/edges sort deterministically. This is **not** a cross-machine, rename-stable or content-addressed package identity. No package registry feature is introduced.
2. **Source identity:** `SourceId` remains a process-assigned ephemeral value, meaningful only with the exact `SourceManager` in a compilation snapshot. Nothing in the API promises replay-stable numeric IDs. A span is validated against its own manager, including UTF-8 byte boundaries, before HIR output is considered verified. The long-term counter exhaustion/persistent-ID policy is outside this scoped decision.
3. **Symbol identity:** numeric `SymbolId` is local to one checked function's symbol allocation; scope shadowing receives distinct IDs, but identical numbers may legitimately occur in different functions/compilations. A cross-function symbol referent must carry its containing function/snapshot identity; a bare numeric ID is not globally unique or persistable.
4. **Query lifetime:** `Compiler::analyze_source` and `analyze_path` build **fresh, independent, noncached** `CompilationSnapshot` values. The same snapshot keeps SourceManager, checked AST/SemanticModel, verified HIR/MIR and optional ModuleGraph mutually aligned. There is intentionally no incremental cache, invalidation mechanism, persisted query key or cross-compilation symbol-resolution API. A future incremental design must establish deterministic invalidation tests and a reviewed ADR before introducing caching.
5. **Semantic authority:** type information, global kinds, binding identities and function signatures are sourced from the checked `SemanticModel`. HIR verification compares derived data against that authority; it is not another source-language type checker.

## Evidence

- [CI 42504573620](https://github.com/Tallenholm/TP/actions/runs/42504573620): initial four noncached source/path snapshot tests, including stale-source independence.
- [CI 42505002345](https://github.com/Tallenholm/TP/actions/runs/42505002345): six query tests; canonical diamond module and same-source multi-alias identity.
- [CI 42505287570](https://github.com/Tallenholm/TP/actions/runs/42505287570): SourceManager span/UTF-8 validation.
- [CI 42506018205](https://github.com/Tallenholm/TP/actions/runs/42506018205): 23 HIR verifier tests for scoped binding and semantic identity.
- [CI 42506366279](https://github.com/Tallenholm/TP/actions/runs/42506366279): 20 MIR/23 HIR/six query tests plus frozen 144-case M1 conformance and full Rust format/Clippy/locked workspace/smoke suite.

## Consequences and boundaries

The current design keeps compiler query correctness inspectable without prematurely committing to distributed/persisted identity semantics. Deterministic module ordering and fresh snapshots are sufficient for early tooling. It does **not** promise incremental performance, stable numeric SourceIds across compilations, global SymbolId uniqueness, type-complete IR validation, or trusted arbitrary external MIR beyond currently enforced verifier invariants.

This decision may be reopened when real editor/incremental compiler work exposes an unmet requirement, but it may not be silently replaced by implementation convenience. Gate 3 remains ACTIVE and the Gate 3 report's final decision remains non-PASS.
