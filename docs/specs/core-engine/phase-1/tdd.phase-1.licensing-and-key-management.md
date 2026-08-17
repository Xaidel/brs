# Core Engine Technical Specification

## Phase 1: Licensing & Key-Management Core

Status: Normative

Owner: Engineering

Date: 2026-08-18

Related documents:

- [Package README](../README.md)
- [Core Engine PRD](../PRD.core-engine.md) — §9.5, §9.6, §11 (`LicenseGrant`), NFR-01, NFR-04, NFR-05
- [Appendix A: Technical Architecture Direction](../appendix-a-technical-architecture-direction.md) — stack direction (superseded to normative by ADR-0001)
- [Appendix B: Key Derivation & Recovery Rationale](../appendix-b-key-derivation-and-recovery.md) — key-derivation and Recovery Code rationale (superseded to normative by ADR-0005)
- [Appendix C: License Reissuance SOP](../appendix-c-license-reissuance-sop.md) — hardware-mismatch support workflow (informative)
- [ADR-0001: Lock the Core Engine application stack](../../../adr/implemented/architecture/2026-08-17-0001-lock-core-engine-application-stack.md)
- [ADR-0002: Fix the Core Engine workspace crate topology](../../../adr/implemented/architecture/2026-08-17-0002-fix-core-engine-workspace-crate-topology.md)
- [ADR-0003: Define the Feature Flag taxonomy](../../../adr/implemented/architecture/2026-08-17-0003-define-feature-flag-taxonomy.md)
- [ADR-0004: Define the RBAC Permission taxonomy and seed roles](../../../adr/implemented/architecture/2026-08-17-0004-define-rbac-permission-taxonomy.md)
- [ADR-0005: Lock the Core Engine licensing & key-management mechanics](../../../adr/implemented/architecture/2026-08-17-0005-lock-core-engine-licensing-key-management-mechanics.md)
- [ADR-0006: Lock the sync-ready schema baseline](../../../adr/implemented/architecture/2026-08-17-0006-lock-sync-ready-schema-baseline.md)
- [backend_arch_docs/architecture.md](../../../../backend_arch_docs/architecture.md)
- [backend_arch_docs/dependency-rules.md](../../../../backend_arch_docs/dependency-rules.md)
- [backend_arch_docs/naming-conventions.md](../../../../backend_arch_docs/naming-conventions.md)
- [backend_arch_docs/testing.md](../../../../backend_arch_docs/testing.md)
- Stack spike evidence: [issue #10](https://github.com/Xaidel/brs/issues/10) (bundled SQLCipher + FTS5 compiles and meets NFR-02; installer size (NFR-03) unverified, deferred to [issue #17](https://github.com/Xaidel/brs/issues/17))

**Supersedes** the removed `docs/specs/core-engine/phase-1/tdd.phase-1.tech-spike-and-licensing.md`. The technical feasibility spike that document gated is no longer part of this design — it has already run and passed as a standalone ticket (issue #10). This document starts from that passed spike and is normative for the licensing/key-management domain and ports slice only.

---

## 1. Purpose

This document is the fresh, normative Core Engine technical design required by HADR-0001 before any Core Engine build ticket proceeds. It does two things:

1. **Coverage map (§5):** dispositions every Core Engine PRD requirement — §9.1 through §9.14, the NFRs, and the full §11 data model — into a build slice (this document's Phase 1, a future Phase 2, a future Phase 3) or an explicit exclusion. This is the map a reader needs to see where any given PRD requirement lands before a Phase 2 or Phase 3 technical design exists to hold it.
2. **Full domain/application/ports design for Phase 1 only:** offline Ed25519 License Key validation (machine-bound, feature-gating) and database encryption key management (bootstrap, derivation, Recovery-Code-driven disaster recovery), under the six locked ADRs (ADR-0001 through ADR-0006).

It answers, for a human or coding-agent implementer:

- What must exist in `app_core::domain`, `app_core::application`, and `app_core::ports` for the licensing and key-management slice, and what must not?
- Which outbound ports must `infra_hardware_id`, `infra_licensing`, `infra_credentials`, and `infra_persistence` satisfy, and which crate owns which capability?
- Where does the Ed25519 verification, PBKDF2 derivation, AES-256-GCM wrap/unwrap, and hardware-fingerprint hashing actually execute — domain or infra? (§4.4 makes this decision explicitly; it reverses the assumption the removed Phase 1 TDD flagged as unconfirmed.)
- How does ADR-0006's `Clock` port get a concrete adapter, and how does the Audit Trail's own timestamping work? (§4.5 — both were explicitly left open by ADR-0006 and assigned to this document.)
- What Gate 1–3 evidence (per HADR-0006 and `backend_arch_docs/testing.md`) is required to consider this slice implemented?
- Why does first-run setup not complete without a Backup Snapshot, and what is the smallest slice of `infra_backup` that requires? (§4.6 — closes a gap ADR-0005 itself flagged and the removed TDD left open.)

This document does **not** define: RBAC/Staff Accounts enforcement, Resident/Household/Certificate domain modeling, the Audit Trail's own entity, the automated backup engine's *scheduling/retention* logic (daily schedule, 14-snapshot rolling purge, manual USB export UI), Barangay Identity/Officials/Templates/Purok/Theming/Dashboard, any UI/Tauri command wiring beyond what §9 requires implementers to know exists, or `infra_*` implementation code. Those are Phase 2 and Phase 3, scoped but not designed here (§5, §13) — with one narrow exception: the single mandatory first-run Backup Snapshot primitive (§4.6) is Phase 1 scope.

## 2. Terminology

Reused verbatim from the [PRD glossary](../PRD.core-engine.md#21-canonical-terms) and [CONTEXT.md](../../../../CONTEXT.md): **License Key**, **Machine Hardware ID**, **Feature Flag**, **Recovery Code**, **LicenseGrant**, **system_secret**, **bootstrap.json**, **Clock**, **Shared Schema Columns**, **sync_status**.

Introduced by this document (implementation vocabulary, not new product concepts):

| Term | Meaning |
|---|---|
| `DatabaseEncryptionKey` | The derived SQLCipher key material (AES-256 key length, PBKDF2-HMAC-SHA256 output). Never the same value as `system_secret`. |
| `EncryptionSalt` | The non-secret PBKDF2 salt persisted in `bootstrap.json`. |
| `LicenseKeyPayload` | The parsed, pre-verification structure of a submitted License Key string — bound `MachineHardwareId`, `FeatureFlagSet`, and an Ed25519 `LicenseSignature`. A smart constructor; malformed input fails before verification is attempted. |
| `LicenseKeyEnvelope` | The concrete wire format of a License Key string (§11). |
| `EncryptionCredentialBootstrap` | The port-owned DTO shape of `bootstrap.json`'s contents (salt + wrapped `system_secret`) exchanged across the `EncryptionCredentialGateway` port. Not a domain entity — see §4.4 for why this document drops the removed TDD's `EncryptionCredential` aggregate. |
| Initial Backup Snapshot | The single, mandatory, on-demand Backup Snapshot `EstablishEncryptionCredentialUseCase` takes before first-run setup can complete (§4.6). Not the automated exit/daily snapshot engine (Phase 3) — a distinct, narrower capability that happens to share `infra_backup` and the archive format. |
| Gate 1 / Gate 2 / Gate 3 | As defined in HADR-0006 and `backend_arch_docs/testing.md`. This document does not redefine them. |

Forbidden synonyms: do not use "hardware fingerprint" for Machine Hardware ID, "master key" for `DatabaseEncryptionKey`, "license token" for License Key, or "encryption key" unqualified for `system_secret` (CONTEXT.md already reserves this) — use the PRD/CONTEXT.md terms so downstream code, tests, and error messages stay searchable.

## 3. Scope

### 3.1 In scope

- `app_core::domain`, `app_core::application`, and `app_core::ports` content for:
  - Offline Ed25519 License Key validation (§9.5) — parsing, machine-binding invariant, and the use case sequencing around signature verification.
  - Database encryption key bootstrap, day-to-day derivation, and Recovery-Code-driven disaster recovery (§9.6, §9.8's Recovery-Code-restore requirement only — not the backup scheduling/retention engine itself).
  - The `LicenseGrant` entity (PRD §11) to the extent Phase 1 needs to represent, validate, and persist it. Concrete SQLCipher migration DDL is implementation-mechanics, not designed here.
  - Outbound port contracts (§8) that `infra_hardware_id`, `infra_licensing`, `infra_credentials`, and `infra_persistence` (ADR-0002's six-crate topology) must satisfy — signatures and semantics only, not implementations.
  - The `Clock` port's concrete adapter placement (ADR-0006's explicit open item) and the Audit Trail's own timestamping approach (ADR-0006's other explicit open item) — resolved here as narrow schema-baseline decisions, without designing the Audit Trail entity itself (§4.5).
  - A single mandatory first-run Backup Snapshot, gating first-run completion (§4.6) — the narrow slice of `infra_backup` this document needs, not the backup engine.
- The License Key wire envelope (§11) — concrete enough for a coding agent (and, eventually, the out-of-scope vendor `bms-keygen` tool) to implement without guessing the signed byte sequence.

### 3.2 Out of scope for this document

- RBAC enforcement, Staff Account authentication, and Role/Permission checks (§9.1) — Phase 2. The 19-key catalog and seed matrix are already fixed by ADR-0004; applying them to actual authorization code is Phase 2 design.
- Resident/Household Registry (§9.2, §9.3), Certificate Generation & Control Number Sequencing (§9.4) — Phase 2.
- The Audit Trail entity and its write path (§9.7) beyond the timestamping decision in §4.5 — Phase 2/3 (see §5).
- The automated Backup & Recovery *engine* — the daily schedule, 14-snapshot rolling retention/purge, and manual USB export UI (§9.8) — Phase 3. This document adds exactly one narrow primitive on top of the same crate (§4.6): a single on-demand snapshot taken once at first-run completion, with no scheduling, retention, or query/list capability. It does not persist a queryable `BackupSnapshot` catalog row — Phase 3 introduces that repository when it needs to list/restore/purge.
- Barangay Identity & Branding (§9.9), Barangay Officials & Signatures (§9.10), Certificate Template & Layout Customization (§9.11), Purok/Sitio/Zone Management (§9.12), Appearance & Theming (§9.13), Dashboard Widget Configuration (§9.14) — Phase 3.
- Any UI or Tauri command-handler wiring, including the "unlock instantly, no restart" reactivity requirement (PRD §9.5 bullet 4) — that is a frontend/composition concern layered on top of the `GetActiveLicense`/`ActivateLicense` use cases this document defines; wiring is Phase 3 ("Plug & Play UI" per the package README's phase map).
- Any `infra_*` crate implementation code (WMI queries, `ed25519-dalek` calls, `keyring` calls, SQLCipher connection opening). This document defines the ports such implementations must satisfy.
- The installer-size spike (NFR-03) — already deferred by the map to [issue #17](https://github.com/Xaidel/brs/issues/17), not re-litigated here.
- The vendor-side `bms-keygen` tool and License Registry ([Appendix C](../appendix-c-license-reissuance-sop.md) §3) — internal tooling, not part of this product.

## 4. Architecture Constraints

### 4.1 Crate topology (ADR-0002, binding)

```text
crates/
  app_core/            private domain + application; only app_core::ports is public
  infra_persistence/   SQLCipher/rusqlite repositories, FTS5 search, audit rows
  infra_backup/        depends on infra_persistence (the one sanctioned infra-to-infra edge)
  infra_hardware_id/   Machine Hardware ID fingerprinting only
  infra_licensing/     Ed25519 license-key verification against the embedded public key
  infra_credentials/   OS-protected credential store + bootstrap.json + Recovery Code
src-tauri/             inbound adapter (Tauri commands) + composition root
```

This document's slice touches `app_core`, `infra_hardware_id`, `infra_licensing`, `infra_credentials`, `infra_persistence` (for `LicenseGrantRepository` only), `src-tauri` (for the `Clock` adapter placement, §4.5), and — narrowly — `infra_backup` (for the single first-run snapshot primitive, §4.6, and nothing else; no scheduling, retention, or repository).

### 4.2 Dependency direction (HADR-0002/0003, `architecture.md`)

`infra_* -> app_core::ports and assembly`; `app_core::application -> app_core::ports and private domain`; `private domain -> no application or infrastructure`. `app_core` depends on no workspace crate; per ADR-0001/0002's confirmation clauses it may depend on ordinary (non-workspace) Cargo crates, subject to review, as a visible diff to `crates/app_core/Cargo.toml`.

### 4.3 Naming (`naming-conventions.md`)

Inbound traits are named after capabilities (`ActivateLicense`, not `LicenseService`); concrete orchestration types use the `UseCase` suffix; outbound ports use `{Noun}Repository`, `{Noun}Source`, or `{Noun}Gateway`; domain events are past-tense business facts; domain services are named after their business rule; transport/port DTO names stay private to `ports`.

### 4.3.1 Windows toolchain (carried from the stack spike)

[Issue #10](https://github.com/Xaidel/brs/issues/10)'s resolution recorded a gap this document must own: bundled SQLCipher + FTS5 compiled and linked cleanly only via the `x86_64-pc-windows-gnu` Rust target in the evaluated environment — the vendored-OpenSSL/MSVC path needs Perl CPAN modules that were not present. **Decision:** the workspace pins `x86_64-pc-windows-gnu` (via `rust-toolchain.toml` at the workspace root) as the supported build target for every crate, not only `infra_persistence`, so the whole workspace builds with one consistent toolchain rather than mixing GNU and MSVC ABIs across crates. This is a workspace-wide constraint recorded here because this is the first technical design to exist; it binds `infra_persistence`'s Phase 2 SQLCipher work as much as it binds nothing in this document's own crate touch-list (§4.1). Revisiting the MSVC path is not scheduled and no longer tracked as an open item: it is revisited only if a concrete blocker appears — a future dependency that fails to build under GNU, or a CI/installer requirement that forces the MSVC ABI — at which point the fix is to install the missing Perl CPAN modules in CI/dev images. Until that trigger fires, `x86_64-pc-windows-gnu` remains the sole binding target.

### 4.4 Decision: crypto primitives execute in `infra_*`, not `app_core::domain`

The removed Phase 1 TDD flagged, as an unconfirmed assumption, that Ed25519 verification and PBKDF2 derivation could live in `app_core::domain` as pure computation (since they are algorithm-library calls, not I/O). **This document resolves that question the opposite way, and the resolution is binding:**

All cryptographic primitive computation — Ed25519 signature verification, the Machine Hardware ID's SHA-256 + Crockford Base32 encoding, PBKDF2-HMAC-SHA256 key derivation, AES-256-GCM wrap/unwrap, and secure random generation of `system_secret`/salt/Recovery Code — executes inside the relevant `infra_*` crate, behind an outbound port that expresses the **capability**, not the mechanism.

Rationale:

- ADR-0002 already assigns these capabilities to specific crates in exactly these terms: `infra_hardware_id` is "Machine Hardware ID fingerprinting... only," `infra_licensing` is "Ed25519 license-key verification against the embedded public key." A domain service that did the actual math would leave those crates with almost nothing to adapt, contradicting the crate topology this document must operate under.
- It keeps `app_core`'s `Cargo.toml` free of any cryptographic crate dependency at all (only ordinary value-shaped crates like `uuid`, per [ticket #8](https://github.com/Xaidel/brs/issues/8)) — a stronger, unambiguous "framework-free" boundary than leaving it as a flagged assumption.
- It gives Gate 2 concrete outbound ports to fake immediately (`LicenseSignatureVerifier`, `MachineHardwareIdSource`, `EncryptionCredentialGateway`) instead of requiring test-local fakes for a domain service that wraps a third-party crate.
- The embedded Ed25519 public key is adapter-owned configuration (compiled into `infra_licensing`), not a value `app_core` needs to carry.

`app_core::domain` therefore owns only: value-object validation/formatting invariants (e.g., `RecoveryCode`'s checksum format, `MachineHardwareId`'s grouped-string shape), the `LicenseGrant` entity's machine-binding invariant (a plain value comparison, no crypto), and typed domain errors. It does not call `ed25519-dalek`, `pbkdf2`, `aes-gcm`, `sha2`, or `wmi` directly.

One consequence: `LicenseGrant`'s construction invariant is narrower than the removed TDD's version. Signature verification now happens in the **application** layer (via the `LicenseSignatureVerifier` port) *before* the entity is constructed, not inside the entity's own constructor — a constructor cannot call an outbound port. The entity still enforces what it *can* enforce purely: a `LicenseGrant` cannot be constructed with a `machine_hardware_id` that disagrees with the local installation's. See §6.2.

### 4.5 Resolving ADR-0006's two open items

ADR-0006 explicitly left two things to this document:

1. **`Clock` port's concrete adapter placement.** No `infra_*` crate in ADR-0002's topology is a natural home for a single trivial `Utc::now()` wrapper, and creating a new crate for it would be workspace ceremony with no second consumer. **Decision:** the concrete `SystemClock` adapter lives in `src-tauri`, which ADR-0002 already designates as the composition root. `src-tauri` constructs it and passes it to `app_core`'s assembly function alongside the five `infra_*` port implementations. No new crate.
2. **Audit Trail's own timestamping.** ADR-0006 excludes `AuditLogEntry` from the five Shared Schema Columns (no `barangay_code`, `updated_at`, `sync_status` — rows are never updated). **Decision:** `AuditLogEntry` carries a single `timestamp_utc` column (already named in PRD §11), stamped through the same `app_core` `Clock` port at the moment the audited use case runs — not a SQLite default or trigger, for the same testability reason ADR-0006 gives for `created_at`/`updated_at` elsewhere. `AuditLogEntry.id` is still UUIDv7 (the map's standalone identifier-scheme decision, [ticket #8](https://github.com/Xaidel/brs/issues/8), applies to every record identifier, not only Shared-Schema-Columns tables). This decision fixes the column shape only; the `AuditLogEntry` entity and its write path remain Phase 2 scope (§5).

Both resolve narrow items ADR-0006 named explicitly; neither expands into full Audit Trail or composition design.

### 4.6 Decision: first-run setup does not complete without a Backup Snapshot

ADR-0005's own Neutral/Risks section named the gap: `bootstrap.json` is a new first-run artifact, and if it is lost or corrupted *before the first automatic Backup Snapshot exists* (the window between first run and the next exit/daily snapshot), the Recovery Code cannot help — it only unwraps `system_secret` out of `bootstrap.json`'s blob; with no blob, there is nothing to unwrap. A barangay that loses its PC in that window loses everything entered so far, holding a Recovery Code that cannot recover it. This is reachable through ordinary first-day use, not a hypothetical.

**Decision:** `EstablishEncryptionCredentialUseCase` (§7.4) does not report first-run setup complete, and does not return the `RecoveryCode` for display, until one Backup Snapshot has been taken and confirmed written to disk. Concretely:

1. `EncryptionCredentialGateway::establish()` runs first (§8.4) — generates and persists `system_secret`, salt, and the Recovery Code's material.
2. `BackupSnapshotWriter::take_snapshot()` (new outbound port, §8.6) runs next, capturing `bootstrap.json` and the current (at this point near-empty) SQLCipher database file into one encrypted archive under the default backup destination (`%APPDATA%\BarangayMS\backups\`, per PRD §9.8/ADR-0001).
3. Only if step 2 succeeds does the use case return the `RecoveryCode` to the caller for one-time display. If it fails, the use case returns `EstablishEncryptionCredentialError::InitialBackupFailed` and displays nothing — the Secretary never sees a Recovery Code she can't trust yet.

Recovery from a failed step 2 is deliberately simple, not a rollback: `establish()` may be safely re-invoked (it overwrites `bootstrap.json` and the keyring entry with a fresh `system_secret`/salt/Recovery Code), because nothing has been shown to the Secretary or written anywhere else yet. The UI's retry path is "run first-run setup again," not a partial-state repair flow.

**Why this is the right amount of scope, not more:** this decision requires exactly one new capability — take one snapshot, once, synchronously, as a gate — not the scheduling, rolling retention, or manual-export UI that make up the rest of `infra_backup`'s eventual job (§3.2). It does not require a queryable `BackupSnapshot` catalog (no `id`/`trigger_type`/`storage_location` row is persisted anywhere by this document) — Phase 3 introduces that repository when it needs to list, restore from, or purge snapshots. The archive file existing on disk is the entire safety property Phase 1 needs.

**Alternatives considered:**

- *Warn instead of block* (display a warning next to the Recovery Code instead of gating on a real snapshot). Rejected: a warning doesn't change the failure mode, only documents it — the whole point of catching this now is that the fix is nearly free (a few seconds at first run, on data that doesn't exist yet to make large) while the alternative is unrecoverable data loss for a real barangay.
- *Accept the risk as documented* (this document's original Open Question framing). Rejected once a cheap, concrete fix was available — the risk is small in probability but total in consequence (all data, on the one day a barangay is most likely to be trying the product for the first time), which is exactly the shape of risk worth spending a few seconds of first-run latency to close.

### 4.7 Decision: Backup Snapshot file-set manifest and `bootstrap.json` location ownership

The §15 open question — how `infra_backup` enumerates and copies the file set — is resolved here as a narrow interface contract that Phase 1's first-run snapshot produces and Phase 3 extends:

1. **`bootstrap.json`'s location is owned by `infra_credentials`, exposed through the port.** `EncryptionCredentialGateway` gains `bootstrap_file_path()` (§8.4); `infra_backup`'s `BackupSnapshotWriter` adapter receives `Arc<dyn EncryptionCredentialGateway>` as an injected dependency (wired by `src-tauri`) and calls it to locate `bootstrap.json`. No path constant is duplicated across crates; the database file's path comes through the already-sanctioned `infra_backup -> infra_persistence` edge (§8.6), not a second hardcoded location.
2. **The archive carries an explicit file manifest, not a hardcoded file list.** A Backup Snapshot archive contains a small JSON manifest — one entry per file, `{ "path": ..., "role": "bootstrap" | "database" }` — plus the files themselves. Phase 1 writes exactly two entries; Phase 3 appends resident photos / logos / signatures (§9.2/§9.9) by extending the manifest, never by rewriting the reader. Discoverability lives in the archive format.
3. **The port is destination-agnostic.** `take_snapshot(&self, destination: BackupDestination)`, where `BackupDestination` is either the default backup directory (`%APPDATA%\BarangayMS\backups\`, per PRD §9.8) or an explicit path. The adapter resolves the default; the same method serves both the first-run gate (§4.6) and Phase 3's manual USB export.

This fixes the *archive contract* Phase 1 produces; it does not design Phase 3's scheduling, retention, purge, or repository (§3.2), which will extend — not re-rip — this contract.

## 5. PRD-to-Implementation Coverage Map

Every Core Engine PRD requirement, dispositioned. "Phase 1" means this document; "Phase 2"/"Phase 3" mean the package README's updated phase map (§13); "Excluded" means out of this release entirely per the PRD's own Non-Goals.

| PRD section | Disposition | Notes |
|---|---|---|
| §9.1 Authentication & RBAC | Phase 2 | Permission catalog and seed matrix already fixed by ADR-0004; enforcement code is Phase 2. |
| §9.2 Resident Registry | Phase 2 | Shared Schema Columns apply (ADR-0006). |
| §9.3 Household Registry | Phase 2 | Shared Schema Columns apply (ADR-0006). |
| §9.4 Certificate Generation & Control Number Sequencing | Phase 2 | Control Number Format token grammar resolved by [ADR-0007](../../../adr/implemented/architecture/2026-08-18-0007-define-control-number-format-token-grammar.md) (formerly open in §15). |
| §9.5 Offline Licensing & Feature Gating | **Phase 1 (this document)** | §6–§9 below. UI reactivity ("unlock instantly, no restart") is Phase 3. |
| §9.6 Data Protection & Key Management | **Phase 1 (this document)** | §6–§9 below. |
| §9.7 Immutable Audit Trail | Phase 2/3 split | Timestamping column resolved here (§4.5); the entity and write path are Phase 2 (each audited action lives where its use case lives); read/query UI is Phase 3. |
| §9.8 Backup & Recovery | Phase 1 (Recovery-Code key recovery + the single mandatory first-run snapshot) / Phase 3 (scheduling, retention, manual export) | This document's `RecoverDatabaseEncryptionKey` use case (§7.5) covers "restore using Recovery Code, no original hardware" for the *key*; §4.6 covers one gating snapshot at first run. The scheduling/retention/manual-export engine itself (daily schedule, 14-snapshot rolling window, manual USB export) remains Phase 3 in `infra_backup`. |
| §9.9 Barangay Identity & Branding | Phase 3 | — |
| §9.10 Barangay Officials & Signatures | Phase 3 | — |
| §9.11 Certificate Template & Layout Customization | Phase 3 | Template Variable enumeration resolved by [ADR-0008](../../../adr/implemented/architecture/2026-08-18-0008-define-template-variable-enumeration.md) (formerly open in §15). |
| §9.12 Purok/Sitio/Zone Management | Phase 3 | — |
| §9.13 Appearance & Theming | Phase 3 | `ThemeSetting` device-local-vs-synced question (PRD §14) resolves by ADR-0006's exclusion list: not a business-record candidate for `.bmssync`, so device-local, not Shared-Schema-Columns. Confirmed here so Phase 3 doesn't re-litigate it. |
| §9.14 Dashboard Widget Configuration | Phase 3 | Same device-local resolution as §9.13, and for the same ADR-0006 reason. |
| NFR-01 (Startup <3s) | Constraint carried into Phase 1 | PBKDF2 iteration count (§8.4) is chosen with this budget in mind. |
| NFR-02 (Search <200ms @ 50k) | Verified by spike (issue #10) | Not re-verified here; applies to Phase 2 (Resident search). |
| NFR-03 (Installer ≤30MB) | Deferred | [issue #17](https://github.com/Xaidel/brs/issues/17), per the map's own decision. |
| NFR-04 (100% offline) | Satisfied by construction, Phase 1 | No port or domain concept in this slice performs network I/O. |
| NFR-05 (Single logical write path) | Constraint carried into Phase 1 and beyond | `LicenseGrantRepository` (§8.3) is `infra_persistence`'s sole writer for license state; no other path may write it. |
| NFR-06 (Auditability) | Phase 2/3 | Depends on the Audit Trail entity (Phase 2) built on the timestamping decision in §4.5. |
| §11 data model: `LicenseGrant` | **Phase 1 (this document)** | §10. |
| §11 data model: `Resident`, `Household`, `HouseholdMembership`, `Certificate`, `ControlNumberSequence`, `StaffAccount`, `DocumentType`, `CertificateTemplate`, `Purok`, `Role`, `Permission`, `RolePermission` | Phase 2 | — |
| §11 data model: `AuditLogEntry` | Column shape Phase 1 (§4.5), entity Phase 2 | — |
| §11 data model: `BackupSnapshot` | Write primitive Phase 1 (§4.6), queryable catalog Phase 3 | Phase 1 writes one archive to disk via `BackupSnapshotWriter` (§8.6) but persists no `id`/`trigger_type`/`storage_location` row; Phase 3 introduces the repository when listing/restoring/purging snapshots needs one. `bootstrap.json`'s presence inside every archive is a Phase 1 constraint Phase 3 inherits. |
| §11 data model: `BarangayProfile`, `BarangayOfficial`, `ThemeSetting`, `DashboardWidgetConfig` | Phase 3 | — |

## 6. Expected Domain Changes

`crates/app_core/src/domain/` (private `mod domain`). No I/O; no ports, adapters, persistence schema, or API contract concepts appear here.

### 6.1 Value objects

| Value object | Validates | Notes |
|---|---|---|
| `MachineHardwareId` | Non-empty Crockford Base32 string, grouped into 13 dashed 4-character blocks (52 data characters) | Equality-comparable. Carries no derivation logic — derivation is entirely `infra_hardware_id`'s job (§4.4). |
| `LicensePublicKey` | Fixed-length Ed25519 public key bytes (32 bytes) | Adapter-owned configuration in `infra_licensing` (§4.4); `app_core` does not hold this value at all — removed from this document's domain vs. the old TDD's version. |
| `LicenseSignature` | Fixed-length Ed25519 signature bytes (64 bytes) | Carried inside `LicenseKeyPayload`; never verified in domain (§4.4) — only structurally validated (correct length). |
| `FeatureFlag` | One of `KpBlotter`, `Treasury`, `BusinessPermits` | The ADR-0003 enum, `#[serde(rename_all = "SCREAMING_SNAKE_CASE")]`. No `Core` variant. Used both inside `LicenseKeyPayload` and `LicenseGrant`. |
| `LicenseKeyPayload` | Parses a submitted License Key string (§11 envelope) into `MachineHardwareId` + `Vec<FeatureFlag>` + `LicenseSignature` | Smart constructor; malformed/undecodable input fails with `LicenseValidationError::MalformedLicenseKey` before any port call is attempted. |
| `EncryptionSalt` | Fixed-length salt bytes (16 bytes) | Construction validates length only; generation is `infra_credentials`'s job (§4.4). |
| `SystemSecret` | Fixed-length installation-secret bytes (32 bytes) | Same generation/validation split as `EncryptionSalt`. Must not implement `Debug`/`Display` in a way that leaks its bytes. |
| `DatabaseEncryptionKey` | Fixed-length derived key bytes (32 bytes) | Output-only; same logging-safety constraint as `SystemSecret`. Never constructed directly — only returned by `EncryptionCredentialGateway` (§8.4). |
| `RecoveryCode` | Crockford Base32, 28 data characters + 1 mod-37 checksum character, formatted `XXXX-XXXX-XXXX-XXXX-XXXX-XXXX-XXXX-X` | Smart constructor: normalizes input (uppercase; strips hyphens; applies Crockford's `O→0`, `I`/`L→1` lookalike substitution) and validates the checksum before returning `Ok`. A bad checksum fails fast as `RecoveryCodeError::InvalidChecksum`, distinct from a wrong-but-well-formed code (which only fails later, at AES-GCM unwrap, inside `infra_credentials`). |
| `Timestamp` | Wraps a UTC instant | Framework-agnostic; sourced only via the `Clock` port (§8.5). |

### 6.2 Entities

- **`LicenseGrant`** (PRD §11). Fields: `id` (UUIDv7), `machine_hardware_id`, `feature_flags: Vec<FeatureFlag>`, `signature` (`LicenseSignature`, retained for audit/support inspection), `activated_at` (`Timestamp`). **Construction invariant:** the only public constructor is
  ```rust
  LicenseGrant::activate(
      payload: LicenseKeyPayload,
      local_machine_id: &MachineHardwareId,
      id: LicenseGrantId,
      activated_at: Timestamp,
  ) -> Result<LicenseGrant, LicenseValidationError>
  ```
  which checks `payload.machine_hardware_id == *local_machine_id`, returning `LicenseValidationError::MachineHardwareMismatch { current_machine_hardware_id }` on mismatch. **Signature verification already happened** in the application layer (§7.2) before this constructor is called — see §4.4. This still makes "a `LicenseGrant` exists" and "its bound machine ID matches this installation" the same fact by construction, satisfying PRD §9.5 bullet 3's ordering requirement, without requiring domain to hold crypto material.

No `EncryptionCredential` aggregate exists in this document (dropped vs. the removed TDD — see §4.4/§2). There is no per-installation collection or identity-bearing lifecycle left for `app_core` to model once credential establishment/recovery is a single cohesive `infra_credentials` capability (§8.4); only `RecoveryCode` and `DatabaseEncryptionKey` remain domain-owned values.

### 6.3 Domain events (past-tense business facts)

- `LicenseGrantActivated { license_grant_id, machine_hardware_id, feature_flags, activated_at }`

No event consumer exists in Phase 1 (no Audit Trail entity yet — §4.5/§5). The event is defined for forward compatibility with the Phase 2 Audit Trail and is drained/discarded by the application layer for now, per `architecture.md`'s "domain events... may later be drained and logged/audited by application code."

**Invariant:** the event carries no `SystemSecret`, `RecoveryCode`, or `DatabaseEncryptionKey` — only non-secret identifiers, the granted flags, and a timestamp.

### 6.4 Domain errors

- `LicenseValidationError`: `MalformedLicenseKey`, `MachineHardwareMismatch { current_machine_hardware_id: MachineHardwareId }`. (`InvalidSignature` is now an application-layer error returned by the `LicenseSignatureVerifier` port call, not a domain error — see §7.2.)
- `RecoveryCodeError`: `InvalidChecksum`, `MalformedFormat`.

## 7. Expected Application Changes

`crates/app_core/src/application/` (private `mod application`). Named use cases only; each sequences domain behavior and port calls.

### 7.1 `GetMachineHardwareIdUseCase`

Calls `MachineHardwareIdSource::current()` (§8.1) and returns the result. Thin passthrough — the capability (WMI reads, hashing, encoding) is entirely infra-side (§4.4); this use case exists so `src-tauri`'s Settings-display command has an inbound port to call rather than reaching into an outbound port directly (which would violate the runtime request flow in `architecture.md`).

### 7.2 `ActivateLicenseUseCase`

Input: the raw License Key string as pasted/uploaded by the Secretary.

Sequence:

1. `LicenseKeyPayload::parse(raw_key)` — domain smart constructor. Fails fast with `LicenseValidationError::MalformedLicenseKey` on bad structure/encoding.
2. `MachineHardwareIdSource::current()` (§8.1) — get the local machine's ID.
3. `LicenseSignatureVerifier::verify(&payload)` (§8.2) — **application-layer port call**, not domain. On failure, the use case returns `ActivateLicenseError::InvalidSignature` without ever constructing a `LicenseGrant`.
4. `LicenseGrant::activate(payload, &local_machine_id, id, clock.now())` (§6.2). On `MachineHardwareMismatch`, the use case returns that error, carrying `current_machine_hardware_id` so the caller can render Appendix C's "Hardware Change Detected" message.
5. `LicenseGrantRepository::save(record)` (§8.3) — append-only insert (§10).
6. Return the persisted `LicenseGrant`'s `feature_flags`.

`ActivateLicenseError` wraps `LicenseValidationError` plus `InvalidSignature` plus any `LicenseGrantRepository`/`MachineHardwareIdSource` port failures (mapped to safe, non-leaking variants per HADR-0005's boundary policy).

### 7.3 `GetActiveLicenseUseCase`

Calls `LicenseGrantRepository::find_current()` (§8.3) and returns the most recently activated grant's `feature_flags`, or an empty set if none exists yet (Core Engine itself needs no flag — §4 of ADR-0003).

### 7.4 `EstablishEncryptionCredentialUseCase`

First-run only. Sequence (§4.6 decision):

1. `EncryptionCredentialGateway::establish()` (§8.4) — generates `system_secret`, salt, and a Recovery Code; persists the wrapped bootstrap state and the OS-keyring copy of `system_secret`.
2. `BackupSnapshotWriter::take_snapshot()` (§8.6) — captures `bootstrap.json` and the current database file into one encrypted archive.
3. Only on success of both steps does the use case return the `RecoveryCode` for one-time display. On step 2's failure, it returns `EstablishEncryptionCredentialError::InitialBackupFailed` instead, and the caller must retry the whole use case (§4.6 — `establish()` is safe to re-invoke; nothing has been shown to the Secretary yet).

The `RecoveryCode` is never persisted in plaintext anywhere `app_core` can see, and this use case does not log it.

### 7.5 `RecoverDatabaseEncryptionKeyUseCase`

Input: a `RecoveryCode` (already checksum-validated by its own smart constructor — §6.1 — so a mistyped code never reaches this use case). Calls `EncryptionCredentialGateway::recover_database_key(&recovery_code)` (§8.4). Returns the recovered `DatabaseEncryptionKey` to the caller (the composition root, not a UI-facing value — §9.5), or a typed error if the wrapped blob fails to unwrap (wrong-but-well-formed code) or `bootstrap.json` is missing/corrupt.

### 7.6 Not a use case: day-to-day key loading

Opening the database on ordinary startup (`EncryptionCredentialGateway::load_database_key()`, §8.4) is composition-root plumbing in `src-tauri`, not an `app_core` use case — no business decision is made, and the database must be open before most use cases can run at all. `architecture.md`'s "infrastructure is the only layer that imports and wires all other layers" applies directly here. This is a deliberate omission, not an oversight.

## 8. Expected Ports Changes

`crates/app_core/src/ports/` — the crate's only `pub mod`. All outbound ports below are `async-trait`, stored behind `Arc<dyn Trait + Send + Sync>` per `architecture.md`.

### 8.1 `MachineHardwareIdSource` (outbound) — implemented by `infra_hardware_id`

```rust
trait MachineHardwareIdSource {
    async fn current(&self) -> Result<MachineHardwareId, HardwareIdError>;
}
```

The adapter performs the full ADR-0005 capability end-to-end: three WMI queries (`Win32_Processor.ProcessorId`, `Win32_BaseBoard.SerialNumber`, `Win32_ComputerSystemProduct.UUID` via the `wmi` crate), concatenation, SHA-256, Crockford Base32 encoding, and grouping — returning the finished domain value object. `HardwareIdError` covers WMI query failure (e.g., unsupported virtualization environment) as a safe, non-leaking variant.

### 8.2 `LicenseSignatureVerifier` (outbound) — implemented by `infra_licensing`

```rust
trait LicenseSignatureVerifier {
    async fn verify(&self, payload: &LicenseKeyPayload) -> Result<(), SignatureVerificationError>;
}
```

The adapter holds the embedded Ed25519 public key as its own baked-in configuration (not a port parameter) and verifies `payload`'s signature over the canonical signed bytes (§11). `infra_licensing` performs no I/O beyond the (synchronous, in-process) verification itself; it is declared `async` per `architecture.md`'s "every outbound port... declared async, even though no concrete adapter exists yet" rule, so Gate 1 does not need to be repeated when the adapter lands.

### 8.3 `LicenseGrantRepository` (outbound) — implemented by `infra_persistence`

```rust
trait LicenseGrantRepository {
    async fn save(&self, record: LicenseGrantRecord) -> Result<(), RepositoryError>;
    async fn find_current(&self) -> Result<Option<LicenseGrantRecord>, RepositoryError>;
}
```

`LicenseGrantRecord` is a port-owned DTO (HADR-0005): `id`, `machine_hardware_id: String`, `feature_flags: Vec<String>`, `signature: String` (base64), `activated_at: String` (ISO-8601 UTC) — mapped to/from the `LicenseGrant` entity by the application layer, never exposed as the private entity itself. `save` is insert-only (§10); `find_current` returns the row with the highest `id` (UUIDv7 sorts chronologically), i.e., the most recently activated grant — reissuance (Appendix C) produces a new row, it never mutates an old one.

### 8.4 `EncryptionCredentialGateway` (outbound) — implemented by `infra_credentials`

```rust
trait EncryptionCredentialGateway {
    async fn establish(&self) -> Result<RecoveryCode, CredentialError>;
    async fn load_database_key(&self) -> Result<DatabaseEncryptionKey, CredentialError>;
    async fn recover_database_key(&self, recovery_code: &RecoveryCode) -> Result<DatabaseEncryptionKey, CredentialError>;
    fn bootstrap_file_path(&self) -> String;
}
```

One cohesive, capability-shaped port — deliberately not split into separate keyring/file/crypto ports, since no `app_core` caller ever needs those primitives independently (§4.4). The adapter owns: OS keyring access (`keyring` crate) for `system_secret`; `bootstrap.json` file I/O for the salt and the AES-256-GCM-wrapped `system_secret`; PBKDF2-HMAC-SHA256 derivation; AES-256-GCM wrap/unwrap; and secure random generation of `system_secret`, salt, and the Recovery Code's underlying bytes.

- `establish()`: first-run only. Generates `system_secret` (32 random bytes), `EncryptionSalt` (16 random bytes), and a `RecoveryCode` (28 random Crockford Base32 data characters + checksum). Derives the AES-256-GCM wrap key by SHA-256-hashing the Recovery Code's canonical 28-character ASCII string (§4.4 rationale reprised in §8.4.1 below), encrypts `system_secret`, writes `bootstrap.json` (salt + wrapped blob), stores `system_secret` in the OS keyring, and returns the `RecoveryCode` for one-time display.
- `load_database_key()`: reads `system_secret` from the OS keyring and the salt from `bootstrap.json`, derives `PBKDF2-HMAC-SHA256(system_secret, salt)`, returns the `DatabaseEncryptionKey`. Fails with `CredentialError::KeyringUnavailable` if the keyring entry is missing (e.g., OS reinstall without a Recovery Code recovery).
- `recover_database_key(recovery_code)`: reads `bootstrap.json`, re-derives the AES-256-GCM unwrap key from the given `RecoveryCode`, unwraps `system_secret`, best-effort re-stores it into the local OS keyring (so subsequent `load_database_key()` calls succeed without re-entering the code), derives and returns the `DatabaseEncryptionKey`. A wrong-but-well-formed code fails AES-GCM authentication and returns `CredentialError::RecoveryCodeMismatch` (distinct from the domain-layer `RecoveryCodeError::InvalidChecksum` a malformed code fails earlier — §6.1).
- `bootstrap_file_path()`: returns the absolute path to `bootstrap.json`. Not `async` (a path query, like `Clock`). Exists so `infra_backup`'s `BackupSnapshotWriter` (§4.7) can locate the file through the port that owns it rather than duplicating the path constant.

#### 8.4.1 Decision: Recovery Code wrap-key derivation

ADR-0005 explicitly deferred the Recovery Code's wrap-key KDF and parameters to this document. **Decision:** SHA-256 the Recovery Code's canonical 28-character ASCII string (post-normalization, pre-checksum-character) directly to produce the 256-bit AES-GCM key — not PBKDF2 or Argon2id.

Rationale: PBKDF2/Argon2id exist to slow down brute-forcing a *low-entropy, human-chosen* secret (a PIN or password). The Recovery Code is not that — it is 140 bits of CSPRNG output (ADR-0005 §3), already far past the point where iterated stretching helps; an attacker who can guess 140 random bits can already break AES-256 with comparable effort. SHA-256 here is expansion (map 140 bits to 256 bits deterministically), not stretching, and adding PBKDF2/Argon2id iterations would only slow down legitimate recovery (relevant to NFR-01-adjacent responsiveness during a support call) for no security benefit. This keeps the KDF surface simple and matches this document's existing SHA-256 use for Machine Hardware ID hashing (§8.1) — one hash primitive, not two.

### 8.5 `Clock` (outbound) — implemented by `SystemClock` in `src-tauri`

```rust
trait Clock {
    fn now(&self) -> Timestamp;
}
```

Already named by ADR-0006; this document fixes its adapter placement (§4.5). Not `async` — reading the system clock is not a suspension point.

### 8.6 `BackupSnapshotWriter` (outbound) — implemented by `infra_backup`

```rust
trait BackupSnapshotWriter {
    async fn take_snapshot(&self, destination: BackupDestination) -> Result<BackupSnapshotLocation, BackupError>;
}
```

`BackupSnapshotLocation` is a minimal port-owned DTO — `path: String`, `created_at: String` (ISO-8601 UTC) — not the full PRD `BackupSnapshot` entity (§4.6/§5: no `id`/`trigger_type`/`storage_location` catalog row is persisted by this document). `BackupDestination` is either the default backup directory (`%APPDATA%\BarangayMS\backups\`) or an explicit path (for Phase 3's manual USB export); the adapter resolves the default. The adapter writes one encrypted archive whose **file manifest** (§4.7) lists `bootstrap.json` (located via the injected `EncryptionCredentialGateway::bootstrap_file_path()`) and the current SQLCipher database file (via `infra_persistence`'s connection, per ADR-0002's sanctioned edge), then the files themselves. No scheduling, retention, purge, or query capability is defined here; §4.6 explains why that boundary is deliberate.

## 9. Expected Adapter and Infrastructure Changes

Named for completeness; implementation is out of scope (§3.2).

- `infra_hardware_id`: implements `MachineHardwareIdSource` via the `wmi` crate. No licensing logic (ADR-0002).
- `infra_licensing`: implements `LicenseSignatureVerifier` via `ed25519-dalek`, holding the embedded public key as adapter configuration.
- `infra_credentials`: implements `EncryptionCredentialGateway` via `keyring`, an AES-256-GCM crate (e.g., `aes-gcm`), and a PBKDF2/SHA-256 crate (e.g., `pbkdf2`, `sha2`) — concrete crate pinning happens in `crates/infra_credentials/Cargo.toml` at implementation time, not this document.
- `infra_persistence`: implements `LicenseGrantRepository` against the SQLCipher connection (§10). This is the only capability this document assigns to `infra_persistence`; its Resident/Household/Certificate repositories are Phase 2.
- `infra_backup`: implements `BackupSnapshotWriter` (§8.6) only — writes one encrypted archive with a file manifest (§4.7) covering `bootstrap.json` (located via the injected `EncryptionCredentialGateway`) and the SQLCipher database file (via `infra_persistence`'s connection/schema code, ADR-0002's sanctioned edge). No scheduling, retention, purge, or the full `BackupSnapshot` repository — those are Phase 3 additions to the same crate.
- `src-tauri`: implements `SystemClock`; at startup on the ordinary (already-established) path, calls `EncryptionCredentialGateway::load_database_key()` directly as composition-root plumbing (§7.6) before wiring `infra_persistence`'s SQLCipher connection; on the first-run path, instead calls `EstablishEncryptionCredentialUseCase` (§7.4, which internally sequences `establish()` then `BackupSnapshotWriter::take_snapshot()` per §4.6) through `app_core`'s public assembly API; exposes Tauri commands for `GetMachineHardwareIdUseCase`, `ActivateLicenseUseCase`, `GetActiveLicenseUseCase`, `EstablishEncryptionCredentialUseCase`, and `RecoverDatabaseEncryptionKeyUseCase`.

## 10. Data Model and Persistence Notes

- **`LicenseGrant` table** (backing `LicenseGrantRepository`): `id` (TEXT, UUIDv7), `machine_hardware_id` (TEXT), `feature_flags` (TEXT, JSON array per ADR-0003), `signature` (TEXT, base64), `activated_at` (TEXT, ISO-8601 UTC). **No** `barangay_code`, `updated_at`, or `sync_status` — ADR-0006 explicitly excludes License state from the Shared Schema Columns. Insert-only: reissuance (Appendix C) always inserts a new row; nothing ever updates or deletes an existing one, mirroring the Audit Trail's own append-only discipline (§4.5) even though `LicenseGrant` is not itself an audit table.
- **`bootstrap.json`** (backing `EncryptionCredentialGateway`, owned by `infra_credentials`, not a SQL table): salt (raw bytes, base64 in JSON) and the AES-256-GCM-wrapped `system_secret` (nonce + ciphertext + tag, base64). Lives at `%APPDATA%\BarangayMS\bootstrap.json`, per ADR-0005, so it is captured by every Backup Snapshot (including the Phase 1 Initial Backup Snapshot, §4.6, and Phase 3's automated snapshots) without new backup plumbing.
- **Initial Backup Snapshot archive** (backing `BackupSnapshotWriter`, §4.6/§4.7/§8.6, owned by `infra_backup`, not a SQL table): one encrypted archive file under `%APPDATA%\BarangayMS\backups\` containing a JSON file manifest (`path` + `role`, two entries: `bootstrap` and `database`) plus `bootstrap.json` and the database file as they exist at first-run completion. No `id`/`trigger_type`/`storage_location` row is persisted anywhere by this document — Phase 3 introduces the `BackupSnapshot` repository when listing/restoring/purging needs one, and extends the manifest with photo/logo/signature entries (§4.7).
- **`AuditLogEntry.timestamp_utc`** column shape only (§4.5): TEXT, ISO-8601 UTC, stamped via `Clock`. No table/entity design here (Phase 2).

## 11. API or Contract Notes: License Key wire envelope

The License Key string the Secretary pastes/uploads is the whole-object Base64 encoding of this JSON envelope:

```json
{
  "machine_hardware_id": "<Crockford Base32 grouped string, e.g. 4ZQK-...>",
  "feature_flags": ["KP_BLOTTER", "TREASURY"],
  "signature": "<Base64 64-byte Ed25519 signature>"
}
```

The Ed25519 signature is computed over the UTF-8 bytes of the canonical 2-field JSON object `{"machine_hardware_id":"...","feature_flags":[...]}` — keys in this fixed order, no whitespace — **before** the `signature` field is added. `infra_licensing`'s `LicenseSignatureVerifier` reconstructs that same canonical 2-field JSON from the parsed payload to verify against. This is concrete enough for `LicenseKeyPayload::parse` (§6.1) and the eventual out-of-scope `bms-keygen` tool to agree on the signed byte sequence without further negotiation; it is not itself a claim about `bms-keygen`'s implementation, which remains out of scope (§3.2).

`feature_flags` uses the exact ADR-0003 SCREAMING_SNAKE_CASE strings; an unrecognized string fails `LicenseKeyPayload::parse` with `LicenseValidationError::MalformedLicenseKey` rather than being silently dropped.

## 12. Testing Strategy

Per HADR-0006 and `testing.md`, gates 1–3 apply to this slice; gates 4–5 do not (no external contract or reference/service composition exists yet).

- **Gate 1 (domain invariant tests + reviewed port contracts):**
  - `LicenseKeyPayload::parse` — valid envelope, malformed JSON, bad Base64, unrecognized feature flag string, wrong-length signature.
  - `LicenseGrant::activate` — matching machine ID succeeds; mismatched machine ID returns `MachineHardwareMismatch` carrying the current ID.
  - `RecoveryCode` smart constructor — valid code accepted; lookalike-character normalization (`O`↔`0`, `I`/`L`↔`1`) accepted; bad checksum rejected; wrong length rejected.
  - `MachineHardwareId`, `FeatureFlag` — construction/formatting/serde-shape tests (no ceremonial trait-shape tests for behaviorless traits, per HADR-0006).
  - Review of all five outbound port signatures (§8) for identity, ordering (`find_current`), atomicity (`save` as pure insert), and error-classification stability.
- **Gate 2 (compiling, intentionally failing use-case tests with local fakes):**
  - `ActivateLicenseUseCase` — fake `LicenseSignatureVerifier` returning success/`InvalidSignature`; fake `MachineHardwareIdSource`; fake `LicenseGrantRepository`. Tests must fail for "use case not yet implemented," not fixture/compilation reasons.
  - `EstablishEncryptionCredentialUseCase` — fake `EncryptionCredentialGateway` and fake `BackupSnapshotWriter`; must cover both orderings: `establish()` succeeds + `take_snapshot()` succeeds → `RecoveryCode` returned; `establish()` succeeds + `take_snapshot()` fails → `InitialBackupFailed` returned and no `RecoveryCode` value reachable from the test's assertions (§4.6).
  - `GetActiveLicenseUseCase`, `RecoverDatabaseEncryptionKeyUseCase`, `GetMachineHardwareIdUseCase` — same pattern, one fake per port, no real I/O (`testing.md`'s "test-local outbound fakes perform no network, filesystem, database, or process I/O").
- **Gate 3 (minimal green core):** the above tests pass; `cargo check -p app_core --all-targets --locked`, `cargo test -p app_core --locked`, `cargo test -p app_core --doc --locked` succeed with no adapter, transport, or persistence implementation present.
- **Beyond this document (future Gate 4/5 tickets, not designed here):** adapter translation tests for each `infra_*` crate (WMI failure mapping, Ed25519 verify failure mapping, keyring-unavailable mapping, AES-GCM authentication-failure mapping), a composition smoke wiring all five adapters plus `SystemClock`, and an end-to-end recovery test matching ADR-0005's Confirmation bullet (enter Recovery Code → checksum validates → AES-GCM unwrap succeeds → `system_secret` recovered → SQLCipher key re-derived, using only a Backup Snapshot and the transcribed code, no Credential Manager or original hardware dependency).

## 13. Implementation Plan

Dependency-ordered, core-first per HADR-0004/hexon guidance:

1. `app_core::domain` value objects, `LicenseGrant` entity, domain errors (§6) — no ports needed yet.
2. `app_core::ports` outbound trait definitions (§8) — signatures only.
3. `app_core::application` use cases against the port traits (§7) — Gate 1 complete at this point.
4. Gate 2: failing use-case tests with local fakes (§12).
5. Gate 3: minimal green core.
6. (Future tickets, HADR-0007 gates 4–5) `infra_hardware_id`, `infra_licensing`, `infra_credentials`, `infra_persistence`'s `LicenseGrantRepository`, `infra_backup`'s `BackupSnapshotWriter` (§4.6/§8.6 only — no scheduling/retention), and `src-tauri`'s `SystemClock` + composition + Tauri commands.
7. (Future map tickets) Core Engine Phase 2 technical design — RBAC enforcement, Resident/Household Registry, Certificate Generation & Control Number Sequencing, and the `AuditLogEntry` entity built on §4.5's timestamping decision.
8. (Future map tickets) Core Engine Phase 3 technical design — Backup & Recovery engine (`infra_backup`), Barangay Identity/Officials/Templates/Purok/Theming/Dashboard, and the License/Feature-Flag UI reactivity requirement deferred in §3.2.

## 14. Acceptance Criteria / Completion Checklist

- [ ] `app_core` compiles with zero Tauri, web, database, OS-credential-store, or cryptographic crate dependencies (only ordinary value crates such as `uuid`) — confirms §4.4.
- [ ] `LicenseGrant::activate` cannot produce a value when `payload.machine_hardware_id != local_machine_id`; no code path constructs `LicenseGrant` without going through it.
- [ ] `ActivateLicenseUseCase` calls `LicenseSignatureVerifier::verify` before `LicenseGrant::activate`, and never constructs a `LicenseGrant` when verification fails.
- [ ] `RecoveryCode`'s smart constructor rejects a bad checksum before any port call — verified by a Gate 1 test, not deferred to `infra_credentials`.
- [ ] All five outbound ports (§8.1–§8.4, §8.6) plus `Clock` (§8.5) are declared `async` (except `Clock`) and object-safe behind `Arc<dyn Trait + Send + Sync>`.
- [ ] `LicenseGrantRepository::save` is insert-only; no port method updates or deletes an existing `LicenseGrant` row.
- [ ] `EstablishEncryptionCredentialUseCase` never returns a `RecoveryCode` unless `BackupSnapshotWriter::take_snapshot()` has also succeeded — verified by a Gate 2 test asserting the failure-ordering case (§4.6, §12).
- [ ] `cargo check -p app_core --all-targets --locked`, `cargo test -p app_core --locked`, and `cargo test -p app_core --doc --locked` all pass with no adapter code present (Gate 3, `testing.md`).
- [ ] Every PRD §9.5/§9.6/§9.8 requirement this document claims (§5) is either covered by a named use case/port or explicitly deferred with a reason; none are silently dropped.

## 15. Open Questions

All four items this section originally carried are now resolved; none remain open. They are retained below as a record of where each resolution lives.

- ~~**Control Number Format token grammar** (PRD §14)~~ — **resolved by [ADR-0007](../../../adr/implemented/architecture/2026-08-18-0007-define-control-number-format-token-grammar.md)**: brace-delimited `{YYYY}` + `{N…}` tokens, literal prefix text, per-(type × PHT-year) sequence scope, integer counter + frozen rendered string. Phase 2 inherits it.
- ~~**Template Variable enumeration** (PRD §14)~~ — **resolved by [ADR-0008](../../../adr/implemented/architecture/2026-08-18-0008-define-template-variable-enumeration.md)**: dotted `{{source.field}}` syntax, closed 32-variable developer-seeded catalog (29 text + 3 image), standard `captain`/`secretary` position keys, empty-string on missing data, computed values frozen at issuance. Phase 3 inherits it.
- ~~**`infra_backup`'s exact interface to `bootstrap.json`**~~ — **resolved by §4.7**: `bootstrap.json`'s location is owned by `infra_credentials` and exposed via `EncryptionCredentialGateway::bootstrap_file_path()`; the archive carries an explicit JSON file manifest (`path` + `role`) rather than a hardcoded list; `take_snapshot` takes a `BackupDestination` (default dir or explicit path). Phase 3 extends the manifest for photos/logos/signatures.
- ~~**MSVC toolchain path** (§4.3.1)~~ — **resolved by §4.3.1**: `x86_64-pc-windows-gnu` is the sole binding target; the MSVC path is revisited only if a concrete blocker (a GNU-incompatible dependency, or a CI/installer requirement forcing MSVC ABI) appears, at which point the fix is installing the Perl CPAN modules in CI/dev images.
