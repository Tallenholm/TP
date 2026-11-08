# M1 Binding, Function, and Control-Flow Semantics

**Status:** Gate 2 frozen M1 semantic surface  
**Scope:** M1 executable core

## Bindings and scopes

`let` declares an immutable local binding. `var` declares a mutable local binding.

A binding may infer its type from its initializer or provide an explicit type annotation. An annotation must be compatible with the initializer; M1 performs no implicit numeric coercion.

Names may not be declared twice in the same lexical scope. A nested block may shadow a name from an outer scope.

The initializer of a shadowing declaration is checked/evaluated before the new binding enters scope, so it may refer to the outer binding of the same name.

Assignment is permitted only to mutable local names. Assignment is an expression whose value is the assigned value. Assignment evaluation first evaluates the right-hand side, then updates the target.

## Functions

Functions are declared at module scope. M1 collects function signatures before checking bodies, so calls may refer to functions declared later in the same loaded module graph.

Calls require exact argument count and compatible argument types. M1 has no overloading or implicit argument conversion.

M1 recursion is semantically allowed. The M1 interpreter currently uses the host Rust stack; resource-depth limitations are documented separately.

Function parameters are immutable local bindings.

A function with no explicit result type has result `Unit`.

## Returns and block values

A block's final unterminated expression is its tail value. A terminated final expression contributes `Unit`.

An explicit `return expression` must match the function's declared/default result type and immediately diverges from the current control-flow path. Bare `return` returns `Unit`.

Code after a definitely diverging statement remains type-checked for diagnostics, but it does not restore fallthrough to the enclosing block.

A non-`Unit` function must either produce a compatible tail value or definitely diverge/return on every path.

## `if`

An `if` condition must be `Bool`; TP has no truthiness conversion.

With an `else`, branch result types must be compatible. A diverging branch (`Never`) unifies with the live branch type.

Without `else`, an `if` expression has `Unit` when it can fall through.

Only the selected branch executes.

## `while`

A `while` condition must be `Bool`.

A non-literal loop condition may be false before the first iteration, so an ordinary `while condition { ... }` is potentially fallthrough and contributes `Unit` even if its body contains a `return`.

A literal `while true { ... }` is definitely diverging in M1 because M1 has no `break` or `continue`. It therefore has the internal divergence type `Never` and can satisfy a non-`Unit` function result.

The condition is evaluated before each iteration and the body executes only while it is true.

## Evaluation order

M1 freezes deterministic left-to-right evaluation for:

- ordinary binary operator operands;
- function-call arguments;
- struct field initializers (documented with data semantics).

Assignment evaluates its right-hand side before updating its target.

Logical `&&` and `||` are left-to-right short-circuit operators:

- `false && rhs` does not evaluate `rhs`;
- `true || rhs` does not evaluate `rhs`;
- the other two cases evaluate `rhs`.

## Conformance evidence

Gate 2 cases under `tests/conformance/scopes-mutability/` and `tests/conformance/functions-control-flow/` freeze this document.
