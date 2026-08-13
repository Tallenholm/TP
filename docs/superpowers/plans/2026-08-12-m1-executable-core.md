# TP M1 Executable Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first working TP compiler core: source input → lexer → parser/AST → name/type checking → HIR/MIR lowering → interpreter execution, exposed through a `tp` CLI with useful diagnostics.

**Architecture:** Use a Rust workspace with a small compiler library and CLI binary. Keep each stage independently testable and data-oriented. M1 uses an interpreter over MIR as the executable backend so language semantics can stabilize before committing to LLVM/native codegen; native codegen remains a later milestone.

**Tech Stack:** Rust 2024 edition, stable Rust toolchain, Cargo workspace, `clap` for CLI, `thiserror` for internal error plumbing only, `insta` for diagnostic/parser snapshots, standard library collections. No parser generator in M1; use a hand-written lexer and Pratt/recursive-descent parser so grammar behavior remains explicit and AI-editable.

## Global Constraints

- Safe TP code must not invoke undefined behavior.
- Simple programs must stay simple.
- Performance must be explainable.
- Concurrency must be safe by construction; concurrency is not implemented in M1.
- Effects and permissions are explicit in the final language; effect checking is deferred to M2.
- Compiler diagnostics are a public interface and must have stable diagnostic codes.
- Interop is core product scope; C/WASM interop is deferred to M3.
- Tooling is part of the language; M1 ships `tp run` and `tp check` first.
- AI is a first-class programmer; parser recovery and deterministic diagnostics are required from the start.
- TP is proprietary; root `LICENSE` governs all source and generated artifacts.
- M1 must not introduce explicit lifetime syntax, mandatory tracing GC, package-registry behavior, macros, async, or native LLVM codegen.

## File Structure

```text
Cargo.toml
rust-toolchain.toml
crates/
  tp-compiler/
    Cargo.toml
    src/
      lib.rs              # public compiler pipeline API
      source.rs           # SourceFile, Span, source mapping
      diagnostic.rs       # diagnostic model/codes/rendering
      token.rs            # TokenKind, Token
      lexer.rs            # UTF-8 lexer
      ast.rs              # parsed syntax tree
      parser.rs           # recursive-descent + Pratt parser
      symbol.rs           # symbol table and scoped name resolution
      types.rs            # type representation and inference helpers
      typecheck.rs        # semantic/type checking
      hir.rs              # typed high-level IR
      lower_hir.rs        # AST -> HIR
      mir.rs              # explicit control-flow IR
      lower_mir.rs        # HIR -> MIR
      value.rs            # runtime values
      interpreter.rs      # MIR interpreter
      pipeline.rs         # parse/check/run orchestration
    tests/
      lexer.rs
      parser.rs
      diagnostics.rs
      typecheck.rs
      runtime.rs
      modules.rs
      m1_programs.rs
      fixtures/
        hello.tp
        arithmetic.tp
        control_flow.tp
        structs.tp
        enums.tp
        match.tp
        options.tp
        modules_main.tp
        util.tp
  tp-cli/
    Cargo.toml
    src/main.rs            # `tp check` and `tp run`
examples/
  hello.tp
  fibonacci.tp
README.md
```

---

### Task 1: Rust workspace and compiler pipeline shell

**Files:**
- Create: `Cargo.toml`
- Create: `rust-toolchain.toml`
- Create: `crates/tp-compiler/Cargo.toml`
- Create: `crates/tp-compiler/src/lib.rs`
- Create: `crates/tp-compiler/src/pipeline.rs`
- Create: `crates/tp-compiler/tests/pipeline.rs`
- Create: `crates/tp-cli/Cargo.toml`
- Create: `crates/tp-cli/src/main.rs`

**Interfaces:**
- Produces: `pub struct Compiler`, `pub fn Compiler::new() -> Self`, `pub fn Compiler::check_source(&self, name: &str, source: &str) -> CompileReport`, `pub struct CompileReport { pub diagnostics: Vec<Diagnostic> }`.

- [ ] **Step 1: Write the failing smoke test**

```rust
use tp_compiler::Compiler;

#[test]
fn empty_source_can_be_checked() {
    let compiler = Compiler::new();
    let report = compiler.check_source("empty.tp", "");
    assert!(report.diagnostics.is_empty());
}
```

