# M1 Lexical and Syntax Semantics

**Status:** Gate 2 frozen semantic surface  
**Scope:** M1 executable core

This document defines the M1 lexical grammar and expression precedence. It records language behavior, not compiler implementation details.

## Source and whitespace

TP source is UTF-8.

Whitespace is not significant syntax. The lexer ignores Unicode characters classified as whitespace. Newlines do **not** terminate statements and do not have indentation meaning.

Braces delimit blocks.

## Comments

M1 supports line comments only:

```tp
// comment until newline
```

Block comments are not part of M1.

## Identifiers and keywords

An identifier starts with `_` or a Unicode alphabetic scalar value.

Subsequent characters may be `_` or Unicode alphanumeric scalar values.

M1 does not normalize Unicode identifier spellings. A combining mark that is not itself alphabetic/alphanumeric is not accepted merely because it could combine visually with a preceding character.

Reserved keywords are:

```text
fn let var if else while return struct enum match true false import as
```

A keyword token cannot be used as an ordinary identifier.

## Literal token forms

Integer and floating-point lexical forms use ASCII decimal digits.

M1 recognizes a floating token only when a decimal point is followed by at least one ASCII digit. Numeric range/runtime semantics are defined in `M1_NUMERIC_SEMANTICS.md`.

Strings use double quotes. M1 supports these escapes:

```text
\\
\"
\n
\r
\t
```

Raw newline/CR characters inside a string literal are lexical errors. Other escape sequences are lexical errors.

## Punctuation and operators

M1 tokenizes:

```text
( ) { } [ ] , . : ; ?
-> =>
+ - * / %
= == ! !=
< <= > >=
&& ||
```

Not every token necessarily has a standalone semantic use in M1; this list defines the current lexical surface.

## Semicolons and statement boundaries

Semicolons are optional where the token grammar already makes the end of a statement unambiguous. Newlines are not separators.

Therefore these are equivalent:

```tp
fn main() {
    print("a")
    print("b")
}
```

```tp
fn main() { print("a") print("b") }
```

For expression statements, a semicolon explicitly terminates/discards the expression value. In a block, the final expression without a semicolon is the block's tail value; a final terminated expression contributes `Unit`.

Bindings and `return` also accept an optional trailing semicolon.

## Expression precedence

From highest to lowest:

| Level | Syntax | Associativity |
|---|---|---|
| Postfix | function call `()`, field access `.`, struct literal continuation | left/chained |
| Unary | `-`, `!` | right-recursive |
| Multiplicative | `*`, `/`, `%` | left |
| Additive | `+`, `-` | left |
| Comparison | `<`, `<=`, `>`, `>=` | left syntactically |
| Equality | `==`, `!=` | left |
| Logical AND | `&&` | left |
| Logical OR | `||` | left |
| Assignment | `=` | right |

Parentheses override precedence.

Comparison operators do not receive special mathematical chaining semantics. A chain is parsed by normal left associativity and must independently satisfy type checking.

Assignment targets in M1 are local names; assignment is right-associative, so `a = b = 7` means `a = (b = 7)`.

## Conformance evidence

The Gate 2 corpus under `tests/conformance/lexical/` and `tests/conformance/precedence/` freezes this document through observable compile/runtime behavior rather than AST shape.
