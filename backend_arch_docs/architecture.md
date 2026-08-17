# Architecture

Hexon combines hexagonal architecture with onion-style inward dependencies. Business
meaning and use-case behavior stay in `app_core`; transport, persistence, runtime
lifecycle, and composition stay in self-contained `infra_*` deployment crates.

## Product Specification Flow

```text
Master PRD (Vision) -> focused PRD (Normative)
                    -> technical design (Normative)
                    -> implementation -> acceptance evidence
```

The master PRD supplies direction but does not authorize implementation. A focused
feature or release PRD and its owning technical design must be normative before work
begins. Application specifications live under `docs/specs/`.

## Construction and Testing Flow

```text
Gate 1 domain + ports approval
    -> Gate 2 dependent failing use-case tests approval (never merge red alone)
    -> Gate 3 minimal green core + core gate approval
    -> Gate 4 exact external contract + runnable reference approval
    -> Gate 5 complete service + public-interface acceptance approval
```

Apply this sequence to each coherent design slice. Each gate progressively constrains
human and AI work to approved predecessor artifacts. It is not an application-wide
waterfall, does not justify speculative concepts or ports, and does not require a
separate technical-design phase document per gate.

## Source Dependency Direction

```text
infra_* -> app_core::ports and app_core assembly
app_core::application -> app_core::ports and private domain
private domain -> no application or infrastructure
```

Cargo enforces the crate boundary. Private modules, visibility, tests, documentation,
and review enforce direction inside `app_core`. Runtime flow through an outbound port
does not reverse source dependency direction.

## Runtime Request Flow

```text
client -> inbound adapter -> inbound port -> use case -> domain/outbound port
       -> outbound adapter -> external system
```

Infrastructure receives core capabilities through the public assembly API. It does
not construct or access private use cases or entities.

## Responsibilities

- `domain::value_objects` owns immutable, self-validating semantic values.
- `domain::events` owns immutable records of significant business facts.
- `domain::entities` owns identity-bearing aggregates and lifecycle behavior.
- `domain::services` owns stateless rules spanning domain objects when justified.
- `ports` owns inbound and outbound contracts, rich boundary values, and stable errors.
- `application` implements named use cases through domain behavior and ports.
- Core assembly constructs private use cases and returns public inbound capabilities.
- `infra_*` modules translate external representations, implement outbound ports,
  configure concrete dependencies, and own deployment lifecycle.

Only `app_core::ports` forms the supported external core API; it re-exports the
assembly function. Repository ports exchange complete port-owned state records so
persistence can operate without access to private aggregates.

Async ports use `async-trait` because assembly stores runtime-selected implementations
behind `Arc<dyn Trait + Send + Sync>`. This object-safe boundary is a deliberate
trade-off for explicit dependency injection.

## Reference and Service Compositions

Gate 4 supplies a runnable reference that uses the real core and exact eventual
external protocol while permitting only outbound capabilities to be simplified. Gate
5 replaces or completes those outbound capabilities and deployment concerns without
changing the approved external contract.

Both compositions run the same external API conformance suite and a focused
composition smoke. The service also owns adapter-specific tests and a functional
process-level public-interface end-to-end test. In-process Axum router tests use
`tower::ServiceExt::oneshot` and prove protocol or composition behavior without being
called process end-to-end tests.

Reference and service describe compositions, not mandatory Cargo crates. Existing
crate criteria remain authoritative: create a crate only for a durable deployment,
reuse, ownership, compilation, or dependency boundary.

## Governance

`arch_docs/` owns reusable Rust Hexon architecture. `docs/` owns application
requirements, technical designs, decisions, and operations. `AGENTS.md` is the
effective engineering contract.

HADR-0006 governs prerequisites and gates 1 through 3. HADR-0007 governs exact contract
authority and gates 4 and 5 while retaining the architecture, crate, privacy, boundary,
and consistency model of HADR-0002 through HADR-0005.