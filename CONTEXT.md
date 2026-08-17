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
A fingerprint derived from CPU ID and motherboard serial number, used exclusively to bind a License Key to one specific installation.
_Avoid_: Hardware fingerprint, device ID

**Permission**:
An individually toggleable capability key from a fixed, developer-seeded catalog, named `<resource>.<action>` (e.g. `resident.create`, `certificate.void`, `staff.manage`). Roles — seeded or Custom — only ever select from existing Permission rows; no Role invents a new Permission key.
_Avoid_: Feature Flag, capability, scope, entitlement

**Role**:
A named, barangay-configurable collection of Permissions, assigned to exactly one Staff Account. Four seeded defaults ship editable, not specially privileged (Admin/Secretary, Encoder, Treasurer, Read-Only/Captain); an Admin/Secretary may also create unlimited Custom Roles with their own Permission selection.
_Avoid_: Group, permission set
