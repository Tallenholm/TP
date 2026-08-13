# Gate 00 — Governance, Repository Control, and IP Report

**Gate:** 00 — Governance, Repository Control, and IP  
**State:** ACTIVE  
**Specification revision:** `docs/MASTER_GATED_DEVELOPMENT_PLAN.md`  
**Report date:** 2026-08-12

> Gate 0 is not yet PASSED. All repository/document/IP/dependency requirements are satisfied except verified server-side protection of `main`.

## Requirements

| ID | Requirement | Status | Evidence |
|---|---|---|---|
| G0-R01 | Root mandatory agent instructions | PASS | `AGENTS.md` |
| G0-R02 | Master gated roadmap | PASS | `docs/MASTER_GATED_DEVELOPMENT_PLAN.md` |
| G0-R03 | Authoritative gate status | PASS | `docs/gates/STATUS.md` |
| G0-R04 | Gate-report template | PASS | `docs/gates/GATE_REPORT_TEMPLATE.md` |
| G0-R05 | Human contribution policy | PASS | `CONTRIBUTING.md` |
| G0-R06 | Pull-request gate checklist | PASS | `.github/pull_request_template.md` |
| G0-R07 | Project-owner review routing | PASS | `.github/CODEOWNERS` on `main` |
| G0-R08 | README makes gate policy visible | PASS | `README.md` |
| G0-R09 | Proprietary root license exists | PASS | `LICENSE` |
| G0-R10 | Repository visibility decision resolved | PASS | `docs/decisions/ADR-0001-public-repository-for-ci.md`; owner intentionally keeps TP public for CI/resources while retaining proprietary licensing. |
| G0-R11 | Dependency-license inventory process/file established | PASS | `THIRD_PARTY_LICENSES.md` |
| G0-R12 | ADR convention/template established | PASS | `docs/decisions/README.md`, `docs/decisions/TEMPLATE.md` |
| G0-R13 | GitHub branch/ruleset enforcement reviewed/configured | FAIL | GitHub reports no repository rulesets; installed integration cannot administer legacy branch protection. Manual owner configuration/verification on `main` remains required. |
| G0-R14 | Governance copies on active M1 branch | PASS | Active branch carries controlling docs. |
| G0-R15 | Active dependency licenses reconciled/verified | PASS | Exact current `Cargo.lock` dependency set reconciled in `THIRD_PARTY_LICENSES.md`, including Unicode-3.0 obligations for `unicode-ident`. |
| G0-R16 | Automated gate-policy workflow exists | PASS | `.github/workflows/gate-policy.yml` on `main`. |

## Review findings

### Critical

None identified.

### Major

- **G0-M01:** GitHub server-side branch/ruleset protection for `main` is not yet proven/configured. Written policy and Gate Policy CI exist, but without server enforcement an administrator could bypass them.

## Final decision

**Decision:** FAIL / ACTIVE  
**Reason:** G0-R13 / G0-M01 remains open; all other Gate 0 requirements are satisfied.

### Owner authorization

**Next gate authorized to open:** Gate 1 stabilization only; Gate 2+ remain locked.  
**Owner:** Timothy Holm  
**Approval record:** Owner approved keeping TP public to maximize free GitHub CI/resources while retaining proprietary licensing.
