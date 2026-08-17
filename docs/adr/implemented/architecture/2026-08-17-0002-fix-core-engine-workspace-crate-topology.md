# ADR-0002: Fix the Core Engine workspace crate topology (app_core + infra_* + src-tauri)

<!-- Location: docs/adr/implemented/architecture/2026-08-17-0002-fix-core-engine-workspace-crate-topology.md.
     The inline Status below must agree with {lifecycle}. -->

- **Date**: 2026-08-17
- **Status**: Implemented
- **Deciders**: Product and Engineering (solo)

## Context and Problem Statement

[HADR-0002](../../../../backend_arch_docs/adr/HADR-0002-adopt-rust-hexon-architecture.md) and [HADR-0003](../../../../backend_arch_docs/adr/HADR-0003-define-crate-and-module-responsibilities.md) fix the reusable pattern: `app_core` (private domain + application, only `app_core::ports` public) depends on no workspace crate; self-contained `infra_*` deployment crates depend only on `app_core`'s public API. The reference template instantiates this with one example adapter crate, `crates/infra_local`, alongside `crates/app_core`.

[ADR-0001](../architecture/2026-08-17-0001-lock-core-engine-application-stack.md) locked the Core Engine technology stack — Tauri v2 desktop shell, SQLite with bundled SQLCipher + FTS5 via `rusqlite`, React/Tailwind/shadcn frontend, Ed25519 offline license verification, Argon2id password hashing, PBKDF2-HMAC-SHA256 database key derivation from an installation `system_secret` (deliberately excluding the Machine Hardware ID), and the OS-protected credential store via the `keyring` crate — but explicitly deferred concrete crate topology to "the crate-topology ticket and the Core Engine technical design." No scaffold exists yet in this repo; this is a fresh-start decision, not a migration of an existing layout.

The [Core Engine PRD](../../../specs/core-engine/PRD.core-engine.md) §2 already treats **Machine Hardware ID**, **License Key**, **Backup Snapshot**, **Recovery Code**, and **Audit Log Entry** as distinct terms. [Appendix B](../../../specs/core-engine/appendix-b-key-derivation-and-recovery.md) is explicit that data-at-rest encryption (which must survive a hardware change) and licensing (which must be hardware-bound) are deliberately separate concerns that must not be conflated — an earlier draft of the key-derivation design made exactly that mistake by deriving the SQLCipher key from the Machine Hardware ID, and Appendix B records backing it out.

## Decision Drivers

