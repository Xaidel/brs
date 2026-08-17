# HADR-0005: Define Boundary and Consistency Policies

- **Date**: 2026-07-20
- **Status**: Accepted
- **Deciders**: Project maintainers

## Context and Problem Statement

Port placement alone does not define semantic types, aggregate privacy, persistence
consistency, error handling, concurrency, or cancellation behavior.

## Decision Drivers

- Preserve domain meaning across boundaries.
- Keep private aggregate behavior inside the core.
- Prevent mutable aliases and stale unconditional updates.
- Return safe external failures without losing internal diagnostics.

## Decision

Inbound ports use rich validated values and operation-specific records. Repository
ports exchange owned port-defined state rather than private aggregates. Application
code maps between state and aggregates where behavior is required.

Create and update use distinct insert and conditional-replace operations. Repository
lookups preserve requested identity, list results have documented deterministic order,
and not-found classification remains stable. Adapters store snapshots rather than
shared aggregate aliases.

External representations stop in adapters. Typed errors retain precise causes;
unexpected errors are logged but never serialized. Async ports are cancellation-safe,
retain no request-scoped state, and never hold a lock across `.await`.

## Alternatives Considered

### Expose Aggregates Through Repository Ports

- Benefits: fewer mapping records.
- Costs and risks: infrastructure must access core entities and lifecycle APIs.

### Port-Owned Complete State

- Benefits: aggregate privacy, validated state, and owned snapshot semantics.
- Costs and risks: explicit mapping is required at the application boundary.

## Consequences

### Positive

- Infrastructure cannot bypass aggregate behavior.
- Persistence semantics are explicit and safe for concurrent use.

### Negative

- Boundary state and mapping add code.

### Neutral / Risks

- Adapter implementations must uphold identity, ordering, and atomicity contracts.

## Confirmation

- Application tests cover error classification and repository integrity.
- Adapter tests cover ordering, isolation, atomic writes, concurrency, and cancellation
  where implementations can suspend internally.
- HTTP tests verify stable safe responses.

## References

- [Architecture](../architecture.md)
- [Testing](../testing.md)