# M1 Known Limitations

**Status:** documented Gate 1 limitations; subject to later gates and Gate 2 semantic freeze  
**Scope:** M1 executable core

This file records known behavior that is intentionally **not** being represented as a completed capability. None of the items below is an unresolved Gate 1 Critical/Major correctness finding.

## Diagnostic presentation

### Tab-stop alignment

Human diagnostic columns are counted in Unicode scalar values, and caret width is character-based rather than byte-based. Source tabs are not expanded to terminal tab stops when calculating visual caret alignment.

Result: a diagnostic on a line containing `\t` may have a caret that is logically column-correct but visually shifted depending on terminal tab width.

This is a presentation limitation. JSON diagnostics retain byte/line/column ranges independently of terminal rendering.

### Line endings

`SourceFile` indexes lines on LF (`\n`). Normal LF and CRLF input are supported; CR is stripped from CRLF line text. A file using standalone CR-only line separators is not modeled as multiple source lines.

CR-only source files are not a supported M1 line-ending mode.

## Interpreter recursion

The M1 MIR interpreter implements function recursion using the host Rust call stack. It does not yet impose a TP-specific recursion depth limit or convert host stack exhaustion into a TP runtime diagnostic.

Very deep or intentionally unbounded recursion can therefore exhaust the host process stack. This is not a memory-unsafe operation in the Rust implementation, but it is not a production-grade runtime containment guarantee.

A deliberate recursion/runtime-resource policy belongs to later runtime/security hardening unless Gate 2 chooses to freeze a stronger requirement earlier.

## Modules

M1 modules are intentionally sibling-file modules, not a package system. Directory-qualified imports, package roots, registries, visibility/export rules, versioned dependencies, and target-specific dependencies are outside M1.

See [`M1_MODULE_SEMANTICS.md`](M1_MODULE_SEMANTICS.md).

## Floating point

M1 accepts only finite `f64` literals, but ordinary runtime arithmetic may produce IEEE Infinity/NaN. Division and remainder by ±0.0 trap.

See [`M1_NUMERIC_SEMANTICS.md`](M1_NUMERIC_SEMANTICS.md).

## Not production-ready

M1 is a semantic/executable core. Ownership/borrowing, explicit unsafe boundaries, effects/capabilities, structured concurrency, native/WASM backends, package integrity, dedicated fuzzing/security hardening, and performance validation belong to later locked gates.
