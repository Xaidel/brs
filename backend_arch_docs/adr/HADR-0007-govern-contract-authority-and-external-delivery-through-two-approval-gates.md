# HADR-0007: Govern Contract Authority and External Delivery Through Two Approval Gates

- **Date**: 2026-07-21
- **Status**: Accepted
- **Deciders**: Project maintainers
- **Depends on**: [HADR-0006](HADR-0006-govern-core-delivery-through-three-approval-gates.md)

## Context and Problem Statement

A passing core does not prove that consumers can use the eventual public protocol or
that completed service adapters preserve its semantics. Building every production
adapter before exercising the external contract delays high-value feedback. Conversely,
a toy reference built with fake core behavior or a different protocol can pass examples
that say nothing about the eventual service.

An executable reference also must not become contract authority merely because it runs.
Without an independently owned normative contract, implementation accidents, fixtures,
or observed output can become de facto compatibility policy. The workflow therefore
needs explicit contract ownership, an early runnable reference, and a later complete
service while retaining the existing crate, privacy, boundary, and consistency model.

## Decision Drivers

- Give the exact external contract an independent normative owner and bounded scope.
- Validate the exact eventual external contract with a consumer-usable executable
  before all service adapters are complete.
- Exercise real core behavior rather than a transport-only mock.
- Reuse one conformance contract across reference and completed service compositions.
- Permit simplification only behind outbound ports.
- Prove production lifecycle through public interfaces without mislabeling in-process
  router tests as end-to-end tests.
- Avoid creating crates solely to represent workflow gates or compositions.

## Decision

After Gate 3 approval under HADR-0006, we will complete delivery through two ordered
approval gates. Each gate requires explicit review approval before an AI contributor
advances. An AI implements only the current gate, consumes approved predecessor
artifacts as fixed constraints, and stops with the required evidence rather than
anticipating later adapters or widening the contract.

### Contract Authority

Before Gate 4 begins, the owning normative technical design must identify the exact
external contract's owner and bounded scope. The technical design owns that contract by
default. It may instead explicitly incorporate a normative feature appendix when
several genuine slices share one contract.

Gate 4 co-develops the exact contract and runnable reference in one pull request. Their
joint approval makes the contract normative before Gate 5 begins. The contract, not
the reference implementation, service implementation, conformance suite, fixture, or
observed output, is the oracle. A disagreement requires an explicit contract or
implementation correction and a return to every affected upstream approval.

### Gate 4: Exact Contract and Runnable Reference Approval

One approved pull request must define the exact eventual external contract and provide
a runnable, consumer-usable reference implementation of that contract. The reference
uses the real approved core and the same eventual protocol, routes, schemas, success
outcomes, safe errors, required headers, limits, and compatibility rules. It may
simplify only outbound capabilities, for example with process-local persistence or a
controlled provider substitute behind an approved port. It must not replace inbound
behavior or the core with mocks.

A shared external API conformance suite runs against the reference composition. A
focused composition smoke test exercises the assembled inbound path with real core
behavior and the selected reference outbound adapters. Contract-only or mock-server-only
pull requests do not satisfy Gate 4 because consumers need a runnable example in the
same approved change.

### Gate 5: Complete Service Approval

The gate supplies the complete service adapters and deployment composition without
changing the approved external contract. The same external API conformance suite and
focused composition smoke test run unchanged against the service composition.
Service-specific tests prove adapter translation, protocol, consistency, failure,
ordering, concurrency, cancellation, security, configuration, and lifecycle semantics
that the shared suite cannot own.

A functional end-to-end test starts the compiled service and exercises it only through
its public interface, including bounded readiness, timeouts, cleanup, and process
lifecycle. In-process Axum tests remain router or composition tests, not process
end-to-end evidence.

Performance acceptance is conditional. It is required only when the normative focused
PRD states a measurable performance target and the normative technical design document
defines a controlled workload, environment, measurement method, and pass criterion.
An ad hoc benchmark or an unstated expectation must not become a release gate.

