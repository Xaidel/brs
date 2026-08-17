# HADR-0006: Govern Core Delivery Through Three Approval Gates

- **Date**: 2026-07-21
- **Status**: Accepted
- **Deciders**: Project maintainers

## Context and Problem Statement

The accepted specification pipeline and core-outward sequence establish authority and
construction order, but they do not define review boundaries. A single large pull
request can make domain vocabulary, port semantics, failing use-case examples, and
their implementation appear inevitable together. This is particularly risky for AI
delivery: an agent can race ahead, hide invalid assumptions behind passing code, and
deny reviewers a meaningful point at which to correct the contract.

The reusable workflow needs approval gates for the core while preserving
test-driven development's red-green evidence, a mergeable default branch, and the
distinction between review gates and technical-design phases.

## Decision Drivers

- Require normative product and implementation authority before delivery begins.
- Review domain meaning and port semantics before application code makes them costly
  to change.
- Make intended use-case behavior visible as genuinely failing tests before it is
  implemented.
- Keep intentionally failing code off the default branch.
- Constrain human and AI delivery progressively instead of authorizing the full design
  at once.
- Preserve Rust crate privacy, owned boundary records, and a fast core-only gate.
- Avoid turning architectural layers or review states into artificial phases.

## Decision

Every new coherent implementation slice requires a normative focused PRD and a
normative owning technical design document. The design may describe the complete
delivery sequence, but each gate requires explicit approval before the next begins.
Approval of the design does not authorize an AI contributor to implement later gates.

The gates are review states within the owning design, not automatically design phases,
branches, crates, releases, or permanent artifacts. Create technical-design phases only
for genuine dependencies, independently verifiable outcomes, material risk, or separate
handoffs. Approval, merge, and release remain distinct events.

For each authorized slice, we will deliver the core through three ordered approval
gates.

### Gate 1: Domain and Ports Approval

The gate contains only the required domain concepts and meaningful domain tests plus
the semantic inbound and outbound ports. Port contracts define owned operation records,
stable errors, and applicable identity, ordering, atomicity, idempotency, concurrency,
and cancellation semantics. Rust visibility keeps aggregates and application
implementations private. Approval confirms vocabulary and contracts; it does not
authorize use cases, adapters, composition, or future contract extensions.

### Gate 2: Failing Use-Case Tests Approval

The gate is a pull request dependent on the approved Gate 1 state. It adds test-local
outbound fakes and compiling use-case tests that fail for the intended missing
behavior. Only the minimum application shell needed to express that failure is
permitted. Review confirms that the tests specify the approved behavior and that the
failure is caused by absent behavior rather than broken fixtures, compilation,
timeouts, or unrelated defects.

The red pull request is evidence, not a standalone deliverable. It must never be
merged alone. Its approved tests proceed with Gate 3 in a dependent stack, combined
merge, or equivalent review system that keeps the default branch green.

### Gate 3: Minimal Green Core Approval

The gate adds the smallest application behavior that makes the approved Gate 2 tests
pass, then permits refactoring while green. It must pass the repository's core-only
Cargo gate without concrete adapters or external I/O. Additional behavior, transport,
persistence implementation, composition, and speculative extension points are
forbidden.

Tokio may execute async core tests as a dev dependency without making deployment
runtime part of the core. Traits without behavior do not require ceremonial signature
tests.

After Gate 3 approval, external contract and deployment work proceeds under
[HADR-0007](HADR-0007-govern-contract-authority-and-external-delivery-through-two-approval-gates.md).
If later work exposes an error in approved domain meaning, ports, or core behavior, the
slice returns to the earliest affected gate and repeats downstream approval rather than
silently repairing the contract.

## Alternatives Considered

### One Core Pull Request

- Benefits: fewer reviews and no intentionally red review state.
- Costs and risks: reviewers see contracts only after their implementation exists, and
  AI can silently optimize the contract around code it has already written.

### Commit Red Tests Directly to the Default Branch

- Benefits: preserves literal red-green history in the main branch.
- Costs and risks: breaks the shared branch and blocks unrelated validation.

### One Phase Per Architectural Layer

- Benefits: makes construction order and ownership visibly explicit.
- Costs and risks: confuses review gates with independently deliverable outcomes and
  adds permanent document ceremony without a genuine dependency.

### Three Dependent Approval Gates (chosen)

- Benefits: creates explicit correction points, preserves observable red evidence,
  keeps merged states green, and narrows delegated work progressively.
- Costs and risks: requires stacked or coordinated pull-request handling and explicit
  approval evidence.

## Consequences

### Positive

- Domain and port mistakes can be corrected before application implementation.
- Test intent is reviewed independently from the code that satisfies it.
- Human and AI contributors receive deliberately narrow, progressively constrained
  tasks.
- The workflow preserves one coherent technical design unless real phase criteria
  justify decomposition.

### Negative

- A coherent capability requires at least three core review decisions.
- Maintainers must preserve dependent pull-request relationships and red evidence.

### Neutral / Risks

- A red test can still be misleading; reviewers must inspect the failure reason, not
  merely observe a nonzero test result.
- Gate 1 can expose a public contract before a consumer exists, so review must reject
  speculative ports and records.
- A later correction can require repeating several approvals.

## Confirmation

- Gate evidence identifies the normative product and technical-design sources, current
  gate, approved predecessor, allowed scope, and exit criteria.
- Gate 1 contains no use-case implementation, adapter, transport, runtime, or wiring.
- Gate 2 CI or review output shows the intended test failure and the pull request is
  never merged without Gate 3 behavior.
- Gate 3 passes:

  ```sh
  cargo check -p app_core --all-targets --locked
  cargo test -p app_core --locked
  cargo test -p app_core --doc --locked
  ```

- Review confirms no adapter, transport, runtime, or future-gate implementation entered
  gates 1 through 3.
- The owning technical design uses phases only when it records a genuine phase reason.

## Relationships and References

- Completes the approval mechanics for the specification pipeline in
  [HADR-0001](HADR-0001-govern-product-and-implementation-specifications.md).
- Completes core review mechanics for the construction order in
  [HADR-0004](HADR-0004-build-and-test-capabilities-from-the-core-outward.md).
- Precedes: [HADR-0007](HADR-0007-govern-contract-authority-and-external-delivery-through-two-approval-gates.md)
- Operational guidance: [Developer runbook](../developer-runbook.md)
- Workflow summary: [Development workflow](../development-workflow.md)