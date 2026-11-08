# M1 Executable and Runtime Semantics

**Status:** Gate 2 frozen M1 semantic surface  
**Scope:** M1 executable core

## Check mode versus run mode

Static checking does not require an executable entry point. A source tree containing only library-style declarations may pass `tp check`.

Execution requires a function named `main`.

## `main` contract

M1 executable `main`:

- must exist in run mode;
- must take zero parameters;
- may return any normal checked M1 value type, including `Unit`, `Bool`, `i64`, `f64`, `String`, structs, or enums.

A missing run-mode entry point or invalid `main` parameter shape is a compile failure in diagnostic family `TP-E0400`, before interpreter execution.

## Runtime traps

A program that passes static checking may still encounter defined runtime traps. M1 uses diagnostic code `TP-E0500` for interpreter/runtime traps.

Frozen examples include checked integer overflow and numeric division/remainder by zero as documented in `M1_NUMERIC_SEMANTICS.md`.

A compile-time-exhaustive match should not reach a non-exhaustive runtime match trap from safe checked M1 source. Such a trap is defensive runtime behavior, not a valid semantic path for a correctly checked program.

## `print`

`print` is a reserved M1 bootstrap builtin.

It:

- takes exactly one argument;
- accepts only `Unit`, `Bool`, `i64`, `f64`, or `String`;
- returns `Unit`;
- appends one textual line to program output.

Primitive formatting is:

- `Unit` → `()`;
- `Bool` → `true` or `false`;
- `i64` → signed decimal;
- finite `f64` → ordinary human-readable decimal representation;
- `String` → the string contents with no added quotes.

Structs, enums, and function values are not printable in M1. This intentionally avoids freezing bootstrap/interpreter-internal formatting as a language contract. A richer display/formatting protocol belongs to later design.

## Recursion resource limitation

Recursion is semantically valid M1 behavior, but the bootstrap MIR interpreter executes recursive calls using the host Rust call stack. M1 does not freeze a language-level maximum recursion depth or a TP-specific stack-exhaustion trap in Gate 2.

Extreme recursion may therefore exhaust the host process stack. This is a documented bootstrap runtime limitation, not a production containment guarantee.

## Conformance evidence

Gate 2 cases under `tests/conformance/entrypoint-runtime/` freeze executable entry-point, primitive print, and representative runtime-trap behavior. Numeric trap details are additionally frozen under `tests/conformance/literals-numerics/`.
