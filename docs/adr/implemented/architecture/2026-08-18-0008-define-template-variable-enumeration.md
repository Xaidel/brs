# ADR-0008: Define the Template Variable enumeration for Certificate Templates

<!-- Location: docs/adr/implemented/architecture/2026-08-18-0008-define-template-variable-enumeration.md.
     The inline Status below must agree with {lifecycle}. -->

- **Date**: 2026-08-18
- **Status**: Implemented
- **Deciders**: Product and Engineering (solo)

## Context and Problem Statement

The [Core Engine PRD](../../../specs/core-engine/PRD.core-engine.md) §9.4 and §9.11 require Certificate Templates to auto-populate from Resident, Household, Certificate, Barangay Official, and Barangay Identity data using named Template Variables (e.g. `{{resident_name}}`, `{{age}}`, `{{address}}`, `{{purpose}}`, `{{date}}`, `{{captain_name}}`), while §14 explicitly defers the "Template Variable availability" — the complete enumeration — to the downstream TDD. The Phase 1 technical design ([tdd.phase-1.licensing-and-key-management.md](../../../specs/core-engine/phase-1/tdd.phase-1.licensing-and-key-management.md)) carries that as an open question (§15), scoped to Phase 3.

Two forces must be reconciled before any enumeration is fixed:

- **Disambiguation.** "Name" appears three times (resident, household head, captain) and "address" twice (resident's, barangay's). A flat namespace (`{{name}}`, `{{address}}`) is ambiguous; the PRD's examples already hint at source-prefixing (`{{resident_name}}`, `{{captain_name}}`).
- **Configurable positions vs. a closed catalog.** PRD §9.10 makes the position list barangay-configurable free text, but §9.11 requires templates to "reference Barangay Officials by position" (`{{captain_name}}`, `{{secretary_name}}`). A variable keyed on a free-text position name has no stable key to resolve against — yet a configurable-position requirement forbids hardcoding a fixed list of position *names*.

The audit-lock requirement (§9.4) also constrains the enumeration: whatever fills a variable at issuance must be *frozen* into the rendered certificate so later edits to a Resident, a Captain change, or a template change never alter an already-issued document.

## Decision Drivers

- **Audit-lock** (PRD §9.4): rendered content, including derived values like age, must be captured at issuance and never retroactively change.
- **Unambiguous by construction**: each variable name must resolve to exactly one field across all five sources.
- **Closed, developer-seeded catalog** — consistent with the RBAC Permission taxonomy ([ADR-0004](2026-08-17-0004-define-rbac-permission-taxonomy.md)): a barangay can pick from the enumerated set, never invent a variable, because a variable is only meaningful where a data field backs it.
- **Configurability of positions** (PRD §9.10) without breaking that closed catalog.
- **Low-to-moderate tech-literacy staff**: a missing or mistyped value must not silently produce garbage on a printed certificate.

## Decision

We will fix the Template Variable enumeration as follows.

### 1. Syntax: dotted `{{source.field}}`

A Template Variable is a brace-delimited, dotted token `{{source.field}}` — source is one of `resident`, `household`, `certificate`, `official`, `barangay`; field is a name in that source's enumeration below. This formalizes the source-prefix intent of the PRD's flat examples (`{{resident_name}}` → `{{resident.name}}`). The convention `.<source>.name` means that entity's display name (a person's assembled full name; the barangay's name).

### 2. Closed, developer-seeded catalog

The enumeration is closed. It maps 1:1 to the enumerable fields of the five source entities. New fields ship as new variables in a later release, never via barangay free-text — the same discipline as ADR-0004's Permission keys. A template referencing a token outside the catalog is rejected at configuration-save time, never at issuance (mirroring the strict-validation decision in [ADR-0007](2026-08-18-0007-define-control-number-format-token-grammar.md)).

### 3. Missing source data renders empty; nothing blocks issuance

Auto-fill is best-effort. A variable whose source value is absent (no occupation, no active Captain, no logo) renders as an empty string, and no variable blocks issuance. The filled-in preview makes empty results visible so the Secretary can catch a genuinely important blank before printing. A per-template "required variable" flag is a possible future option, not now.

### 4. Computed values are enumerated and frozen at issuance

Derived values are first-class variables, computed at the moment of issuance and frozen into the rendered text: `resident.name` (assembled full name), `resident.age` (from birthdate), and `household.head.name` (resolved head-of-household). This extends the audit-lock requirement to derived values — a certificate shows the age *at issuance*, not the resident's current age.

### 5. Barangay Officials are referenced by standard position keys

`BarangayOfficial` gains a nullable `position_key` (fixed enum, seeded with `captain` and `secretary`) alongside its fully-configurable free-text `position_title`. Templates reference officials only through the standard keys (`{{official.captain.name}}`, `{{official.secretary.name}}`); auto-selection resolves each key to the currently-Active official tagged with it, and renders empty (per §3) if none is tagged. The key set grows later (e.g. `treasurer`) as a backward-compatible catalog extension, never from barangay free-text. The free-text position list from §9.10 is unaffected.

### 6. Exactly one date variable

`{{certificate.issue_date}}` is the single date variable — for a certificate, "today" and "issue date" are the same moment. A fixed date string in body text is literal text. Date *formatting* styles ("this 5th day of August, 2026") are a Phase 3 layout concern, not additional variables.

### 7. The enumeration

