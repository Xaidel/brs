# HADR-0003: Define Crate and Module Responsibilities

- **Date**: 2026-07-20
- **Status**: Accepted
- **Deciders**: Project maintainers

## Context and Problem Statement

Layer names alone do not prevent misplaced behavior. The durable core/deployment
boundary and the conceptual direction inside the core need explicit ownership without
turning every internal layer into a public crate.

## Decision Drivers

- Make placement deterministic during design and review.
- Let Cargo enforce the durable dependency boundary.
- Keep private domain and application concepts cohesive.
- Avoid public API and manifest ceremony for internal layers.

## Decision

We will use `app_core` and self-contained `infra_*` deployment crates. `app_core`
depends on no workspace crate. Deployment crates depend on the public core API.

Inside `app_core`, value objects depend on no other domain concepts; events may use
value objects; entities may use values and events; services may use entities and
values. Application uses domain behavior and ports. Assembly constructs private use
cases and returns public inbound capabilities.

Only `app_core::ports` is public, including its re-exported assembly function.
Deployment crates own translation, port implementations, configuration, wiring,
lifecycle, and their executable. A shared adapter crate requires demonstrated reuse
and an accepted HADR.

## Alternatives Considered

### One Crate Per Layer

- Benefits: stronger mechanical dependency enforcement.
- Costs and risks: excessive public surface and independently consumable-looking layers.

### Core and Deployment Crates

- Benefits: expresses the durable boundary while preserving private internals.
- Costs and risks: module direction inside `app_core` relies on privacy, tests, and review.

## Consequences

### Positive

- The workspace graph is small and communicates its real deployment boundary.
- Concrete entities and use cases remain private.

### Negative

- Cargo cannot enforce all conceptual direction inside `app_core`.

### Neutral / Risks

- Shared adapter extraction must remain evidence-driven rather than speculative.

## Confirmation

- `Cargo.toml` contains only justified workspace members and edges.
- Review verifies module imports and visibility when core responsibilities change.
- New crates and public boundary changes require a HADR.

## References

- [Dependency rules](../dependency-rules.md)
- [HADR-0002](HADR-0002-adopt-rust-hexon-architecture.md)