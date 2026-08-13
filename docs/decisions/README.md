# TP Architecture Decision Records (ADRs)

TP uses Architecture Decision Records for durable language, compiler, runtime, security, tooling, package, ABI, and governance decisions.

## When an ADR is required

Create an ADR before implementation when a decision:

- defines or changes observable language semantics;
- changes compiler-stage architecture or an IR invariant;
- chooses a memory/ownership/unsafe/effect/concurrency model;
- establishes an ABI, package format, lockfile, registry, or compatibility promise;
- creates a security/trust boundary;
- selects a backend/runtime strategy with long-term consequences;
- changes public machine-readable compiler/tooling interfaces;
- intentionally reverses a prior accepted decision.

Small implementation details that do not affect these areas do not need ADRs.

## Naming

Use:

```text
ADR-0001-short-kebab-title.md
ADR-0002-next-decision.md
```

Numbers are never reused, including superseded ADRs.

## States

- `PROPOSED`
- `ACCEPTED`
- `REJECTED`
- `SUPERSEDED`
- `DEPRECATED`

Implementation of a decision that requires an ADR must not begin while the ADR remains PROPOSED unless the active gate explicitly authorizes a reversible prototype used to decide the ADR.

## Required structure

Copy `TEMPLATE.md` and complete every section. No `TBD` placeholders are permitted in an ACCEPTED ADR.

## Relationship to gates

Gate reports must reference every ADR that materially affects their requirements. Reopening a decision may require reopening an earlier gate if the old decision was part of that gate's proof.
