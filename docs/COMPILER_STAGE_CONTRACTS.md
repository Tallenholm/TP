# TP Compiler Stage Contracts — Gate 3 (initial freeze)

**Revision:** 2026-11-08  
**Status:** Gate 3 architecture contract; frozen M1 semantics are immutable.  
**Owner:** Compiler architecture Gate 3.

## Stage ownership and failure boundaries

| Stage | Owns | Input → output | What it must not do |
|---|---|---|---|
| SourceManager | Source file identity, text, spans | file/path + UTF-8 → SourceFile/SourceId/Span | Infer types or silently rebind source identity |
| Lexer | Tokenization, lexical diagnostics | SourceFile → tokens/diagnostics | Mutate source text, build modules |
| Parser | Syntax and recoverable parse diagnostics | tokens + source → AST/diagnostics | Decide expression types, resolve imported symbols |
| ModuleLoader / name resolution | Canonical module graph and imports | file graph + AST → resolved module/diagnostics | Duplicate source modules under equivalent aliases |
| TypeChecker | Binding, expression types, semantic decisions | resolved AST → SemanticModel + diagnostics | Leak unresolved type information downstream |
| HirLowerer | Deterministic lowering from checked semantics | AST + SemanticModel → typed HIR | Re-infer values/types or bypass rejected source code |
| HIR verifier (planned) | HIR structural/types/bindings contracts | HIR → verified HIR or typed error | Accept mismatched symbols or panic on malformed IR |
| MirLowerer | CFG/local lowering, left-to-right value capture | verified HIR → MIR | Change evaluation order or implicit runtime behavior |
| MIR verifier (planned) | CFG/local indexing and use integrity | MIR → verified MIR or typed error | Silently normalize malformed IR or execute it |
| Interpreter | M1 run-time behavior and defined traps | verified MIR → value/stdout or runtime trap | Read outside verified local/block tables or re-typecheck |

As of 2026-11-08, `Compiler::lower_source`/`lower_path` call `HirVerifier`, and `Compiler::lower_mir_source`/`lower_mir_path` call `MirVerifier`. Both check internal IR before returning it to callers. The public interpreter also runs `MirVerifier` at `run_main` time even when constructed from externally fabricated MIR; invalid modules fail with code `TP-E0600`, as proved by [CI 42504027480](https://github.com/Tallenholm/TP/actions/runs/42504027480). This does **not** prove every type/definite-assignment invariant.

## Established M1 semantic invariant

The Gate 2 conformance oracle (144 cases) controls all observable behavior. In particular, equalities/print/function-symbol restrictions, exact left-to-right evaluation and value capture, short-circuit logic, module canonicalization, entrypoint diagnostics, checked integer traps, and runtime output must not change.

## HIR scoped binding and checked-type contract — G3-T03

- A function's parameter symbols and all let/pattern binding SymbolIds must be unique within that function (shadowing still receives a fresh SymbolId).
- Lexical scopes are tracked across nested blocks and distinct match arms. No local use/assignment may refer to a not-yet-declared or out-of-scope SymbolId.
- HIR literal types must match their primitive kind; local use type metadata must match the resolved binding. `if`/`while` conditions require `Bool` or `Never` (unreachable conditions are allowed by frozen M1).
- Assignment to an immutable symbol is invalid IR. A source span with `start > end` is invalid IR; there is not yet a complete check against the source's byte length.
- **Critical provenance rule:** `HirStmt::Let.ty` must originate from the type checker's `SemanticModel`, **not** `value.ty`. An explicit `let x: Maybe<i64> = None;` is valid even though the uncontextualized generic `None` initializer carries a less-specific type. MIR local metadata must use the authoritative HIR `ty`.
- Scope and primitive-shape verification is not a second source type checker; richer HIR type rules remain Gate 3 work.
- Typed `HirVerificationError/HirVerificationKind` map to external diagnostic `TP-E0601` on the public Compiler API when invalid lowered HIR is encountered.

**Evidence:** intended RED CI [42503096400](https://github.com/Tallenholm/TP/actions/runs/42503096400), annotation-provenance regression exposed by [42503348927](https://github.com/Tallenholm/TP/actions/runs/42503348927), and GREEN CI [42503647514](https://github.com/Tallenholm/TP/actions/runs/42503647514): 9/9 HIR tests and frozen 144-case M1 corpus.

## MIR-specific structural invariants — initial G3-T02 contract

The interpreter currently indexes `function.blocks[BlockId.0]` and `Frame.locals[LocalId.0]` directly. Therefore **existence alone is insufficient**: block IDs and local IDs must correspond to their positions in their respective arrays.

For every `MirFunction`:

1. `blocks[index].id == BlockId(index)` for every block, with no duplicate, hole or out-of-order IDs.
2. `locals[index].id == LocalId(index)` for every local, with no duplicate, hole or out-of-order IDs.
3. `entry` must refer to an existing block.
4. Every Goto/Branch/Match successor must refer to an existing block.
5. Every function parameter `LocalId` must exist and no parameter may be duplicated.
6. Every statement assignment target and local operand referenced by Rvalues/Terminators/Match pattern bindings must exist.
7. Verification is pure, deterministic and reports typed errors; it must not panic on malformed IR or try to fix input.
8. Integration: both `Compiler::lower_mir_source` and `Compiler::lower_mir_path` must reject invalid MIR as compile failures before it is given to the interpreter.

A `Terminator::Unreachable` may be a legitimate lowering artifact in unreachable generated blocks; it is not rejected solely because the variant exists. Stronger reachable-path and type invariants will be evaluated in subsequent Gate 3 tasks, not guessed here.

## Error boundary

The verifier returns an internal `MirVerificationError { function, kind }`, where `kind` is a structured enum of invariant violations. At the external compiler API boundary, internal failure maps deterministically to a coded compiler diagnostic, rather than to an interpreter runtime trap or a panic. Error messages may contain useful function/block IDs, but consumers should rely on structured error variants and stable external diagnostic codes.

## Unresolved architecture decisions (not silently frozen)

- `G3-D01`: cross-module SymbolId identity and session/version stability.
- `G3-D02`: complete internal error mapping to source spans for synthesized IR.
- `G3-D03`: replay/caching guarantees of atomic SourceId allocation.
- `G3-D04`: query caching and invalidation.

Those decisions are deliberately not resolved by the initial MIR verifier contract.

## Test evidence

The Gate 3 plan requires an actual RED → GREEN test cycle before coding the MIR verifier. `crates/tp-compiler/tests/mir_verifier.rs` first introduces compile-failing tests for the missing public verifier API; after that failure is observed on CI, the minimal verified implementation may be added. Subsequent negative tests ensure broken CFG/indexing is rejected.