Image variables (marked 🖼) are referenceable in the catalog but rendered by the layout engine (signature per §9.10's display mode; logos per §9.9), not by text substitution.

| Source | Variables |
| --- | --- |
| `resident` (13) | `name` ★, `first_name`, `middle_name`, `last_name`, `suffix`, `sex`, `birthdate`, `age` ★, `civil_status`, `occupation`, `contact_number`, `address`, `purok` |
| `household` (3) | `head.name` ★, `purok`, `address` |
| `certificate` (3) | `control_number`, `issue_date`, `purpose` |
| `official` (6) | `captain.name`, `captain.title`, `captain.signature` 🖼, `secretary.name`, `secretary.title`, `secretary.signature` 🖼 |
| `barangay` (7) | `name`, `address`, `municipality_city`, `province`, `contact_phone`, `contact_email`, `logo` 🖼 |

Total: 32 variables (29 text + 3 image). ★ = computed and frozen at issuance.

### 8. Deliberate exclusions

- **Sector Tags** (`Senior Citizen`, `PWD`, …) — not in §9.11's template-source list; they are a resident filter, not certificate prose.
- **`barangay.zip_code`, `barangay.facebook_url`, `barangay.website_url`** — not in §9.11's list; they are settings/UI-chrome, not certificate body text.
- **`barangay.municipality_logo` / `barangay.province_logo`** — §9.9 lists them, but §9.11's "logo references" is singular; the barangay's own logo is what renders on certificates.

## Alternatives Considered

### Flat syntax (`{{resident_name}}`, `{{captain_name}}`)

- Benefits: matches the PRD's example text literally.
- Costs and risks: no consistent rule for disambiguating the three "name" and two "address" fields; grows ad hoc prefixes as the catalog expands. Rejected in favor of dotted `{{source.field}}`.

### Dynamic position-name keys (`{{official.<position-name>.name}}`)

- Benefits: lets a barangay reference any configured position by name.
- Costs and risks: breaks the closed catalog (a typo'd or renamed position becomes a silently empty variable) and makes the enumeration unbounded. Rejected in favor of standard position keys (§5).

### Extensible (barangay-defined) variable catalog

- Benefits: maximum flexibility.
- Costs and risks: a variable with no backing field is meaningless; free-text invention reintroduces exactly the ambiguity and typo surface the enumeration exists to eliminate. Rejected in favor of a developer-seeded closed set.

### Missing data renders the literal placeholder (or blocks issuance)

- Benefits: makes an unfilled variable visually obvious on the raw template.
- Costs and risks: a literal `{{resident.occupation}}` printed on a real certificate is worse than a blank; blocking issuance on an optional field (occupation) frustrates routine work. Rejected in favor of empty-string rendering with preview visibility.

### Separate "today" and "issue date" variables

- Benefits: could distinguish a static "as of" date from the issuance date in body text.
- Costs and risks: for a certificate the two coincide; a second date variable invites templates that silently diverge from the audited issue date. Rejected in favor of one `{{certificate.issue_date}}`.

## Consequences

### Positive

- Phase 3's template renderer inherits a single, validated, closed enumeration with unambiguous names.
- The audit-lock requirement covers derived values (age, assembled names), not only stored fields.
- Configurable positions and the closed catalog coexist via standard position keys, satisfying §9.10 and §9.11 simultaneously.

### Negative

- `BarangayOfficial` gains a `position_key` field (§9.10) that the barangay must tag for `captain`/`secretary` to resolve; untagged officials render empty for those variables.
- A barangay that wants a variable for a field outside the enumeration (e.g. a Sector Tag on a certificate) must wait for a catalog extension rather than adding it themselves.

### Neutral / Risks

- The image variables (signature, logo) are enumerated but their rendering is governed by layout/§9.10, so their exact behavior is fixed by Phase 3's layout design, not this ADR.
- The catalog is deliberately minimal; a future release may extend it (new fields, new position keys, Sector Tags) backward-compatibly.

## Confirmation

- A template saved with a token outside the enumeration is rejected (Phase 3 Gate 1 validation test).
- Rendering freezes both stored and computed values at issuance; a later Resident edit or Captain change does not alter an already-issued certificate (Phase 3 audit-lock test).
- A missing source value renders as an empty string and does not block issuance.
- The catalog resolves each variable to exactly one field across all five sources (no ambiguous name).

## Relationships and References

- Refines the [Core Engine PRD](../../../specs/core-engine/PRD.core-engine.md) §9.4/§9.11 by fixing its "Template Variable availability" open question (§14) and formalizing the `{{source.field}}` syntax.
- Resolves the Template Variable enumeration open question in the [Phase 1 technical design §15](../../../specs/core-engine/phase-1/tdd.phase-1.licensing-and-key-management.md), which had scoped it to Phase 3.
- Builds on [ADR-0004](2026-08-17-0004-define-rbac-permission-taxonomy.md) (closed, developer-seeded catalog precedent), [ADR-0006](2026-08-17-0006-lock-sync-ready-schema-baseline.md) (the source entities' schema), and [ADR-0007](2026-08-18-0007-define-control-number-format-token-grammar.md) (strict config-time validation, `issue_date`/`control_number` it names).
- Owning spec: [Core Engine PRD](../../../specs/core-engine/PRD.core-engine.md) §9.4, §9.10, §9.11.
- Glossary: [CONTEXT.md](../../../../CONTEXT.md) (Template Variable — added).
