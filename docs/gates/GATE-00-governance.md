# Gate 00 — Governance, Repository Control, and IP Report

**Gate:** 00 — Governance, Repository Control, and IP  
**State:** ACTIVE  
**Specification revision:** `docs/MASTER_GATED_DEVELOPMENT_PLAN.md`  
**Implementation PR:** governance changes applied directly to `main` by project owner instruction  
**Report date:** 2026-08-12

> Gate 0 is not yet PASSED. All repository/document/IP/dependency requirements are satisfied except verified server-side protection of `main`.

## 1. Requirements

| ID | Requirement | Status | Evidence |
|---|---|---|---|
| G0-R01 | Root mandatory agent instructions | PASS | `AGENTS.md` |
| G0-R02 | Master gated roadmap | PASS | `docs/MASTER_GATED_DEVELOPMENT_PLAN.md` |
| G0-R03 | Authoritative gate status | PASS | `docs/gates/STATUS.md` |
| G0-R04 | Gate-report template | PASS | `docs/gates/GATE_REPORT_TEMPLATE.md` |
| G0-R05 | Human contribution policy | PASS | `CONTRIBUTING.md` |
| G0-R06 | Pull-request gate checklist | PASS | `.github/pull_request_template.md` |
| G0-R07 | Project-owner review routing | PASS | `.github/CODEOWNERS` |
| G0-R08 | README makes gate policy visible | PASS | `README.md` |
| G0-R09 | Proprietary root license exists | PASS | `LICENSE` |
| G0-R10 | Repository visibility decision resolved | PASS | `docs/decisions/ADR-0001-public-repository-for-ci.md`; owner intentionally keeps TP public for CI/resources while retaining proprietary licensing. |
| G0-R11 | Dependency-license inventory process/file established | PASS | `THIRD_PARTY_LICENSES.md`. |
| G0-R12 | ADR convention/template established | PASS | `docs/decisions/README.md` and `docs/decisions/TEMPLATE.md`. |
| G0-R13 | GitHub branch/ruleset enforcement reviewed/configured where available | FAIL | Repository ruleset API currently reports no repository rulesets. The GitHub integration cannot read/write legacy branch protection administration (403 for protection read). Manual owner configuration/verification is required; see `docs/gates/GATE-00-MANUAL-BRANCH-PROTECTION.md`. |
| G0-R14 | Governance copies on active M1 branch | PASS | Active branch contains the mandatory governance controls and Gate 1 report. |
| G0-R15 | Active dependency licenses reconciled/verified | PASS | Exact third-party set from active M1 `Cargo.lock` reconciled in `THIRD_PARTY_LICENSES.md`; permissive dependency set recorded, including Unicode-3.0 obligations for `unicode-ident`. |
| G0-R16 | Automated gate-policy workflow exists | PASS | `.github/workflows/gate-policy.yml` rejects PRs that lack a PASS gate report, explicit owner authorization, or contain OPEN findings. |

## 2. Verification

### Repository state

- Repository: `Tallenholm/TP`
- Default branch: `main`
- Visibility: `public`
- Owner/admin access: confirmed for `Tallenholm`
- Repository rulesets returned by GitHub API during Gate 0 review: none.
- Legacy branch-protection read through the installed GitHub integration: inaccessible (403 / administration permission limitation), so protection cannot be claimed without manual owner verification.

### Public CI rationale

GitHub's current documentation states that standard GitHub-hosted runners are free and unlimited for public repositories. Larger runners remain separately billable. ADR-0001 records the owner's decision to keep TP public for this reason.

### Dependency audit

The active M1 `Cargo.lock` was enumerated and every third-party registry package was entered in `THIRD_PARTY_LICENSES.md`. The set consists of permissive MIT / Apache-2.0 family dependencies plus `unicode-ident`, whose declared expression additionally requires Unicode-3.0 compliance for Unicode-derived data.

## 3. Review Findings

### Critical

None identified.

### Major

- **G0-M01 — `main` server-side enforcement is not yet proven/configured.** Written policy, CODEOWNERS, PR templates, and Gate Policy CI exist, but without branch protection/ruleset enforcement an administrator could still merge/push around those checks. Gate 0 cannot pass until this is resolved.

### Minor

None currently recorded.

## 4. Architecture / Governance Review

The repository has mutually reinforcing discovery and enforcement surfaces:

- `AGENTS.md` for agent-aware tooling;
- README warning for casual readers;
- `CONTRIBUTING.md` for humans;
- PR template for proposed changes;
- gate status + master roadmap for current/long-term direction;
- Gate 0/1 evidence reports;
- ADR policy/template for durable decisions;
- third-party license ledger;
- CODEOWNERS for owner routing;
- Gate Policy GitHub Actions workflow for automated gate validation.

The controlling rule remains: passing ordinary CI never substitutes for a gate report or owner authorization.

## 5. Security / IP Review

The root proprietary license remains present. Public source visibility is an intentional owner decision recorded in ADR-0001; TP is public for CI/infrastructure economics, not because it is open source.

Secrets must never be committed to the public repository. Future credentials must use GitHub secrets/permissions or other approved secret-management mechanisms.

The current Rust dependency graph has been reconciled. Release/distribution gates must still generate exact third-party notices from their release lockfile, including Unicode-3.0 obligations where applicable.

## 6. Documentation Audit

PASS for current governance documentation. The only unresolved Gate 0 requirement is server-side `main` protection.

## 7. Final Decision

**Decision:** FAIL / ACTIVE  
**Reason:** G0-R13 / G0-M01 remains open. GitHub server-side protection of `main` has not been verified/configured, so the written gates are not yet technically enforced against bypass.

### Owner authorization

**Next gate authorized to open:** Gate 1 remains ACTIVE for stabilization only; Gate 2 and later gates remain LOCKED.  
**Owner:** Timothy Holm  
**Approval record:** Owner adopted the gated roadmap and explicitly approved keeping TP public to maximize free public-repository CI/resources while retaining proprietary licensing.
