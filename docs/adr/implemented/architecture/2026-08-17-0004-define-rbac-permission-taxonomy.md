# ADR-0004: Define the RBAC Permission taxonomy and seed roles

<!-- Location: docs/adr/implemented/architecture/2026-08-17-0004-define-rbac-permission-taxonomy.md.
     The inline Status below must agree with {lifecycle}. -->

- **Date**: 2026-08-17
- **Status**: Implemented
- **Deciders**: Product and Engineering (solo)

## Context and Problem Statement

The [Core Engine PRD](../../../specs/core-engine/PRD.core-engine.md) §9.1 fixes the shape of Role-Based Access Control — a `Role` is a named, editable collection of individually toggleable `Permission`s, each Staff Account holds exactly one active Role, four seeded default Roles ship (Admin/Secretary, Encoder, Treasurer, Read-Only/Captain), and v2.4 (§14) re-resolved the product to full custom RBAC: Admins may create unlimited Custom Roles with their own Permission selection, and the seeded defaults are "fully editable and not specially privileged." The PRD's data model (§11) already fixes the schema — `Role { id, name, is_seeded, enabled, ... }`, `Permission { id, key, description, ... }`, `RolePermission { id, role_id, permission_id, granted }` — and a safety constraint: at least one active Role must always retain `staff.manage`, refusing any edit that would leave zero Staff Accounts able to manage Roles and Staff.

What the PRD explicitly defers (§14, "Permission taxonomy... is TDD-level detail") is the exact set of Permission key strings and the seeded Role→Permission mapping. The PRD offers only an illustrative, non-exhaustive example list (`resident.create`, `resident.view`, `household.edit`, `certificate.issue`, `staff.manage`, `settings.manage`, `license.manage`, `backup.manage`, `audit.view`) and a per-role narrative (§7 Actors, §8 User Stories) that is suggestive but not a complete, implementable contract. Without a fixed catalog and seed matrix, neither the `RolePermission` seed migration nor any enforcement check in `app_core` has a concrete contract to build against.

## Decision Drivers

- Give the `Role`/`Permission`/`RolePermission` schema (already fixed by the PRD) a concrete, implementable Permission catalog and seed data.
- Preserve v2.4's full-custom-RBAC intent: seeded Roles must be ordinary editable data, and the catalog must support barangays picking a *different* combination than the defaults — not just re-deriving the four narrative roles.
- Keep the catalog a fixed, developer-controlled taxonomy (mirroring [ADR-0003](2026-08-17-0003-define-feature-flag-taxonomy.md)'s `FeatureFlag` precedent) rather than something end users can extend at runtime, so enforcement code can pattern-match against a known enum-like set.
- Respect the PRD's explicit narrowing of Treasurer's Core Engine scope to "minimal certificate/payment-matching fields, not full Resident profiles" without conflating a field-level projection concern with a resource-action RBAC gate.
- Don't invent product capability the PRD never described (e.g., household archiving) merely to fill out a taxonomy symmetrically.
- Leave room for KP Blotter, Treasury, and Business Permits — each "Not yet specified" on the parent map — to extend the same catalog later without a taxonomy rework.

## Decision

We will define the RBAC Permission taxonomy as follows:

1. **Key grammar**: `<resource>.<action>` — both segments lowercase snake_case, resource singular (`resident`, not `residents`), action drawn from a controlled verb vocabulary: `{view, create, edit, archive, draft, issue, void, manage}`. This is the exact style the PRD's own illustrative examples already use.

2. **Fixed, developer-seeded catalog**: `Permission` rows are seeded by the application (initial install and, later, module-install migrations), never created by an Admin at runtime. Custom Roles (v2.4) select an arbitrary subset of *existing* Permission rows; they never introduce a new Permission key. This mirrors ADR-0003's `FeatureFlag` precedent — the taxonomy is a developer-controlled vocabulary, not open-ended admin input.

3. **The complete Core Engine catalog** (19 keys):

   - Business entities (granular verbs): `resident.view`, `resident.create`, `resident.edit`, `resident.archive`, `household.view`, `household.create`, `household.edit`, `certificate.view`, `certificate.draft`, `certificate.issue`, `certificate.void`
   - Admin-configuration (single `manage` verb per resource): `staff.manage`, `license.manage`, `backup.manage`, `purok.manage`, `official.manage` (Barangay Officials), `document_type.manage` (Document Types + Certificate Templates + Control Number Format), `barangay_profile.manage` (Barangay Identity + Theme + Dashboard widget config)
   - Audit: `audit.view`

   This splits the PRD's single illustrative `settings.manage` into four keys along an existing pattern already visible in the PRD's own examples: legally consequential admin-configuration resources (`document_type.manage`, affecting issued-certificate content) are kept separate from low-risk cosmetic/identity resources (`barangay_profile.manage`) and clerical config (`purok.manage`, `official.manage`) — each independently toggleable on a Custom Role, which is the entire point of v2.4's full-custom-RBAC re-resolution.

   Two keys not in the PRD's illustrative list were added to close gaps: `household.view` (the PRD's example list had `household.create`/`household.edit` but no view key, despite Read-Only/Captain needing to browse households per §5 Actors) and `certificate.view` (needed for Read-Only/Captain to review issued certificates and for Treasurer to see certificates at all, per driver 4). No `household.archive` was added — the PRD never describes household archiving as a product capability, and this ADR does not invent one.

