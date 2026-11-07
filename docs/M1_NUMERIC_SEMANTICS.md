# M1 Numeric Semantics

**Status:** Gate 1 stabilized behavior; subject to Gate 2 semantic freeze  
**Scope:** M1 executable core only

## No implicit numeric coercion

M1 has two numeric primitive types:

- `i64`
- `f64`

Binary arithmetic/comparison requires matching numeric operand types. TP does not implicitly widen `i64` to `f64` or narrow `f64` to `i64`.

## `i64`

Integer literals must fit the signed 64-bit range. The exact minimum value `-9223372036854775808` is representable.

Runtime integer arithmetic is checked:

- addition, subtraction, multiplication, division, remainder, and unary negation trap on signed overflow;
- division by zero traps;
- remainder by zero traps.

These runtime failures use diagnostic family `TP-E0500`.

## `f64`

Source float literals must parse to a **finite** IEEE-754 binary64 value. A decimal literal that parses to Infinity or is otherwise non-finite is rejected at compile time.

Once execution begins, ordinary floating-point arithmetic follows host Rust/IEEE-754 binary64 behavior:

- addition, subtraction, multiplication, nonzero division, and nonzero remainder may produce finite values, Infinity, or NaN;
- comparisons use IEEE behavior;
- NaN is not equal to itself;
- Infinity/NaN produced by arithmetic are values, not automatic traps.

TP M1 deliberately adds one runtime rule beyond ordinary IEEE division/remainder behavior:

- division by `+0.0` or `-0.0` traps;
- remainder by `+0.0` or `-0.0` traps.

This is an M1 semantic choice, not a performance claim. Gate 2 may freeze or deliberately revise it only through the gated semantic process.

## Optional numeric types

The parser supports `T?` optional-type sugar. In M1 it is only valid when the program declares a nominal arity-1 `Option<T>` type. The compiler does not silently invent `Option`.

Example:

```tp
enum Option<T> {
    Some(T),
    None
}

fn maybe_number(value: i64?) {
    // `i64?` is the declared `Option<i64>` nominal type.
}
```
