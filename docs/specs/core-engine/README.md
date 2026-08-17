# Core Engine Package

Status: Proposed — pending Phase 1 technical spike

Owner: Product and Engineering (solo)

## Reading Order

1. [PRD.core-engine.md](PRD.core-engine.md) — normative product contract for this release.
2. [appendix-a-technical-architecture-direction.md](appendix-a-technical-architecture-direction.md) — informative technology direction and the mandatory Phase 1 feasibility spike.
3. [appendix-b-key-derivation-and-recovery.md](appendix-b-key-derivation-and-recovery.md) — informative rationale behind the PRD's data-protection requirements.
4. [appendix-c-license-reissuance-sop.md](appendix-c-license-reissuance-sop.md) — informative support process satisfying the PRD's reissuance requirement.

## Source Precedence

`PRD.core-engine.md` is the sole normative source for this package's product scope. All three appendices are supporting/informative only: they explain rationale and suggest mechanics, but must not be read as expanding or narrowing what the PRD requires. If a downstream TDD needs to deviate from an appendix's suggested mechanics while still satisfying the PRD's binding requirement, that is permitted — the appendix does not bind implementation choice.

## Supersession

This package supersedes the following prior documents, whose content has been fully incorporated here:

- `docs/PRD.md` (flat PRD v2.2, single-document draft — now split across the [Master PRD](../master/PRD.md) and this package)
- `docs/.init-plan` (PRD v1.0 draft)

## Phase Map

This package covers the first three phases of the original product roadmap. Later phases belong to their own packages (see the [Master PRD feature map](../master/PRD.md#10-feature-map)) and are out of scope here.

| Phase | Milestone | Core Deliverable |
|---|---|---|
| Phase 1 | Tech Spike & Core Licensing Engine — [designed](phase-1/tdd.phase-1.tech-spike-and-licensing.md) | SQLCipher + FTS5 compilation spike, Ed25519 key verification, Credential Manager integration for encryption key derivation (per [Appendix B](appendix-b-key-derivation-and-recovery.md)), first-run Recovery Code generation. See [Phase 1 TDD](phase-1/tdd.phase-1.tech-spike-and-licensing.md) and its [spike results log](phase-1/spike-results.md). |
| Phase 2 | Core Engine & RBAC MVP | Per-person staff accounts (Argon2id), Resident Registry (FTS5 search), Household Registry, transactional Certificate Generator with atomic Control Number sequencing. |
| Phase 3 | Plug & Play UI & Retention Engine | License Settings screen, dynamic feature-flag UI wrappers, automated rolling 14-snapshot backup engine. |

Phases 4 (add-on modules) and 5 (municipal export suite) belong to the [KP Blotter](../kp-blotter/README.md), [Treasury](../treasury/README.md), [Business Permits](../business-permits/README.md) packages and the Exploratory municipal-sync direction respectively — see the [Master PRD](../master/PRD.md).
