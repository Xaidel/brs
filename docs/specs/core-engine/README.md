# Core Engine Package

Status: Proposed

Owner: Product and Engineering (solo)

## Reading Order

1. [PRD.core-engine.md](PRD.core-engine.md) — normative product contract for this release.
2. [appendix-a-technical-architecture-direction.md](appendix-a-technical-architecture-direction.md) — technology direction, promoted to normative by [ADR-0001](../../adr/implemented/architecture/2026-08-17-0001-lock-core-engine-application-stack.md). The mandatory feasibility spike it named has already run and passed as [issue #10](https://github.com/Xaidel/brs/issues/10).
3. [appendix-b-key-derivation-and-recovery.md](appendix-b-key-derivation-and-recovery.md) — key-derivation/recovery rationale, promoted to normative on its fixed points by [ADR-0005](../../adr/implemented/architecture/2026-08-17-0005-lock-core-engine-licensing-key-management-mechanics.md).
4. [appendix-c-license-reissuance-sop.md](appendix-c-license-reissuance-sop.md) — informative support process satisfying the PRD's reissuance requirement.
5. [ADR-0001](../../adr/implemented/architecture/2026-08-17-0001-lock-core-engine-application-stack.md) through [ADR-0006](../../adr/implemented/architecture/2026-08-17-0006-lock-sync-ready-schema-baseline.md) — the six locked architecture decisions every Core Engine technical design operates under.
6. [phase-1/tdd.phase-1.licensing-and-key-management.md](phase-1/tdd.phase-1.licensing-and-key-management.md) — the current normative technical design (Phase 1 only; see Phase Map below).

## Source Precedence

`PRD.core-engine.md` is the sole normative source for this package's product scope. The three appendices explain rationale and suggest mechanics; ADR-0001 and ADR-0005 promote specific points within Appendices A and B to normative (see each ADR's Relationships section for exactly which points). Where a downstream TDD needs to deviate from an appendix's suggested mechanics not fixed by an ADR while still satisfying the PRD's binding requirement, that is permitted — the appendix does not bind implementation choice beyond what an ADR has locked.

## Supersession

This package supersedes the following prior documents, whose content has been fully incorporated here:

- `docs/PRD.md` (flat PRD v2.2, single-document draft — now split across the [Master PRD](../master/PRD.md) and this package)
- `docs/.init-plan` (PRD v1.0 draft)

## Phase Map

This package covers the first three phases of the original product roadmap. Later phases belong to their own packages (see the [Master PRD feature map](../master/PRD.md#10-feature-map)) and are out of scope here.

| Phase | Milestone | Core Deliverable | Status |
|---|---|---|---|
| Phase 1 | Licensing & Key-Management Core | Offline Ed25519 License Key validation, database-encryption-key bootstrap/derivation, Recovery-Code-driven key recovery. See [phase-1/tdd.phase-1.licensing-and-key-management.md](phase-1/tdd.phase-1.licensing-and-key-management.md). | Technical design normative; build not started. |
| Phase 2 | Core Engine & RBAC MVP | RBAC enforcement (ADR-0004's 19-key catalog + seed matrix), Resident Registry (FTS5 search), Household Registry, transactional Certificate Generator with atomic Control Number sequencing, the `AuditLogEntry` entity. | Technical design not yet written. |
| Phase 3 | Plug & Play UI & Retention Engine | License Settings screen + dynamic feature-flag UI reactivity, automated rolling 14-snapshot backup engine (`infra_backup`), Barangay Identity/Officials/Templates/Purok/Theming/Dashboard. | Technical design not yet written. |

The technical feasibility spike originally scoped to Phase 1 (bundled SQLCipher + FTS5 compilation, search performance, installer size) ran as a standalone ticket ahead of any phase TDD — see [issue #10](https://github.com/Xaidel/brs/issues/10) (installer size unverified, deferred to [issue #17](https://github.com/Xaidel/brs/issues/17)) — rather than being gated inside Phase 1's own document.

### Phase dependency matrix

| Phase | Required predecessor | Capabilities unlocked | Explicitly deferred |
|---|---|---|---|
| Phase 1 | The six locked ADRs (ADR-0001–0006) and the stack spike (issue #10) | `LicenseGrant` validation/persistence, `DatabaseEncryptionKey` bootstrap/derivation/recovery, the `Clock` port and its `SystemClock` adapter placement | RBAC enforcement, all business-record domains, the Audit Trail entity, backup scheduling, UI wiring |
| Phase 2 | Phase 1's `Clock` port and crate topology conventions | Authenticated, permission-checked Resident/Household/Certificate CRUD; immutable Audit Trail | Barangay configuration/branding, theming, dashboard, backup engine |
| Phase 3 | Phase 1's License/Feature-Flag persistence and `bootstrap.json` location; Phase 2's data domains to back up | License/Feature-Flag UI reactivity, automated backup & manual export, Barangay Identity/Officials/Templates/Purok/Theming/Dashboard | — |

Phases 4 (add-on modules) and 5 (municipal export suite) belong to the [KP Blotter](../kp-blotter/README.md), [Treasury](../treasury/README.md), [Business Permits](../business-permits/README.md) packages and the Exploratory municipal-sync direction respectively — see the [Master PRD](../master/PRD.md).
