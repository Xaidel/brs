# ADR-0007: Define the Control Number Format token grammar and per-year sequence scope

<!-- Location: docs/adr/implemented/architecture/2026-08-18-0007-define-control-number-format-token-grammar.md.
     The inline Status below must agree with {lifecycle}. -->

- **Date**: 2026-08-18
- **Status**: Implemented
- **Deciders**: Product and Engineering (solo)

## Context and Problem Statement

The [Core Engine PRD](../../../specs/core-engine/PRD.core-engine.md) §9.4 fixes the binding requirements for Certificate Control Numbers — "unique, monotonically increasing, and free of gaps within its Document Type," rendered from a configurable Control Number Format pattern (e.g. `BC-2026-00001`), allocated atomically in a single transaction, and never renumbered — while §14 explicitly defers the "Control Number Format token grammar" (exact syntax and tokens) to the downstream TDD. The Phase 1 technical design ([tdd.phase-1.licensing-and-key-management.md](../../../specs/core-engine/phase-1/tdd.phase-1.licensing-and-key-management.md)) carries that as an open question (§15), scoped to Phase 2.

Two forces inside §9.4 collide unless resolved deliberately:

- **Real-world convention vs. the literal wording.** Philippine barangay control numbers conventionally reset each calendar year (`BC-2026-00001`, then `BC-2027-00001` in January) — this is what a COA/DILG auditor actually checks ("gapless within the year"). But "monotonically increasing … within its Document Type," read literally, forbids a yearly reset.
- **Configurability vs. uniqueness-by-construction.** A barangay-editable format string can, if left unconstrained, break the audit invariant: a format that omits the year would (under a yearly reset) emit `00001` in 2026 and again in 2027; a format that omits the sequence would stamp every certificate identically.

The grammar and the sequence's scope must be fixed together so that "gapless," "unique within type," "configurable," and "never renumber" all hold simultaneously.

## Decision Drivers

- **Audit-grade gaplessness** (PRD §9.4): no skipped or reused numbers, visible to a COA/DILG-style audit.
- **The real barangay numbering convention**: type prefix + calendar year + zero-padded sequence, resetting each year.
- **Uniqueness must hold by construction**, not by operator discipline — a malformed format must be rejected at configuration time, never discovered at issuance.
- **Atomic, monotonic allocation** inside the issuance transaction (PRD §9.4), independent of concurrency or interruption.
- **No renumbering ever**: a format change applies only to newly allocated numbers; previously issued certificates are never re-rendered.
- **Configurability without a "type code" concept**: each Document Type already owns its own format and its own sequence.

## Decision

We will fix the Control Number format and sequence scope as follows.

### 1. Sequence scope: per (Document Type × PHT calendar year)

The underlying allocation counter is keyed by Document Type and calendar year. The gapless, monotonic guarantee holds within (type, year); the full rendered Control Number — which includes the year — is what remains unique and unambiguous within a type across years. The PRD phrase "within its Document Type" is refined to "within its Document Type and year" to match the PRD's own `BC-2026-00001` example and real audit practice.

### 2. Calendar year basis: PHT, never UTC

Both the sequence key's year and the `{YYYY}` token's value use the Philippine Standard Time calendar year (UTC+8, no DST), not UTC. Keying on UTC would flip the sequence at 08:00 PHT on Jan 1, so a Jan-1-morning issuance would get the previous year's number and the previous year's sequence. The `Clock` port ([ADR-0006](2026-08-17-0006-lock-sync-ready-schema-baseline.md)) returns UTC only, so Phase 2 needs a local-calendar-date source distinct from it; the decision here is the *basis* (PHT year), not that mechanism.

### 3. Token grammar: brace-delimited, literal text, no type token

A Control Number Format is a string of literal characters with two token forms:

- `{YYYY}` — the 4-digit PHT year of issuance.
- `{N}` repeated for sequence width — `{NNNNN}` is a 5-digit zero-padded sequence, `{N}` alone is unpadded.

Everything outside braces is literal text (e.g. `BC-`, `-`). There is no `{TYPE}` token: each Document Type owns its own format and sequence, so a type prefix is ordinary literal text the barangay types. The token set is fixed to exactly `{YYYY}` and `{N…}`; unknown tokens are rejected at configuration time, so adding a token later (e.g. `{MM}`) is a backward-compatible grammar extension, not a rework.

### 4. Format constraints

Every Control Number Format must contain exactly one `{YYYY}` and exactly one `{N…}` (each at most once), and `{N…}` is capped at 10 digits. A format violating these is rejected when the Document Type is saved, never at issuance. This makes "unique within type" hold by construction.

### 5. Padding is a minimum, not a ceiling

The zero-pad width is a floor: at `9999 → 10000` the number renders full-width without error, truncation, or wrap. Gaplessness lives in the numeric sequence, not the display width.

### 6. Storage: integer counter + frozen rendered string

`ControlNumberSequence` (`doc_type_id`, `year`, `last_allocated`) holds the raw integer — the gapless source of truth — and `Certificate.control_number` holds the rendered string, frozen at issuance. A mid-year format change (e.g. `{YYYY}-{NNNN}` → `BC-{YYYY}-{NNNNN}`) keeps the counter running and applies the new format only to new issues: the 42nd certificate keeps `2026-0042`; the 43rd renders `BC-2026-00043`. This reconciles "no gaps" with "no renumbering" simultaneously.