- Give `Cargo.toml` and the Core Engine technical design a fixed, concrete crate list instead of an open choice (per ADR-0001's deferral).
- Preserve the PRD's own separation between hardware-fingerprinting and licensing at the architecture level, not just in prose (Appendix B).
- Follow HADR-0003's "demonstrated reuse, not speculative sharing" bar for any crate that isn't a straightforward `app_core -> infra_*` leaf.
- Keep `app_core` free of Tauri, web, database, and OS-credential-store dependencies (HADR-0002/0003 conformance).
- Avoid workspace ceremony (extra crates) that has no demonstrated second consumer.

## Decision

We will use a six-crate workspace:

- **`app_core`** — unchanged from HADR-0002/0003: private domain + application, only `app_core::ports` public.
- **`infra_persistence`** — SQLCipher/rusqlite repositories, FTS5 search, and Audit Log Entry writes (audit rows are ordinary immutable rows in the same encrypted database, not a separate technology).
- **`infra_backup`** — encrypted point-in-time export/import (Backup Snapshot) and Recovery-Code-driven restore. **Depends on `infra_persistence`** to reuse its connection, schema, and key-setup code rather than re-implementing SQLCipher file handling from scratch. This is the workspace's first infra-to-infra dependency edge — an explicit, demonstrated-reuse exception to [dependency-rules.md](../../../../backend_arch_docs/dependency-rules.md)'s documented `infra_* -> app_core`-only edge table, justified under HADR-0003's own "demonstrated reuse" bar for shared adapter code.
- **`infra_hardware_id`** — Machine Hardware ID fingerprinting (CPU ID + motherboard serial) only; no licensing logic. Mirrors the PRD's own separation of concerns (Appendix B).
- **`infra_licensing`** — Ed25519 license-key verification against the embedded public key; takes a hardware id as an input value rather than fingerprinting it itself.
- **`infra_credentials`** — OS-protected credential store (Windows Credential Manager via the `keyring` crate).

`src-tauri` is not a `crates/`-namespaced `infra_*` crate. It is itself the inbound adapter — Tauri commands translate IPC calls into `app_core::ports` calls — **and** the composition root: `main.rs` wires all five `infra_*` crates and calls `app_core`'s public assembly function. This matches [architecture.md](../../../../backend_arch_docs/architecture.md)'s treatment of "reference and service compositions" as executables, not mandatory extra crates. `src-tauri` keeps Tauri's own conventional top-level directory name — Tauri's CLI tooling expects a directory literally named `src-tauri` — rather than being renamed `infra_tauri`.

Workspace layout:

```text
crates/
  app_core/
  infra_persistence/
  infra_backup/
  infra_hardware_id/
  infra_licensing/
  infra_credentials/
src-tauri/      (Tauri shell + composition root)
src/            (React + Tailwind + shadcn/ui frontend)
Cargo.toml      (workspace root)
```

This is the reference layout (`crates/app_core` + `crates/infra_local`) carrying over, with `infra_local` split into five responsibility-named crates instead of one generic one.

Governance scope: this decision is recorded entirely as this application ADR. [`backend_arch_docs/`](../../../../backend_arch_docs/) (the reusable Hexon template, including `dependency-rules.md`'s edge table) is intentionally left unmodified — this application's use of one sanctioned infra-to-infra edge, justified by demonstrated reuse, is an application-level instantiation of the existing HADR-0002/0003 pattern, not a proposed change to the reusable template's general rule for other consumers of the template.

## Alternatives Considered

### Merge `infra_backup` into `infra_persistence`

- Benefits: no infra-to-infra dependency edge; the workspace graph stays a pure star (`infra_* -> app_core` only), matching `dependency-rules.md` exactly as documented.
- Costs and risks: backup/restore (point-in-time export/import, Recovery-Code-driven restore) is a distinct responsibility from routine repository CRUD even though it touches the same SQLCipher file; merging them blurs ownership inside one crate. Rejected — kept separate for clarity of ownership, accepting the infra-to-infra edge as the cost.

### Fold hardware-id fingerprinting into `infra_licensing`

- Benefits: one fewer crate; fingerprinting today only feeds the licensing use case and the Settings display of the same value.
- Costs and risks: Appendix B explicitly treats Machine Hardware ID as a standalone capability, independent of license validation, and calls out that conflating hardware-binding concerns with a different concern is exactly the mistake this project already made and backed out of once (the DB-key-derivation story in Appendix B). Rejected — splitting mirrors the PRD's own boundary and prevents that mistake from recurring at the crate level.

### Fold `infra_credentials` into `infra_persistence`

- Benefits: one fewer crate.
- Costs and risks: the OS-protected credential store (`keyring`) is a distinct OS-level technology with no shared code with rusqlite/SQLCipher. Rejected — folding it in would be a false economy that couples two unrelated technologies.

### Split `src-tauri` into a thin shell + separate `infra_tauri_ipc` crate

- Benefits: IPC command-handler/translation logic becomes testable independent of the Tauri runtime.
- Costs and risks: only one inbound adapter exists today; splitting the translation layer out is speculative until a second inbound adapter demonstrates the reuse (mirrors HADR-0003's "demonstrated reuse" bar, applied to inbound adapters). Rejected as premature.

### Promote the `infra_backup -> infra_persistence` edge to a new HADR in `backend_arch_docs/`

- Benefits: would make infra-to-infra dependencies a documented, reusable pattern in the template for every derived service, not just this application.
- Costs and risks: this is this application's instantiation choice, not a proposed change to the reusable template's rule for other template consumers; promoting it prematurely risks over-generalizing a single-application decision. Rejected in favor of application-ADR-only scope.

## Consequences

### Positive

- `Cargo.toml` and the Core Engine technical design inherit a concrete, unblocked crate list instead of ADR-0001's open deferral.
- The PRD's Machine-Hardware-ID/licensing separation is preserved at the architecture level, not just in prose.
- Backup/restore has a clear, independently reviewable ownership boundary from day-to-day persistence.

### Negative

- `infra_backup -> infra_persistence` is a documented exception to the template's own dependency table; future readers of `dependency-rules.md` need this ADR's cross-reference to understand why this application's graph isn't a pure star topology.
- Six crates plus `src-tauri` is more workspace ceremony than the reference template's two-crate example.

### Neutral / Risks

- If a second application ever needs to reuse `infra_hardware_id`, `infra_licensing`, or `infra_credentials`, HADR-0003's "demonstrated reuse + accepted HADR" bar applies before extracting a shared crate; that extraction is not addressed here.

## Confirmation

- `Cargo.toml` workspace members are exactly: `app_core`, `infra_persistence`, `infra_backup`, `infra_hardware_id`, `infra_licensing`, `infra_credentials`, `src-tauri` (the frontend's `package.json` under `src/` is not a Cargo workspace member).
- `infra_backup` is the only crate with a workspace-internal crate dependency (on `infra_persistence`); every other `infra_*` crate depends only on `app_core`.
- `app_core` has zero Tauri, web, database, or OS-credential-store dependencies (HADR-0002/0003 conformance, carried from ADR-0001).

## Relationships and References

- Retains [HADR-0002 (hexon architecture)](../../../../backend_arch_docs/adr/HADR-0002-adopt-rust-hexon-architecture.md) and [HADR-0003 (crate responsibilities)](../../../../backend_arch_docs/adr/HADR-0003-define-crate-and-module-responsibilities.md).
- Refines [ADR-0001 (Core Engine application stack)](../architecture/2026-08-17-0001-lock-core-engine-application-stack.md) by pinning the concrete crate topology ADR-0001 deferred.
- Owning spec: [Core Engine PRD](../../../specs/core-engine/PRD.core-engine.md) §2 (Machine Hardware ID, License Key, Backup Snapshot, Recovery Code, Audit Log Entry definitions), [Appendix B](../../../specs/core-engine/appendix-b-key-derivation-and-recovery.md) (key derivation & recovery, hardware-id/licensing separation).
- Supporting issue: [Fix the workspace crate topology (app_core + infra_* + src-tauri)](https://github.com/Xaidel/brs/issues/3).
