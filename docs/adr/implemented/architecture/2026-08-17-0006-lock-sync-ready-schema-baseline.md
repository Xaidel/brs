# ADR-0006: Lock the sync-ready schema baseline (UUIDv7 + shared columns)

<!-- Location: docs/adr/implemented/architecture/2026-08-17-0006-lock-sync-ready-schema-baseline.md.
     The inline Status below must agree with {lifecycle}. -->

- **Date**: 2026-08-17
- **Status**: Implemented
- **Deciders**: Product and Engineering (solo)

## Context and Problem Statement

The [Master PRD](../../../specs/master/PRD.md) §8 (Cross-Cutting Requirements) binds every table storing product-owned records to a shared set of "sync-ready schema conventions," pointing to [Appendix A](../../../specs/master/appendix-a-sync-ready-schema-and-municipal-integration.md) §3 for the concrete columns: an `id` primary key, `barangay_code`, `updated_at`, and `sync_status`. The stated reason to fix these now, while Municipal Sync itself remains Exploratory and unscheduled, is that retrofitting them onto an already-populated production database (real resident records, in real barangays) is far more disruptive than including them from each package's first schema migration.

Appendix A leaves three things underspecified or, on one point, already superseded by a later map decision:

- **`id` type**: Appendix A's literal text says "a UUIDv4 (or ULID) string primary key." This map already settled UUIDv7 as the record-identifier scheme during charting and confirmed a concrete crate via [ticket #8](https://github.com/Xaidel/brs/issues/8) (the `uuid` crate, `v7`+`serde` features, `Uuid::now_v7()`). This ADR is where that supersession gets formally recorded against Appendix A's text.
- **`sync_status` value set**: Appendix A names `PENDING`/`SYNCED` but does not say whether that is exhaustive.
- **Which tables the four columns apply to**: Appendix A says "every product-owned table" without drawing the boundary against Core Engine's administrative/security tables (Staff Account, Role, Permission, Feature Flags, License state) or reference/config tables (Purok/Sitio/Zone, Document Type templates) or the Audit Trail.

Two further points are not named by Appendix A at all and need fixing before any package's first migration can be written: whether a `created_at` column belongs alongside `updated_at`, and which layer is responsible for stamping these timestamps given the hexagonal crate boundary [ADR-0002](2026-08-17-0002-fix-core-engine-workspace-crate-topology.md) already drew (`app_core` framework-free, `infra_persistence` the SQLite adapter).

## Decision Drivers

