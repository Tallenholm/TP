# M1 Data, Generic, and Pattern Semantics

**Status:** Gate 2 freeze candidate — independent conformance verification in progress  
**Scope:** M1 executable core

## Nominal data types

M1 supports nominal `struct` and `enum` declarations. A nominal type is identified by its declared type name plus generic arguments; unrelated declarations with structurally identical fields are not interchangeable.

Generic parameter arity is exact. Type arguments do not have implicit defaults.

Recursive nominal declarations are permitted at the type level.

## Structs

Struct fields are named. A struct literal:

- must provide every declared field exactly once;
- may list fields in any source order;
- may not provide unknown fields;
- checks each field value against the declared field type.

Field access uses `value.field` and returns the field's instantiated type.

Generic struct arguments may be inferred from field values. If the same generic parameter receives incompatible evidence from multiple fields, the literal is rejected.

If a generic parameter cannot be inferred from the literal itself, the inferred expression may contain an internal unknown type argument and may be refined by a compatible surrounding annotation. M1 does not expose this internal placeholder as user syntax.

Struct field initializer expressions are evaluated left-to-right in source order, and each value is captured at its evaluation point.

## Enums and variant construction

Enum variants may have zero or more payload values.

A payload variant is constructed by calling its variant name:

```tp
Some(42)
```

A zero-payload variant may be used as a bare value and may also be called with an empty argument list.

Constructor payload arity and types are checked.

Generic enum arguments are inferred from payload values when possible. Conflicting inference is rejected. A zero-payload generic variant may rely on an expected/annotated enum type to provide otherwise-unresolved generic arguments.

## Optional type sugar

`T?` is exactly type sugar for a declared arity-1 nominal `Option<T>`. The compiler does not synthesize `Option`.

## Match expressions

A match evaluates its scrutinee once and then evaluates only the first matching reachable arm.

Arm result types must be compatible. A diverging arm is compatible with the live result type.

Supported M1 patterns are:

- wildcard: `_`;
- binding name;
- `i64`, `Bool`, and `String` literals;
- enum variants with recursively nested payload patterns.

Pattern bindings are immutable and scoped to the selected arm.

## Bare pattern names and zero-payload variants

When matching an enum, a bare name that exactly matches a zero-payload variant of that enum denotes the variant. Otherwise a bare name is a catch-all binding.

This resolution is target-enum-aware; M1 does not use capitalization to distinguish variables from variants.

## Exhaustiveness

A match must be exhaustive at compile time.

- `Bool`: both `true` and `false`, or a catch-all.
- Enum: every variant must be fully covered, or a catch-all.
- Other/open domains such as `i64` and `String`: require a wildcard or binding catch-all.

A variant counts as fully covered only when all of its payload subpatterns are irrefutable. Therefore:

```tp
Some(value)
```

covers all `Some` values, but:

```tp
Some(1)
```

does not.

## Unreachable and duplicate arms

M1 rejects:

- an arm after an already exhaustive/catch-all arm;
- duplicate literal arms;
- duplicate fully covered variants;
- exact duplicate recursive/refutable patterns.

Conservative overlap that is not exact and does not fully cover a variant need not be diagnosed as unreachable in M1.

## Conformance evidence

Gate 2 cases under `tests/conformance/structs-enums-generics/` and `tests/conformance/patterns/` freeze this document.
