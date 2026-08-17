# Core Engine PRD

## Barangay Management System — Core Engine (Foundational Release)

Status: Proposed

Owner: Product and Engineering (solo)

Version: 2.4 (2026-08-05)
Release tag: v1 / Foundational MVP

Related documents:

- [Master PRD](../master/PRD.md)
- [Package README](README.md)
- [Appendix A: Technical Architecture Direction](appendix-a-technical-architecture-direction.md)
- [Appendix B: Key Derivation & Recovery Rationale](appendix-b-key-derivation-and-recovery.md)
- [Appendix C: License Reissuance SOP](appendix-c-license-reissuance-sop.md)

---

## 1. Summary

Core Engine is the first shippable release of the Barangay Management System: a per-person staff login and role model, an offline resident and household registry with full-text search, an audit-grade certificate generator, an offline licensing engine that will gate all future add-on modules, an immutable audit trail, and an automated encrypted backup system. It is a complete, standalone product a barangay can purchase and use with zero add-on modules — every add-on module described in the [Master PRD](../master/PRD.md) depends on this release's licensing engine to unlock.

## 2. Product Framing

### 2.1 Canonical Terms

- **Resident** — a demographic profile for one person served by the barangay.
- **Household** — a group of Residents mapped to a Purok/Zone, with one designated Head of Household and relationship links between members.
- **Sector Tag** — a demographic/eligibility classification on a Resident (Senior Citizen, PWD, Solo Parent, 4Ps Beneficiary, Registered Voter, OFW, Indigenous People).
- **Certificate** — an issued official document generated from Resident data under a Document Type.
- **Control Number** — the audit-grade, gapless, monotonic identifier assigned to a Certificate at the moment of issuance, unique and unambiguous within its Document Type.
- **Document Type** — a barangay-configurable official certificate or credential type (e.g., Barangay Clearance, Certificate of Indigency) with its own Certificate Template and Control Number Format.
- **Certificate Template** — the editable text, layout, header/footer, seal, and variable placeholders that define a Document Type's appearance and content.
- **Template Variable** — a named placeholder in a Certificate Template (e.g., `{{resident_name}}`, `{{date}}`, `{{captain_name}}`) that auto-populates from Resident, Household, Certificate, Barangay Official, or Barangay Identity data.
- **Control Number Format** — a per-Document-Type configurable pattern (e.g., year + zero-padded sequence + type code) that defines how Control Numbers are rendered while preserving atomic allocation and gaplessness.
- **Staff Account** — an individual login belonging to one named staff member; never shared between people.
- **Role** — a named, editable collection of Permissions that define what a Staff Account can do in the system. Roles are barangay-configurable; the system ships four seeded default Roles (Admin/Secretary, Encoder, Treasurer, Read-Only/Captain) that can be edited or supplemented with Custom Roles.
- **Permission** — an individual capability or action category (e.g., `resident.create`, `certificate.issue`, `staff.manage`) that can be individually toggled on or off within a Role.
- **Custom Role** — a barangay-defined Role beyond the four seeded defaults, with an Admin-chosen name and custom Permission set.
- **License Key** — a signed, offline credential that unlocks Feature Flags for a specific Machine Hardware ID.
- **Machine Hardware ID** — a fingerprint derived from CPU ID and motherboard serial number, used exclusively to bind License Keys to a specific installation.
- **Feature Flag** — a runtime-toggleable capability unlocked by a valid License Key (used by add-on modules; Core Engine itself is always unlocked once installed).
- **Audit Log Entry** — an immutable record of an action taken against sensitive Resident data or a Certificate.
- **Recovery Code** — a human-transcribable secret generated at first run that can independently unlock the encrypted database if the installation's protected credential store is lost.
- **Backup Snapshot** — an encrypted, point-in-time export of the local database.
- **Barangay Identity** — a single, configurable profile record per installation: barangay name, logo, municipality/city, province, address, contact information, and official web presence, rendered on certificates and in-app UI chrome.
- **Barangay Official** — a configurable record for one current or historical staff member or position-holder: full name, position, photo, signature image, term dates, and status (Active/Inactive), auto-populated into certificates when the template requires a signatory.
- **Purok/Sitio/Zone** — a barangay-configurable administrative subdivision used to group Residents and Households; replaces hardcoded defaults.

### 2.2 Product Definition

