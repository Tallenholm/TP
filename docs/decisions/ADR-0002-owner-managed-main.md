# ADR-0002 — Owner-managed main without mandatory branch protection

**Status:** ACCEPTED  
**Date:** 2026-11-07  
**Decision owner:** Timothy Holm  
**Gate(s):** Gate 0 and all later gates  
**Supersedes:** None  
**Superseded by:** None

## Context

TP is currently a solo-owner proprietary project hosted in a public GitHub repository primarily to take advantage of public-repository CI/resources. Gate 0 previously treated server-side branch protection or an active repository ruleset as a hard blocker even though the controlling roadmap only required a branch policy and owner-resolved governance.

The repository already has layered procedural controls: `AGENTS.md`, the master gated roadmap, gate status/report files, draft milestone PRs, a PR template, CODEOWNERS, and the Gate Policy workflow. Server-side protection would add defense in depth, but it also adds setup/maintenance friction for a solo owner and is not required for the language/compiler correctness guarantees themselves.

## Decision Drivers

- preserve the gated development discipline;
- avoid unnecessary repository-administration friction for a solo owner;
- keep the project moving without weakening compiler/language acceptance criteria;
- retain clear evidence of who may authorize advancement;
- keep branch protection available as optional future hardening.

## Options Considered

### Option A — Mandatory server-side branch protection

Require an active branch rule/ruleset before Gate 0 can pass.

Benefits: stronger technical prevention of accidental direct pushes or bypassed checks.

Drawbacks: adds administrative friction; GitHub integrations may not be able to configure or verify it; not necessary to establish the semantic/testing gates themselves.

### Option B — Owner-managed main with optional branch protection

Keep `main` as the gated/stable branch by policy, require milestone work through the documented gate process, and treat server-side branch protection as optional defense in depth rather than a Gate 0 blocker.

Benefits: lower friction while preserving the repository's governing process and explicit owner authority.

Drawbacks: the owner can technically bypass the process, so enforcement depends partly on owner discipline.

## Decision

Adopt Option B.

Server-side branch protection/rulesets are **not required for Gate 0 to pass**. Their absence is an explicitly accepted governance risk for the current solo-owner phase.

The following remain mandatory:

- `main` represents the last gated/stable state;
- milestone PRs remain draft/unmerged until their gate passes;
- gate reports remain the authoritative evidence for advancement;
- later gates remain locked until their dependencies pass;
- passing ordinary CI alone never authorizes advancement;
- Timothy Holm must explicitly authorize opening the next major gate.

Branch protection may be enabled later as defense in depth without reopening Gate 0 unless the owner decides it has become a required control.

## Consequences

### Positive

- removes a non-semantic administrative blocker;
- preserves the project's gated engineering process;
- makes the policy match the project's current solo-owner operating model.

### Negative / Tradeoffs

- GitHub may still permit direct owner pushes or merges that bypass CI/gate workflows;
- process integrity therefore depends partly on owner discipline until stronger server-side enforcement is adopted.

### Risks

Primary risk: accidental or intentional bypass of the documented gate process.

Mitigation: mandatory repository instructions, status/gate reports, draft PR discipline, PR template, CODEOWNERS, Gate Policy workflow, and explicit owner authorization.

## Semantic / Compatibility Impact

None. This decision changes repository governance only and does not alter TP language semantics, compiler behavior, APIs, ABI, or conformance requirements.

## Security / Safety Impact

No change to safe-language guarantees. This reduces repository-process enforcement only. It does not waive any compiler correctness, safety, security, test, or review requirement in later gates.

## Performance Impact

None.

## Tooling / AI Impact

Agents must treat the repository documents as authoritative even when GitHub technically allows a bypass. The absence of branch protection is not permission to skip a gate.

## Validation Plan

- update the master gated plan to record branch protection as optional defense in depth;
- update Gate 0 report and status;
- retain Gate Policy and other governance controls;
- verify Gate 1 remains the only active engineering gate and Gate 2+ remain locked.

## Reversal / Migration Plan

If multiple contributors are added, accidental bypass becomes a recurring problem, or the owner wants stronger enforcement, enable branch protection/rulesets and update this ADR or supersede it with a stricter policy.

## Approval

**Owner decision:** APPROVED  
**Approval reference:** Timothy Holm explicitly directed the project to drop branch protection as a Gate 0 blocker and continue on 2026-11-07.
