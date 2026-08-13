# TP Third-Party Dependency and License Ledger

TP is proprietary software. Third-party dependencies remain subject to their own licenses.

Any new runtime, compiler, build, test, CI, packaging, or documentation dependency must be reviewed and added here before its gate can pass.

For every dependency record: package/project, exact version/version policy, authoritative source, license identifier/text source, purpose, distribution status, linkage/consumption mode, notice/source obligations, proprietary-distribution compatibility, reviewer, and date.

Do not copy third-party source into TP merely because it is publicly visible; provenance and permission must be established first.

## Current M1 dependency audit

The active M1 Rust workspace uses dependencies recorded in `Cargo.lock`. Before Gate 0 passes, this ledger must be reconciled against that lockfile and authoritative package/project license metadata.

| Dependency | Version | Purpose | License status |
|---|---:|---|---|
| `clap` | 4.6.6 in current lockfile | CLI argument parsing | PENDING authoritative verification |
| transitive dependencies | See `Cargo.lock` | CLI/toolchain support | PENDING reconciliation/verification |

## Required Gate 0 action

1. Reconcile this ledger against active `Cargo.lock`.
2. Verify each license from authoritative metadata/source.
3. Record attribution/notice obligations.
4. Confirm compatibility with TP's proprietary distribution model.
5. Record evidence in `docs/gates/GATE-00-governance.md`.