- [ ] **Step 2: Run the test and verify RED**

Run: `cargo test -p tp-compiler --test pipeline`
Expected: FAIL because `tp_compiler::Compiler` does not exist.

- [ ] **Step 3: Add the minimal public pipeline shell**

```rust
// crates/tp-compiler/src/lib.rs
mod pipeline;

pub use pipeline::{CompileReport, Compiler};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
}
```

```rust
// crates/tp-compiler/src/pipeline.rs
use crate::Diagnostic;

#[derive(Default)]
pub struct Compiler;

impl Compiler {
    pub fn new() -> Self { Self }

    pub fn check_source(&self, _name: &str, _source: &str) -> CompileReport {
        CompileReport { diagnostics: Vec::new() }
    }
}

#[derive(Debug, Default)]
pub struct CompileReport {
    pub diagnostics: Vec<Diagnostic>,
}
```

- [ ] **Step 4: Run the test and verify GREEN**

Run: `cargo test -p tp-compiler --test pipeline`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml rust-toolchain.toml crates/tp-compiler crates/tp-cli
git commit -m "feat: scaffold TP compiler workspace"
```

---

### Task 2: Source model and stable diagnostics

**Files:**
- Create: `crates/tp-compiler/src/source.rs`
- Create: `crates/tp-compiler/src/diagnostic.rs`
- Modify: `crates/tp-compiler/src/lib.rs`
- Test: `crates/tp-compiler/tests/diagnostics.rs`

**Interfaces:**
- Produces: `SourceFile`, `Span`, `LineCol`, `Diagnostic`, `Severity`, `render_diagnostic`.
- `Span` uses byte offsets and always refers to a specific `SourceId`.

- [ ] **Step 1: Write failing span mapping test**

```rust
use tp_compiler::{SourceFile, Span};

#[test]
fn byte_span_maps_to_line_and_column() {
    let src = SourceFile::new("main.tp", "let x = 1\nprint(x)\n");
    let loc = src.line_col(Span::new(src.id(), 10, 15).start).unwrap();
    assert_eq!((loc.line, loc.column), (2, 1));
}
```

- [ ] **Step 2: Run and verify RED**

Run: `cargo test -p tp-compiler --test diagnostics`
Expected: FAIL because source/diagnostic types do not exist.

- [ ] **Step 3: Implement source IDs, byte spans, line mapping, and diagnostics**

Required diagnostic model:

```rust
pub enum Severity { Error, Warning, Note }

pub struct Diagnostic {
    pub code: &'static str,
    pub severity: Severity,
    pub message: String,
    pub primary: Option<Span>,
    pub labels: Vec<Label>,
    pub help: Option<String>,
}
```

Use codes beginning with `TP-E` for errors. Reserve:
- `TP-E0001` invalid token
- `TP-E0100` parse error
- `TP-E0200` unknown name
- `TP-E0300` type mismatch
- `TP-E0400` invalid control flow
- `TP-E0500` runtime trap

- [ ] **Step 4: Add snapshot rendering test and verify GREEN**

Run: `cargo test -p tp-compiler --test diagnostics`
Expected: PASS with deterministic `file:line:column`, code, message, source line, caret span, and optional help.

- [ ] **Step 5: Commit**

```bash
git add crates/tp-compiler/src/source.rs crates/tp-compiler/src/diagnostic.rs crates/tp-compiler/src/lib.rs crates/tp-compiler/tests/diagnostics.rs
git commit -m "feat: add source spans and structured diagnostics"
```

---

### Task 3: UTF-8 lexer

**Files:**
- Create: `crates/tp-compiler/src/token.rs`
- Create: `crates/tp-compiler/src/lexer.rs`
- Modify: `crates/tp-compiler/src/lib.rs`
- Test: `crates/tp-compiler/tests/lexer.rs`

**Interfaces:**
- Produces: `Token { kind: TokenKind, span: Span }`, `Lexer::new(&SourceFile)`, `Lexer::lex() -> LexResult`.
- Token kinds must include identifiers, integer/float/string literals, punctuation, operators, EOF, and keywords `fn`, `let`, `var`, `if`, `else`, `while`, `return`, `struct`, `enum`, `match`, `true`, `false`, `import`, `as`.

- [ ] **Step 1: Write failing lexer test**

```rust
#[test]
fn lexes_function_and_expression() {
    let kinds = lex_kinds("fn add(a: i64, b: i64) -> i64 { a + b }");
    assert_eq!(kinds, vec![
        Fn, Identifier, LParen, Identifier, Colon, Identifier, Comma,
        Identifier, Colon, Identifier, RParen, Arrow, Identifier,
        LBrace, Identifier, Plus, Identifier, RBrace, Eof,
    ]);
}
```

- [ ] **Step 2: Run and verify RED**

Run: `cargo test -p tp-compiler --test lexer`
Expected: FAIL because lexer is missing.

- [ ] **Step 3: Implement minimal lexer**

Requirements:
- UTF-8-safe cursor advancement.
- ASCII keywords/operators for M1.
- Unicode allowed in identifier continuation/start using Rust `char::is_alphabetic`/`is_alphanumeric` plus `_`.
- `//` line comments.
- Escapes `\\`, `\"`, `\n`, `\r`, `\t` in strings.
- Invalid characters emit `TP-E0001` and continue lexing.

