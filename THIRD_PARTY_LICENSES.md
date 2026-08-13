# TP Third-Party Dependency and License Ledger

TP is proprietary software. Third-party dependencies remain subject to their own licenses; those licenses do not change TP's proprietary licensing except where their terms legally require otherwise.

This file is a controlled ledger. Any new runtime, compiler, build, test, CI, packaging, or documentation dependency must be reviewed and added here before its gate can pass.

## Review rules

For every dependency record:

- package/project name;
- exact version or version policy;
- source/homepage/repository;
- license identifier and license text/source;
- dependency purpose;
- whether it ships in TP binaries/distributions or is development-only;
- whether it is statically linked, dynamically linked, executed as a tool, or otherwise consumed;
- attribution/notice/source-disclosure obligations;
- compatibility notes with TP's proprietary distribution model;
- reviewer and review date.

Do not copy source code from a third-party project into TP merely because it is publicly visible. Provenance and license permission must be established first.

## Gate 0 audit scope

The active M1 Rust workspace has one external direct dependency: `clap` with the `derive` feature. The complete transitive registry set was reconciled against `Cargo.lock` on branch `m1-executable-core-clean` on 2026-08-12.

Cargo lockfile checksums remain the build-integrity reference for the exact packages used by the workspace.

## Current locked Rust dependency ledger

| Dependency | Locked version | Relationship / purpose | Declared license | Gate 0 disposition |
|---|---:|---|---|---|
| `anstream` | 1.0.0 | transitive CLI terminal output | MIT OR Apache-2.0 | ACCEPTED |
| `anstyle` | 1.0.14 | transitive CLI styling | MIT OR Apache-2.0 | ACCEPTED |
| `anstyle-parse` | 1.0.0 | transitive ANSI parsing | MIT OR Apache-2.0 | ACCEPTED |
| `anstyle-query` | 1.1.5 | transitive terminal capability queries | MIT OR Apache-2.0 | ACCEPTED |
| `anstyle-wincon` | 3.0.11 | transitive Windows terminal support | MIT OR Apache-2.0 | ACCEPTED |
| `clap` | 4.6.6 | direct CLI argument parsing | MIT OR Apache-2.0 | ACCEPTED |
| `clap_builder` | 4.6.6 | transitive clap builder implementation | MIT OR Apache-2.0 | ACCEPTED |
| `clap_derive` | 4.6.4 | transitive procedural macros for clap derive | MIT OR Apache-2.0 | ACCEPTED |
| `clap_lex` | 1.1.0 | transitive CLI lexer | MIT OR Apache-2.0 | ACCEPTED |
| `colorchoice` | 1.0.5 | transitive terminal color policy | MIT OR Apache-2.0 | ACCEPTED |
| `heck` | 0.5.0 | transitive derive-case conversion | MIT OR Apache-2.0 | ACCEPTED |
| `is_terminal_polyfill` | 1.70.2 | transitive terminal detection compatibility | MIT OR Apache-2.0 | ACCEPTED |
| `once_cell_polyfill` | 1.70.2 | transitive compatibility helper | MIT OR Apache-2.0 | ACCEPTED |
| `proc-macro2` | 1.0.107 | transitive procedural macro token support | MIT OR Apache-2.0 | ACCEPTED |
| `quote` | 1.0.47 | transitive procedural macro code generation | MIT OR Apache-2.0 | ACCEPTED |
| `strsim` | 0.11.1 | transitive CLI suggestion/string similarity | MIT | ACCEPTED |
| `syn` | 3.0.3 | transitive Rust syntax parsing for derive macros | MIT OR Apache-2.0 | ACCEPTED |
| `unicode-ident` | 1.0.24 | transitive Unicode identifier tables | (MIT OR Apache-2.0) AND Unicode-3.0 | ACCEPTED WITH UNICODE NOTICE |
| `utf8parse` | 0.2.2 | transitive UTF-8 parsing | MIT OR Apache-2.0 | ACCEPTED |
| `windows-link` | 0.2.1 | transitive Windows linking support | MIT OR Apache-2.0 | ACCEPTED |
| `windows-sys` | 0.61.2 | transitive Windows API bindings | MIT OR Apache-2.0 | ACCEPTED |

## Verification basis

The exact package/version set above is taken from the active M1 `Cargo.lock`. License expressions were checked against packaged crate metadata/readmes/source listings on docs.rs and the corresponding upstream crate workspaces where applicable.

The dependency family used by TP is permissively licensed. For dual-licensed `MIT OR Apache-2.0` dependencies, TP may comply under an available license option; distribution tooling must still preserve all required copyright/license notices. `unicode-ident` additionally declares `Unicode-3.0` for Unicode-derived data and therefore requires the Unicode license obligations to be preserved when distributed.

This ledger is governance evidence, not a substitute for legal advice. If TP distribution changes materially, the notice/attribution package must be reviewed again for that distribution form.

## Distribution policy

Before any TP binary/toolchain is distributed outside development/testing:

1. generate or assemble a third-party notices bundle from the exact release lockfile;
2. include applicable MIT/Apache license texts/notices required by the selected compliance path;
3. include the Unicode-3.0 license/notice for `unicode-ident` data;
4. ensure the notices correspond to the exact release dependency graph;
5. record the resulting notice artifact in the release gate evidence.

## Change control

Any dependency addition, removal, or version update reopens the dependency-license review for the affected gate. A dependency PR must update this ledger or provide automated evidence that feeds an equivalent controlled ledger before its gate may pass.

**Gate 0 audit status:** COMPLETE for the dependency set locked on `m1-executable-core-clean` as of 2026-08-12.
