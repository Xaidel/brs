# Core Engine Technical Specification

## Phase 1: Technical Feasibility Spike & Offline Licensing / Key-Management Core

Status: Proposed

Owner: Engineering

Date: 2026-08-06

Related documents:

- [Package README](../README.md)
- [Core Engine PRD](../PRD.core-engine.md) — §9.5, §9.6, §11 (`LicenseGrant`), NFR-01, NFR-03, NFR-04, NFR-05
- [Appendix A: Technical Architecture Direction](../appendix-a-technical-architecture-direction.md) — stack choice and mandatory spike
- [Appendix B: Key Derivation & Recovery Rationale](../appendix-b-key-derivation-and-recovery.md) — key-derivation and Recovery Code rationale
- [Appendix C: License Reissuance SOP](../appendix-c-license-reissuance-sop.md) — hardware-mismatch support workflow
- [ADR-0001: Place app_core in its own workspace crate](../../../adr/ADR-0001-place-app-core-in-its-own-workspace-crate.md)
- [backend_arch_docs/architecture.md](../../../../backend_arch_docs/architecture.md)
- [backend_arch_docs/dependency-rules.md](../../../../backend_arch_docs/dependency-rules.md)
- [backend_arch_docs/naming-conventions.md](../../../../backend_arch_docs/naming-conventions.md)
- [backend_arch_docs/testing.md](../../../../backend_arch_docs/testing.md)
- [Spike Results Log](spike-results.md) (phase-local, non-normative evidence record)

---

## 1. Purpose

This document defines the first implementation slice of the Core Engine package: the
mandatory Phase 1 technical feasibility spike, and the `crates/app_core` domain,
application, and ports content needed for offline Ed25519 license-key validation and
at-rest database key management/recovery.

It answers, for a human or coding-agent implementer:

- What must the Phase 1 spike prove, and what evidence counts as pass/fail, before any
  further Phase 1 work proceeds?
- Which domain concepts, entities, value objects, domain services, and domain events
  belong to `app_core::domain` for licensing and key management, and which do not?
- Which named use cases belong to `app_core::application`, and how do they sequence
  authorization-adjacent, validation, and persistence concerns?
- Which inbound and outbound port contracts `app_core::ports` must expose so that a
  later `infra_*` crate can implement hardware identification, OS-protected credential
  storage, and persistence without re-deciding this slice's contracts?
- What Gate 1–3 evidence (per `backend_arch_docs/testing.md`) is required to consider
  this slice implemented?

This document does **not** define: adapter implementations, `infra_*` crate structure,
Tauri command wiring, SQLCipher schema/migration mechanics, or any UI. It also does not
re-litigate PRD product scope — the PRD remains normative; this document maps its
Phase 1-relevant requirements to `app_core` responsibilities.

## 2. Terminology

