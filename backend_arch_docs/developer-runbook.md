# Five-Gate Developer Runbook

This runbook operationalizes HADR-0006 and HADR-0007 for Rust repositories. It keeps a
shared conceptual order across implementations while using Rust-specific crate,
privacy, ownership, Axum, process, and Cargo mechanics.

It is practical guidance, not a competing source of authority. If this runbook differs
from an accepted HADR, `AGENTS.md`, a normative focused PRD, or its owning normative
technical design, follow the authoritative source and correct this runbook.

## First Day

Install Rust with `rustup`. This workspace tracks stable in `rust-toolchain.toml`, uses
Rust 2024, and declares Rust 1.85 as its minimum supported Rust version. From the
repository root, confirm the selected toolchain and build the locked workspace:

```sh
rustc --version
cargo --version
cargo check --workspace --all-targets --locked
```

Start the Notes service:

```sh
HTTP_ADDR=127.0.0.1:3000 RUST_LOG=info cargo run -p infra_local --locked
```

The default address is `0.0.0.0:8080`. `HTTP_ADDR` must parse as a socket address.
`RUST_LOG` controls the tracing filter, and the executable emits structured JSON logs.
The process reports its actual bound address after startup.

From another shell, verify the public interface:

```sh
curl -i http://127.0.0.1:3000/healthz
curl -i -X POST http://127.0.0.1:3000/notes \
  -H 'content-type: application/json' \
  -d '{"title":"First note","content":"A small example"}'
curl -i http://127.0.0.1:3000/notes
```

Notes are process-local and disappear on restart. Stop with `Ctrl-C`; Unix deployments
also handle `SIGTERM`. Both paths trigger graceful Axum shutdown. If startup fails,
check `HTTP_ADDR`, whether the port is occupied, and the structured error log before
changing code.

## Authority and Reading Order

Read in this order:

1. `README.md` for the runnable application.
2. `AGENTS.md` for the binding engineering contract.
3. `arch_docs/adr/README.md` and accepted HADRs for reusable decisions.
4. `arch_docs/architecture.md`, `dependency-rules.md`, `development-workflow.md`, and
   `testing.md` for consolidated guidance.
5. `docs/specs/README.md`, then the owning feature README, focused PRD, and technical
   design for the capability being changed.
6. Code from `app_core::domain` through `app_core::ports`, application, assembly,
   `infra_local` adapters, wiring, server, and executable.

The focused PRD owns product-visible capabilities, observable outcomes, and measurable
quality obligations. The technical design owns implementation boundaries, exact
protocol mechanics, delivery sequencing, and acceptance design. Accepted HADRs own
reusable architecture decisions. The runbook explains how to execute them.

## Four Flows to Keep Separate

### Specification Flow

```text
Master PRD (Vision)
    -> focused PRD (Normative)
    -> owning technical design (Normative)
    -> approved gate evidence
```

The master PRD supplies direction but never authorizes implementation.

### Construction Flow

```text
domain meaning -> ports -> red use-case tests -> green core
    -> exact external contract plus runnable reference -> complete service
```

This is the order in which uncertainty is reduced, not a source dependency graph.

### Source Dependency Flow

```text
infra_* -> app_core
app_core -> no deployment crate
```

Inside `app_core`, domain and application depend only on inward concepts and core-owned
ports. Runtime callbacks through trait objects do not reverse source dependencies.

### Runtime Flow

```text
consumer -> inbound adapter -> inbound capability -> application -> domain
                                                    -> outbound port -> adapter
```

Do not turn runtime arrows into imports or adapter-to-adapter calls.

## Before Starting a Feature

Confirm all of the following:

- A normative focused PRD owns the bounded product outcome.
- A normative technical design owns the coherent slice and exact gate sequence.
- All predecessor gates and true delivery phases are approved.
- Requirement coverage, non-goals, and deferred scope are explicit.
- Gate 4 identifies the owner and bounded scope of the exact external contract.
- Performance acceptance appears only when the PRD has a measurable target.
- The handoff identifies allowed crates, modules, dependencies, commands, and exit
  criteria.

Do not turn the five gates into five technical-design phases automatically. Create a
phase only for a genuine dependency, independently verifiable outcome, material risk,
or separate handoff.

## 1. Establish Authority

Before opening a delivery branch, record:

