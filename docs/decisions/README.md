# TP Architecture Decision Records (ADRs)

TP uses ADRs for durable language, compiler, runtime, security, tooling, package, ABI, and governance decisions.

Create an ADR before implementation when a decision changes observable semantics, compiler-stage architecture/IR invariants, memory/ownership/unsafe/effect/concurrency models, ABI/package/compatibility promises, security boundaries, backend/runtime strategy, public machine-readable tooling interfaces, or intentionally reverses a prior accepted decision.

Use numbered files such as `ADR-0001-short-kebab-title.md`; numbers are never reused.

States: `PROPOSED`, `ACCEPTED`, `REJECTED`, `SUPERSEDED`, `DEPRECATED`.

Implementation of an ADR-required decision must not begin while it remains PROPOSED unless the active gate explicitly authorizes a reversible prototype to decide the ADR.

Copy `TEMPLATE.md` and complete every section. ACCEPTED ADRs may not contain `TBD` placeholders.

Gate reports reference every ADR that materially affects their proof; reopening an ADR may reopen an earlier gate.
