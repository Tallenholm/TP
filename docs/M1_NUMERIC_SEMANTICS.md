# M1 Numeric Semantics

**Status:** Gate 2 frozen M1 semantic surface  
**Scope:** M1 executable core

## Numeric types and coercion

M1 has two numeric primitive types:

- `i64`
- `f64`

Arithmetic and ordered comparison require matching numeric operand types. M1 performs no implicit `i64` ↔ `f64` conversion.

Equality also requires matching types.

## `i64` literals

Integer source literals are decimal ASCII digits with optional unary negation.

The representable range is exactly signed 64-bit:

- minimum: `-9223372036854775808`;
- maximum: `9223372036854775807`.

A source integer outside that range is a compile-time `TP-E0100` error. The exact minimum value is recognized as a valid negative literal even though its positive magnitude is one greater than `i64::MAX`.

## Checked integer execution

M1 integer execution is checked.

The following trap with `TP-E0500` rather than wrapping:

- addition overflow;
- subtraction overflow;
- multiplication overflow;
- division overflow such as `i64::MIN / -1`;
- remainder overflow where the host checked operation cannot represent the result;
- unary negation overflow, including negating `i64::MIN`.

Division by zero and remainder by zero also trap with `TP-E0500`.

No build mode is permitted to silently switch M1 integer arithmetic from checked to wrapping semantics.

## `f64` literals

Float source literals use decimal ASCII digits with a decimal point and at least one digit following the point.

A source float must parse to a finite IEEE-754 binary64 value. A literal that parses directly to Infinity or NaN is rejected at compile time with `TP-E0100`.

M1 does not currently provide exponent notation or source spellings for Infinity/NaN.

## `f64` execution

Once execution begins, ordinary `f64` arithmetic follows IEEE-754 binary64 behavior for finite operands:

- addition, subtraction, multiplication, nonzero division, and nonzero remainder may produce finite values, Infinity, or NaN;
- ordered comparisons follow IEEE behavior;
- NaN is unequal to itself;
- `+0.0 == -0.0`.

M1 deliberately traps division or remainder by either `+0.0` or `-0.0` with `TP-E0500`, rather than producing ordinary IEEE Infinity/NaN for those two operations.

## Equality

Numeric equality is defined only between matching numeric types:

```tp
1 == 1       // valid
1.0 == 1.0   // valid
1 == 1.0     // TP-E0300
```

There is no implicit widening for equality.

## Optional numeric types

`T?` is syntax sugar for the program's declared nominal `Option<T>`. It is valid only when an arity-1 type named `Option` is declared.

The compiler does not synthesize a hidden built-in `Option`.

## Conformance evidence

Gate 2 cases under `tests/conformance/literals-numerics/` freeze these rules through compile/runtime behavior.
