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
