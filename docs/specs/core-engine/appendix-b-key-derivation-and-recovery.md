# Appendix B

## Key Derivation & Recovery Rationale

Back to the owning PRD: [Core Engine PRD](PRD.core-engine.md)

The owning PRD remains authoritative for product scope. This appendix clarifies supporting detail and must not broaden or contradict that scope. The binding requirements are [PRD §9.6](PRD.core-engine.md#96-data-protection--key-management); this appendix explains why they are shaped the way they are and suggests supporting mechanics for the downstream TDD.

---

## 1. Purpose

Document why the database encryption key must not depend on the Machine Hardware ID, and describe a recovery mechanism that keeps that guarantee intact even if the local credential store is unavailable.

## 2. The Problem This Requirement Prevents

An earlier draft of this design derived the SQLCipher encryption key directly from the Machine Hardware ID (CPU ID + motherboard serial), combined with an installation secret. That is a natural-seeming choice, since the same Hardware ID already gates license validation ([PRD §9.5](PRD.core-engine.md#95-offline-licensing--feature-gating)) — but it silently breaks two other requirements in the same release:

- **License reissuance ([PRD §9.5](PRD.core-engine.md#95-offline-licensing--feature-gating), [Appendix C](appendix-c-license-reissuance-sop.md))** exists specifically for the case where a barangay's hardware changes (repair or replacement) and existing records must remain intact. If the Hardware ID feeds the encryption key, that same hardware change makes the *existing database file* undecryptable the moment it changes — the reissuance SOP would restore the *license*, but the *data* would already be lost.
- **Backup & Recovery ([PRD §9.8](PRD.core-engine.md#98-backup--recovery))** exists specifically for disaster scenarios — a destroyed, stolen, or unrecoverable PC — where the barangay restores onto **different hardware** by definition. A backup encrypted under a key tied to the dead machine's Hardware ID could never be opened anywhere else, defeating the point of having a backup at all.

The root cause is conflating two distinct concerns that happen to both involve "the machine": **licensing** (which should be hardware-bound — that is the anti-piracy mechanism) and **data-at-rest encryption** (which must survive the machine changing, since protecting the data is the actual goal).

## 3. Suggested Derivation Approach (Informative)

- At first run, generate a cryptographically strong random salt and an installation secret (`system_secret`).
- Store `system_secret` in OS-level protected storage (e.g., Windows Credential Manager via the Rust `keyring` crate) for day-to-day frictionless unlock.
- Derive the SQLCipher key as `PBKDF2-HMAC-SHA256(system_secret, salt)` — deliberately excluding the Machine Hardware ID.
- Keep the Machine Hardware ID entirely within the Ed25519 license-validation path ([PRD §9.5](PRD.core-engine.md#95-offline-licensing--feature-gating)), where hardware-binding is the intended behavior.

This keeps the original security property intact: a raw database file extracted from a stolen hard drive still cannot be decrypted by reverse-engineering the app binary, since the key is never a static value embedded in the binary itself.

## 4. The Recovery Code (Single Point of Failure Mitigation)

Storing `system_secret` only in Windows Credential Manager creates a new single point of failure: if that credential store is lost (OS reinstall, disk swap, full machine replacement), the barangay could be locked out of its own decryptable-in-principle data. To prevent this:

- At first-run setup, display a one-time, human-transcribable Recovery Code (the `system_secret`, or a wrapping key for it, formatted for manual transcription — e.g., `XXXX-XXXX-XXXX-XXXX`).
- Instruct the Secretary to print or write it down and store it separately from the PC — the same pattern used by BitLocker or password-manager recovery codes.
- Use this Recovery Code as the fallback unlock path for both the live database and any Backup Snapshot, independent of Credential Manager and independent of any specific machine's hardware.

## 5. Usage Guidance

The downstream Phase 1 TDD should treat the derivation approach above as its starting point, subject to normal engineering judgment (e.g., PBKDF2 iteration count, salt length), and must preserve the two binding properties from [PRD §9.6](PRD.core-engine.md#96-data-protection--key-management): the key must not depend on Machine Hardware ID, and a Recovery Code must exist as an independent unlock path.