An installation of Core Engine belongs to exactly one barangay. It holds that barangay's Residents, grouped into Households, and issues Certificates against them. Every write to a Resident, Household, or Certificate is performed by an authenticated Staff Account and recorded as an Audit Log Entry. The installation's data is encrypted at rest and independently recoverable via Backup Snapshots, regardless of the specific PC hardware it currently runs on.

## 3. Problem Statement

Barangay secretaries currently track residents and issue certificates from paper logs or unstructured spreadsheets, with no reliable search, no gapless control-number discipline for auditors, no per-staff accountability, and no backup discipline beyond whatever a given employee happens to do. Any digital alternative must work fully offline on the hardware barangays already own, must not lose data when that hardware eventually fails or is replaced, and must produce records that would satisfy a DPA 2012 or COA-style audit from day one — not as a later add-on.

## 4. Product Vision

Core Engine should function as the barangay's single, always-available source of truth for who its residents are, which household they belong to, and what has been officially issued to them — usable on the same low-spec PC for years, across hardware replacements, without losing a single record. See the [Master PRD](../master/PRD.md) for how this release fits into the broader modular product direction.

## 5. Goals

### 5.1 Primary Goals

- Every action affecting Resident or Certificate data must be attributable to one authenticated Staff Account.
- Staff must be able to create, edit, archive, and search Resident and Household records, with search returning in under 200ms at 50,000 records.
- Staff must be able to generate the four core Certificates with a zero-gap, audit-grade Control Number per document type.
- The local database must be encrypted at rest and remain decryptable independent of the specific Machine Hardware ID it was created on.
- A barangay must be able to unlock any future add-on module entirely offline via a signed License Key.
- Data must survive a hardware failure via automated and manual encrypted backups.

### 5.2 Secondary Goals

- The schema should follow the sync-ready conventions in [Master Appendix A](../master/appendix-a-sync-ready-schema-and-municipal-integration.md), even though sync execution is out of scope for this release.
- The UI should be usable by low-to-moderate tech-literacy staff without requiring outside technical support for routine operation.

## 6. Non-Goals

