# Anti-Patterns

## Unapproved Implementation

Do not implement from a vision, proposed focused PRD, or proposed technical design.
Normative focused requirements and a normative owning design authorize work.

## Artificial Architecture Phases

Do not create phases solely for domain, ports, application, adapters, and
infrastructure. A phase requires a real dependency and verifiable outcome.

## Technical Designs That Redefine Product Scope

Resolve product changes in the focused PRD before updating implementation design.

## Adapter-First Contract Design

Do not derive domain concepts or ports from HTTP DTOs, schemas, or provider types.
Define domain meaning and core-owned contracts first.

## Application Logic Before Tests

Guide each new use-case behavior with a failing test and test-local outbound fakes.
Review that evidence in a dependent Gate 2 pull request and never merge it red without
Gate 3. Do not begin adapters until the Gate 3 core approval passes.

## Gate Skipping and AI Overreach

Do not treat approval of the complete technical design as permission to implement all
five gates at once. Human and AI contributors implement only the current gate. Do not
hide later behavior, adapters, or contract changes in an earlier approval.

## Permanently Red Default Branch

Do not merge Gate 2 alone. Preserve intentionally failing test evidence in a dependent
pull request, then merge it only with the minimal Gate 3 implementation.

## Contract-Only or Mock-Core Reference

Do not call schemas, a mock server, or a transport stub a Gate 4 reference. The exact
eventual contract and runnable consumer-usable implementation ship in the same approved
pull request, using the real core and same eventual protocol.

## Simplifying the Inbound Side

Do not use a different protocol, route set, schema, or error contract for the reference.
Only outbound capabilities behind approved ports may be simplified.

## Composition Crates by Default

Do not create crates merely because the workflow names reference and service
compositions. Apply the same durable deployment, reuse, ownership, compilation, and
dependency criteria as any other crate decision.

## Duplicated Conformance Suites

Do not fork external expectations for reference and service. Run the same contract
suite and focused smoke intent against both, then add service-specific adapter tests.

## In-Process E2E Claims

Do not call an Axum `Router` exercised with `oneshot` a process end-to-end test. Gate 5
functional E2E starts the compiled service and uses only its public interface.

## Uncontrolled Performance Gates

Do not block acceptance on an ad hoc benchmark. Performance acceptance requires both a
normative PRD target and a controlled measurement design in the normative technical
design document.

## Ceremonial Trait Tests

Do not restate method signatures in tests. Exercise observable port semantics through
consumers and adapters.

## Boundary Leakage

Do not import Axum, Serde, Tokio runtime, database, environment, or provider types into
domain, ports, or application. Do not make private entities or use cases public to let
deployment code bypass ports.

## Misplaced Responsibilities

Do not put business rules in handlers, translation in wiring, or composition in
adapters. Adapters do not call one another. HTTP handlers do not construct use cases
or access storage directly.

## Shared Mutable Aggregates

Do not persist `Arc<Mutex<Note>>` or expose aggregate aliases. Exchange and store owned
port state. Never hold a lock guard across `.await`.

## Unsafe Error Exposure

Preserve typed errors internally, but never serialize dependency details. Return the
stable external error contract.

## Speculative Topology

Do not split every conceptual layer or hypothetical deployment into a crate. Add a
shared adapter only after demonstrated reuse.

## Misplaced Documentation

Do not put application requirements, technical designs, or project decisions in
`arch_docs/`. Do not put reusable Rust Hexon rules in `docs/`.

## Checker Theater

Do not present a partial dependency scanner as complete architectural enforcement.
Use Cargo boundaries, Rust privacy, tests, explicit contracts, and review.