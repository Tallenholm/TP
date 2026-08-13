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

## Current Rust dependencies

The active M1 Rust workspace currently uses dependencies recorded in `Cargo.lock`. Before Gate 0 passes, this table must be reconciled against the active branch and the exact dependency/license metadata verified from authoritative package/project sources.

| Dependency | Version | Purpose | Distribution | License | Verification status |
|---|---:|---|---|---|---|
| `clap` | 4.6.6 in current M1 lockfile | CLI argument parsing | Bootstrap compiler/toolchain dependency | Verify authoritative package license | PENDING |
| transitive dependencies | See `Cargo.lock` | Transitive support for CLI/toolchain | Varies | Verify individually/through dependency audit | PENDING |

## Required Gate 0 action

Before Gate 0 can be marked PASSED:

1. reconcile this ledger against `Cargo.lock` on the active branch;
2. verify each dependency's license from authoritative metadata/source;
3. record any notice/attribution requirements;
4. confirm compatibility with the proprietary TP distribution model;
5. record the audit evidence in `docs/gates/GATE-00-governance.md`.
