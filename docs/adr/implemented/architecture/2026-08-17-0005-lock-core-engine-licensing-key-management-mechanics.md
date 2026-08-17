# ADR-0005: Lock the Core Engine licensing & key-management mechanics (Hardware ID, bootstrap storage, Recovery Code)

<!-- Location: docs/adr/implemented/architecture/2026-08-17-0005-lock-core-engine-licensing-key-management-mechanics.md.
     The inline Status below must agree with {lifecycle}. -->

- **Date**: 2026-08-17
- **Status**: Implemented
- **Deciders**: Product and Engineering (solo)

## Context and Problem Statement

The [Core Engine PRD](../../../specs/core-engine/PRD.core-engine.md) §9.5–9.6 fixes the binding product requirements for offline licensing and data-protection key management, and [Appendix B](../../../specs/core-engine/appendix-b-key-derivation-and-recovery.md) / [Appendix C](../../../specs/core-engine/appendix-c-license-reissuance-sop.md) record *informative* supporting mechanics that [ADR-0001](2026-08-17-0001-lock-core-engine-application-stack.md) promoted to normative direction (Ed25519 license validation, Argon2id password hashing, PBKDF2-HMAC-SHA256 database-key derivation, `keyring`-backed credential storage) while explicitly deferring three concrete mechanics to a later ticket:

- **Machine Hardware ID derivation**: the PRD names CPU ID + motherboard serial as inputs (§9.5) but not how they are read, combined, or displayed.
- **Encryption-credential bootstrap storage**: Appendix B establishes that `system_secret` lives in the OS-protected credential store and the SQLCipher key is `PBKDF2-HMAC-SHA256(system_secret, salt)`, but leaves open where the `salt` itself lives before that derivation can run — the chicken-and-egg the salt is needed to derive the key that opens the database that would otherwise hold the salt.
- **Recovery Code format**: Appendix B §4 requires a "human-transcribable Recovery Code" as an independent unlock path but leaves its character set, grouping, and error-detection format open, and explicitly leaves open whether it represents `system_secret` directly or a wrapping key for it.

[ADR-0002](2026-08-17-0002-fix-core-engine-workspace-crate-topology.md) already carves out `infra_hardware_id`, `infra_licensing`, and `infra_credentials` as separate crates along exactly this PRD-drawn boundary, but those crates have no concrete contract to build against until these three mechanics are fixed.

## Decision Drivers

- **The DB key must never depend on Machine Hardware ID** (Appendix B's core invariant) — a hardware repair/replacement or a disaster restore onto different hardware must never render existing data or backups permanently unreadable.
- **Recovery must work "independent of Credential Manager"** (Appendix B §4), including a full restore onto new hardware where Windows Credential Manager — being machine-local — does not travel with the restore, while a Backup Snapshot (PRD §9.8) does.
- **Anti-piracy binding intent of PRD §9.5**: the Machine Hardware ID should discriminate between distinct installations to the extent feasible without paid hardware-fingerprinting SDKs, which would be disproportionate for this product.
- **Human-transcribable, typo-resistant Recovery Code** (Appendix B §4) — it is copied once onto paper at first run and re-entered by hand during a real recovery event, so transcription-error detection matters.
- **Stay within the already-locked crypto profile** (ADR-0001): Ed25519, Argon2id, PBKDF2-HMAC-SHA256, `keyring` — introduce new primitives only where the existing profile doesn't cover a need (the Recovery Code's wrap cipher).
- **No new runtime network dependency or installer-size regression** (NFR-03, NFR-04) — every mechanic here is local-only.

## Decision

We will fix the three mechanics as follows.

### 1. Machine Hardware ID derivation

- `infra_hardware_id` reads three inputs via the Rust `wmi` crate (structured WMI access, no subprocess/`wmic` dependency): `Win32_Processor.ProcessorId`, `Win32_BaseBoard.SerialNumber`, and `Win32_ComputerSystemProduct.UUID` (SMBIOS UUID) — one more input than the PRD's literal two, added to reduce collision risk (see Alternatives).
- The three raw strings are concatenated with a fixed delimiter and hashed with SHA-256.
- The digest is Crockford Base32-encoded and displayed grouped into dashed 4-character blocks (52 data characters → 13 groups) in Settings and in the hardware-mismatch error state ([Appendix C](../../../specs/core-engine/appendix-c-license-reissuance-sop.md) Step 1), so the Secretary reads or copies one consistent code style across the product rather than a raw hex string.
- This Machine Hardware ID is the value bound into the signed License Key and compared during Ed25519 validation in `infra_licensing`; `infra_hardware_id` performs no licensing logic itself, per ADR-0002.

### 2. Encryption-credential bootstrap storage

