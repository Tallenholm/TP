# M1 Value, Equality, String, and Unicode Semantics

**Status:** Gate 2 freeze candidate — independent conformance verification in progress  
**Scope:** M1 executable core

## Primitive values

M1's primitive value types are:

- `Unit`
- `Bool`
- `i64`
- `f64`
- `String`

M1 has no implicit null value and no broad truthiness conversion.

## M1 equality contract

For M1, `==` and `!=` are defined only for operands of the same supported comparable type:

- `Unit`;
- `Bool`;
- `i64`;
- `f64`;
- `String`.

Struct values, enum/generic nominal values, and function values are **not** equality-comparable in M1. Attempting to compare them is `TP-E0300`.

This restriction is deliberate. The bootstrap interpreter internally has Rust representations capable of broader equality, but that representation is not the TP language contract. Rich nominal/derived equality is deferred to a later explicit design.

## Strings

A `String` is the decoded sequence of Unicode scalar values represented by its UTF-8 source literal after M1 escape processing.

Strings support equality/inequality. M1 does not define `+` as string concatenation.

String equality is exact. TP M1 does not apply Unicode normalization, locale-sensitive comparison, case folding, or grapheme equivalence.

Therefore a precomposed spelling such as `"é"` and the decomposed spelling `"e◌́"` are distinct string values.

## Unicode identifiers

Identifier lexical rules are defined in `M1_LEXICAL_AND_SYNTAX_SEMANTICS.md`.

M1 does not normalize identifier spellings. Unicode alphabetic/alphanumeric scalars are accepted according to the lexical rule; visually similar or canonically equivalent spellings are not silently merged.

## Optional values

M1 has no compiler-invented optional runtime value. `T?` is type sugar for a user-declared arity-1 `Option<T>` enum.

## Conformance evidence

Gate 2 cases under `tests/conformance/equality-strings-unicode/` and `tests/conformance/literals-numerics/` freeze this document.