- the normative focused PRD and requirement identifiers;
- the normative owning technical design document and coherent slice;
- accepted HADR and ADR constraints;
- the current gate and approved predecessor evidence; and
- whether the PRD contains a measurable performance target.

Use one technical design for a coherent capability unless real dependencies or handoffs
justify phases. The five gates are approval checkpoints, not automatic phase documents.

For an AI contributor, state the allowed files and current gate explicitly. Instruct it
to stop after evidence for that gate. Reject speculative later-gate code even when it
appears correct.

## 2. Gate 1: Domain and Ports

### Change Set

- Add only required domain values, entities, typed errors, events, or services.
- Add meaningful invariant and lifecycle tests.
- Define semantic inbound and outbound traits, owned records, and stable errors.
- Keep domain and application modules private; expose the smallest supported surface
  through `app_core::ports` and core assembly only when required.
- For runtime-selected local dependencies, retain the repository's current
  `Arc<dyn Trait + Send + Sync>` choice. Do not generalize it into a universal rule.

### Review

- Confirm contracts derive from product and domain meaning, not Axum DTOs or storage.
- Confirm repository ports exchange owned state rather than aggregate aliases.
- Confirm async semantics include cancellation and never require a lock across `.await`.
- Confirm no use-case behavior, adapter, wiring, or future extension entered the gate.

Approve Gate 1 before creating Gate 2.

## 3. Gate 2: Failing Use-Case Tests

### Branch and Pull Request

Branch from the approved Gate 1 commit and mark the pull request as dependent and
intentionally red. Link its predecessor. Do not target it for standalone merge.

### Change Set

- Add test-local hand-written outbound-port fakes.
- Add compiling tests for the next approved application behaviors.
- Add only enough private application shell for tests to compile and reach the intended
  missing behavior.
- Avoid concrete adapters and external I/O.

### Evidence

Run the narrow tests and capture the test names and failure output. Confirm failures are
assertion failures caused by absent behavior, not panics in setup, compilation errors,
timeouts, or permissive fakes.

Review and approve the red evidence. Keep the pull request unmerged until Gate 3 is
combined through stacked review, merge queue support, or an equivalent green merge.

## 4. Gate 3: Minimal Green Core

### Change Set

- Implement only behavior required by approved Gate 2 tests.
- Preserve private entities and use cases.
- Pass owned commands and records; clone deliberately at boundaries.
- Refactor only after green and do not prebuild Gate 4 protocol code.

### Evidence

```sh
cargo check -p app_core --all-targets --locked
cargo test -p app_core --locked
cargo test -p app_core --doc --locked
```

The core gate performs no external I/O. Tokio as a dev dependency may run async use-case
tests, but deployment runtime and adapters remain outside this gate.

Merge Gate 2 tests only together with or immediately beneath this passing Gate 3 state.

## 5. Gate 4: Exact Contract and Runnable Reference

### Change Set

Define the exact eventual external protocol and provide its consumer-usable runnable
reference in the same pull request. The reference must use real Gate 3 core behavior.
The only simplifications are outbound capabilities behind approved ports, such as an
in-memory repository or controlled provider substitute.

Reference and service are composition roles. Prefer wiring functions or executable
configuration inside an existing justified deployment crate. Do not create a Cargo
crate merely to match a gate name.

### Shared Conformance

Build one external API conformance harness whose assertions are composition-neutral.
For Axum, a harness can accept a `Router` or router factory and use
`tower::ServiceExt::oneshot`. Assert exact methods, paths, statuses, headers, media
types, typed bodies, error envelopes, invalid inputs, and unsupported operations.

### Focused Composition Smoke

Assemble real core capabilities and reference outbound adapters, then exercise one
representative public journey through the inbound adapter. Keep this narrow; the shared
conformance suite owns external breadth. Label it composition smoke, not E2E.

### Consumer Check

Run the documented start command and exercise the contract as an external consumer
would. Verify the reference is runnable without importing private test helpers.

## 6. Gate 5: Complete Service

### Change Set

- Implement complete outbound and inbound adapters, configuration, wiring,
  observability, runtime, and lifecycle required by the service design.
- Preserve the Gate 4 external contract and run the same conformance harness unchanged.
- Run the same focused smoke harness against real core behavior and service-selected
  adapters.