- [ ] **Step 4: Add invalid-character recovery test and verify GREEN**

Run: `cargo test -p tp-compiler --test lexer`
Expected: PASS; invalid token creates one diagnostic and tokens after it remain available.

- [ ] **Step 5: Commit**

```bash
git add crates/tp-compiler/src/token.rs crates/tp-compiler/src/lexer.rs crates/tp-compiler/src/lib.rs crates/tp-compiler/tests/lexer.rs
git commit -m "feat: implement TP lexer"
```

---

### Task 4: AST and parser for functions, expressions, and statements

**Files:**
- Create: `crates/tp-compiler/src/ast.rs`
- Create: `crates/tp-compiler/src/parser.rs`
- Modify: `crates/tp-compiler/src/lib.rs`
- Test: `crates/tp-compiler/tests/parser.rs`

**Interfaces:**
- Produces: `Module`, `Item`, `FnDecl`, `Stmt`, `Expr`, `TypeRef`, `Parser::parse_module()`.
- Expression parsing uses Pratt precedence.

- [ ] **Step 1: Write failing precedence test**

```rust
#[test]
fn multiplication_binds_tighter_than_addition() {
    let module = parse_ok("fn main() { let x = 1 + 2 * 3 }");
    assert_expr_shape(&module, "(+ 1 (* 2 3))");
}
```

- [ ] **Step 2: Run and verify RED**

Run: `cargo test -p tp-compiler --test parser multiplication_binds_tighter_than_addition`
Expected: FAIL because parser/AST is missing.

- [ ] **Step 3: Implement parser for M1 core syntax**

Required grammar in this task:
- function declarations with typed parameters and optional return type;
- blocks;
- `let` and `var` declarations;
- expression statements;
- `return`;
- `if/else`;
- `while`;
- literals;
- unary `-` and `!`;
- binary `+ - * / % == != < <= > >= && ||`;
- assignment to mutable names;
- calls;
- parenthesized expressions.

- [ ] **Step 4: Add parser recovery test**

```rust
#[test]
fn parser_recovers_at_statement_boundary() {
    let result = parse("fn main() { let x = ; let y = 2; }");
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result.module.is_some());
}
```