- Split by secrecy, not convenience. The PBKDF2 `salt` is not secret — its role is uniqueness/anti-rainbow-table, not confidentiality — so it lives in a small **unencrypted bootstrap file** in the application data directory (e.g. `%APPDATA%\BarangayMS\bootstrap.json`, alongside the `%APPDATA%\BarangayMS\backups\` convention already fixed by PRD §9.8/ADR-0001). Living in the app data directory means it is automatically carried by every Backup Snapshot and every restore.
- `system_secret` remains exclusively in the OS-protected credential store (Windows Credential Manager via `keyring`, per ADR-0001) and is never written to disk in the clear, in `bootstrap.json` or anywhere else.
- The SQLCipher key remains `PBKDF2-HMAC-SHA256(system_secret, salt)` as ADR-0001/Appendix B already fixed, deliberately excluding the Machine Hardware ID; the Hardware ID stays confined to the Ed25519 license-validation path in `infra_licensing`.

This resolves the chicken-and-egg without inventing a new primitive: the salt was never the secret, so it never needed protected storage in the first place — only availability, which the bootstrap file's location (backed up with the data, unlike Credential Manager) satisfies.

### 3. Recovery Code

- The Recovery Code is a **wrapping key**, not a direct transcription of `system_secret`. `bootstrap.json` additionally stores `system_secret` AES-256-GCM-encrypted (nonce + ciphertext + tag), wrapped under a key derived from the Recovery Code. This decouples the code's transcribable length from `system_secret`'s full entropy and allows the code to be rotated later (re-wrap under a new code) without touching `system_secret` or re-encrypting the database. The exact KDF and parameters used to turn the entered code into the AES-256-GCM unwrap key are deferred to the Core Engine technical design, consistent with ADR-0001's precedent of deferring crate/parameter pinning.
- **Character set**: Crockford Base32 (`0123456789ABCDEFGHJKMNPQRSTVWXYZ`) — excludes `I`, `L`, `O`, `U` to eliminate visual confusion with `1`/`0` and avoid accidental profanity, and has a well-defined optional check-symbol extension used below.
- **Length and grouping**: 28 data characters (140 bits — a comfortable margin over a 128-bit wrapping-key floor), grouped in blocks of 4, plus one trailing Crockford check symbol as an 8th block-of-1: `XXXX-XXXX-XXXX-XXXX-XXXX-XXXX-XXXX-X`.
- **Checksum**: the trailing Crockford check symbol (mod-37 over the 28 data symbols) is validated at entry, before any AES-GCM unwrap attempt, so a mistyped code is rejected immediately and distinctly from a genuinely wrong code.

## Alternatives Considered

### Machine Hardware ID: PRD's literal two inputs only (CPU ID + motherboard serial)

- Benefits: matches PRD §9.5 text exactly; one fewer WMI query.
- Costs and risks: `Win32_Processor.ProcessorId` reflects only CPUID feature/family/model/stepping bits — identical across every CPU of the same model/stepping, not a per-unit serial (per-chip processor serial numbers died with early Pentium III). `Win32_BaseBoard.SerialNumber` commonly returns literal placeholder text (e.g. `"To be filled by O.E.M."`) on cheap/generic boards, exactly the price point a barangay PC is likely to be. Two barangays running the same off-the-shelf budget PC model could derive an identical Machine Hardware ID, letting one License Key silently validate on both installations — undermining PRD §9.5's anti-piracy intent. Rejected in favor of a third input (SMBIOS UUID) that reduces, without eliminating, this collision risk; the residual risk is accepted rather than solved further (see Consequences), since fully solving it needs paid hardware-fingerprinting SDKs disproportionate to this product.

### Machine Hardware ID: shell out to `wmic`

- Benefits: no new crate dependency.
- Costs and risks: `wmic` is deprecated and removed from newer Windows builds; a subprocess dependency is also a worse fit for the offline, self-contained posture than a structured COM/WMI crate call. Rejected in favor of the `wmi` crate.

### Bootstrap storage: OS-protected store also holds the salt (the ticket's named alternative)

- Benefits: one storage location instead of two; no plaintext file on disk at all.
- Costs and risks: Windows Credential Manager is machine-local and is exactly what Recovery must work *independent of* (Appendix B §4) — a full disaster restore onto new hardware, by definition, does not carry the old machine's Credential Manager contents. If the salt lived only there, a restore onto new hardware would have `system_secret` (recoverable via the Recovery Code) but no salt to combine it with, defeating the recovery guarantee. Rejected; only data-directory storage (which travels with the Backup Snapshot) satisfies the "independent of Credential Manager" requirement.

### Recovery Code: direct transcription of `system_secret`

- Benefits: one fewer moving part — no wrap/unwrap step, no AES-GCM blob to manage.
- Costs and risks: forces the Secretary to hand-transcribe the full 256-bit `system_secret` (a much longer code than a wrapping key needs), and ties the code's shape permanently to `system_secret`'s own size and value — a suspected-compromised code could only be "rotated" by also rotating `system_secret` itself, which would require re-deriving and re-applying the SQLCipher key. Rejected in favor of a wrapped design that keeps the code short and independently rotatable.

### Recovery Code: plain hex or unrestricted alphanumeric character set

- Benefits: simplest possible encoding; no custom alphabet.
- Costs and risks: no built-in transcription-error detection, and hex/mixed-case alphanumeric strings are more error-prone to hand-copy and read aloud over a support call than a purpose-built human-transcription alphabet. Rejected in favor of Crockford Base32, which excludes visually confusable characters and carries a standard checksum extension.

## Consequences

### Positive

- `infra_hardware_id`, `infra_licensing`, and `infra_credentials` (ADR-0002) now have a concrete contract each: fingerprint inputs and combination method, license-validation input shape, and credential-store usage, respectively.
- The recovery flow is testable end-to-end independent of both the original hardware and Credential Manager, matching Appendix B §4's requirement directly.
- The bootstrap file rides naturally inside the existing Backup Snapshot mechanism (PRD §9.8) with no new backup plumbing required.
- The Machine Hardware ID and Recovery Code share one visual grammar (Crockford Base32, dashed 4-character blocks), so the Secretary learns one code style for the whole product.

### Negative

- `bootstrap.json` is a new first-run artifact with its own loss mode, distinct from Credential Manager loss: if it is lost or corrupted before any Backup Snapshot exists (the gap between first run and the first automatic exit/daily snapshot), the salt is unrecoverable — the Recovery Code alone cannot help, since it only unwraps `system_secret`, not the salt. This is a genuine edge case in the earliest window of an installation's life, not fully closed by this ADR (see Confirmation and Neutral/Risks).
- Three WMI queries (`ProcessorId`, `BaseBoard.SerialNumber`, `ComputerSystemProduct.UUID`) add a small amount of startup-path surface area versus two, though WMI queries are local and fast enough not to threaten NFR-01's cold-start budget.

### Neutral / Risks

- The three-input Machine Hardware ID reduces but does not eliminate cross-installation collision risk on generic/clone hardware or in VM environments; this is accepted as a documented risk rather than solved with paid fingerprinting SDKs.
- The Recovery Code's wrap-key KDF (algorithm, iteration/parameter choice for turning the entered code into an AES-256-GCM key) is deferred to the Core Engine technical design, which must still land on concrete parameters before implementation.
- The `bootstrap.json`-before-first-backup gap noted above may warrant a first-run safeguard (e.g., forcing an immediate backup, or displaying the Recovery Code together with an explicit warning that recovery also depends on an existing Backup Snapshot) — left to the technical design to resolve, not re-litigated here.

## Confirmation

- `infra_hardware_id` combines exactly `Win32_Processor.ProcessorId` + `Win32_BaseBoard.SerialNumber` + `Win32_ComputerSystemProduct.UUID` via the `wmi` crate, SHA-256, and displays the result as Crockford Base32 in dashed 4-character groups.
- `bootstrap.json` contains the PBKDF2 salt and the AES-256-GCM-wrapped `system_secret` blob; no code path ever writes `system_secret` to disk unencrypted.
- The Backup Snapshot file set includes `bootstrap.json` alongside the encrypted database file (code/dependency review against PRD §9.8).
- An end-to-end recovery test (enter Recovery Code → checksum validates → AES-GCM unwrap succeeds → `system_secret` recovered → SQLCipher key re-derived) succeeds using only a Backup Snapshot and the transcribed Recovery Code, without any dependency on Credential Manager or the original machine's hardware.
- The database key derivation path (`app_core`/`infra_persistence`) never takes the Machine Hardware ID as an input; the Machine Hardware ID is read only by `infra_licensing`'s validation path.

## Relationships and References

- Refines [ADR-0002 (Core Engine workspace crate topology)](2026-08-17-0002-fix-core-engine-workspace-crate-topology.md) by fixing concrete contracts for `infra_hardware_id`, `infra_licensing`, and `infra_credentials`.
- Promotes [Core Engine Appendix B](../../../specs/core-engine/appendix-b-key-derivation-and-recovery.md) key-derivation/recovery guidance and [Appendix C](../../../specs/core-engine/appendix-c-license-reissuance-sop.md) reissuance SOP from informative to normative on the three points this ADR fixes; retains [ADR-0001 (application stack)](2026-08-17-0001-lock-core-engine-application-stack.md)'s crypto profile (Ed25519, Argon2id, PBKDF2-HMAC-SHA256, `keyring`) unchanged.
- Owning spec: [Core Engine PRD](../../../specs/core-engine/PRD.core-engine.md) §9.5 (Offline Licensing & Feature Gating), §9.6 (Data Protection & Key Management), §9.8 (Backup & Recovery).
- Glossary: [CONTEXT.md](../../../../CONTEXT.md) (Machine Hardware ID — definition updated to reflect the three-input derivation).
- Supporting issue: [Decide licensing & key-management mechanics](https://github.com/Xaidel/brs/issues/6).