- **Retrofitting is more expensive than including from the start** (Master PRD §8's own stated rationale) — this baseline must be fixed before the Core Engine technical design and any package's first migration, not after.
- **Appendix A's municipal-hub model only ever moves resident-linked business data across a barangay boundary** ("Municipal services link to Resident UUID"; "a future municipal aggregator... never becomes a second writer of resident data" — Master PRD line 54) — the inclusion boundary for these columns should track that, not sweep in unrelated administrative data.
- **Municipal Sync is Exploratory and out of this map's scope** (Master PRD §7, Out of Scope) — this ADR must not invent export-pipeline mechanics (extra `sync_status` states, conflict handling) that belong to a future Municipal Sync PRD.
- **Consistency with the already-locked crate topology** ([ADR-0002](2026-08-17-0002-fix-core-engine-workspace-crate-topology.md)) — timestamp-stamping responsibility must respect the `app_core`/`infra_persistence` boundary, not blur it.
- **Testability** (`tdd` skill) — anything that stamps a business-meaningful timestamp must be fake-able in unit tests, not left to database-side defaults.
- **No re-litigating already-settled decisions** — the UUIDv7 crate choice ([ticket #8](https://github.com/Xaidel/brs/issues/8)) and the RBAC Permission taxonomy ([ADR-0004](2026-08-17-0004-define-rbac-permission-taxonomy.md)) are inputs here, not open questions.

## Decision

We will fix the sync-ready schema baseline as follows.

### 1. The Shared Schema Columns

Every table that qualifies under the inclusion principle below (§2) carries exactly these five columns:

| Column | Type | Notes |
| --- | --- | --- |
| `id` | TEXT | UUIDv7, via the `uuid` crate (`v7`+`serde` features, `Uuid::now_v7()`) per [ticket #8](https://github.com/Xaidel/brs/issues/8). Supersedes Appendix A §3's literal "UUIDv4 (or ULID)" text — UUIDv7's per-process monotonic ordering was already the map's charting-time decision; this is where it becomes binding schema. |
| `barangay_code` | TEXT | PSGC code identifying the owning barangay, per Appendix A §3. |
| `created_at` | TEXT (ISO-8601 UTC) | New column, not named by Appendix A — see §3 below. |
| `updated_at` | TEXT (ISO-8601 UTC) | Per Appendix A §3. |
| `sync_status` | TEXT | Exactly `PENDING` \| `SYNCED` — see §4 below. |

### 2. Table inclusion principle

The five columns apply only to tables holding barangay-owned records that are candidates for a future resident-linked `.bmssync` export: Resident, Household, Certificate/Document, and future module business objects (KP Blotter Case, Treasury Transaction, Business Permit).

They do **not** apply to Core Engine's administrative, security, or configuration tables: Staff Account, Role, the fixed Permission catalog ([ADR-0004](2026-08-17-0004-define-rbac-permission-taxonomy.md)), Feature Flags ([ADR-0003](2026-08-17-0003-define-feature-flag-taxonomy.md)), License state ([ADR-0005](2026-08-17-0005-lock-core-engine-licensing-key-management-mechanics.md)), Purok/Sitio/Zone reference entries, Document Type templates, or the Audit Trail. These tables use whatever columns their own domain needs instead.

Two cases are worth calling out because they are per-barangay mutable yet still excluded:

- **Purok/Sitio/Zone and Document Type templates** — a barangay adds, renames, or disables its own entries, but they are local configuration, not resident data that would ever cross a barangay boundary in Appendix A's model.
- **Audit Trail** — individually attributable and immutable (Master PRD §36; Core Engine PRD acceptance criterion: "no entry can be edited or deleted"), but not resident-exportable data under Appendix A's municipal-hub model. Its own timestamping (a plain `created_at`-equivalent, since rows are never updated) is left to the Core Engine technical design, outside this ADR's scope.

### 3. `created_at` added alongside `updated_at`

Appendix A names only `updated_at`. This ADR adds `created_at` now, on the same rationale Master PRD §8 already gives for the whole baseline: retrofitting a column onto populated production tables is materially more expensive than including it in the first migration.

### 4. `sync_status` is exactly `PENDING` \| `SYNCED`

No third state is added. A record's own business lifecycle (archived, voided, disabled, etc.) lives in that table's own columns, separate from this sync-bookkeeping field. Since no BMS package hard-deletes records — residents/households are archived, certificates are voided, never removed (Core Engine PRD §9.2, §9.4) — there is no "deleted" state competing for a third value either.

### 5. Timestamp ownership: an `app_core` `Clock` port

`app_core` stamps both `created_at` and `updated_at` via an injected `Clock` port trait at the moment a use case runs. Neither column is set by SQLite `DEFAULT CURRENT_TIMESTAMP` nor by a trigger in `infra_persistence`. The concrete `Clock` adapter's crate placement and construction are deferred to the Core Engine technical design ([ticket #9](https://github.com/Xaidel/brs/issues/9)), consistent with how [ADR-0001](2026-08-17-0001-lock-core-engine-application-stack.md) and [ADR-0005](2026-08-17-0005-lock-core-engine-licensing-key-management-mechanics.md) deferred crate/parameter pinning to the same design.

## Alternatives Considered

### `sync_status` with additional states now (e.g. `EXCLUDED`, `CONFLICT`)

- Benefits: might save a future migration if Municipal Sync eventually needs richer export bookkeeping.
- Costs and risks: Municipal Sync is Exploratory, unscheduled, and explicitly out of this map's scope (Master PRD §7); any state beyond `PENDING`/`SYNCED` would be speculation about an export pipeline's mechanics that doesn't exist yet, encoded into a column every business table carries. Rejected in favor of the two states Appendix A already names; a future Municipal Sync PRD is free to add states via its own migration when the pipeline is real.

### Broad table-inclusion principle (every product table gets the columns)

- Benefits: one rule, no borderline cases to reason about; matches Appendix A's literal "every product-owned table" wording most directly.
- Costs and risks: Appendix A's own municipal-hub model never moves administrative data (Staff Accounts, Roles, Permissions, License state) across a barangay boundary, so `barangay_code`/`sync_status` would be dead weight on those tables. It also cuts against this map's own earlier decision ([ADR-0004](2026-08-17-0004-define-rbac-permission-taxonomy.md)) to keep the RBAC Permission catalog fixed, developer-seeded, and identical across every installation — a table with no meaningful notion of "this barangay's copy" has no meaningful `barangay_code` either. Rejected in favor of the narrower, export-candidate-scoped principle.

### SQLite `DEFAULT CURRENT_TIMESTAMP` / triggers for `created_at`/`updated_at`

- Benefits: one fewer port/trait to define and inject; timestamping "just happens" at the storage layer.
- Costs and risks: untestable without a real wall clock in unit tests, and it puts business-meaningful data — `updated_at` feeds `sync_status` semantics and future export ordering — inside `infra_persistence` rather than `app_core`, which is exactly the boundary [ADR-0002](2026-08-17-0002-fix-core-engine-workspace-crate-topology.md) drew between the framework-free domain crate and its SQLite adapter. Rejected in favor of an `app_core`-owned `Clock` port.

### `updated_at` only, no `created_at` (matching Appendix A literally)

- Benefits: exactly matches Appendix A's named columns; no scope creep beyond what the PRD asked for.
- Costs and risks: `created_at` is a near-universal need (record age, sort-by-creation, audit correlation) and cheap to add today; omitting it now only to add it later triggers the exact retrofit cost Master PRD §8 cites as the reason to fix this baseline in the first place. Rejected in favor of adding it now.

## Consequences

### Positive

- Every future package (KP Blotter, Treasury, Business Permits) inherits one settled answer for identifiers and change-tracking columns on its business tables, with no ambiguity about which of its own tables qualify.
- The `id` supersession is now recorded where a future reader will look for it (this ADR) instead of only living in a closed research ticket and the map's own notes.
- `app_core` gains a concrete, testable `Clock` port contract that the Core Engine technical design can build against immediately.
- The narrow inclusion principle keeps `barangay_code`/`sync_status` meaningful everywhere they appear, rather than diluting them onto tables (Permission catalog, Feature Flags) that are identical across every installation by design.

### Negative

- Two parallel column sets now exist in the schema going forward — the five Shared Schema Columns on export-candidate business tables, and whatever ad hoc timestamp/identifier columns administrative tables use — which a future contributor must learn to distinguish rather than applying one rule everywhere.
- The Audit Trail's own timestamping is explicitly left open by this ADR rather than fixed here, so it remains a small piece of unfinished baseline work carried into the Core Engine technical design.

### Neutral / Risks

- Because Municipal Sync itself is unbuilt, the `sync_status`/`barangay_code` columns on business tables will sit unused (always `PENDING`, always the one local barangay's code) until that capability exists — an accepted cost of "build the hooks now, use them later," not a defect of this decision.
- If a future package needs a business table that doesn't cleanly fit "resident-linked business record" (e.g. a cross-cutting audit-adjacent table), its owning technical design will need to argue inclusion or exclusion against this ADR's principle rather than finding a table-by-table answer already written down here.

## Confirmation

- Every Resident, Household, and Certificate/Document table migration includes exactly the five Shared Schema Columns with the types in §1.
- No Staff Account, Role, Permission, Feature Flag, License, Purok/Sitio/Zone, Document Type template, or Audit Trail table carries `barangay_code` or `sync_status` (code/schema review against this ADR's §2 exclusion list).
- `sync_status` is constrained (application-level validation or a `CHECK` constraint) to exactly `PENDING`/`SYNCED`.
- `created_at` and `updated_at` are set exclusively through an `app_core` `Clock` port in use-case code — no migration defines a SQLite `DEFAULT CURRENT_TIMESTAMP` or an `UPDATE` trigger on any Shared-Schema-Columns table.
- `id` values on new records are UUIDv7 (verifiable by decoding the leading 48 bits as a plausible Unix millisecond timestamp), never UUIDv4.

## Relationships and References

- Refines [Master PRD](../../../specs/master/PRD.md) §8 (Cross-Cutting Requirements) and [Appendix A](../../../specs/master/appendix-a-sync-ready-schema-and-municipal-integration.md) §3 (Shared Schema Conventions) by fixing their open points; supersedes Appendix A §3's literal "UUIDv4 (or ULID)" text for `id` with UUIDv7.
- Builds on [ticket #8](https://github.com/Xaidel/brs/issues/8)'s resolution (UUIDv7 crate selection — `docs/research/uuidv7-crate-selection.md`).
- Refines [ADR-0002](2026-08-17-0002-fix-core-engine-workspace-crate-topology.md) (workspace crate topology) by fixing the `app_core`/`infra_persistence` timestamping boundary.
- Consistent with, and excludes the tables owned by, [ADR-0003](2026-08-17-0003-define-feature-flag-taxonomy.md) (Feature Flag taxonomy), [ADR-0004](2026-08-17-0004-define-rbac-permission-taxonomy.md) (RBAC Permission taxonomy), and [ADR-0005](2026-08-17-0005-lock-core-engine-licensing-key-management-mechanics.md) (licensing mechanics).
- Feeds the Core Engine technical design ([ticket #9](https://github.com/Xaidel/brs/issues/9)) and the not-yet-written KP Blotter/Treasury/Business Permits technical designs, which apply the Shared Schema Columns to their own tables.
- Glossary: [CONTEXT.md](../../../../CONTEXT.md) — "Shared Schema Columns," "sync_status," "Clock."
- Supporting issue: [Record the sync-ready schema baseline (UUIDv7 + shared columns)](https://github.com/Xaidel/brs/issues/7).