The Gate 4 reference and Gate 5 service are compositions of capabilities, not
automatically Cargo crates. They may be wiring functions, executable selections, or
test harness compositions within an existing justified deployment boundary. A new
crate still requires an independent deployment, reuse, ownership, compilation, or
dependency reason and the repository's HADR process.

## Alternatives Considered

### Complete Service Before Any External Validation

- Benefits: only one executable and adapter set needs maintenance.
- Costs and risks: consumer and protocol feedback arrives after expensive adapter work.

### Treat the Reference as Executable Contract Authority

- Benefits: minimizes normative protocol documentation.
- Costs and risks: accidental behavior becomes compatibility policy and can constrain
  consumers and the service incorrectly.

### Approve the Contract Before Any Reference Work

- Benefits: establishes authority before implementation begins.
- Costs and risks: exact protocol details cannot receive feedback from the simplest
  real-core executable before approval.

### Mock or Different-Protocol Reference

- Benefits: rapid demonstration with minimal dependencies.
- Costs and risks: does not prove real core integration or the contract consumers will
  eventually use.

### Co-Develop an Independently Owned Contract and Reference (chosen)

- Benefits: combines executable feedback with explicit contract governance, validates
  consumer usability early, and detects drift when complete adapters replace simplified
  outbound capabilities.
- Costs and risks: Gate 4 review must distinguish contract decisions from
  implementation defects and maintain two explicit compositions during delivery.

## Consequences

### Positive

- Consumers can integrate against the eventual protocol before service completion.
- Consumers receive a runnable endpoint backed by an independently reviewable contract.
- Gate 5 adapters cannot redefine protocol behavior from implementation convenience.
- The completed service is checked against evidence first established by the reference.
- Adapter-specific depth and public-interface behavior have distinct owners.

### Negative

- Gate 4 review must cover both normative contract text and executable evidence.
- Reference outbound adapters and composition require maintenance until the service is
  accepted.
- Shared contract appendices and conformance setup require explicit authority and must
  avoid coupling tests to one composition's internals.

### Neutral / Risks

- Exact external contract changes after Gate 4 require returning to the governing
  product, design, and affected gate approvals rather than silently changing Gate 5.
- Simplified outbound capabilities can conceal service-specific constraints; Gate 5
  adapter tests and end-to-end evidence remain mandatory.
- Later evidence can reopen the contract, but the owning artifact and downstream
  evidence must be revised explicitly.

## Confirmation

- The normative technical design identifies the contract owner and bounded scope before
  Gate 4 begins.
- Gate 4 approves the exact contract, runnable reference, shared conformance, and
  focused composition smoke together.
- Gate 4 uses the real core and eventual protocol and simplifies only outbound
  capabilities.
- Gate 5 begins only after the exact contract is normative.
- Review and documentation name the contract rather than an implementation as the
  behavior oracle.
- The same external API conformance suite and focused composition smoke test pass for
  both Gate 4 and Gate 5 compositions.
- Gate 5 includes service-specific tests and a functional process-level
  public-interface end-to-end test.
- Any performance criterion traces to a normative PRD target and controlled technical
  design; otherwise no performance acceptance claim is made.
- Cargo review confirms reference and service names did not create unjustified crates,
  and the complete workspace validation passes.

## Relationships and References

- Depends on: [HADR-0006](HADR-0006-govern-core-delivery-through-three-approval-gates.md)
- Retains the architecture and composition model of
  [HADR-0002](HADR-0002-adopt-rust-hexon-architecture.md).
- Retains the crate and privacy model of
  [HADR-0003](HADR-0003-define-crate-and-module-responsibilities.md).
- Completes external delivery mechanics for the construction order in
  [HADR-0004](HADR-0004-build-and-test-capabilities-from-the-core-outward.md).
- Retains the boundary and consistency policies of
  [HADR-0005](HADR-0005-define-boundary-and-consistency-policies.md).
- Operational guidance: [Developer runbook](../developer-runbook.md)
- Testing guidance: [Testing](../testing.md)