- Katarungang Pambarangay (KP Blotter) case management — see the separate [KP Blotter package](../kp-blotter/README.md).
- Barangay Treasury fee ledger and official receipts — see the separate [Treasury package](../treasury/README.md).
- Business Clearances & Permits — see the separate [Business Permits package](../business-permits/README.md).
- Municipal data export/sync execution — Exploratory at the master level; not committed for any release yet.
- Concurrent multi-PC/LAN access to a single database — this release must not prevent it architecturally (see NFR-05), but must not implement it.
- Any cloud-hosted account, cloud backup, or internet-dependent operation.
- Print-preview or per-job margin calibration UI for the printer output — fixed standard A4/Letter margins only. A fast-follow UX pass to address real-world printer margin variance is planned immediately after this release (tracked outside this PRD's scope, not a Core Engine v1 requirement).
- General ledger or accounting functionality beyond what the Treasury module defines separately.
- WYSIWYG drag-and-drop certificate template designer — certificate customization uses structured field editing and named-variable substitution only, not freeform layout design.
- Template localization or multi-language support — all barangay-configurable text (templates, official titles, Purok names) is single-language at barangay discretion.
- Legally-certified PKI/E-Signature support — signature images are stored photos or placeholders for wet signature only; no legally-binding digital certification.
- Per-user theming or appearance preferences — theme settings (colors, dark/light mode) are barangay-wide, not per-staff-account.

## 7. Target Users

| Role | Permission family | Primary reason they use Core Engine |
|---|---|---|
| Admin / Secretary | Full administrative access | Manage residents, households, staff accounts, licensing, backups, and audit logs. |
| Encoder | Data entry | Add/edit resident records and draft certificates. |
| Treasurer | Financial operations (Core Engine scope: view-only on the minimal certificate/payment-matching fields, not full Resident profiles) | View the fields needed to match a fee payment to a certificate (e.g. resident name, certificate type/control number, issue date) once Treasury is licensed; full ledger operations live in the Treasury package. |
| Read-Only / Captain | View-only | View resident/household records and dashboard reports; approve/sign issued documents. |

## 8. Core User Stories

**Authentication & Session**
- As any staff member, I want to log in with my own username and PIN so my actions are attributable to me, not a shared account.
- As any staff member, I want the app to auto-lock after 15 minutes idle so I don't leave resident data exposed if I step away.

**Resident & Household Management**
- As a Secretary or Encoder, I want to add, edit, and archive resident profiles so the registry stays current.
- As a Secretary or Encoder, I want to search residents by name in under a second so I can serve a walk-in quickly.
- As a Secretary, I want to group residents into households so I can track family units and designate a head of household.

**Certificate Issuance**
- As a Secretary or Encoder, I want to generate a certificate auto-filled from a resident's profile so I don't retype data.
- As an Admin/Secretary, I want every issued certificate's control number to be gapless and auditable so the barangay passes a COA-style audit.
- As an Admin/Secretary, I want to void a certificate with a reason note, without losing or reusing its control number.

**Licensing & Module Unlock**
- As an Admin/Secretary, I want to view and copy my installation's Machine Hardware ID so I can request a license key from vendor support.
- As an Admin/Secretary, I want to paste a signed license key and have new modules unlock instantly without restarting the app.
- As an Admin/Secretary whose PC was replaced or repaired, I want a documented path to re-license the new hardware without losing existing data.

**Backup & Recovery**
- As an Admin/Secretary, I want the app to automatically back up on exit and on a daily schedule so I don't have to remember to do it manually.
- As an Admin/Secretary, I want to export an encrypted backup to a USB drive on demand.
- As an Admin/Secretary recovering from a destroyed or replaced PC, I want to restore from a backup using a recovery code, without needing the original hardware.

**Audit**
- As an Admin/Secretary, I want to view an immutable log of who did what to resident data and when, so I can answer a DPA 2012 or COA inquiry.

**Barangay Configuration**
- As an Admin/Secretary, I want to configure my barangay's name, logo, contact information, and officials' details once so they automatically appear on all issued certificates without re-entry.
- As an Admin/Secretary, I want to define or customize Certificate Templates and Control Number Formats for my barangay's specific document types so every barangay's branding and numbering needs are met without code changes.
- As an Admin/Secretary, I want to add my barangay officials (Barangay Captain, Secretary, Kagawads, Health Workers, Tanods) with their photos, signatures, and term dates so certificates auto-select the current active signatory.
- As an Admin/Secretary, I want to create custom staff Roles with individually toggled permissions tailored to my barangay's workflow, not restricted to the four default Roles.
- As an Admin/Secretary, I want to manage my barangay's Puroks/Sitios/Zones within the system so staff do not need to free-type addresses and data stays consistent.
- As an Admin/Secretary, I want to set my barangay's preferred colors and dark/light appearance so the interface reflects my barangay's brand.

## 9. Product Requirements

### 9.1 Authentication & Role-Based Access Control

- The system must require every staff member to authenticate with an individual Staff Account before performing any action on Resident, Household, or Certificate data.
- The system must not support a Staff Account shared by more than one person.
- The system must store Staff Account credentials as a salted, memory-hard password hash (Argon2id or bcrypt); it must not store credentials in plaintext or reversible form.
- The system must implement a configurable role-based permission model where a Role is a named collection of individually toggleable Permissions (e.g., `resident.create`, `resident.view`, `resident.edit`, `resident.archive`, `household.create`, `household.edit`, `certificate.draft`, `certificate.issue`, `certificate.void`, `staff.manage`, `settings.manage`, `license.manage`, `backup.manage`, `audit.view`; exact Permission taxonomy is TDD-level detail). Each Staff Account is assigned exactly one active Role.
- The system must ship four pre-seeded default Roles (Admin/Secretary, Encoder, Treasurer, Read-Only/Captain) with Permission sets matching the boundaries historically implied in prior releases; these defaults are fully editable and not specially privileged.
- An Admin/Secretary must be able to create, edit, and deactivate Custom Roles; deactivating a Role must not delete existing Staff Accounts assigned to it, only prevent new assignment.
- The system must enforce a safety constraint: at least one active Role must always retain the `staff.manage` Permission, and the system must refuse any edit that would leave zero active Staff Accounts capable of managing Roles and Staff (to prevent accidental lockout).
- The system must provide a one-action "lock" control that returns to a per-person unlock screen without ending the underlying session.
- The system must automatically lock the session after 15 minutes of user inactivity.

### 9.2 Resident Registry

- The system must support create, edit, archive (soft-delete), and view operations on Resident profiles.
- The system must persist, at minimum: full name (first/middle/last/suffix), sex, birthdate, civil status, occupation, contact number, and address/Purok/Zone.
- Every Resident record must use a UUIDv4 (or ULID) string identifier as its primary key, per [Master Appendix A](../master/appendix-a-sync-ready-schema-and-municipal-integration.md).
- The system must support tagging a Resident with one or more Sector Tags: Senior Citizen, PWD, Solo Parent, 4Ps Beneficiary, Registered Voter, OFW, Indigenous People.
- The system must support attaching a photo to a Resident profile, captured via a connected webcam or uploaded as JPEG/PNG, stored in the application's local data directory.
- Authorized users must be able to search Residents by name and return matching results in under 200ms at a database size of up to 50,000 Resident records (see NFR-02).

### 9.3 Household Registry

- The system must support creating a Household mapped to a specific Purok/Zone and address.
- The system must require every Household to have exactly one designated Head of Household, selected from existing Residents.
- The system must support linking additional Residents to a Household with an explicit relationship (spouse, child, parent, sibling, relative, tenant).

### 9.4 Certificate Generation & Control Number Sequencing

- The system must auto-populate Certificate Templates from Resident, Household, Certificate, Barangay Official, and Barangay Identity data using named Template Variables (e.g., `{{resident_name}}`, `{{age}}`, `{{address}}`, `{{purpose}}`, `{{date}}`, `{{captain_name}}`; exact available variables are TDD-level detail).
- The system must support, at minimum, four seeded Document Types with pre-configured Templates and Control Number Formats: Barangay Clearance, Certificate of Indigency, Certificate of Residency, and Certificate of Good Moral Character. These four types cannot be permanently deleted but may be disabled; their historical Control Number sequences and issued Certificates must remain intact and reportable even when disabled.
- Barangays may add Custom Document Types beyond the seeded four, each with its own editable Certificate Template and Control Number Format; each Custom Document Type gets its own independent, gapless Control Number sequence from the moment of creation.
- Each Certificate Template must include editable text body, header image (optional), watermark (optional), footer text (optional), seal placement (optional), QR code toggle (optional), and document border (optional).
- The system must audit-lock each Certificate's rendered content at the moment of issuance: a Certificate's template and variables must be captured such that editing a Document Type's Template after issuance does not retroactively alter any previously printed or issued Certificate; template edits apply only to new issuances.
- The system must render certificate output for standard A4 and Letter paper sizes with fixed, pre-configured margins suitable for USB-connected Windows printers.
- The system must assign each issued Certificate a Control Number that is unique, monotonically increasing, and free of gaps within its Document Type. Control Numbers are formatted according to the Document Type's configurable Control Number Format pattern (e.g., `BC-2026-00001` for Barangay Clearance, `2026-0001` for a custom type).
- The system must allocate a Control Number inside a single atomic database transaction at the exact moment of issuance, such that no two certificates of the same Document Type can ever receive the same number and no number is skipped under concurrent or interrupted operation.
- Changing a Document Type's Control Number Format must apply only to newly allocated Control Numbers; previously allocated Control Numbers must never be altered, renumbered, or reformatted.
- The system must not delete or reassign a Control Number once allocated. A voided or cancelled Certificate must retain its Control Number, be marked `VOIDED` rather than removed, and require a mandatory reason note.

### 9.5 Offline Licensing & Feature Gating

- The system must compute a Machine Hardware ID at startup from the local CPU ID and motherboard serial number, and must display it in Settings in a form the Secretary can copy.
- The system must validate a submitted License Key entirely offline, using an Ed25519 signature check against a public key embedded in the application binary; it must not require network access to validate a key.
- The system must verify that a submitted License Key's bound Machine Hardware ID matches the local installation before enabling any Feature Flag it grants.
- Upon successful validation, the system must persist the resulting Feature Flags locally and make any newly unlocked module's navigation available immediately, without requiring an application restart.
- The system must provide a documented path for re-issuing a License Key when an installation's hardware changes (repair or replacement), without requiring alteration of existing Resident, Household, or Certificate records. See [Appendix C](appendix-c-license-reissuance-sop.md) for the supporting process.

### 9.6 Data Protection & Key Management

- The system must encrypt the local database at rest.
- The database encryption key must be derivable without the Machine Hardware ID as an input, so that a legitimate hardware change or a disaster-recovery restore never renders existing data or backups permanently unreadable. (Machine Hardware ID remains an input to license validation in [9.5](#95-offline-licensing--feature-gating), which is a distinct concern — see [Appendix B](appendix-b-key-derivation-and-recovery.md) for why these must not be conflated.)
- The system must generate, at first run, a human-transcribable Recovery Code that can independently unlock the encrypted database or a Backup Snapshot if the installation's protected credential store is lost.
- The system must not store the database encryption key, or a value trivially equivalent to it, in a form recoverable by inspecting the installed application binary alone.

### 9.7 Immutable Audit Trail

- The system must create an Audit Log Entry for every create, edit, soft-delete, print, or export action performed against Resident, Household, or Certificate data.
- Each Audit Log Entry must record, at minimum: a UTC timestamp, the authenticated Staff Account that performed the action, the action type, and the target record's identifier.
- The system must prevent updates or deletions of existing Audit Log Entries once written.

### 9.8 Backup & Recovery

- The system must take an automatic encrypted backup snapshot on application exit.
- The system must take automatic encrypted backup snapshots on a daily schedule while the application remains open.
- The system must retain a rolling window of at least the last 14 automatic snapshots and purge older ones automatically.
- The default backup destination must be fixed at `%APPDATA%\BarangayMS\backups\` with the 14-snapshot rolling retention above; first-run setup must not prompt the Secretary to choose a different destination. This can still be changed later via Settings if that capability is otherwise planned.
- The system must provide a one-action manual export of an encrypted backup archive to a user-selected destination (e.g., a USB drive), independent of the automatic rolling retention limit.
- A Backup Snapshot must be restorable using the Recovery Code described in [9.6](#96-data-protection--key-management), without requiring the original machine's hardware.

### 9.9 Barangay Identity & Branding

- The system must persist a single Barangay Identity profile per installation, containing: barangay name, barangay logo, municipality/city name, municipality/city logo (optional), province name, province logo (optional), ZIP code, barangay address, contact phone number, email address, official Facebook page URL (optional), and website URL (optional).
- Admin/Secretary must be able to edit the Barangay Identity at any time; changes must reflect in all newly issued Certificates and in relevant in-app UI chrome (e.g., login screen, sidebar, dashboard header) without requiring an application restart.
- Logo and image uploads must be stored as JPEG/PNG in the application's local data directory, following the same discipline as Resident photos in [9.2](#92-resident-registry).
- Barangay Identity data must be rendered on every generated Certificate's header and/or footer per [9.4](#94-certificate-generation--control-number-sequencing).

### 9.10 Barangay Officials & Signatures

- The system must persist Barangay Official records, one per individual currently or historically serving the barangay in an official capacity.
- Each Barangay Official record must include: full name, position (title or role, e.g., Barangay Captain, Secretary, Treasurer, Kagawad, SK Chairperson, Lupon Chairman, Barangay Health Worker, Tanod, Staff — the list of position types is barangay-configurable, not hardcoded), optional photo, optional signature image, term start date, term end date, and status (Active/Inactive).
- Admin/Secretary must be able to create, edit, and mark Barangay Official records Inactive; Inactive records must remain in the audit trail but must not be auto-selected for new certificate signatories.
- Certificate Templates must be able to reference Barangay Officials by position (e.g., `{{captain_name}}`, `{{secretary_name}}`); at certificate generation time, the system must auto-select the currently Active official holding that position and auto-populate the template.
- Each Barangay Official's signature image must support a per-Certificate/per-Template display mode: show (render the uploaded signature image on the certificate), hide (omit the signature line entirely), electronic (render the stored signature image), or wet-signature placeholder (render a blank line for physical signing later).
- A change of Barangay Officials must not alter historical Certificates; only future issuances auto-pick up the new signatory.

### 9.11 Certificate Template & Layout Customization

- Each Document Type must have an editable Certificate Template consisting of: structured body text (using Template Variables like `{{resident_name}}`, `{{age}}`, `{{address}}`, `{{purpose}}`, `{{date}}`, `{{captain_name}}`), header image (optional), watermark (optional), footer text (optional), seal placement (optional), QR code toggle (optional), and document border (optional).
- Admin/Secretary must be able to edit the body text and layout elements of any Document Type's Template; changes apply to newly issued Certificates only.
- The system must audit-lock each Certificate's rendered content at issuance: a Certificate must record the exact text and layout used to produce it, independent of later Template edits. If a Template is edited after a Certificate is issued, the historical Certificate must remain unchanged; only future issuances use the updated Template.
- Available Template Variables include those sourced from Resident profile (name, age, birthdate, civil status, occupation, contact number, address/Purok), Household data (head-of-household name, Purok/Zone), Certificate data (issue date, control number, purpose), Barangay Official data (by position: name, title, signature), and Barangay Identity data (barangay name, address, contact, municipality/city, province, logo references).

### 9.12 Purok/Sitio/Zone Management

- The system must support a barangay-configurable list of Purok/Sitio/Zone entries instead of hardcoded defaults.
- Admin/Secretary must be able to create, rename, reorder, and deactivate Purok/Sitio/Zone entries; Resident and Household records must select a Purok from this list rather than free-text entry, ensuring consistency.
- Deactivating a Purok/Sitio/Zone entry in use by existing Resident or Household records must not orphan those records; existing data must remain intact and reportable, but the entry must not be available for selection in new Resident/Household creation.

### 9.13 Appearance & Theming

- The system must persist a barangay-wide Theme setting affecting only in-app UI appearance (colors, contrast, dark/light mode), distinct from certificate branding in [9.9](#99-barangay-identity--branding) and [9.11](#911-certificate-template--layout-customization).
- Admin/Secretary must be able to set: primary color, accent color, and light/dark/system-preference mode.
- Theme changes must apply to the in-app UI without requiring an application restart.
- Theming is barangay-wide, not per-staff-account; all users see the same configured appearance.

### 9.14 Dashboard Widget Configuration

- The system must support an optional barangay dashboard (post-login or as a dedicated view) with toggleable informational widgets.
- Available widgets include: total population, senior citizens count, PWD count, solo parents count, registered voters count, upcoming birthdays, certificates issued (count or breakdown by type), cases (if KP Blotter is licensed), and revenue (if Treasury is licensed).
- Admin/Secretary must be able to toggle the visibility of each widget from a Settings panel.
- Widgets whose data comes from an unlicensed add-on module (e.g., cases from KP Blotter, revenue from Treasury) must be hidden from the toggle list until that module is licensed, consistent with feature-gating in [9.5](#95-offline-licensing--feature-gating).

## 10. API, Interaction, or Workflow Requirements

- **License activation:** Secretary pastes/uploads a License Key in Settings → the system validates it offline → on success, relevant module navigation appears immediately with no restart.
- **Session lock/unlock:** any staff member can trigger a one-click lock → unlock requires selecting a Staff Account and entering that person's own PIN.
- **Backup restore:** Admin/Secretary selects a Backup Snapshot or imported archive → the system prompts for the Recovery Code if the local protected credential store cannot supply the key → on success, the database is restored in place.
- **Certificate void:** Admin/Secretary selects an issued Certificate → provides a mandatory reason → the system marks it `VOIDED` while preserving its Control Number.
- **Barangay identity configuration:** Admin/Secretary navigates Settings → Barangay Profile → edits barangay name, logos, address, contact information, and web presence → changes appear immediately on newly issued Certificates and in relevant UI chrome.
- **Barangay officials management:** Admin/Secretary navigates Settings → Officials → adds or edits an Official (full name, position, photo, signature, term dates, status) → system auto-selects the current Active official for each Certificate Template variable requiring a signatory; no manual re-entry needed.
- **Certificate template customization:** Admin/Secretary navigates Settings → Document Types → selects a Document Type → edits template body text (using Template Variables), layout elements (header, footer, seal, QR code), and Control Number Format → changes apply to newly issued Certificates only; historical Certificates remain unchanged.
- **Custom document type creation:** Admin/Secretary navigates Settings → Document Types → creates a new Document Type → assigns it a name, Control Number Format, and Certificate Template → the system allocates an independent gapless Control Number sequence for this type.
- **Custom role creation:** Admin/Secretary navigates Settings → User Roles → creates a new Role → names it and selects which Permissions this Role grants → the new Role immediately becomes available for assignment to new Staff Accounts.
- **Purok/Sitio management:** Admin/Secretary navigates Settings → Puroks → adds, renames, or deactivates Purok entries → staff use the updated list when creating or editing Residents and Households.
- **Theme customization:** Admin/Secretary navigates Settings → Appearance → selects primary color, accent color, and light/dark/system mode → UI appearance updates immediately without restart.

## 11. Data Model Requirements

Conceptual entities and fields this release must persist (implementation-level schema, indexing, and migration mechanics belong to the downstream TDD):

- **Resident** — `id`, name fields, sex, birthdate, civil_status, occupation, contact_number, address/purok_id, sector_tags, photo_reference, plus the shared sync-ready columns (`barangay_code`, `updated_at`, `sync_status`).
- **Household** — `id`, purok_id, address, head_of_household_resident_id, plus shared sync-ready columns.
- **HouseholdMembership** — `id`, household_id, resident_id, relationship.
- **Certificate** — `id`, document_type_id (replaces `certificate_type` enum), control_number, resident_id, issue_date, purpose, template_snapshot (the rendered template content at issuance for audit-locking), status (`ISSUED` / `VOIDED`), void_reason (nullable), plus shared sync-ready columns.
- **ControlNumberSequence** — per-document-type monotonic counter state, updated only within the issuance transaction.
- **StaffAccount** — `id`, full_name, username, designation, role_id (replaces `role` enum), password_hash, plus shared sync-ready columns.
- **AuditLogEntry** — `id`, timestamp_utc, staff_account_id, action_type, target_entity_id, plus shared sync-ready columns.
- **LicenseGrant** — `id`, machine_hardware_id, feature_flags, signature_metadata.
- **BackupSnapshot** — `id`, created_at, trigger_type (`exit` / `scheduled` / `manual`), storage_location.
- **BarangayProfile** — `id`, barangay_name, barangay_logo_reference, municipality_city, municipality_city_logo_reference (nullable), province, province_logo_reference (nullable), zip_code, barangay_address, contact_phone, contact_email, facebook_url (nullable), website_url (nullable), plus shared sync-ready columns.
- **BarangayOfficial** — `id`, full_name, position_title, photo_reference (nullable), signature_image_reference (nullable), term_start_date, term_end_date, status (`Active` / `Inactive`), plus shared sync-ready columns.
- **DocumentType** — `id`, name (e.g., "Barangay Clearance"), is_seeded (boolean, true for the four core types), enabled (boolean), control_number_format (pattern), certificate_template_id, plus shared sync-ready columns.
- **CertificateTemplate** — `id`, document_type_id, body_text, header_image_reference (nullable), watermark_image_reference (nullable), footer_text (nullable), seal_placement (nullable), qr_code_enabled (boolean), border_style (nullable), plus shared sync-ready columns.
- **Purok** — `id`, name, display_order, enabled (boolean), plus shared sync-ready columns.
- **Role** — `id`, name, is_seeded (boolean, true for the four default roles), enabled (boolean), plus shared sync-ready columns.
- **Permission** — `id`, key (e.g., `resident.create`), description, plus shared sync-ready columns.
- **RolePermission** — `id`, role_id, permission_id, granted (boolean).
- **ThemeSetting** — `id`, primary_color (hex), accent_color (hex), dark_mode_enabled (boolean or enum for system/light/dark). TDD may treat this as device-local preference rather than synced data.
- **DashboardWidgetConfig** — `id`, widget_key (e.g., `population`, `senior_citizens`, `certificates_issued`, `cases`, `revenue`), enabled (boolean). TDD may treat this as device-local preference rather than synced data.

## 12. Non-Functional Requirements

- **NFR-01 (Startup Performance):** Cold startup must complete in under 3 seconds on an Intel i3-class PC with 4GB RAM.
- **NFR-02 (Search Performance & Capacity):** The system must support up to 50,000 Resident records and return name-search results in under 200ms.
- **NFR-03 (Installer Size):** The distributed installer must be under 30MB. This must be validated by the technical spike in [Appendix A](appendix-a-technical-architecture-direction.md) before Phase 1 is considered complete.
- **NFR-04 (Offline Operation):** The system must provide 100% of its functionality with no network connection present.
- **NFR-05 (Single Logical Write Path):** All reads and writes to the local database must be funneled through a single backend data-access boundary; no UI or frontend component may query the database directly. This preserves the option to expose that same boundary over a local network in a future release without a rewrite, and prevents any accidental multi-process file access that could corrupt the database. See [Appendix A](appendix-a-technical-architecture-direction.md).
- **NFR-06 (Auditability):** Every Audit Log Entry, once written, must be immutable for the lifetime of the installation.

## 13. Success Criteria

This release is successful when an authorized staff member can, entirely offline on a target-spec PC:

1. Log in with an individual Staff Account and have the app auto-lock after 15 minutes idle.
2. Create a Resident profile, tag it with one or more Sector Tags, and find it via search in under 200ms.
3. Group Residents into a Household with a designated Head of Household, selecting from a barangay-configurable Purok list (no free-text entry).
4. Generate any of the four core Certificates with a correctly auto-filled template (including barangay branding, current official signatory, and a gapless Control Number).
5. Void an issued Certificate with a reason note and confirm its Control Number is neither reused nor missing from the sequence.
6. Copy the installation's Machine Hardware ID, submit a License Key, and see a module unlock instantly with no restart.
7. Simulate a hardware replacement and confirm, via the documented reissuance path, that existing records remain fully readable.
8. Restore a Backup Snapshot on a different machine using only the Recovery Code, with no dependency on the original hardware.
9. Review an Audit Log Entry attributing a given action to the correct Staff Account, and confirm no entry can be edited or deleted.
10. Configure the barangay name, logo, officials, and contact information once in Settings; verify these appear automatically on newly issued Certificates without re-entry.
11. Add a new Barangay Official (e.g., a new Secretary), mark a previous Secretary Inactive, and confirm future Certificates auto-pick up the new signatory while historical Certificates remain unchanged.
12. Edit a Certificate Template's body text and layout; issue a new Certificate and confirm it uses the updated template, while a previously issued Certificate is unchanged.
13. Create a custom Document Type (e.g., "Employment Certificate"), assign it a Control Number Format, and confirm it receives its own independent, gapless Control Number sequence separate from other types.
14. Create a Custom Role (e.g., "Data Verifier") with a subset of Permissions (e.g., `resident.view`, `resident.edit` but not `certificate.issue`), assign it to a Staff Account, and confirm that staff member can only perform the permitted actions.
15. Add a new Purok (e.g., "Purok 4") to the barangay's administrative divisions; verify it appears in the Purok dropdown for new Resident/Household creation.
16. Set the barangay's theme to a custom primary color and dark mode; verify the in-app UI reflects these changes immediately without an app restart.

## 14. Open Questions

v2.3 resolved earlier open questions (see [9.8](#98-backup--recovery) and [6](#6-non-goals) for their resulting requirements):

- **Treasurer scope:** resolved to minimal certificate/payment-matching fields only, not full Resident profiles.
- **Backup destination:** resolved to a fixed default (`%APPDATA%\BarangayMS\backups\`, 14-snapshot retention), no first-run prompt.
- **Print-preview/margin calibration:** resolved to a scheduled fast-follow UX pass immediately after release.

v2.4 amendments supersede prior v2.2 decisions on RBAC and Certificate Types:

- **RBAC: fixed 4 roles vs. custom Roles with Permissions** (v2.2: resolved to "4 fixed roles"; v2.4: **re-resolved to full custom RBAC** — see [9.1](#91-authentication--role-based-access-control)). The system now ships the four defaults as editable seed Roles; Admins can create unlimited Custom Roles with individually toggled Permissions.
- **Certificate types & Control Number Formats: fixed types & format vs. fully configurable** (v2.2: resolved to "Barangay Clearance, Certificate of Indigency, Certificate of Residency, Certificate of Good Moral Character, with example format BC-2026-00001"; v2.4: **re-resolved to fully custom per-type Formats and Custom Document Types** — see [9.4](#94-certificate-generation--control-number-sequencing)). The four core types cannot be deleted but may be disabled; barangays may add Custom Document Types; each Document Type has its own configurable Control Number Format pattern, all while preserving atomic, gapless allocation per type.

v2.4 introduces new open questions deferred to the downstream TDD:

- **Permission taxonomy:** Exact set of Permission keys (e.g., `resident.create`, `resident.view`, `household.edit`, `certificate.issue`, `certificate.void`, `staff.manage`, `settings.manage`, `license.manage`, `backup.manage`, `audit.view`, or a different breakdown) is TDD-level detail subject to implementation considerations.
- **Control Number Format token grammar:** Exact syntax and tokens for configurable Control Number Formats (e.g., `YYYY-NNNN` for year + zero-padded sequence, or `TYPE-YYYY-NN`, or other pattern language) is TDD-level detail.
- **Template Variable availability:** Complete enumeration of available Template Variables for Certificate Templates (currently outlined in [9.11](#911-certificate-template--layout-customization)) must be finalized during TDD based on data model feasibility.
- **Theme and Widget Config as device-local vs. synced:** `ThemeSetting` and `DashboardWidgetConfig` are noted in [11](#11-data-model-requirements) as candidates for device-local-preference storage rather than sync-ready columns; the TDD must decide whether these belong in the sync-ready schema or in a local-device config file.