4. **Treasurer's Core Engine field restriction stays outside the Permission model**: the PRD's "minimal certificate/payment-matching fields, not full Resident profiles" scoping is a response-shaping/DTO-projection concern, not a distinct RBAC gate. Treasurer's Core Engine seed grants `certificate.view` (full detail at the RBAC layer); the eventual Treasury module's technical design owns how much of that certificate is actually rendered to a Treasurer-role caller.

5. **Seeded Role → Permission mapping**:

   | Permission | Admin/Secretary | Encoder | Treasurer | Read-Only/Captain |
   |---|---|---|---|---|
   | `resident.view` | ✅ | ✅ | | ✅ |
   | `resident.create` | ✅ | ✅ | | |
   | `resident.edit` | ✅ | ✅ | | |
   | `resident.archive` | ✅ | ✅ | | |
   | `household.view` | ✅ | ✅ | | ✅ |
   | `household.create` | ✅ | | | |
   | `household.edit` | ✅ | | | |
   | `certificate.view` | ✅ | ✅ | ✅ | ✅ |
   | `certificate.draft` | ✅ | ✅ | | |
   | `certificate.issue` | ✅ | | | |
   | `certificate.void` | ✅ | | | |
   | `staff.manage` | ✅ | | | |
   | `license.manage` | ✅ | | | |
   | `backup.manage` | ✅ | | | |
   | `purok.manage` | ✅ | | | |
   | `official.manage` | ✅ | | | |
   | `document_type.manage` | ✅ | | | |
   | `barangay_profile.manage` | ✅ | | | |
   | `audit.view` | ✅ | | | |

   Encoder's household access is view-only — the PRD attributes household grouping ("group residents into households, designate a head of household") only to the Secretary, not the Encoder, so Encoder gets enough visibility for data-entry context without household-management rights. Read-Only/Captain does not get `audit.view` in the seed — "review resident/household records and dashboard reports" is about product data, not the internal audit ledger, which the PRD attributes only to Admin/Secretary's management cluster; a barangay that wants its Captain to see the audit log can build that as a Custom Role.

6. **Naming convention binds future modules**: KP Blotter, Treasury, and Business Permits will each eventually seed their own Permission rows into the same catalog (e.g. `case.create`, `receipt.issue`). This ADR fixes only the *convention* those future keys must follow — the same `<resource>.<action>` grammar and fixed-seeding discipline — without enumerating any module-specific key, mirroring how ADR-0003 fixed `FeatureFlag` naming without anticipating unapproved modules. Those modules' own technical designs own their exact keys and seed mappings.

## Alternatives Considered

### Single `settings.manage` catch-all (as the PRD's illustrative list literally shows)

- Benefits: fewer keys, matches the PRD's example text verbatim.
- Costs and risks: directly undercuts v2.4's stated reason for re-resolving to full custom RBAC — a barangay cannot grant "edit our Puroks" without also granting "edit our legal certificate templates and control-number formats." Rejected in favor of splitting along the resource's actual risk profile.

### Field-level Permission variant for Treasurer (`certificate.view_minimal` vs `certificate.view`)

