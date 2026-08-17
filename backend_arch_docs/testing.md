# Testing

Testing follows the architecture and the five approval gates. Evidence is introduced at
the earliest boundary that can own it and is reused unchanged when it describes a
shared external contract.

HADR-0006 governs test evidence for gates 1 through 3. HADR-0007 governs the normative
contract oracle and test evidence for gates 4 and 5.

## Gate Evidence

| Gate | Required evidence |
| --- | --- |
| 1 | Domain invariant tests and reviewed semantic port contracts |
| 2 | Compiling use-case tests with local fakes, captured failing for the intended reason |
| 3 | Approved tests green plus the core-only Cargo gate |
| 4 | Shared external API conformance against a runnable real-core reference and a focused composition smoke |
| 5 | The same conformance and smoke against the service, service-specific tests, functional process E2E, and conditional controlled performance evidence |

Gate 2's red pull request is dependent evidence and must never merge alone. A nonzero
test command is insufficient: review the named failure and show that missing use-case
behavior, not compilation or fixture failure, caused it.

## Test Layers

### Domain and Application

Test value validation and normalization, entity identity, aggregate invariants,
identity-preserving lifecycle behavior, typed errors, and application orchestration.
Test-local outbound fakes perform no network, filesystem, database, or process I/O.
Compilation proves behaviorless trait shapes; do not add ceremonial trait tests.

### Shared External API Conformance

Define the conformance suite from the exact external contract, independent of reference
or service internals. Run the same cases against both compositions. Assert statuses,
headers, content types, typed bodies, safe errors, malformed input, unsupported methods
and media types, and every public operation required by the focused PRD.

The harness may accept a router factory, bound address, or client abstraction, but it
must not branch expected behavior by composition. Composition-specific setup belongs in
the harness adapter, not in the shared assertions.

### Adapter and Service-Specific Tests

Inbound tests cover protocol parsing, validated translation, and safe error mapping.
Outbound tests cover provider translation, error classification, identity, ordering,
conditional and atomic writes, owned snapshot isolation, and logical concurrency. Test
cancellation at real suspension points when an adapter can suspend internally. Never
hold a lock across `.await` in production or test orchestration.

### Composition Smoke

A focused smoke assembles real core behavior with the selected outbound adapters and
exercises a representative inbound path. For Axum, use an in-process `Router` with
`tower::ServiceExt::oneshot` when socket behavior is not the subject. Run the same smoke
intent for reference and service compositions; do not call it full E2E.

### Functional End-to-End

Gate 5 starts the compiled process and interacts exclusively through public interfaces.
Use an ephemeral port, bounded readiness, request timeouts, isolated state, and a child
guard that guarantees kill and reap on failure. Cover at least one critical functional
journey plus startup and public reachability; add shutdown assertions when lifecycle is
part of the contract.

### Performance

Performance acceptance is conditional. Require it only when a normative focused PRD
contains a measurable target and its normative technical design defines controlled
inputs, environment, warmup, sampling, statistics, and pass criteria. Keep ordinary
benchmarks outside acceptance claims when those sources do not exist.

Safe Rust prevents many memory races, but it does not prove logical concurrency, ID
uniqueness, lock scope, ordering, atomic replacement, or cancellation behavior.

## Cargo Gates

Run at Gate 3:

```sh
cargo check -p app_core --all-targets --locked
cargo test -p app_core --locked
cargo test -p app_core --doc --locked
```

Run before Gate 5 approval and final review:

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo test --workspace --doc --locked
```