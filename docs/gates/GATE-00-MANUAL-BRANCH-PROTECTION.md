# Gate 00 Manual Action — Protect `main`

Gate 0 cannot be marked PASSED until GitHub itself enforces the gate checks on `main`.

The installed GitHub integration used during Gate 0 can manage repository content and PRs but does not expose the Administration write permission required to create branch protection/rulesets. GitHub's repository rulesets endpoint returned no configured rulesets during review.

## Required owner action in GitHub

Configure a protection rule/ruleset targeting `main` with the following minimum policy:

1. **Enforcement:** Active.
2. **Target:** `main` / default branch.
3. **Require a pull request before merge/push to `main`.**
4. **Require status checks to pass before merge.**
5. Require the TP **Gate Policy** check once it has appeared as a status check in the repository.
6. Require the normal TP **CI** check used by the active compiler branch.
7. Prefer requiring the branch to be up to date before merge when practical.
8. **Block force pushes.**
9. **Block branch deletion.**
10. Do not create an ordinary bypass path that lets routine milestone work skip the gate.

Because TP is currently a solo-owner project, do **not** create a review-approval requirement that makes it impossible for the only owner to merge. The hard requirements are PR-based changes plus required Gate Policy/CI checks. Human review requirements can be strengthened later when additional trusted reviewers exist.

## Verification after configuration

Gate 0 verification must demonstrate all of the following:

- GitHub reports an active protection rule/ruleset covering `main`.
- A PR whose gate report is `FAIL / ACTIVE` cannot be merged because Gate Policy is red.
- A branch with failing normal CI cannot be merged.
- Direct/force update behavior matches the configured policy.
- The final settings/evidence are recorded in `docs/gates/GATE-00-governance.md`.

## Gate 0 status

Until that evidence exists, **G0-R13 remains FAIL and Gate 0 remains ACTIVE.**