- Benefits: makes the field-trim restriction visible and enforceable at the RBAC layer itself, with no separate mechanism needed.
- Costs and risks: conflates two different concerns — *which resource actions a Role may perform* vs. *how much of a resource's data a given caller may see* — and would require a parallel `_minimal` variant anywhere a future field-scoped role appears, ballooning the catalog. Rejected; the projection concern is deferred to the Treasury module's technical design, which already owns "the fields needed to match a fee payment to a certificate."

### Admin-invented Permission keys (fully dynamic catalog)

- Benefits: maximum flexibility — a barangay could express any access rule it wants without waiting for a developer-shipped key.
- Costs and risks: no fixed vocabulary for `app_core` enforcement code to pattern-match against; every check would need to resolve an arbitrary string at runtime with no compile-time or catalog-level guarantee it corresponds to a real gate. Rejected — the PRD's "custom Permission set" language already means selecting from existing Permissions, not inventing new ones, consistent with ADR-0003's precedent.

### Symmetric CRUD across every resource (add `household.archive`, `resident.void`-style parity everywhere)

- Benefits: a uniformly shaped catalog is easier to reason about and slightly more future-proof.
- Costs and risks: invents product capability (household archiving) the PRD never described, which is this ADR's job to avoid — RBAC taxonomy should reflect real actions, not manufacture symmetry. Rejected.

## Consequences

### Positive

- `app_core`'s Role/Permission/RolePermission seed migration has a concrete 19-key catalog and a concrete seed matrix to build against — no more "TDD-level detail" placeholder.
- The four admin-configuration keys (`purok.manage`, `official.manage`, `document_type.manage`, `barangay_profile.manage`) give v2.4's Custom Roles genuine fine-grained control, matching the PRD's own stated reason for moving off fixed roles.
- Treasurer's Core Engine seed is deliberately thin (`certificate.view` only), correctly reflecting that most of Treasurer's described capability only activates once the Treasury module is licensed — nothing in Core Engine's seed pretends otherwise.
- Future modules have a fixed naming convention to extend, keeping the eventual full-product Permission catalog internally consistent.

### Negative

- Nineteen keys is more upfront enumeration than the PRD's four-key illustrative list; every seed migration and any documentation referencing "the Permission taxonomy" must track all nineteen, not a shorthand.
- Splitting `settings.manage` into four keys means four RolePermission rows must be kept in sync wherever the old single-key mental model might have been assumed elsewhere in existing documents.

### Neutral / Risks

- If a future module needs a field-level access concern (à la Treasurer's minimal-fields restriction) that can't be cleanly deferred to a DTO/projection layer, this taxonomy's assumption that Permission gates are resource-action-only, never field-scoped, will need revisiting.
- The `document_type.manage`/`official.manage`/`barangay_profile.manage`/`purok.manage` split is a judgment call about risk grouping, not a PRD-mandated boundary; a future barangay pattern of use might argue for a different split (e.g., separating Theme from Barangay Identity within `barangay_profile.manage`).

## Confirmation

- The `Permission` table is seeded, at first migration, with exactly the 19 keys listed in the Decision section, each with a human-readable `description`.
- The four seed `Role` rows (`is_seeded = true`) have `RolePermission` grants matching the table in the Decision section exactly.
- No application code path allows creating a `Permission` row outside a developer-authored migration.
- `app_core`'s RBAC enforcement checks a resource-action Permission key only — no code path checks a field-level or `_minimal`-style variant.

## Relationships and References

- Refines [ADR-0002 (workspace crate topology)](2026-08-17-0002-fix-core-engine-workspace-crate-topology.md) by fixing `app_core`'s RBAC seed contract.
- Follows the developer-controlled-taxonomy precedent set by [ADR-0003 (Feature Flag taxonomy)](2026-08-17-0003-define-feature-flag-taxonomy.md); the two taxonomies are deliberately kept distinct (see [CONTEXT.md](../../../../CONTEXT.md) — Feature Flag's `_Avoid_: Permission`).
- Owning spec: [Core Engine PRD](../../../specs/core-engine/PRD.core-engine.md) §9.1 (Authentication & RBAC), §5/§7 (Actors), §11 (data model: `Role`, `Permission`, `RolePermission`), §14 (Permission taxonomy deferred as TDD-level detail).
- Glossary: [CONTEXT.md](../../../../CONTEXT.md) (Permission, Role).
- Supporting issue: [Define the RBAC Permission taxonomy and seed roles](https://github.com/Xaidel/brs/issues/5).