### 7. Grammar validation is a domain value object

`ControlNumberFormat` is a Phase 2 `app_core::domain` value object whose smart constructor enforces §3–§5 (token set, arity, width cap). A malformed format fails at configuration-save time, never at issuance.

### 8. Default formats for the four seeded Document Types

The four seeded types ship with a uniform 5-digit width, editable by the barangay:

- Barangay Clearance — `BC-{YYYY}-{NNNNN}`
- Certificate of Indigency — `COI-{YYYY}-{NNNNN}`
- Certificate of Residency — `COR-{YYYY}-{NNNNN}`
- Certificate of Good Moral Character — `CGMC-{YYYY}-{NNNNN}`

## Alternatives Considered

### Sequence scope: all-time monotonic per type (literal PRD reading)

- Benefits: matches §9.4's wording most literally; one ever-increasing number across a type's entire history.
- Costs and risks: contradicts the PRD's own `BC-2026-00001` example and the real barangay yearly-reset convention auditors expect; a barangay cannot express "start over each January," which is standard practice. Rejected.

### Grammar: bare tokens `YYYY-NNNN` (PRD §14's informal hint)

- Benefits: matches the PRD's example text most directly.
- Costs and risks: a bare token cannot be distinguished from a literal of the same letters, so a prefix like `BC` is ambiguous and the grammar cannot be validated. Rejected in favor of brace delimiting.

### `{TYPE}` token for the prefix

- Benefits: a barangay could share one format across types and have the type code auto-filled.
- Costs and risks: redundant — each Document Type already owns its own format and sequence, so a "type code" concept is a second source of the same fact. Rejected.

### Fixed width (wrap or error on overflow)

- Benefits: visual width stays constant.
- Costs and risks: a busy front desk reaching `9999` could not issue (error) or would wrap into duplicate numbers — catastrophic, and forbidden by §9.4's no-skip/no-reuse requirement. Rejected in favor of minimum-width padding.

### Year basis: UTC

- Benefits: reuses the existing UTC-only `Clock` port directly, no new capability.
- Costs and risks: the yearly reset lands at 08:00 PHT on Jan 1, mislabeling Jan-1-morning issuances with the prior year. Rejected in favor of PHT.

## Consequences

### Positive

- Phase 2's Certificate Generator inherits a single, validated answer for Control Number syntax and scope instead of re-litigating it.
- "Gapless + unique + configurable + never renumber" now hold simultaneously, with uniqueness enforced at configuration time.
- The four seeded types ship with concrete, editable defaults that match real barangay practice.

### Negative

- Phase 2 must source a local calendar date (PHT) beyond the UTC-only `Clock` port — a small new capability (a `LocalCalendar` port, or a local-date method on `Clock`) that Phase 1 does not need.
- A yearly reset means "gapless" is a per-year property, not a property of a type's entire history; auditors and support staff must be told the reset is expected, not a defect.

### Neutral / Risks

- The counter is keyed by year, so a format that legitimately wants a continuous all-time number is not expressible; this is accepted as outside the product's real use case, and the grammar can be extended later if a barangay demands it.
- The token set is deliberately minimal (`{YYYY}`, `{N…}`); month/day tokens are left as a future grammar extension, not pre-built now.

## Confirmation

- A `ControlNumberFormat` smart constructor rejects any format without exactly one `{YYYY}`, without exactly one `{N…}`, with a `{N…}` wider than 10 digits, or containing an unknown token (Phase 2 Gate 1 domain-invariant tests).
- The sequence counter is keyed by (Document Type, PHT year); the issuance transaction increments it atomically and never reuses or skips a value.
- `Certificate.control_number` is immutable after issuance; no code path re-renders an issued Control Number.
- A mid-year format change produces a continuing numeric sequence rendered under the new format, with previously issued strings unchanged (Phase 2 test).

## Relationships and References

- Refines the [Core Engine PRD](../../../specs/core-engine/PRD.core-engine.md) §9.4 by fixing its "Control Number Format token grammar" open question (§14) and tightening "within its Document Type" to "within its Document Type and year."
- Resolves the Control Number Format token grammar open question in the [Phase 1 technical design §15](../../../specs/core-engine/phase-1/tdd.phase-1.licensing-and-key-management.md), which had scoped it to Phase 2.
- Builds on [ADR-0006](2026-08-17-0006-lock-sync-ready-schema-baseline.md): the `Clock` port it defines returns UTC only, which this ADR's PHT-year decision sits alongside; the `Certificate`/`ControlNumberSequence` tables it includes under the Shared Schema Columns carry the columns this decision populates.
- Owning spec: [Core Engine PRD](../../../specs/core-engine/PRD.core-engine.md) §9.4 (Certificate Generation & Control Number Sequencing), §11 (`ControlNumberSequence`, `Certificate`).
- Glossary: [CONTEXT.md](../../../../CONTEXT.md) (Control Number, Control Number Format — added).