- Add service-specific tests for real provider or persistence behavior, translation,
  ordering, atomicity, logical concurrency, cancellation, failures, and lifecycle.

Never hold a synchronization guard across `.await`. Test cancellation at a known
suspension point and verify state and lock availability afterward.

### Functional Process E2E

Use `CARGO_BIN_EXE_<name>` from a Cargo integration test or an equivalent maintained
process harness. Start the compiled binary on an ephemeral address, continuously drain
stdout and stderr, wait for readiness with a deadline, and exercise only public HTTP or
other public interfaces. A guard must kill and reap the child if any assertion fails.

In-process Axum tests remain protocol or composition tests. They do not satisfy this
process-level evidence.

### Conditional Performance Acceptance

If and only if the normative focused PRD states a measurable target, execute the
controlled workload and pass criterion defined by the normative technical design.
Record toolchain, build profile, environment, inputs, warmup, sample method, and result.
Without both sources, report no performance acceptance gate.

## 7. Complete Verification

Run the repository gate from the workspace root:

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo test --workspace --doc --locked
```

Then verify the shared conformance suite, both composition-smoke targets where both
remain in the repository, the Gate 5 process E2E, and any authorized performance check.

## 8. Record Delivery

Update the owning specification package with gate status and durable evidence links.
Record exact commands and outcomes. If a gate changes an approved contract, return to
the earliest affected gate and repeat downstream approval rather than patching history.

The existing Notes reference predates adoption of the five-gate workflow. Do not
manufacture historical gate approvals for it; apply this runbook to future capabilities
and successor references.

## Responsibility Placement

Use this diagnostic when reference and service work exposes an unclear responsibility:

| Question | Owner |
| --- | --- |
| Is it a product-visible capability, outcome, or measurable target? | Focused PRD |
| Is it an invariant or lifecycle rule of one aggregate or value? | Domain |
| Is it technology-independent orchestration for one caller intention? | Application |
| Is it a semantic capability the application needs from outside? | Outbound port |
| Is it the exact consumer protocol, schema, error, or compatibility rule? | Technical design or incorporated normative appendix |
| Is it serialization, database, provider SDK, runtime, or deployment behavior? | Adapter or infrastructure |

A deterministic fake provider may return port-valid structures and stable failure
classes. It must not decide product policy. Behavior needed identically from reference
and service is evidence of a shared contract, but shared protocol translation can still
belong in adapters. If a real adapter cannot satisfy a port naturally, revisit the port,
technology choice, or technical design explicitly instead of hiding the mismatch.

## Coding-Agent Delegation

Earlier gates contain more conceptual freedom and require stronger human review. Later
gates are intentionally more mechanical: approved domain language constrains tests,
approved tests constrain the core, and the approved core plus Gate 4 contract constrain
service adapters on both sides. That progressive reduction in degrees of freedom is the
reason this workflow is suitable for AI-assisted delivery.

Use this handoff format:

```text
Owning focused PRD and requirements:
Owning normative technical design:
Current gate:
Approved predecessor evidence:
In scope:
Explicitly deferred:
Allowed crates, modules, and dependencies:
Required starting test state:
Required validation commands:
Exit criteria and evidence:
```

The agent must implement only the current gate and stop. Do not ask a Gate 5 agent to
rediscover product behavior, redesign ports, or infer the public protocol from the
reference implementation.

## Review Checklists

### Gates 1 Through 3

- Domain names and invariants trace to the focused PRD.
- Ports use validated owned values and operation-specific records.
- No framework, Serde, Tokio runtime, storage, or provider types entered the core.
- Gate 2 compiles and fails only for the intended missing behavior.
- The red pull request cannot merge alone.
- Gate 3 is the smallest passing behavior and passes the core-only gate.

### Gate 4

- The contract specifies exact operations, encoding, schemas, errors, limits, ordering,
  compatibility, and consumer-visible cancellation or timeout behavior where relevant.
- The runnable reference invokes the real core and exposes the eventual service protocol.
- Only outbound capabilities are simplified.
- An external consumer can use the documented endpoint without private helpers.
- Shared conformance and focused composition smoke are distinct and both pass.

### Gate 5

- The same conformance cases and focused smoke run without implementation-specific
  expected behavior.
- Service-specific translation, persistence/provider, failure, concurrency,
  cancellation, security, configuration, and lifecycle risks have focused tests.
- Functional E2E starts the compiled process and uses only public interfaces.
- Any performance gate traces to a measurable PRD obligation and controlled design.
- New crates satisfy HADR-0003 rather than mirroring workflow labels.

## Common Gotchas

### Treating the Reference as a Mock Application

The reference can use a memory repository or deterministic fake provider, but handlers
cannot return canned business results. It must exercise real domain and application
behavior through public inbound capabilities.

### Letting the Reference Define the Contract

Gate 4 co-develops text and executable evidence, but the normative technical design is
the oracle. Unexpected output is a defect or an explicit contract proposal, not an
automatic compatibility rule.

### Sharing Mutable Aggregates

Persist owned state snapshots. Do not keep `Arc<Mutex<Aggregate>>`, retain request
aliases, or hold any lock guard across `.await`.

### Calling Every Cross-Component Test Integration or E2E

Name the evidence precisely. Axum `Router::oneshot` can prove protocol or composition;
it does not prove process startup, TCP behavior, signals, or shutdown. A comprehensive
conformance suite is not a smoke test.

### Treating Safe Rust as a Concurrency Test

Safe Rust prevents classes of memory failure. It does not prove logical ordering,
uniqueness, atomic replacement, cancellation behavior, or lock availability. Force
important interleavings with `Barrier`, `Notify`, or `oneshot`, and bound waits with
`tokio::time::timeout`.

## Troubleshooting

### The Red Test Does Not Compile

Gate 2 requires compiling tests. Add only the minimum private application shell needed
to express missing behavior. A compiler error is not red behavior evidence.

### The Red Test Passes

The fake may be permissive, the behavior may already exist, or the assertion may test
the wrong boundary. Tighten observations or choose the next absent approved behavior.

### Conformance Differs Between Implementations

Keep startup, credentials, reset, readiness, and cleanup in implementation-specific
runners. Expected statuses, headers, schemas, errors, and ordering must not branch. If
the difference is legitimate, revise and reapprove the Gate 4 contract.

### An Async Test Hangs

Use bounded timeouts, await every `JoinHandle`, continuously drain child output, and
wait for a known suspension point rather than sleeping or relying on scheduler order.
After cancellation, release helper tasks and verify both durable state and lock access.

### A Process Test Leaks a Child

Put the child in a guard immediately after spawn. The guard must kill and reap it on
all exits. Use an ephemeral address and a readiness deadline; never rely on a fixed
sleep.

### Clippy or Cargo Changes the Lockfile

This application template commits `Cargo.lock`. Run the documented commands with
`--locked`. Change dependencies deliberately in their own reviewed scope rather than as
a side effect of workflow work.

## FAQ

### Why five gates when it creates more work?

Each gate catches a different class of drift while correction is cheap. The later,
most repetitive implementation work is also the most tightly bounded and therefore the
best place to use coding agents.

### Can Gate 2 and Gate 3 be one pull request?

Yes when tooling cannot support a stack, provided red tests and green implementation
remain separately reviewable commits and only a green result lands.

### Must reference and service be separate crates?

No. They are compositions. Add a crate only for a durable deployment, reuse, ownership,
compilation, or dependency boundary.

### Can Gate 4 use REST before a GraphQL service exists?

Not if GraphQL is the intended service contract. Reference and service must expose the
same exact protocol. A temporary API does not qualify consumers or bound Gate 5.

### Is the in-memory adapter disposable?

It may be lifecycle-limited, but it remains tested code while used for Gate 4,
conformance, smoke, onboarding, or consumer integration. Its simplification does not
permit incorrect port semantics.

### Are benchmarks E2E acceptance?

No. Criterion or `iai-callgrind` can diagnose code paths. Performance acceptance needs
a product target and a controlled technical-design measurement plan.

## Quick Reference

```text
Gate 1  domain + ports                 approve meaning and boundaries
Gate 2  compiling red use-case tests  approve expected behavior; never merge alone
Gate 3  minimal green core             pass app_core without adapters or I/O
Gate 4  exact contract + reference     same eventual API; shared conformance + smoke
Gate 5  complete service               same evidence + service tests + process E2E
```

Core gate:

```sh
cargo check -p app_core --all-targets --locked
cargo test -p app_core --locked
cargo test -p app_core --doc --locked
```

Full gate:

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo test --workspace --doc --locked
```