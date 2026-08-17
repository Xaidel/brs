# HADR-0002: Adopt Rust Hexon Architecture

- **Date**: 2026-07-20
- **Status**: Accepted
- **Deciders**: Project maintainers

## Context and Problem Statement

The template needs an architecture that preserves business meaning while allowing
transport, persistence, and deployment choices to change. Rust crate boundaries,
module privacy, ownership, and typed errors should enforce this without mechanically
copying patterns from another language.

## Decision Drivers

- Keep domain and application behavior independent of frameworks and deployment.
- Let the core own contracts used by callers and implementations.
- Use Rust visibility and ownership idiomatically.
- Keep composition explicit and replaceable.

## Decision

We will combine hexagonal ports and adapters with onion-style inward dependencies.
`app_core` owns private domain and application modules plus a public `ports` module
that includes a narrow assembly API. Self-contained `infra_*` crates own adapters,
configuration, concrete wiring, observability, runtime lifecycle, and executables.

Ports use rich validated values and typed errors. Concrete dependencies are injected
manually through constructors and `Arc<dyn Trait + Send + Sync>`. Domain code performs
no I/O. Adapters translate external representations; infrastructure composes them.

## Alternatives Considered

### Framework-Centered Application

- Benefits: rapid transport-first development.
- Costs and risks: framework and persistence representations shape business contracts.

### Rust Hexon Core and Deployments

- Benefits: durable inward dependencies, private implementation, replaceable edges.
- Costs and risks: explicit translation and dependency injection require more code.

## Consequences

### Positive

- Core behavior is testable without concrete adapters or I/O.
- Deployment choices can change without rewriting business behavior.

### Negative

- Boundary records and translation create deliberate duplication.

### Neutral / Risks

- Architecture labels still require responsibility-aware review.

## Confirmation

- Cargo manifests retain an acyclic deployment-to-core graph.
- Domain and application compile without infrastructure dependencies.
- Composition tests exercise the public core boundary.

## References

- [Architecture](../architecture.md)
- [HADR-0003](HADR-0003-define-crate-and-module-responsibilities.md)