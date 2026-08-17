# HADR-0004: Build and Test Capabilities From the Core Outward

- **Date**: 2026-07-20
- **Status**: Accepted
- **Deciders**: Project maintainers

## Context and Problem Statement

Dependency direction does not prescribe construction order. Starting with handlers or
schemas lets external representations shape the core, while tests written after use
cases lose value as design feedback.

## Decision Drivers

- Let domain meaning determine port contracts.
- Design application behavior with isolated tests before implementation.
- Keep the core executable without concrete adapters.
- Preserve short feedback loops per coherent capability.

## Decision

For each normative technical-design slice, we will define required domain meaning and
tests, define ports, write failing application tests with test-local fakes, implement
the smallest passing behavior, refactor, and pass the core gate before implementing
adapters or infrastructure.

```sh
cargo check -p app_core --all-targets --locked
cargo test -p app_core --locked
cargo test -p app_core --doc --locked
```

Adapter tests then verify translation and port semantics. Deployment composition and
acceptance evidence complete the slice. A failing test need not be committed. Traits
without behavior do not need ceremonial signature tests.

## Alternatives Considered

### Implementation First

- Benefits: exposes technology constraints and visible endpoints early.
- Costs and risks: external types shape core contracts and isolated testing becomes hard.

### Core-Outward Test-Driven Development

- Benefits: domain vocabulary and tests guide contracts before technologies intervene.
- Costs and risks: requires disciplined increments and maintained fakes.

## Consequences

### Positive

- Core regressions are detected without I/O.
- Adapters conform to passing core behavior rather than defining it.

### Negative

- Adapter work waits for meaningful core behavior and tests.

### Neutral / Risks

- Oversized slices delay integration feedback and should be narrowed in design.

## Confirmation

- Application tests perform no external I/O.
- The core gate passes before adapter work for the slice.
- Acceptance includes adapter and composition evidence afterward.

## References

- [Development workflow](../development-workflow.md)
- [Testing](../testing.md)
- [HADR-0001](HADR-0001-govern-product-and-implementation-specifications.md)