Run: `cargo test -p tp-compiler --test parser`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/tp-compiler/src/ast.rs crates/tp-compiler/src/parser.rs crates/tp-compiler/src/lib.rs crates/tp-compiler/tests/parser.rs
git commit -m "feat: parse TP functions and expressions"
```

---

### Task 5: Structs, enums, match, and modules syntax

**Files:**
- Modify: `crates/tp-compiler/src/ast.rs`
- Modify: `crates/tp-compiler/src/parser.rs`
- Test: `crates/tp-compiler/tests/parser.rs`
- Test: `crates/tp-compiler/tests/modules.rs`

**Interfaces:**
- Adds `StructDecl`, `EnumDecl`, `VariantDecl`, `Pattern`, `MatchArm`, `ImportDecl`.

- [ ] **Step 1: Write failing enum/match parse test**

```rust
#[test]
fn parses_enum_and_exhaustive_match_shape() {
    let src = r#"
        enum Option<T> { Some(T), None }
        fn value(x: Option<i64>) -> i64 {
            match x { Some(v) => v, None => 0 }
        }
    "#;
    let module = parse_ok(src);
    assert_eq!(module.items.len(), 2);
}
```

- [ ] **Step 2: Run and verify RED**

Run: `cargo test -p tp-compiler --test parser parses_enum_and_exhaustive_match_shape`
Expected: FAIL.

- [ ] **Step 3: Implement syntax only**

Support:
- `struct User { id: i64, name: String }`
- struct construction `User { id: 1, name: "A" }`
- field access `user.name`
- `enum Option<T> { Some(T), None }`
- variant construction `Some(3)` and `None`
- `match expr { pattern => expr, ... }`
- patterns: wildcard `_`, literal, identifier binding, enum variant with nested bindings.
- `import util` resolving later to sibling `util.tp`.

- [ ] **Step 4: Run parser/module tests and verify GREEN**

Run: `cargo test -p tp-compiler --test parser --test modules`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/tp-compiler/src/ast.rs crates/tp-compiler/src/parser.rs crates/tp-compiler/tests/parser.rs crates/tp-compiler/tests/modules.rs
git commit -m "feat: parse TP data types match and imports"
```

---

### Task 6: Symbol resolution and primitive type system

**Files:**
- Create: `crates/tp-compiler/src/symbol.rs`
- Create: `crates/tp-compiler/src/types.rs`
- Create: `crates/tp-compiler/src/typecheck.rs`
- Modify: `crates/tp-compiler/src/lib.rs`
- Test: `crates/tp-compiler/tests/typecheck.rs`

**Interfaces:**
- Produces `Type`, `TypeId`, `SymbolId`, `TypeChecker::check_module`.
- Primitive types: `Bool`, `i64`, `f64`, `String`, `Unit`.
- User types: structs, enums, generic type parameters.

- [ ] **Step 1: Write failing unknown-name test**

```rust
#[test]
fn reports_unknown_name() {
    let report = check("fn main() { print(missing) }");
    assert_has_code(&report, "TP-E0200");
}
```

- [ ] **Step 2: Run and verify RED**

Run: `cargo test -p tp-compiler --test typecheck reports_unknown_name`
Expected: FAIL.

- [ ] **Step 3: Implement scoped resolution and primitive typing**

Rules:
- function names visible module-wide;
- parameters/local bindings lexical-scoped;
- duplicate binding in same scope is an error;
- `let` immutable, `var` mutable;
- assignment requires mutable target;
- arithmetic operands must be compatible numeric types;
- conditions require `Bool`;
- function call arity/types checked;
- declared return type checked;
- inferred local type comes from initializer.

- [ ] **Step 4: Add mismatch and mutability tests**

```rust
#[test]
fn assignment_to_let_is_rejected() {
    let report = check("fn main() { let x = 1; x = 2 }");
    assert_has_message(&report, "immutable");
}

#[test]
fn boolean_condition_is_required() {
    let report = check("fn main() { if 1 { } }");
    assert_has_code(&report, "TP-E0300");
}
```

