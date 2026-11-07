# ADR-0001 — Keep TP Public for CI While Retaining Proprietary Licensing

**Status:** ACCEPTED  
**Date:** 2026-08-12  
**Decision owner:** Timothy Holm  
**Gate(s):** Gate 0 and all later CI-dependent gates  
**Supersedes:** None  
**Superseded by:** None

## Context

TP is proprietary, All Rights Reserved software, but the repository is hosted publicly on GitHub. Gate 0 required an explicit owner decision on whether public visibility was accidental or intentional.

The project expects to use GitHub Actions heavily for compiler verification, conformance testing, cross-platform CI, fuzz/property checks, and later release gates. Current GitHub documentation states that standard GitHub-hosted runners are free and unlimited for public repositories, while private-repository use draws from plan-specific included minutes and can incur charges beyond those allowances. Larger runners remain billable even for public repositories.

The repository already contains a proprietary root `LICENSE`; public visibility is not being treated as an open-source grant.

## Decision Drivers

- Maximize access to standard GitHub-hosted CI without consuming private-repository Actions-minute quotas.
- Preserve TP's proprietary licensing and ownership position.
- Keep the gated verification model inexpensive enough to run aggressively.
- Avoid weakening test coverage merely to conserve CI minutes.
- Maintain the option to move private later if confidentiality becomes more important than public-repository CI economics.

## Options Considered

### Option A — Keep the repository public

Benefits:

- standard GitHub-hosted runner use is free and unlimited for public repositories under GitHub's current policy;
- permits aggressive CI across Linux, Windows, macOS, and supported architectures without private-minute pressure;
- preserves straightforward GitHub Actions operation.

Tradeoffs:

- source code is publicly visible;
- public visibility cannot provide source confidentiality;
- GitHub platform functionality may permit viewing/forking within the scope of GitHub's terms even though TP is not open source.

### Option B — Make the repository private

Benefits:

- substantially better source confidentiality;
- reduced casual discovery/access.

Tradeoffs:

- standard GitHub-hosted runner use consumes plan-specific private-repository minutes and may incur charges after included usage;
- could discourage running the broad CI matrix TP's gate policy requires.

## Decision

TP will remain a **public GitHub repository** at this stage.

This is an intentional infrastructure decision so TP can use standard public-repository GitHub-hosted CI extensively. It is **not** a decision to open-source TP.

The root proprietary `LICENSE` remains controlling for rights not otherwise required by GitHub's platform terms or applicable law. Contributors and agents must continue to treat TP as proprietary software.

The project may revisit repository visibility if confidentiality needs later outweigh the CI/resource benefit.

## Consequences

### Positive

- Gate verification can use standard GitHub-hosted runners aggressively without private Actions-minute pressure.
- Cross-platform and repeated regression CI are economically practical.
- The repository's public status is no longer an unresolved governance ambiguity.

### Negative / Tradeoffs

- TP source is publicly readable.
- Proprietary licensing does not make publicly visible source confidential.
- Public visibility may make copying possible in practice even where the license does not authorize reuse.

### Risks

- **Unauthorized reuse:** mitigated by the proprietary root license, copyright notices, provenance records, and retaining development history.
- **Future CI policy changes:** GitHub billing/runner policy can change; Gate 0 governance should be revisited if the economics materially change.
- **Confidentiality needs increase:** repository visibility may be changed later through a new ADR.

## Semantic / Compatibility Impact

None on TP language semantics, ABI, source compatibility, or package behavior.

## Security / Safety Impact

Public source visibility increases information exposure but does not change compiler/runtime safety guarantees. Secrets must never be committed to the repository. CI must rely on GitHub secrets/permissions rather than repository-stored credentials.

## Performance Impact

None on generated TP programs. Public-repository CI availability supports broader performance testing later.

## Tooling / AI Impact

Agents and human contributors may rely on public GitHub Actions as a primary independent verifier. The gate system must still require evidence; free CI is not permission to skip gates.

## Validation Plan

- Confirm repository visibility is `public`.
- Confirm the root proprietary `LICENSE` remains present.
- Record this ADR in the Gate 0 report.
- Periodically re-check GitHub Actions billing policy when materially relevant.

## Reversal / Migration Plan

If TP later requires confidentiality, create a superseding ADR, change the repository to private, review CI quotas/costs, and update Gate 0 governance documents. No language gate needs to reopen solely because of repository visibility unless CI verification capability materially changes.

## Approval

**Owner decision:** APPROVED  
**Approval reference:** Timothy Holm explicitly chose on 2026-08-12 to keep TP public so the project can use free GitHub CI/resources while retaining proprietary licensing.
