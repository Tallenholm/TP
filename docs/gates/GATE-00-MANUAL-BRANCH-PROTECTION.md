# Optional Hardening — Protect `main`

Server-side branch protection is **not required for Gate 0** under `docs/decisions/ADR-0002-owner-managed-main.md`.

TP currently uses an owner-managed `main` policy. The project owner explicitly accepted the risk that GitHub may technically permit direct pushes or merges that bypass automated checks during the solo-owner phase.

This file is retained as a future hardening recipe.

## Optional GitHub hardening

If/when stronger server-side enforcement is desired, configure a protection rule/ruleset targeting `main` with:

1. **Enforcement:** Active.
2. **Target:** `main` / default branch.
3. Require a pull request before merge/push to `main`.
4. Require status checks to pass before merge.
5. Require the TP **Gate Policy** check once available.
6. Require the normal TP **CI** check used by the active compiler branch.
7. Prefer requiring the branch to be up to date before merge when practical.
8. Block force pushes.
9. Block branch deletion.
10. Avoid routine bypass paths.

Because TP is currently a solo-owner project, do not create a review-approval requirement that makes it impossible for the only owner to merge.

## Current status

This is optional defense in depth. Its absence does **not** block Gate 0, Gate 1, or later work unless a future owner decision/ADR makes branch protection mandatory again.