Run: `cargo test -p tp-compiler --test typecheck`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/tp-compiler/src/symbol.rs crates/tp-compiler/src/types.rs crates/tp-compiler/src/typecheck.rs crates/tp-compiler/src/lib.rs crates/tp-compiler/tests/typecheck.rs
git commit -m "feat: add TP name resolution and type checking"
```

---

### Task 7: Type-check structs, enums, match, Option and Result

**Files:**
- Modify: `crates/tp-compiler/src/types.rs`
- Modify: `crates/tp-compiler/src/typecheck.rs`
- Test: `crates/tp-compiler/tests/typecheck.rs`

**Interfaces:**
- Adds nominal struct/enum typing, generic substitution, and match exhaustiveness.
- `Option<T>` and `Result<T, E>` are prelude enum definitions represented by the same enum machinery, not special runtime magic.

- [ ] **Step 1: Write failing exhaustiveness test**

```rust
#[test]
fn non_exhaustive_option_match_is_rejected() {
    let report = check(r#"
        fn unwrap_or_zero(x: Option<i64>) -> i64 {
            match x { Some(v) => v }
        }
    "#);
    assert_has_message(&report, "non-exhaustive");
}
```

- [ ] **Step 2: Run and verify RED**

Run: `cargo test -p tp-compiler --test typecheck non_exhaustive_option_match_is_rejected`
Expected: FAIL.

- [ ] **Step 3: Implement nominal data-type checking**

Required behavior:
- struct field names/types validated;
- enum variant payloads validated;
- generic arguments substituted for enum/struct fields;
- match pattern types checked;
- duplicate/unreachable variant arm warning may be deferred, but missing enum variants is an error;
- wildcard arm counts as exhaustive;
- all match expression arms unify to one result type.

- [ ] **Step 4: Run full typechecker suite and verify GREEN**

Run: `cargo test -p tp-compiler --test typecheck`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/tp-compiler/src/types.rs crates/tp-compiler/src/typecheck.rs crates/tp-compiler/tests/typecheck.rs
git commit -m "feat: type check TP structs enums and match"
```

---

### Task 8: Typed HIR

**Files:**
- Create: `crates/tp-compiler/src/hir.rs`
- Create: `crates/tp-compiler/src/lower_hir.rs`
- Modify: `crates/tp-compiler/src/typecheck.rs`
- Test: `crates/tp-compiler/tests/hir.rs`

**Interfaces:**
- Produces `HirModule`, `HirFunction`, `HirStmt`, `HirExpr { ty: TypeId, kind: HirExprKind, span: Span }`.
- All identifiers become `SymbolId`; all expression types are explicit.

- [ ] **Step 1: Write failing HIR lowering test**

```rust
#[test]
fn hir_contains_resolved_symbol_and_type() {
    let hir = check_and_lower("fn main() { let x = 1; x + 2 }").unwrap();
    let expr = find_last_expr(&hir, "main");
    assert_eq!(expr.type_name(), "i64");
    assert!(expr.contains_resolved_symbol());
}
```

- [ ] **Step 2: Run and verify RED**

Run: `cargo test -p tp-compiler --test hir`
Expected: FAIL.

- [ ] **Step 3: Lower AST + semantic tables to deterministic typed HIR**

No name strings may remain as the semantic identity of local variables in HIR; retain source names only for diagnostics/debug printing.

- [ ] **Step 4: Snapshot HIR and verify GREEN**

Run: `cargo test -p tp-compiler --test hir`
Expected: PASS and stable HIR snapshots.

- [ ] **Step 5: Commit**

```bash
git add crates/tp-compiler/src/hir.rs crates/tp-compiler/src/lower_hir.rs crates/tp-compiler/src/typecheck.rs crates/tp-compiler/tests/hir.rs
git commit -m "feat: lower checked TP code to typed HIR"
```

---

### Task 9: MIR control-flow lowering

**Files:**
- Create: `crates/tp-compiler/src/mir.rs`
- Create: `crates/tp-compiler/src/lower_mir.rs`
- Test: `crates/tp-compiler/tests/mir.rs`

**Interfaces:**
- Produces CFG-oriented `MirFunction { blocks: Vec<BasicBlock>, locals: Vec<LocalDecl> }`.
- Terminators: `Goto`, `Branch`, `Return`, `MatchEnum`, `Trap`.
- Statements: `Assign`, `Call`, `ConstructStruct`, `ConstructEnum`, `ReadField`.

- [ ] **Step 1: Write failing `if` lowering test**

```rust
#[test]
fn if_expression_creates_branch_and_join_blocks() {
    let mir = lower("fn main() -> i64 { if true { 1 } else { 2 } }");
    let f = mir.function("main").unwrap();
    assert!(f.has_terminator("Branch"));
    assert!(f.blocks.len() >= 4);
}
```

- [ ] **Step 2: Run and verify RED**

Run: `cargo test -p tp-compiler --test mir`
Expected: FAIL.

- [ ] **Step 3: Implement explicit control-flow lowering**

Lower:
- if/else to branch/join blocks;
- while to header/body/exit blocks;
- return to `Return` terminator;
- match to enum/literal dispatch with one arm block per branch;
- expression temporaries to explicit locals;
- function calls to explicit call statements.

- [ ] **Step 4: Snapshot MIR and verify GREEN**

Run: `cargo test -p tp-compiler --test mir`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/tp-compiler/src/mir.rs crates/tp-compiler/src/lower_mir.rs crates/tp-compiler/tests/mir.rs
git commit -m "feat: lower TP HIR to MIR control flow"
```

---

### Task 10: MIR interpreter

**Files:**
- Create: `crates/tp-compiler/src/value.rs`
- Create: `crates/tp-compiler/src/interpreter.rs`
- Test: `crates/tp-compiler/tests/runtime.rs`

**Interfaces:**
- Produces `Interpreter::run_main(&MirModule) -> Result<Value, RuntimeError>`.
- Runtime values: Unit, Bool, I64, F64, String, Struct, Enum.

- [ ] **Step 1: Write failing arithmetic runtime test**

```rust
#[test]
fn executes_arithmetic_program() {
    let value = run("fn main() -> i64 { let x = 6; x * 7 }").unwrap();
    assert_eq!(value.as_i64(), Some(42));
}
```

- [ ] **Step 2: Run and verify RED**

Run: `cargo test -p tp-compiler --test runtime executes_arithmetic_program`
Expected: FAIL.

- [ ] **Step 3: Implement interpreter**

Requirements:
- stack frame per function call;
- local slots indexed by MIR local IDs;
- deterministic left-to-right expression/call evaluation;
- checked division by zero → `TP-E0500` runtime trap;
- match dispatch for enum variants and literals;
- struct construction/field reads;
- builtin `print(value)` writes through an injectable output sink for tests.

- [ ] **Step 4: Add control-flow/data-type runtime tests and verify GREEN**

Run: `cargo test -p tp-compiler --test runtime`
Expected: PASS for arithmetic, functions, loops, structs, enums, match, Option.

- [ ] **Step 5: Commit**

```bash
git add crates/tp-compiler/src/value.rs crates/tp-compiler/src/interpreter.rs crates/tp-compiler/tests/runtime.rs
git commit -m "feat: execute TP MIR with interpreter"
```

---

### Task 11: Multi-file module loading

**Files:**
- Modify: `crates/tp-compiler/src/pipeline.rs`
- Modify: `crates/tp-compiler/src/symbol.rs`
- Test: `crates/tp-compiler/tests/modules.rs`
- Create fixtures: `crates/tp-compiler/tests/fixtures/modules_main.tp`, `crates/tp-compiler/tests/fixtures/util.tp`

**Interfaces:**
- Produces `Compiler::check_path(&Path) -> CompileReport` and `Compiler::run_path(&Path) -> RunReport`.
- `import util` resolves `util.tp` relative to the importing file's directory in M1.

- [ ] **Step 1: Write failing two-file import test**

```rust
#[test]
fn imported_function_can_be_called() {
    let result = run_fixture("modules_main.tp").unwrap();
    assert_eq!(result.as_i64(), Some(42));
}
```

Fixture:

```tp
// util.tp
fn twice(x: i64) -> i64 { x * 2 }
```

```tp
// modules_main.tp
import util
fn main() -> i64 { util.twice(21) }
```

- [ ] **Step 2: Run and verify RED**

Run: `cargo test -p tp-compiler --test modules`
Expected: FAIL.

- [ ] **Step 3: Implement deterministic relative module loading**

Rules:
- no network/package lookup in M1;
- detect import cycles and emit a diagnostic;
- canonicalize paths for module identity;
- module public/private visibility is deferred; imported top-level declarations are visible through module qualification.

- [ ] **Step 4: Run module tests and verify GREEN**

Run: `cargo test -p tp-compiler --test modules`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/tp-compiler/src/pipeline.rs crates/tp-compiler/src/symbol.rs crates/tp-compiler/tests/modules.rs crates/tp-compiler/tests/fixtures
git commit -m "feat: add TP multi-file module loading"
```

---

### Task 12: `tp check` and `tp run` CLI

**Files:**
- Modify: `crates/tp-cli/src/main.rs`
- Modify: `crates/tp-cli/Cargo.toml`
- Test: `crates/tp-cli/tests/cli.rs`

**Interfaces:**
- CLI:
  - `tp check <file.tp>`: exit 0 if no errors, 1 on compiler errors.
  - `tp run <file.tp>`: check then execute `main`; exit 1 on compile/runtime error.
  - `--diagnostic-format human|json` supported by both commands.

- [ ] **Step 1: Write failing CLI test**

```rust
#[test]
fn check_returns_zero_for_valid_program() {
    let output = run_tp(["check", fixture("hello.tp")]);
    assert!(output.status.success());
}
```

- [ ] **Step 2: Run and verify RED**

Run: `cargo test -p tp-cli --test cli`
Expected: FAIL.

- [ ] **Step 3: Implement CLI**

Use `clap` derive API. Human diagnostics go to stderr. Program output goes to stdout. JSON diagnostics emit one JSON object per diagnostic with at least:

```json
{"schema":1,"code":"TP-E0300","severity":"error","message":"...","file":"main.tp","start":0,"end":1}
```

- [ ] **Step 4: Run CLI tests and verify GREEN**

Run: `cargo test -p tp-cli --test cli`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/tp-cli
git commit -m "feat: add tp check and tp run commands"
```

---

### Task 13: M1 conformance programs and documentation

**Files:**
- Create: `crates/tp-compiler/tests/m1_programs.rs`
- Create fixtures under: `crates/tp-compiler/tests/fixtures/`
- Create: `examples/hello.tp`
- Create: `examples/fibonacci.tp`
- Create: `README.md`

**Interfaces:**
- No new compiler API. This task proves the M1 surface end-to-end.

- [ ] **Step 1: Add end-to-end conformance tests for every M1 feature**

Required fixtures/tests:
- hello/print;
- arithmetic/comparisons;
- mutable locals;
- if/else;
- while;
- functions and returns;
- strings;
- structs/fields;
- enums;
- match;
- `Option<T>`;
- `Result<T, E>` declaration/use;
- imports;
- representative syntax/type/runtime failures.

- [ ] **Step 2: Run the suite before README work**

Run: `cargo test --workspace`
Expected: PASS with zero ignored M1 tests.

- [ ] **Step 3: Write README with exact current capabilities**

README must include:
- proprietary notice;
- `cargo build --workspace` bootstrap instructions;
- `cargo run -p tp-cli -- check examples/hello.tp`;
- `cargo run -p tp-cli -- run examples/hello.tp`;
- syntax examples that are covered by tests;
- explicit `Not implemented yet` section for ownership, effects, async, package registry, C/WASM interop, LLVM/native output, LSP, formatter.

- [ ] **Step 4: Run verification again**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all three commands exit 0.

- [ ] **Step 5: Commit**

```bash
git add README.md examples crates/tp-compiler/tests
git commit -m "docs: complete TP M1 executable core"
```

---

## Plan Self-Review

### Spec coverage

M1 requirements mapped:
- functions/primitives/locals/arithmetic/comparisons/conditionals/loops → Tasks 3, 4, 6, 9, 10;
- structs/enums/match → Tasks 5, 7, 9, 10;
- strings → Tasks 3, 6, 10;
- Option/Result → Task 7 and conformance Task 13;
- modules → Task 11;
- useful diagnostics → Tasks 2, 3, 4, 6, 12;
- HIR/MIR architecture → Tasks 8–9;
- executable behavior → Task 10;
- CLI/tooling entrypoint → Task 12;
- AI-readable diagnostics → Tasks 2 and 12.

Deferred exactly as intended by the design: ownership/borrow checking, effects/capabilities, concurrency/async, package manager, C/WASM interop, formatter/LSP/docs generator, LLVM/native backend, self-hosting.

### Placeholder scan

No `TBD`, `TODO`, or implementation-placeholder steps are permitted in this plan. Deferred features are explicitly scoped to later milestones rather than left undefined inside M1.

### Type/interface consistency

Pipeline progression is fixed as:

```text
SourceFile
  -> Vec<Token>
  -> AST Module
  -> resolved/type-checked semantic tables
  -> HirModule
  -> MirModule
  -> Interpreter::run_main
```

`Compiler::check_source`, `Compiler::check_path`, and `Compiler::run_path` remain the stable orchestration layer used by the CLI and tests.
