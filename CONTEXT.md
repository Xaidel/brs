# BRS (Barangay Management System)

Offline-first desktop product for Philippine barangays: a foundational Core Engine plus separately-licensed add-on modules.

## Language

**Module**:
A separately licensed unit of product capability, built on top of Core Engine (KP Blotter, Treasury, Business Permits). Core Engine itself is not a module — it is the foundational release every module depends on.
_Avoid_: Package, plugin, add-on (use "add-on module" only when contrasting with Core Engine)

**Feature Flag**:
A runtime-toggleable gate, one per add-on module, unlocked by a valid License Key. Core Engine has no Feature Flag — it is unconditionally unlocked once installed. Flags map 1:1 to modules; no module currently has independently-unlockable sub-features.
_Avoid_: Permission, capability toggle, entitlement

**License Key**:
A signed, offline credential bound to one installation's Machine Hardware ID that grants a set of Feature Flags. Validated entirely offline via Ed25519 signature against a public key embedded in the application binary.
_Avoid_: License code, activation key

**LicenseGrant**:
The persisted record of a validated License Key's effect on an installation: which Feature Flags it unlocked, for which Machine Hardware ID, with what signature metadata.
_Avoid_: License record, activation record

**Machine Hardware ID**:
A fingerprint derived from CPU ID, motherboard serial number, and SMBIOS UUID (SHA-256 of the three concatenated, Crockford Base32-displayed), used exclusively to bind a License Key to one specific installation. Never an input to database-key derivation — see `system_secret`.
_Avoid_: Hardware fingerprint, device ID

**system_secret**:
A cryptographically random, per-installation value stored exclusively in the OS-protected credential store (Windows Credential Manager), never written to disk. Combined with the salt in `bootstrap.json` via PBKDF2-HMAC-SHA256 to derive the SQLCipher database key. Deliberately independent of Machine Hardware ID, so a hardware change or disaster restore never makes existing data unreadable.
_Avoid_: Encryption key, master password, DB secret

**bootstrap.json**:
The unencrypted first-run file (`%APPDATA%\BarangayMS\bootstrap.json`) holding the non-secret PBKDF2 salt and the AES-256-GCM-wrapped copy of `system_secret`. Lives in the application data directory so it travels with every Backup Snapshot, resolving the chicken-and-egg where the salt is needed before the database it might otherwise live in can be opened.
_Avoid_: Bootstrap file (informal use is fine in prose; the glossary term is the literal filename), credential store (that term is reserved for the OS-protected store holding `system_secret`)

**Recovery Code**:
A one-time, human-transcribable code (Crockford Base32, 28 data characters + 1 checksum character, grouped `XXXX-XXXX-XXXX-XXXX-XXXX-XXXX-XXXX-X`) generated at first run. Wraps `system_secret` (AES-256-GCM) rather than transcribing it directly, so the code stays short and independently rotatable. The fallback unlock path for the database or a Backup Snapshot when the OS-protected credential store is unavailable — independent of both Credential Manager and the original machine's hardware.
_Avoid_: Recovery key, backup code, unlock code

**Permission**:
An individually toggleable capability key from a fixed, developer-seeded catalog, named `<resource>.<action>` (e.g. `resident.create`, `certificate.void`, `staff.manage`). Roles — seeded or Custom — only ever select from existing Permission rows; no Role invents a new Permission key.
_Avoid_: Feature Flag, capability, scope, entitlement

**Role**:
A named, barangay-configurable collection of Permissions, assigned to exactly one Staff Account. Four seeded defaults ship editable, not specially privileged (Admin/Secretary, Encoder, Treasurer, Read-Only/Captain); an Admin/Secretary may also create unlimited Custom Roles with their own Permission selection.
_Avoid_: Group, permission set

**Shared Schema Columns**:
The four-column convention — `id` (UUIDv7), `barangay_code`, `created_at`, `updated_at`, `sync_status` — carried by every table holding a barangay-owned business record that is a candidate for a future resident-linked `.bmssync` export (Resident, Household, Certificate/Document, and future module business objects like KP Blotter Case, Treasury Transaction, Business Permit). Deliberately *not* carried by administrative/config tables local to one installation (Staff Account, Role, Permission, Feature Flags, License state, Purok/Sitio/Zone, Document Type templates, Audit Trail) — those use whatever columns their own domain needs. Master PRD Appendix A names these columns informatively; ADR-0006 fixes them as binding, superseding Appendix A's literal "UUIDv4 (or ULID)" text for `id` with the already-decided UUIDv7.
_Avoid_: Sync columns (informal use is fine in prose; the glossary term is the full name), audit columns (that term is reserved for the Audit Trail's own timestamping, which is separate)

**sync_status**:
A per-record enum on every table carrying the Shared Schema Columns, exactly `PENDING` or `SYNCED` — no third state. Tracks whether a record has been included in a future `.bmssync` export package, not a record's own business lifecycle (archived/voided/etc., which lives in that table's own columns).
_Avoid_: Status, state (too generic — always qualify as `sync_status` when referring to this column)

**Clock**:
An `app_core` port trait, injected into use cases, that supplies the current UTC time. Stamps `created_at`/`updated_at` on Shared-Schema-Columns tables at the domain layer, rather than via SQLite `DEFAULT`/triggers in `infra_persistence` — keeping timestamping testable (fake-able in unit tests) and inside the hexagonal boundary ADR-0002 already drew.
_Avoid_: Clock service, time provider

**Control Number**:
The audit-grade, gapless identifier assigned to a Certificate at the moment of issuance. Unique within its Document Type and Philippine Standard Time (UTC+8) calendar year; the raw allocation counter is never reused, skipped, or renumbered, and a voided Certificate retains its number (marked `VOIDED`). Rendered from the Document Type's `Control Number Format`.
_Avoid_: Serial number, reference number, sequence ID

**Control Number Format**:
A per-Document-Type configurable pattern that renders a Control Number: literal text plus exactly two brace tokens — `{YYYY}` (the PHT year of issuance) and `{N…}` (a zero-padded decimal sequence; the number of `N`s sets the *minimum* width, never a ceiling). Each format must contain exactly one of each token. The underlying sequence resets each PHT calendar year, so gaplessness holds within (Document Type × year).
_Avoid_: Numbering scheme, serial format, sequence pattern

**Template Variable**:
A named placeholder in a Certificate Template, written `{{source.field}}` (e.g. `{{resident.name}}`, `{{certificate.control_number}}`), that auto-populates from Resident, Household, Certificate, Barangay Official, or Barangay Identity data at issuance. The set is a closed, developer-seeded catalog (not barangay-defined); a missing value renders empty, and computed values (age, assembled names) are frozen into the rendered certificate at issuance. Barangay Officials are referenced through the standard `captain`/`secretary` position keys.
_Avoid_: Placeholder, merge field, template token
