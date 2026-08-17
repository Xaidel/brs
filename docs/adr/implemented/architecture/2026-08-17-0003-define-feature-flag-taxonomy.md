# ADR-0003: Define the Feature Flag taxonomy

<!-- Location: docs/adr/implemented/architecture/2026-08-17-0003-define-feature-flag-taxonomy.md.
     The inline Status below must agree with {lifecycle}. -->

- **Date**: 2026-08-17
- **Status**: Implemented
- **Deciders**: Product and Engineering (solo)

## Context and Problem Statement

The [master PRD](../../../specs/master/PRD.md) fixes a perpetual-license, pay-per-module model: one Core Engine license per installation, with each add-on module — KP Blotter, Treasury, Business Permits — unlocked separately via a signed License Key. The [Core Engine PRD](../../../specs/core-engine/PRD.core-engine.md) defines **Feature Flag** as "a runtime-toggleable capability unlocked by a valid License Key (used by add-on modules; Core Engine itself is always unlocked once installed)," validated entirely offline via an Ed25519 signature check against a public key embedded in the binary, bound to the installation's Machine Hardware ID. Each module PRD states its module's Feature Flag "must be gated by Core Engine's offline licensing engine," phrased in the singular.

[ADR-0002](2026-08-17-0002-fix-core-engine-workspace-crate-topology.md) placed Ed25519 license-key verification in the `infra_licensing` crate. The Core Engine PRD's data model sketches a `LicenseGrant` entity — `id`, `machine_hardware_id`, `feature_flags`, `signature_metadata` — but neither the PRD nor ADR-0002 fixes what `feature_flags` actually contains: the concrete key strings, whether they map one-to-one to modules, whether Core Engine itself needs a key, or the wire/storage representation shared between the signed payload and the persisted `LicenseGrant` row. Without this, `infra_licensing` and every module's gating check have no concrete contract to build against.

## Decision Drivers

- Give `infra_licensing` and the three module PRDs' "this module's Feature Flag" language a concrete, implementable contract.
- Keep the signed License Key payload debuggable by a human inspecting it, not just machine-parseable.
- Avoid a representation that breaks when a flag is later added or retired, since the product's own model is "additional modules unlocked via separately purchased signed keys" over the life of an installation.
- Preserve the PRD's explicit statement that Core Engine is unconditionally unlocked once installed — don't model an always-true condition as a flag.
- Don't design for modules that have no approved PRD yet (Municipal Sync, LAN Mode are explicitly out of scope on the parent map).

## Decision

We will define the Feature Flag taxonomy as follows:

1. **Key strings**: `KP_BLOTTER`, `TREASURY`, `BUSINESS_PERMITS` — SCREAMING_SNAKE_CASE, each matching its module's plain-English name.
2. **Mapping**: strict 1:1 between flags and modules. One flag unlocks its entire module; no module currently has an independently-unlockable sub-feature.
3. **No `CORE` key**: the flag enum contains only the three gate-able add-ons. Core Engine code paths perform no flag check at all — Core Engine access is unconditional, not gated by an always-true flag.
4. **Representation**: a single Rust type, `FeatureFlag` — a 3-variant enum (`KpBlotter`, `Treasury`, `BusinessPermits`) — serialized via serde as a JSON array of the SCREAMING_SNAKE_CASE strings above. The identical `Vec<FeatureFlag>` shape is used both inside the Ed25519-signed License Key payload (pre-signing) and persisted in `LicenseGrant.feature_flags` as a SQLite TEXT column (JSON-encoded).
5. **No forward-compatibility reservation now**: nothing is reserved for Municipal Sync, LAN Mode, or any other future module. The JSON-array representation already permits adding new keys later without a breaking change; reserving placeholder keys for unapproved, exploratory modules is deferred to whichever future map eventually specifies them.

## Alternatives Considered

### Short/abbreviated codes (`KP`, `TREAS`, `BIZ`)

- Benefits: shorter payload and column values.
- Costs and risks: less self-documenting in a signed payload a human might inspect while debugging a licensing issue; abbreviation choices (`BIZ` vs `PERMITS` vs `BP`) are arbitrary and harder to keep consistent as more modules are added. Rejected in favor of full module names.

