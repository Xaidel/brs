# Development Workflow

Work moves from approved product intent to approved technical design, then through five
explicit delivery approvals. Apply this workflow per coherent capability. Each gate
limits the next change set; this progressive constraint is especially important for AI
contributors, which must not anticipate later gates.

[HADR-0006](adr/HADR-0006-govern-core-delivery-through-three-approval-gates.md)
governs prerequisites and gates 1 through 3.
[HADR-0007](adr/HADR-0007-govern-contract-authority-and-external-delivery-through-two-approval-gates.md)
governs contract authority and gates 4 and 5.

## Planning Authority

1. Confirm the master PRD contains the initiative in its direction or feature map.
2. Make a focused feature or foundational-release PRD `Normative` through review.
3. Write one complete technical design document, or phases justified by real
   dependencies, independently verifiable outcomes, material risk, or separate handoffs.
4. Make the owning technical design `Normative` and satisfy its prerequisites.
5. Record how its coherent slices pass gates 1 through 5 without treating the gates as
   automatic design phases.

The feature README owns reading order, precedence, dependencies, status, and progress.
Every requirement must be covered, explicitly deferred, excluded with justification,
or blocked by a recorded question.

## Gate 1: Domain and Ports

Approve only the domain meaning and semantic core contracts required by the slice.

- Add required values, entities, typed errors, events, or services and meaningful tests.
- Define inbound capabilities and outbound requirements in `app_core::ports`.
- Use validated values and owned operation-specific records. Keep aggregates and use
  cases private.
- Specify identity, ordering, atomicity, idempotency, conflict, concurrency, and
  cancellation behavior when applicable.
- Reject HTTP, Serde, Axum, database, SDK, runtime, adapter, and speculative future needs.

Gate 1 approval freezes the reviewed vocabulary and contracts for Gate 2. A discovered
contract defect returns to Gate 1 rather than being silently repaired downstream.

## Gate 2: Failing Use-Case Tests

Create a dependent pull request from the approved Gate 1 state. Add test-local outbound
fakes and compiling application tests that fail only because the intended use-case
behavior is absent. Capture the command, failing test names, and failure output.

The reviewer confirms that each test describes approved behavior, observes the correct
boundary, and fails for the intended reason. The red pull request is never merged alone.
Carry it into Gate 3 with a stacked pull request, combined merge, or equivalent branch
workflow that leaves the default branch green.

## Gate 3: Minimal Green Core

Add the smallest application behavior that passes the approved Gate 2 tests. Refactor
only while green. Do not add adapters, transport, persistence implementations, or
unapproved extension points.

Run the core gate without external I/O:

```sh
cargo check -p app_core --all-targets --locked
cargo test -p app_core --locked
cargo test -p app_core --doc --locked
```

Tokio may be a core dev dependency for async orchestration tests. The current local
composition uses `Arc<dyn Trait + Send + Sync>` and object-safe async ports; use owned
records, keep synchronization inside adapters, never hold a lock across `.await`, and
preserve cancellation safety.

## Gate 4: Exact Contract and Runnable Reference

One approved pull request must include both:

- the exact external contract consumers will use in the eventual service; and
- a runnable, consumer-usable reference implementation of that contract.

Before this gate starts, the normative technical design identifies the contract owner
and bounded scope. The design owns the contract by default, or explicitly incorporates
a normative feature appendix when genuine slices share it. Gate 4 joint approval makes
the exact contract normative before Gate 5. The contract, not either implementation or
the conformance suite, is the oracle.

The reference uses the real Gate 3 core and the same eventual protocol, routes,
schemas, headers, success outcomes, and safe errors. Only outbound capabilities may be
simplified behind approved ports. A mock core, transport-only stub, different protocol,
or contract-only pull request does not qualify.

Run one shared external API conformance suite against the reference composition. Add a
focused composition smoke that assembles the real core and reference outbound adapters
and exercises a representative inbound path. For Axum, prefer an in-process `Router`
request with `tower::ServiceExt::oneshot`; this is a composition smoke, not process E2E.

## Gate 5: Complete Service

Implement the complete service adapters and deployment composition without changing
the approved external contract. Run the same shared external API conformance suite and
focused composition smoke against the service composition. Add focused service tests
for adapter translation, real protocol semantics, consistency, errors, ordering,
atomicity, concurrency, cancellation, configuration, and lifecycle as applicable.

Add a functional end-to-end test that starts the compiled service, waits for readiness
with a deadline, uses only the public interface, verifies a critical journey, requests
shutdown where supported, and always kills and reaps the child on failure.

Performance acceptance is present only when the normative focused PRD states a
measurable target and the normative technical design controls workload, environment,
measurement, and pass criterion. Otherwise benchmarks are diagnostic evidence, not an
acceptance gate.

Reference and service are compositions. Do not create `reference` or `service` crates
unless ordinary crate-boundary criteria independently require them.

## Approval and Completion

Each gate records its normative sources, predecessor approval, scope, commands, and
evidence. Approval of one gate authorizes only the next gate. An AI contributor stops
after producing the current gate's evidence and must not use spare time to implement
later behavior.

After Gate 5 implementation is complete and before approval, run complete validation,
verify every acceptance criterion, and update delivery status:

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo test --workspace --doc --locked
```

See the [developer runbook](developer-runbook.md) for operational mechanics and the
[testing guide](testing.md) for evidence ownership.