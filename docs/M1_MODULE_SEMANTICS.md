# M1 Module Semantics

**Status:** Gate 1 stabilized behavior; subject to Gate 2 semantic freeze  
**Scope:** M1 executable core only

This document records the module behavior implemented and tested during Gate 1. It is intentionally narrower than the eventual TP package/module system.

## Import syntax

M1 imports one sibling TP source file by module identifier:

```tp
import util
import util as helpers
```

An import named `util` resolves to `util.tp` in the importing file's directory.

M1 import syntax does **not** accept filesystem path separators or directory-qualified import paths. A future package/module gate may expand this model.

## Canonical module identity

A source file has one module identity per compilation, determined by its canonical filesystem path.

Consequences:

- importing the same file repeatedly loads and emits it once;
- importing the same file under multiple aliases does not duplicate its declarations;
- diamond imports reuse one shared module instance;
- on platforms that support symlinks, aliases/symlinks resolving to the same canonical file share one module identity;
- canonical identity is used in addition to cycle detection, not only for cycle detection.

The compiler's internal namespace chosen for a canonical module is an implementation detail and is **not** a user-facing stability promise.

## Aliases

An import alias is local to the importing module.

```tp
import util as helpers

fn main() -> i64 {
    helpers.twice(21)
}
```

Rules:

- repeating the exact same module/alias import is deduplicated;
- one canonical module may be imported through more than one alias;
- two different modules may not claim the same alias in one importing module;
- a conflicting alias emits `TP-E0200` at the import boundary.

## Qualified imported declarations

Imported declarations are referenced through the import name or alias.

Functions:

```tp
helpers.twice(21)
```

Nominal types:

```tp
fn read(user: model.User) -> i64 {
    user.id
}
```

Struct literals:

```tp
let user: model.User = model.User { id: 21 };
```

Generic enum types and constructors:

```tp
let maybe: model.Maybe<i64> = model.Some(21);
```

Qualified enum patterns:

```tp
match maybe {
    model.Some(value) => value,
    model.None => 0
}
```

## Cycles

Import cycles are compile errors. The diagnostic reports the canonical path chain involved in the cycle.

## Determinism

M1 traverses imports in source order. Given the same source tree and filesystem resolution, repeated lowering produces the same declaration/function ordering.

Canonical module reuse prevents diamond/repeated imports from introducing order-dependent duplicate declarations.

## Filesystem and platform behavior

M1 deliberately delegates filename case sensitivity to the host filesystem.

- TP does not perform its own Unicode or ASCII case folding for module filenames.
- `import util` asks the host filesystem to resolve `util.tp`.
- Path joining uses the host path implementation.
- Canonicalization resolves supported symlinks before module identity comparison.

This policy is documented rather than pretending Linux, macOS, and Windows filesystems have identical case behavior.

## Current M1 limitations

The following are outside the M1 module surface:

- package names and package roots;
- directory-qualified import syntax;
- absolute imports;
- package registries;
- visibility/export declarations;
- re-export syntax;
- versioned dependencies;
- target-specific dependency selection.

Because M1 imports only sibling module identifiers, directly expressing two same-named modules from different directories is not part of the current grammar. That case belongs to the later package/module design rather than being silently guessed in M1.

## Gate 1 evidence

The Gate 1 module regression matrix covers:

- import parsing and aliases;
- valid sibling function calls;
- cycle rejection;
- repeated identical import deduplication;
- multiple aliases to one canonical module;
- duplicate alias rejection;
- diamond dependency deduplication;
- qualified imported nominal types;
- qualified struct literals;
- qualified enum constructors and patterns;
- deterministic lowering order;
- canonical symlink identity on Unix.

The authoritative RED -> GREEN run IDs and commits are recorded in `docs/gates/GATE-01-M1-adversarial-stabilization.md`.
