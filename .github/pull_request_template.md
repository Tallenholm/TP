# TP Pull Request Gate Checklist

> Read `AGENTS.md`, `docs/MASTER_GATED_DEVELOPMENT_PLAN.md`, and `docs/gates/STATUS.md` before completing this PR.

## Gate declaration

**Gate:**  
**Gate state when PR opened:**  
**Requirement IDs / acceptance criteria:**

If this PR belongs to a LOCKED gate, stop and do not proceed until the controlling status/plan is explicitly changed by the project owner.

## Scope

Describe exactly what changes and what is intentionally out of scope.

## TDD evidence

- [ ] Failing test was written first.
- [ ] RED was observed for the intended reason.
- [ ] Minimal correct implementation was added.
- [ ] Targeted test is GREEN.
- [ ] Negative/boundary/regression tests were added.

**RED evidence:**  
**GREEN evidence:**

## Verification

Record exact commands and results.

- [ ] Subsystem tests pass.
- [ ] Full test suite passes.
- [ ] Formatter check passes.
- [ ] Linter passes with warnings denied.
- [ ] Required conformance/fuzz/property/stress tests pass.
- [ ] Fresh CI reproduces verification.

```text
<command + result>
```

## Architecture / semantics

- [ ] No unintended semantic change is introduced.
- [ ] Any intended semantic change was approved in the controlling spec/ADR before implementation.
- [ ] No duplicate semantic source-of-truth was introduced.
- [ ] Required HIR/MIR/compiler invariants remain valid.

## Security / dependencies

- [ ] No new unsafe/security-sensitive behavior, OR it is documented below.
- [ ] No new dependency, OR dependency justification/license is documented below.
- [ ] No generated build artifacts are included.

**Security/unsafe notes:**  
**Dependency/license notes:**

## Documentation

- [ ] Documentation/specifications/examples/CLI help were updated where behavior changed.
- [ ] Machine-readable interfaces remain documented and versioned where applicable.

## Review findings

**Critical unresolved:** None / list  
**Major unresolved:** None / list  
**Minor unresolved:** None / list

- [ ] There are zero unresolved Critical findings.
- [ ] There are zero unresolved Major findings.

## Gate evidence

**Gate report:** `docs/gates/GATE-XX-<name>.md` / not yet created  
**CI run:**  
**Head commit:**

## Merge authorization

- [ ] Applicable gate is PASSED.
- [ ] Gate report is complete.
- [ ] Project owner has explicitly authorized the next gate/merge where required.

**If any required checkbox above is false, this PR must remain draft/unmerged.**