Reused verbatim from the [PRD glossary](../PRD.core-engine.md#21-canonical-terms):
**License Key**, **Machine Hardware ID**, **Feature Flag**, **Recovery Code**,
**LicenseGrant** (PRD §11 conceptual entity).

Introduced by this document (implementation vocabulary, not new product concepts):

| Term | Meaning |
|---|---|
| `system_secret` | The Appendix B installation secret. Modeled here as the `SystemSecret` domain value object. |
| `EncryptionCredential` | New aggregate introduced by this TDD to hold the non-secret, persisted parts of the encryption key material (a salt and metadata) — distinct from `LicenseGrant`. Not a PRD-named entity; do not confuse with `LicenseGrant`. |
| `DatabaseEncryptionKey` | The derived SQLCipher key material. Never the same value as `system_secret`; it is PBKDF2-HMAC-SHA256 output. |
| Spike | The Appendix A §4 mandatory technical feasibility check. Distinct from the Gate 1–3 approval sequence in `backend_arch_docs/testing.md`; it is a prerequisite that gates Gate 1, not a gate itself. |
| Gate 1 / Gate 2 / Gate 3 | As defined in HADR-0006 and `backend_arch_docs/testing.md`. This document does not redefine them. |

Forbidden synonyms: do not use "hardware fingerprint" for Machine Hardware ID, "master
key" for `DatabaseEncryptionKey`, or "license token" for License Key — use the PRD terms
so downstream code, tests, and error messages stay searchable.

## 3. Scope

### 3.1 In scope

- The Appendix A §4 mandatory technical spike (static-linked SQLCipher + FTS5
  compilation, FTS5 performance with both features enabled, installer size budget) and
  its pass/fail evidence.
- `app_core::domain` and `app_core::application` content for:
  - Offline Ed25519 License Key validation (signature check + Machine Hardware ID
    binding check) per PRD §9.5.
  - Database encryption key derivation, first-run credential establishment, and
    Recovery Code generation/redemption per PRD §9.6 and Appendix B.
- `app_core::ports` inbound capabilities (use-case-facing traits, commands, results,
  errors) and outbound port contracts (repository/store/gateway traits) that a later
  `infra_*` crate must satisfy — signatures and semantics only, not implementations.
- The `LicenseGrant` conceptual entity from PRD §11, to the extent Phase 1 needs to
  represent and persist validated license state. Concrete SQLCipher schema/migration
  mechanics are explicitly deferred (see §11 of this document).
- A new `EncryptionCredential` aggregate (introduced by this TDD, not the PRD) to
  represent the persisted, non-secret half of the encryption key material.

### 3.2 Out of scope for this document

- Resident/Household registry, Certificate generation and Control Number sequencing,
  RBAC/Staff Accounts/Roles/Permissions, Immutable Audit Trail, the automated
  rolling-snapshot Backup & Recovery engine, Barangay Identity/Officials/Templates/
  Purok/Theming/Dashboard configuration — all Phase 2/3 per the
  [package README's phase map](../README.md#phase-map).
- Any UI or Tauri command-handler wiring. The License Settings screen is Phase 3. Per
  [ADR-0001](../../../adr/ADR-0001-place-app-core-in-its-own-workspace-crate.md),
  wiring `app_core` into `src-tauri`'s command handlers is explicitly deferred to a
  later, separately scoped change — this document does not authorize it.
- Any `infra_*` crate or adapter implementation (hardware ID reading, OS credential
  store access, SQLCipher persistence). No `infra_*` crate exists yet; this document
  defines the ports such implementations must satisfy, not the implementations.
- Gate 4/5 evidence (`backend_arch_docs/testing.md`, HADR-0007): these require an
  exact external contract and a runnable reference/service composition, neither of
  which exists at Phase 1. Only Gates 1–3 apply here.
- The vendor-side `bms-keygen` tool and License Registry (Appendix C §3) — internal
  tooling, not part of this product.
- Exact PBKDF2 iteration count, salt/secret byte lengths, and Recovery Code
  character/grouping format — left as implementation-mechanics judgment per Appendix B
  §5 ("subject to normal engineering judgment"), bounded by the invariants stated in
  §7 below.

## 4. Architecture Constraints

- This repository's real Hexon layout is Rust, not the illustrative Go paths in the
  `tdd-writer` skill's `hexon-tdd-guidance.md` reference. Per
  `backend_arch_docs/architecture.md`, `dependency-rules.md`, and ADR-0001, the
  authoritative paths are:
  - `crates/app_core/src/domain/` — private `mod domain` (crate-only visibility).
  - `crates/app_core/src/application/` — private `mod application` (crate-only
    visibility).
  - `crates/app_core/src/ports/` — the crate's only `pub mod`; re-exports the core
    assembly function. This is the sole supported external API of `app_core`.
  - Adapters and infrastructure (transport, persistence, provider translation,
    composition root, runtime lifecycle) belong exclusively to self-contained
    `infra_*` deployment crates (HADR-0002, HADR-0003) — never inside `app_core`.
    No `infra_*` crate exists yet in this repository.
- Source dependency direction (architecture.md): `infra_* -> app_core::ports and
  assembly`; `app_core::application -> app_core::ports and private domain`; `private
  domain -> no application or infrastructure`. `app_core` depends on no workspace
  crate (ADR-0001); it may depend on ordinary (non-workspace) Cargo crates, subject to
  review, as a visible diff to `crates/app_core/Cargo.toml`.
- **Working interpretation — flagged for reviewer confirmation (see §15 Open
  Questions):** `domain` code performs no I/O and depends on no port, application, or
  infrastructure concept (per `crates/app_core/src/domain/mod.rs` and
  `dependency-rules.md`). This document reads "no I/O" as excluding external-system
  access (filesystem, OS credential store, hardware queries, OS RNG syscalls) but
  permitting deterministic, pure cryptographic-primitive computation (Ed25519
  signature verification, PBKDF2-HMAC-SHA256 derivation) as ordinary domain-service
  logic, since `architecture.md` describes `app_core` as "framework-free," not
  "dependency-free," and these crates are algorithm libraries, not
  transport/storage/framework/SDK types. All *sourcing* of random bytes (salts,
  secrets, entity IDs) and all *reading* of external state (hardware IDs, OS
  credential store, persisted records) still go through outbound ports — only the
  deterministic verify/derive computation itself is domain-owned. §7 and §15 apply
  this consistently and flag it for explicit sign-off.
- Async ports use `async-trait`, stored behind `Arc<dyn Trait + Send + Sync>`
  (architecture.md) — every outbound port defined in §9 is declared async, even though
  no concrete adapter exists yet, so Gate 1 does not need to be repeated when an
  `infra_*` crate implements it.
- Naming: inbound traits are named after capabilities (`ActivateLicense`, not
  `LicenseService`); concrete orchestration types use the `UseCase` suffix; outbound
  ports use `{Noun}Repository`, `{Noun}Store`, or `{Noun}Gateway`; domain events are
  past-tense business facts; domain services are named after their business rule, not
  as generic "manager" containers; transport/port DTO names stay private to `ports`
  (`backend_arch_docs/naming-conventions.md`).
- HADR-0006 governs Gates 1–3, which this slice must satisfy. HADR-0007's Gates 4–5
  require an external contract and reference/service composition that do not exist at
  Phase 1 — this document does not attempt to satisfy them.
- Per PRD NFR-05 (single logical write path), any future `infra_*` persistence adapter
  for `LicenseGrantRepository` and `EncryptionCredentialRepository` must be the sole
  writer of the underlying database file; this document's port contracts must not
  presuppose a design that would require more than one writer.
- Appendix A, B, and C are informative only (per the package README's source
  precedence rule). Where this document's design deviates from an appendix's
  suggested mechanics, the deviation and rationale are called out explicitly rather
  than silently substituted.

## 5. PRD-to-Implementation Mapping

| PRD requirement | Disposition | Notes |
|---|---|---|
| §9.5 bullet 1 — compute Machine Hardware ID at startup; display in Settings | Partially covered | Computation capability is covered via the `MachineHardwareIdGateway` outbound port and `GetMachineHardwareIdUseCase` inbound capability (§8, §9). The Settings *display* is UI — deferred to Phase 3 per the package README. |
| §9.5 bullet 2 — validate License Key entirely offline via Ed25519 signature against embedded public key; no network required | Covered | §7 domain signature verification; no outbound port performs network I/O anywhere in this slice, satisfying NFR-04 by construction. |
| §9.5 bullet 3 — verify submitted Machine Hardware ID matches local installation before enabling Feature Flags | Covered | Enforced as an entity-construction invariant on `LicenseGrant` (§7); a `LicenseGrant` cannot exist in an unvalidated state. |
| §9.5 bullet 4 — persist resulting Feature Flags; make newly unlocked module navigation available without restart | Partially covered | Persistence of Feature Flags is covered (`LicenseGrantRepository`, `GetActiveLicenseUseCase`). "Navigation available without restart" is a UI/nav concern — deferred to Phase 3. |
| §9.5 bullet 5 — documented re-issuance path on hardware change, without altering existing records | Covered (app behavior only) | `ActivateLicenseUseCase` surfaces a distinct `MachineHardwareMismatch` error carrying the current Machine Hardware ID, which is what Appendix C's SOP requires the app to display. The SOP's vendor-side signing process is out of scope (Appendix C §3). |
| §9.6 bullet 1 — encrypt local database at rest | Partially covered | This slice supplies `DatabaseEncryptionKey` derivation and establishment; actually opening an encrypted `rusqlite`/SQLCipher connection with that key is persistence-adapter work with no `infra_*` crate to own it yet — deferred, stated explicitly rather than silently omitted. |
| §9.6 bullet 2 — key derivable without Machine Hardware ID as input | Covered | `DatabaseEncryptionKeyDerivation` domain service takes only `SystemSecret` + `EncryptionSalt`; Machine Hardware ID never appears in its inputs (§7). This is a deliberate, testable separation from license validation's Machine Hardware ID usage. |
| §9.6 bullet 3 — first-run human-transcribable Recovery Code, independent unlock path | Covered | `EstablishEncryptionCredentialUseCase` and `RecoverDatabaseEncryptionKeyUseCase` (§8) model generation and independent redemption. |
| §9.6 bullet 4 — key must not be recoverable by inspecting the installed binary alone | Covered as a constraint | `SystemSecret` and `RecoveryCode` are never persisted by `app_core` in plaintext; §9 and §11 state this explicitly as a binding constraint on any future adapter. |
| §11 `LicenseGrant` entity | Covered (entity + port), persistence deferred | See §7 (entity), §9 (repository port), §11 of this document (data model notes — schema/migration mechanics deferred). |
| NFR-01 (startup <3s on i3/4GB) | Constraint noted | PBKDF2 iteration count and hardware-ID/license-check cost must fit the startup budget; exact tuning is implementation-mechanics per Appendix B §5, not fixed by this document. |
| NFR-03 (installer <30MB) | Covered by spike | §6. |
| NFR-04 (100% offline operation) | Covered | No port or domain concept in this slice performs network I/O. |
| NFR-05 (single logical write path) | Constraint noted | See §4 and §9; enforcement is a future `infra_*` concern, but this document's ports do not preclude it. |
| All other PRD sections (§9.1–§9.4, §9.7–§9.14, other §11 entities, other NFRs) | Excluded | Out of scope for Phase 1 per the package README's phase map; not designed here. |

## 6. Technical Feasibility Spike (Prerequisite Gate)

Per [Appendix A §4](../appendix-a-technical-architecture-direction.md#4-mandatory-technical-spike-phase-1),
this spike must run and pass **before** any Gate 1 domain/ports work in §7–§9 is
merged. It is not one of the HADR-0006 Gates 1–3; it is a prerequisite feasibility
check on the chosen stack itself.

### 6.1 What must be proven

| # | Check | Pass criterion |
|---|---|---|
| S1 | Static linking of `sqlcipher` in `rusqlite` alongside the `fts5` Cargo feature compiles cleanly | A workspace build with both features enabled succeeds with no linker errors, on the target Windows toolchain, from a clean checkout. |
| S2 | FTS5 search performance holds with both features enabled simultaneously | A representative FTS5 query against a seeded dataset returns in a time consistent with NFR-02's 200ms/50,000-record target (exact benchmark harness is implementation-mechanics; the number and method used must be recorded in the spike results log). |
| S3 | Resulting installer binary size stays under the NFR-03 30MB budget | A built installer (or a representative proxy artifact if a full installer pipeline isn't wired yet) measures under 30MB, with the measurement method recorded. |

### 6.2 Evidence

Record raw results in [`spike-results.md`](spike-results.md) (phase-local, non-normative
evidence log — not part of this document's normative contract). At minimum capture: build
command and output for S1, benchmark methodology and measured latency for S2, artifact
size and measurement method for S3, target hardware/toolchain versions, and a pass/fail
verdict with date.

### 6.3 Failure path

If any of S1–S3 fail, Appendix A §4 requires revisiting the stack choice **before**
committing further implementation time on §7–§9. This document does not define a
fallback stack; a failed spike returns the Phase 1 slice to re-scoping (a new or amended
technical direction, likely requiring an ADR) rather than authorizing a workaround
inside this TDD.

## 7. Expected Domain Changes

`crates/app_core/src/domain/` (private `mod domain`). No I/O; no ports, adapters,
persistence schema, or API contract concepts appear here — only value objects,
entities, domain services, domain events, and domain errors.

### 7.1 Value objects

| Value object | Validates | Notes |
|---|---|---|
| `MachineHardwareId` | Non-empty opaque fingerprint string | Reused PRD term. Equality-comparable; carries no derivation logic (derivation is I/O — see §9). |
| `LicensePublicKey` | Fixed-length Ed25519 public key bytes (32 bytes) | The "embedded in the application binary" key from PRD §9.5. Modeled as a domain-owned constant (see assumption below), not injected configuration, since Phase 1 has no key-rotation requirement. |
| `LicenseSignature` | Fixed-length Ed25519 signature bytes (64 bytes) | — |
| `FeatureFlagSet` | Non-empty, de-duplicated set of validated flag-key strings | Exact flag-key taxonomy (`CORE`, `KP`, `TREASURY`, …) is left open, mirroring how the PRD leaves the Permission taxonomy TDD-level — low risk, does not affect this slice's contracts. |
| `LicenseKeyPayload` | Parses/decodes a submitted License Key string into its structural parts (bound `MachineHardwareId`, `FeatureFlagSet`, `LicenseSignature`, signed metadata) | Smart constructor; malformed input fails with a domain error before signature verification is attempted. |
| `EncryptionSalt` | Fixed-length random salt bytes | Exact length is implementation-mechanics (Appendix B §5); construction validates length only, does not generate the bytes (generation is I/O — see §9). |
| `SystemSecret` | Fixed-length random installation-secret bytes | Same generation/validation split as `EncryptionSalt`. Must not implement `Debug`/`Display` in a way that leaks its bytes — a logging-safety constraint carried into implementation. |
| `DatabaseEncryptionKey` | Fixed-length derived key bytes | Output-only value object; never constructed directly from user input, only from `DatabaseEncryptionKeyDerivation` (below). Same logging-safety constraint as `SystemSecret`. |
| `RecoveryCode` | Human-transcribable formatted string (e.g., grouped alphanumeric) | Exact character set/grouping is implementation-mechanics (Appendix B §5); construction validates format, not content correctness. |
| `Timestamp` | Wraps a UTC instant | Framework-agnostic; concrete time-source crate is an adapter concern behind the `Clock` port (§9). |

### 7.2 Entities

- **`LicenseGrant`** (PRD §11 conceptual entity). Fields: `id`, `machine_hardware_id`,
  `feature_flags`, `signature_metadata` (the `LicenseSignature` plus any signed,
  non-secret metadata carried by the payload), `activated_at`. **Construction
  invariant:** the only public constructor is
  `LicenseGrant::activate(payload: LicenseKeyPayload, local_machine_id:
  &MachineHardwareId, public_key: &LicensePublicKey, id: LicenseGrantId, activated_at:
  Timestamp) -> Result<LicenseGrant, LicenseValidationError>`, which internally
  performs signature verification (via `LicenseSignatureVerification`, §7.3) and the
  Machine Hardware ID equality check before a `LicenseGrant` value can exist. This
  makes "a `LicenseGrant`'s Feature Flags are active" and "a `LicenseGrant` passed
  validation" the same fact by construction — there is no code path that produces an
  unvalidated `LicenseGrant`, directly satisfying PRD §9.5 bullet 3's ordering
  requirement ("only then treat its Feature Flags as active").
- **`EncryptionCredential`** (new aggregate, introduced by this TDD — not a PRD-named
  entity). Fields: `id`, `salt` (`EncryptionSalt`), `established_at` (`Timestamp`).
  Deliberately holds **no** secret material — `system_secret` and `RecoveryCode` are
  never fields of this entity or persisted anywhere by `app_core` (see §7.4 and §9).

### 7.3 Domain services (stateless, named after their business rule)

- **`LicenseSignatureVerification`** — verifies a `LicenseKeyPayload`'s
  `LicenseSignature` against a `LicensePublicKey` using Ed25519. Pure computation;
  fails with `LicenseValidationError::InvalidSignature`.
- **`DatabaseEncryptionKeyDerivation`** — derives a `DatabaseEncryptionKey` from a
  `SystemSecret` and `EncryptionSalt` via PBKDF2-HMAC-SHA256, deliberately excluding
  `MachineHardwareId` from its inputs (PRD §9.6 bullet 2; Appendix B §3). Pure,
  deterministic: same secret+salt always yields the same key.
- **`RecoveryCodeEncoding`** — a reversible, deterministic transform between
  `SystemSecret` and `RecoveryCode` (`encode`/`decode`). Recommend an embedded
  checksum/parity in the encoding so a mistyped Recovery Code fails fast with a
  domain error rather than silently deriving a wrong key — exact scheme is
  implementation-mechanics.

### 7.4 Domain events (past-tense business facts)

- `LicenseGrantActivated { license_grant_id, machine_hardware_id, feature_flags,
  activated_at }`
- `EncryptionCredentialEstablished { encryption_credential_id, established_at }`
- `RecoveryCodeIssued { encryption_credential_id, issued_at }`

**Invariant:** none of these events carry `SystemSecret`, `RecoveryCode`, or
`DatabaseEncryptionKey` payloads — only non-secret identifiers and timestamps. This
matters because domain events may later be drained and logged/audited by application
code (architecture.md), and secret material must never enter that path.

No event consumer exists in Phase 1 (no Audit Trail, no UI) — events are defined for
forward compatibility with Phase 2/3 consumers and are drained/discarded by the
application layer for now.

### 7.5 Domain errors

- `LicenseValidationError`: `MalformedLicenseKey`, `InvalidSignature`,
  `MachineHardwareMismatch { current_machine_hardware_id: MachineHardwareId }`.
- Value-object construction errors (one small typed error per value object, e.g.
  invalid length/format) — exact enum shapes are Gate 1 review detail, not fixed here.

### 7.6 Explicit domain exclusions

No ports, no adapters, no persistence schema, no API/Tauri-command contracts, no
`keyring`/`rusqlite`/hardware-query crates, no application orchestration, no
infrastructure wiring appear in `domain`.

## 8. Expected Application Changes

`crates/app_core/src/application/` (private `mod application`). Depends only on domain
and ports. Named use cases, each with an `{Verb}{Noun}UseCase` concrete type
implementing a capability-named inbound port trait from §9.

| Use case | Inbound port trait | Summary |
|---|---|---|
| `ActivateLicenseUseCase` | `ActivateLicense` | Parses a submitted License Key, activates a `LicenseGrant` (signature + binding check happens inside `LicenseGrant::activate`, §7.2), persists it, emits `LicenseGrantActivated`. |
| `GetActiveLicenseUseCase` | `GetActiveLicense` | Returns the currently persisted `LicenseGrant`'s Feature Flags (empty set if none activated — Core Engine itself is always unlocked regardless). |
| `GetMachineHardwareIdUseCase` | `GetMachineHardwareId` | Returns the local installation's current Machine Hardware ID, for a future Settings screen and for the Appendix C reissuance flow. |
| `EstablishEncryptionCredentialUseCase` | `EstablishEncryptionCredential` | First-run only. Generates salt + `system_secret`, derives the initial `DatabaseEncryptionKey`, generates the Recovery Code, persists non-secret credential metadata, stores the secret in OS-protected storage. |
| `DeriveDatabaseEncryptionKeyUseCase` | `DeriveDatabaseEncryptionKey` | Normal (non-first-run) path: loads persisted salt + stored `system_secret`, derives the current `DatabaseEncryptionKey`. |
| `RecoverDatabaseEncryptionKeyUseCase` | `RecoverDatabaseEncryptionKey` | Fallback path: given a submitted Recovery Code, decodes it back to `system_secret` independent of OS-protected storage, then derives the same `DatabaseEncryptionKey`. |

### 8.1 Sequencing — `ActivateLicenseUseCase`

1. Read `local_machine_id` from `MachineHardwareIdGateway` (outbound port, §9). Gateway
   failure maps to `HardwareIdentityUnavailable`, distinct from a binding mismatch.
2. Generate a new `LicenseGrantId` via `IdGenerator` (§9) and read `activated_at` via
   `Clock` (§9).
3. Call `LicenseGrant::activate(payload, &local_machine_id, &EMBEDDED_LICENSE_PUBLIC_KEY,
   id, activated_at)`. Malformed input, invalid signature, or a hardware mismatch all
   surface as typed `LicenseValidationError` variants (§7.5), mapped to
   `ActivateLicenseError` at the port boundary (§9).
4. On success, persist via `LicenseGrantRepository::save_current` (replaces any prior
   grant — see §11 on singleton semantics). A repository failure maps to an opaque
   `ActivateLicenseError::PersistenceFailure` (logged, never serialized, per
   HADR-0005).
5. Emit `LicenseGrantActivated`; drain/discard (no consumer yet, §7.4).
6. Return `ActivateLicenseResult { feature_flags, activated_at }`.

### 8.2 Sequencing — `EstablishEncryptionCredentialUseCase`

1. Guard: `EncryptionCredentialRepository::load_current()` must return `None`; if
   `Some`, fail with `EstablishEncryptionCredentialError::AlreadyEstablished` — this
   use case must run at most once per installation, since re-running it would orphan
   the encrypted data created under the previous credential.
2. Generate `id` (`IdGenerator`), salt bytes and secret bytes (`RandomnessGateway`,
   §9); construct `EncryptionSalt` and `SystemSecret` (domain validation).
3. Derive `DatabaseEncryptionKey` via `DatabaseEncryptionKeyDerivation` — fail fast
   here (pure computation) before any I/O is attempted.
4. Encode `RecoveryCode` via `RecoveryCodeEncoding`.
5. **Ordering rule:** call `SystemSecretStore::store(secret)` **before**
   `EncryptionCredentialRepository::save_current(record)`. If the credential-store
   write fails, no orphaned `EncryptionCredentialRecord` exists yet. If the repository
   write fails after the credential-store write succeeded, the use case may safely
   retry the repository write alone (the store operation is idempotent-safe to
   overwrite with the same secret).
6. Emit `EncryptionCredentialEstablished` and `RecoveryCodeIssued` (metadata only, no
   secret payload — §7.4).
7. Return `EstablishEncryptionCredentialResult { recovery_code, database_encryption_key }`.
   The caller (a future `infra_*` crate) uses `database_encryption_key` to initialize
   the encrypted database for the first time and displays `recovery_code` exactly once
   — both are out of scope for this document to implement, but this use case's return
   shape is what makes them possible without further `app_core` changes later.

### 8.3 Sequencing — `DeriveDatabaseEncryptionKeyUseCase`

1. `EncryptionCredentialRepository::load_current()` — `None` maps to
   `DeriveDatabaseEncryptionKeyError::NotEstablished` (caller should route to
   `EstablishEncryptionCredentialUseCase` instead; first run has not happened).
2. `SystemSecretStore::load()` — `None` maps to
   `DeriveDatabaseEncryptionKeyError::SystemSecretUnavailable` (caller should fall back
   to `RecoverDatabaseEncryptionKeyUseCase`).
3. Derive via `DatabaseEncryptionKeyDerivation` using the loaded secret and persisted
   salt. Read-only; no writes.
4. Return `DeriveDatabaseEncryptionKeyResult { database_encryption_key }`.

### 8.4 Sequencing — `RecoverDatabaseEncryptionKeyUseCase`

1. Parse the submitted Recovery Code string into `RecoveryCode` (domain validation) —
   malformed input fails with `RecoverDatabaseEncryptionKeyError::MalformedRecoveryCode`.
2. Decode via `RecoveryCodeEncoding::decode` to recover `SystemSecret` — a checksum/
   parity failure maps to `RecoverDatabaseEncryptionKeyError::InvalidRecoveryCode`.
3. `EncryptionCredentialRepository::load_current()` — `None` maps to `NotEstablished`.
4. Derive via `DatabaseEncryptionKeyDerivation` using the recovered secret and
   persisted salt.
5. Return `RecoverDatabaseEncryptionKeyResult { database_encryption_key }`. **Scope
   boundary:** this use case cannot itself confirm the recovered key actually opens
   the encrypted database — that confirmation only happens when a future persistence
   adapter attempts to open it. This document proves the Recovery Code path is
   independent of `SystemSecretStore`, not that any specific database is readable with
   it.

### 8.5 Sequencing — `GetActiveLicenseUseCase` / `GetMachineHardwareIdUseCase`

Simple read-throughs of `LicenseGrantRepository::load_current()` and
`MachineHardwareIdGateway::current()` respectively; no domain invariants beyond value
construction.

### 8.6 Core assembly

`ports::assemble(...)` (the crate's public assembly function, per architecture.md)
takes the outbound port implementations in §9 as `Arc<dyn Trait + Send + Sync>`
constructor arguments and returns a bundle of the six inbound port trait objects
above. Phase 1 defines and tests this function against fakes (§12); no concrete
`infra_*` implementation is wired to it yet.

## 9. Expected Ports Changes

`crates/app_core/src/ports/` (the crate's only `pub mod`). Owns inbound/outbound
contracts, port-level commands/results/errors, and re-exports `assemble(...)`.

### 9.1 Inbound ports (capability-named traits)

| Trait | Command/Query | Result | Error (key variants) |
|---|---|---|---|
| `ActivateLicense` | `ActivateLicenseCommand { license_key: String }` | `ActivateLicenseResult { feature_flags, activated_at }` | `MalformedLicenseKey`, `InvalidSignature`, `MachineHardwareMismatch { current_machine_hardware_id }`, `HardwareIdentityUnavailable`, `PersistenceFailure` |
| `GetActiveLicense` | — | `GetActiveLicenseResult { feature_flags, machine_hardware_id: Option<..>, activated_at: Option<..> }` | `PersistenceFailure` |
| `GetMachineHardwareId` | — | `GetMachineHardwareIdResult { machine_hardware_id }` | `HardwareIdentityUnavailable` |
| `EstablishEncryptionCredential` | — | `EstablishEncryptionCredentialResult { recovery_code, database_encryption_key }` | `AlreadyEstablished`, `SecretStoreFailure`, `PersistenceFailure` |
| `DeriveDatabaseEncryptionKey` | — | `DeriveDatabaseEncryptionKeyResult { database_encryption_key }` | `NotEstablished`, `SystemSecretUnavailable` |
| `RecoverDatabaseEncryptionKey` | `RecoverDatabaseEncryptionKeyCommand { recovery_code: String }` | `RecoverDatabaseEncryptionKeyResult { database_encryption_key }` | `MalformedRecoveryCode`, `InvalidRecoveryCode`, `NotEstablished` |

Errors are stable, typed, and never wrap unsafe-to-serialize internals (HADR-0005);
underlying causes are logged, not returned to the caller beyond a classified variant.

### 9.2 Outbound ports

| Port | Suffix rationale | Responsibility | Async |
|---|---|---|---|
| `MachineHardwareIdGateway` | Gateway — external system query | Reads the real CPU ID + motherboard serial and returns the current `MachineHardwareId`. Reading real hardware is I/O — this is exactly why it is a port, not a domain computation. | Yes |
| `SystemSecretStore` | Store — protected secret storage | Stores/loads `SystemSecret` in OS-protected credential storage (Appendix B suggests Windows Credential Manager via the `keyring` crate — informative, not mandated). | Yes |
| `LicenseGrantRepository` | Repository — persisted aggregate state | `save_current(record: LicenseGrantRecord)`, `load_current() -> Option<LicenseGrantRecord>`. Exchanges a port-owned record, not the private `LicenseGrant` entity (HADR-0005). | Yes |
| `EncryptionCredentialRepository` | Repository — persisted aggregate state | `save_current(record: EncryptionCredentialRecord)`, `load_current() -> Option<EncryptionCredentialRecord>`. Never exchanges secret material (§9.3). | Yes |
| `RandomnessGateway` | Gateway — external entropy source | `generate_bytes(length: usize) -> Vec<u8>`, CSPRNG-grade. Introduced so salt/secret *generation* — which ultimately reads OS entropy — stays out of `domain`, consistent with the "no I/O in domain" rule; only *validation* of already-generated bytes is domain-owned (§7.1). | Yes |
| `IdGenerator` | Gateway — identity generation | `new_id() -> LicenseGrantId` / `new_id() -> EncryptionCredentialId` (or a shared generic identifier type). Keeps entity IDs out of domain-internal generation for the same reason as `RandomnessGateway`, and for deterministic test fakes. | Yes |
| `Clock` | Neither Repository nor Store nor Gateway by strict suffix, but follows the same "external world" pattern | `now() -> Timestamp`. Used to timestamp `LicenseGrant` activation and `EncryptionCredential` establishment deterministically in tests. | Yes |

All outbound ports are declared `async` and consumed as `Arc<dyn Trait + Send + Sync>`
per architecture.md, even though no concrete adapter exists yet — this avoids a
Gate 1 contract change when an `infra_*` crate later implements them.

### 9.3 Explicit port-level constraint

`LicenseGrantRecord` and `EncryptionCredentialRecord` (§11) must never carry
`SystemSecret`, `RecoveryCode`, or raw `DatabaseEncryptionKey` fields. This is the
port-level enforcement of PRD §9.6 bullet 4 (must not be recoverable by inspecting the
installed binary/app data alone) — it is stated here, not only in §7, because it binds
whichever adapter implements these ports later.

## 10. Expected Adapter and Infrastructure Changes

Out of scope for this document's normative content. No `infra_*` crate exists yet in
this repository (only `src-tauri`, which ADR-0001 explicitly does not wire to
`app_core` in this change). This section names the obligations a future adapter set
must satisfy, without designing them:

- An adapter for `MachineHardwareIdGateway` must read the real CPU ID and motherboard
  serial and combine them into the `MachineHardwareId` format `domain` expects.
- An adapter for `SystemSecretStore` should use OS-protected credential storage (the
  `keyring` crate against Windows Credential Manager, per Appendix B §3) — informative
  suggestion, not mandated; any equally OS-protected mechanism satisfies PRD §9.6.
- An adapter for `RandomnessGateway` must use a CSPRNG source (e.g., the OS entropy
  source via a standard Rust crate), not a non-cryptographic PRNG.
- Adapters for `LicenseGrantRepository` and `EncryptionCredentialRepository` must be
  the sole writers of whatever underlying store they use, per NFR-05, and must resolve
  the bootstrap-storage question raised in §11 and §15 before implementation.
- No adapter in this future set may perform network I/O for any operation defined in
  this document (NFR-04).
- Wiring any of the above into `src-tauri` command handlers, or into a new `infra_*`
  crate's composition root, is a separately scoped follow-up per ADR-0001 — not
  authorized by this document.

## 11. Data Model and Persistence Notes

Concrete SQLCipher schema, migration mechanics, and the physical storage location for
these records are deferred to a later phase/ADR that also decides `infra_*` crate
topology — stated explicitly per this document's scope, rather than silently omitted.
What Phase 1 fixes is the **port-owned record shape** consumed at the repository
boundary (HADR-0005: repositories exchange owned state, not private aggregates):

- **`LicenseGrantRecord`** — `id`, `machine_hardware_id` (string), `feature_flags`
  (list of strings), `signature_metadata` (opaque bytes: the Ed25519 signature plus
  any signed, non-secret metadata), `activated_at` (UTC timestamp). Maps directly to
  the PRD §11 `LicenseGrant` conceptual entity's fields.
- **`EncryptionCredentialRecord`** — `id`, `salt` (bytes), `established_at` (UTC
  timestamp). Deliberately excludes `system_secret` and any Recovery Code
  material — see §9.3.
- **Singleton semantics:** both records are single-row-per-installation ("current")
  rather than general collections. `save_current`/`load_current` (§9.2) reflect this;
  the usual HADR-0005 insert-vs-conditional-replace distinction collapses to one
  operation because there is only ever one current row, not because this document
  waives that principle for general collections.

## 12. Testing Strategy

Follows `backend_arch_docs/testing.md`'s Gate 1–3 evidence table. Gates 4–5 do not
apply (§3.2) — no external contract or reference/service composition exists yet.

| Gate | Evidence for this slice |
|---|---|
| Gate 1 | Domain invariant tests: value-object validation (`MachineHardwareId` non-empty, `LicenseSignature`/`LicensePublicKey` fixed lengths, `EncryptionSalt`/`SystemSecret` length, `RecoveryCode` format, `FeatureFlagSet` de-duplication); entity invariant tests (`LicenseGrant::activate` rejects malformed input, bad signature, and a machine-ID mismatch, and only ever produces a validated grant); domain-service tests (`LicenseSignatureVerification` true/negative cases with test key pairs; `DatabaseEncryptionKeyDerivation` determinism — same secret+salt → same key, different salt → different key; `RecoveryCodeEncoding` round-trip and corrupted-input rejection). Plus reviewed semantic port contracts for every port in §9 (signatures, errors, and applicable identity/ordering/atomicity semantics per HADR-0005). |
| Gate 2 | Compiling, intentionally failing use-case tests with test-local outbound fakes (e.g. `InMemoryLicenseGrantRepository`, `InMemoryEncryptionCredentialRepository`, `FixedMachineHardwareIdGateway`, `StaticSystemSecretStore`, `StaticClock`, `FixedRandomnessGateway`, `SequentialIdGenerator`) for all six use cases in §8 — including the negative cases (invalid signature, machine mismatch, already-established, missing secret, malformed/invalid recovery code). Each red test's failure reason is reviewed to confirm it is caused by absent use-case behavior, not fixture or compilation failure. The red PR is dependent evidence and is never merged alone. |
| Gate 3 | Minimal green implementation of the six use cases and `ports::assemble(...)` that makes the approved Gate 2 tests pass, plus the core-only Cargo gate: `cargo check -p app_core --all-targets --locked`, `cargo test -p app_core --locked`, `cargo test -p app_core --doc --locked`. No concrete adapter or external I/O is introduced at this gate. |

Domain and application tests perform no filesystem, network, database, or process I/O
(`testing.md` "Domain and Application" test layer) — every fake in Gate 2 is
in-memory/deterministic.

### 12.1 Spike evidence (not a Gate)

The §6 spike is a prerequisite, evaluated separately from Gates 1–3, using the S1–S3
pass criteria in §6.1 and recorded in [`spike-results.md`](spike-results.md).

## 13. Implementation Plan

1. **Run the technical feasibility spike (§6).** Blocking — do not start step 2 until
   S1–S3 pass and are recorded in `spike-results.md`. If any check fails, stop and
   return to stack re-scoping per §6.3.
2. **Gate 1:** implement `domain` value objects, `LicenseGrant` and
   `EncryptionCredential` entities, the three domain services, domain events, and
   domain errors (§7); define all inbound and outbound port traits, commands,
   results, and errors (§9). Open for review as domain+ports only — no use-case
   implementation.
3. **Gate 2:** add the six use-case inbound port implementations as failing tests
   against local fakes (§12), stacked/dependent on the approved Gate 1 state. Never
   merge this alone.
4. **Gate 3:** implement the minimal `application` behavior (§8) to turn the Gate 2
   tests green, implement `ports::assemble(...)` (§8.6), and pass the core-only Cargo
   gate (§12).
5. **Explicitly deferred beyond this plan** (no step number — not part of this
   document's delivery): `infra_*` crate scaffolding and adapter implementations for
   every port in §9.2, `src-tauri`/Tauri command wiring, SQLCipher schema/migration
   design, and Gates 4–5. These require their own ADR/TDD per ADR-0001 and
   `backend_arch_docs/architecture.md`.

## 14. Acceptance Criteria / Completion Checklist

**Spike:**

- [ ] S1, S2, S3 (§6.1) each have recorded pass evidence in `spike-results.md`, dated
      and attributable.
- [ ] If any check failed, the failure and the resulting stack re-scoping decision are
      recorded before any Gate 1 work merges.

**Domain and ports (Gate 1):**

- [ ] Every value object in §7.1 validates on construction and rejects the stated
      invalid inputs in a unit test.
- [ ] `LicenseGrant::activate` has no code path that returns a `LicenseGrant` without
      both a valid signature and a matching Machine Hardware ID.
- [ ] `DatabaseEncryptionKeyDerivation` never takes `MachineHardwareId` as an input
      (reviewable by trait/function signature inspection).
- [ ] No domain event in §7.4 carries `SystemSecret`, `RecoveryCode`, or
      `DatabaseEncryptionKey` payloads.
- [ ] Every port in §9 is reviewed for signature, typed errors, and applicable
      identity/ordering/atomicity semantics.

**Application (Gates 2–3):**

- [ ] All six use cases in §8 have Gate 2 failing tests reviewed for correct failure
      reason, then Gate 3 green implementations.
- [ ] `cargo check -p app_core --all-targets --locked`, `cargo test -p app_core
      --locked`, and `cargo test -p app_core --doc --locked` all pass.
- [ ] `ActivateLicenseUseCase` rejects a syntactically valid but wrongly-signed key
      with `InvalidSignature`, and a validly-signed key bound to a different machine
      with `MachineHardwareMismatch { current_machine_hardware_id }`.
- [ ] `RecoverDatabaseEncryptionKeyUseCase`, given the same underlying secret,
      produces the same `DatabaseEncryptionKey` as `DeriveDatabaseEncryptionKeyUseCase`
      would from `SystemSecretStore` — proving the two paths are independent but
      consistent.
- [ ] `EstablishEncryptionCredentialUseCase` refuses to run a second time
      (`AlreadyEstablished`) against a fake repository that already has a current
      record.

**Layer boundaries:**

- [ ] `app_core::domain` and `app_core::application` remain unreachable from outside
      the crate (the existing `compile_fail` doctest in `crates/app_core/src/lib.rs`
      continues to pass for `domain`; an equivalent check exists or is added for
      `application`).
- [ ] `ports` is the only `pub mod` in `crates/app_core/src/lib.rs`.
- [ ] No `infra_*`, `tauri`, `rusqlite`, or `keyring` import appears anywhere in
      `domain` or `application`.
- [ ] No PRD-out-of-scope concept (Resident, Household, Certificate, Staff Account,
      Role, Audit Log Entry, Backup Snapshot, Barangay Identity/Official, Purok,
      Theme, Dashboard Widget) is introduced anywhere in this slice.

## 15. Open Questions

- **Domain ownership of pure cryptographic computation.** §4 documents this TDD's
  working interpretation: Ed25519 verification and PBKDF2-HMAC-SHA256 derivation are
  domain-owned pure computation, while all randomness sourcing and external reads stay
  behind outbound ports. The alternative is to push signature verification and key
  derivation behind a dedicated outbound `CryptoGateway`-style port instead, with
  `domain` only consuming its result. This changes which layer owns two of §7.3's
  three domain services — a named boundary decision — so it needs explicit reviewer
  sign-off at Gate 1 review rather than being silently assumed. Moving it later would
  reopen Gate 1.
- **Bootstrap storage location for `LicenseGrantRecord`/`EncryptionCredentialRecord`.**
  §11 fixes the port-owned record shape but not where these records physically live.
  If they were stored inside the same SQLCipher-encrypted database that
  `DatabaseEncryptionKey` is used to open, deriving the key would require reading a
  salt that is itself locked behind the key it helps derive — a bootstrap
  chicken-and-egg problem Appendix B does not address. Candidate resolutions (an
  unencrypted local bootstrap file/config, or extending `SystemSecretStore`'s
  OS-protected storage to also hold the salt) are not decided here because deciding
  would fix a concrete `infra_*` adapter target, which is out of this document's
  scope — but whoever designs that adapter must resolve it before Gate 4/5 work
  begins, since it may also affect NFR-05 (single writer) enforcement.
- **Exact PBKDF2 iteration count and salt/secret byte lengths, against NFR-01.**
  Appendix B §5 explicitly leaves this to engineering judgment. It is noted as an
  open item because an under-tuned iteration count could threaten the 3-second
  cold-start budget (NFR-01) once a real adapter exists, but it does not change any
  contract in this document and can be tuned without a Gate 1 revisit — included here
  for visibility, not as a blocking decision.