### kebab-case (`kp-blotter`, `treasury`, `business-permits`)

- Benefits: readable, common in URL-like/config contexts.
- Costs and risks: inconsistent with Rust enum/const naming conventions used elsewhere in the workspace; SCREAMING_SNAKE_CASE serializes directly from a serde `rename_all` on a Rust enum with no translation layer. Rejected.

### Numeric bitmask (`0x01`, `0x02`, `0x04`, …)

- Benefits: compact; single-integer storage and comparison.
- Costs and risks: opaque in a signed payload meant to be debuggable; brittle the moment a flag is retired or bit positions need to shift; requires a separate mapping table kept in sync with the enum. Rejected — the product's own module-unlock lifecycle (buy more modules over an installation's life) makes flag churn a real, not hypothetical, case.

### Explicit always-true `CORE` flag

- Benefits: symmetry — every gate-able concept, including Core Engine, has a corresponding enum variant.
- Costs and risks: the Core Engine PRD is explicit that Core Engine is unconditionally unlocked once installed; a flag that can never be false isn't modeling a runtime toggle, it's modeling a constant, and any code that checked it would be dead logic. Rejected as modeling something unconditional as if it were conditional.

### Reserve placeholder keys for Municipal Sync / LAN Mode now

- Benefits: avoids a future taxonomy change when those modules are eventually specified.
- Costs and risks: both are explicitly out of scope on the parent map with no approved PRD; reserving keys for undecided, exploratory scope is speculative design, and the JSON-array representation already avoids a breaking change when they are eventually added. Rejected.

## Consequences

### Positive

- `infra_licensing` has a concrete `FeatureFlag` enum to build the License Key payload parser and `LicenseGrant.feature_flags` (de)serialization against.
- Module gating becomes a single, uniform check (`flags.contains(&FeatureFlag::KpBlotter)`, etc.) with no special-casing for Core Engine.
- Adding a fourth module later is an enum variant + JSON string addition, not a schema migration or bitmask renumbering.
- The three module PRDs' "this module's Feature Flag" language is now concretely specified end-to-end (payload → signature → persisted column).

### Negative

- SCREAMING_SNAKE_CASE full names make the JSON payload and column marginally larger than a bitmask or short code — judged acceptable given the small, bounded flag count.

### Neutral / Risks

- If a module is ever specified with an independently-unlockable sub-feature, this taxonomy's strict 1:1 assumption will need a follow-up decision (not addressed here).
- Municipal Sync and LAN Mode's eventual flag keys, if those modules are ever approved, are unspecified by this ADR and left to a future map.

## Confirmation

- `infra_licensing` defines `FeatureFlag` as a 3-variant enum (`KpBlotter`, `Treasury`, `BusinessPermits`) with serde `rename_all = "SCREAMING_SNAKE_CASE"` (or equivalent explicit renames), serializing to/from a JSON array of strings.
- No `Core` or `CORE` variant exists in `FeatureFlag`, and no code path checks a flag to gate Core Engine functionality.
- `LicenseGrant.feature_flags` is a SQLite TEXT column holding the JSON-encoded array, using the same `FeatureFlag` type as the signed payload.

## Relationships and References

- Refines [ADR-0002 (workspace crate topology)](2026-08-17-0002-fix-core-engine-workspace-crate-topology.md) by fixing the concrete contents of `infra_licensing`'s `LicenseGrant.feature_flags`.
- Owning spec: [Core Engine PRD](../../../specs/core-engine/PRD.core-engine.md) §9.5 (offline licensing & feature-gating), data model (`LicenseGrant`); [KP Blotter](../../../specs/kp-blotter/PRD.kp-blotter.md), [Treasury](../../../specs/treasury/PRD.treasury.md), and [Business Permits](../../../specs/business-permits/PRD.business-permits.md) PRDs (each module's Feature Flag gating requirement).
- Glossary: [CONTEXT.md](../../../../CONTEXT.md) (Feature Flag, License Key, LicenseGrant, Module).
- Supporting issue: [Define the Feature Flag taxonomy](https://github.com/Xaidel/brs/issues/4).
