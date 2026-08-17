# Barangay Management System (BMS) Master Product Requirements Document

Status: Vision

Owner: Product and Engineering (solo)

Scope: Release-independent product direction

Related documents:

- [Sync-ready schema & municipal integration concepts](appendix-a-sync-ready-schema-and-municipal-integration.md)
- [Core Engine release package](../core-engine/README.md)
- [Katarungang Pambarangay (KP Blotter) package](../kp-blotter/README.md)
- [Barangay Treasury package](../treasury/README.md)
- [Business Clearances & Permits package](../business-permits/README.md)

---

## 1. Product Summary

The Barangay Management System (BMS) is a modular, offline-first desktop application for Philippine barangays and LGUs operating under weak, intermittent, or zero internet connectivity. It replaces manual/paper-based resident record-keeping with a single-installation desktop app that runs entirely on local hardware, with no runtime dependency on the internet or any remote server.

## 2. Problem

Barangay offices are the front line of Philippine local government but typically operate on low-spec PCs with unreliable or absent internet access. Existing government digitization efforts assume connectivity (cloud dashboards, hosted databases, SaaS portals) that many barangays cannot rely on day-to-day. Meanwhile, manual paper records for residents, households, and certificates are slow to search, easy to lose, and produce no audit trail for compliance with the Data Privacy Act of 2012 (DPA 2012). Barangays also vary widely in which extra functions they need (dispute mediation, fee collection, business permitting), so a single monolithic system risks being either underpowered for busier barangays or bloated for smaller ones.

## 3. Vision

BMS should be the durable, offline record-of-truth for a barangay's residents, households, and issued documents — reliable on a four-year-old office PC with no internet, auditable enough to satisfy DPA 2012 and COA expectations, and extensible so a barangay can add specialized capability (dispute mediation, treasury, business permits) only when it needs it and is willing to pay for it, without ever being forced onto a always-connected architecture.

## 4. Product Principles

- **Offline-first, always.** No product capability may require internet access or a remote server to function. Connectivity is optional and, when it exists at all, is used only for out-of-band license delivery or future municipal data export — never for runtime operation.
- **Plug-and-play, feature-as-a-service licensing.** The Core Engine is sold as a perpetual license. Specialized capability ships as separately licensed modules, unlocked entirely offline via cryptographically signed keys tied to the installation's hardware.
- **Barangay data ownership.** Each barangay installation is the sole authoritative owner and editor of its own resident and household data. No other system, including any future municipal aggregator, may create or edit that data — only mirror it read-only.
- **Audit-grade by default.** Any action touching resident personal information, or any officially numbered document, must be individually attributable to a real staff member and immutably logged, from the first release onward — not bolted on later.
- **Low-spec hardware as the design constraint, not the edge case.** Every product decision is made assuming a Core i3, 4GB RAM Windows PC with no internet — not a developer workstation.
- **Recoverability independent of any single machine.** Data-at-rest protection must never make a barangay's own records unrecoverable to the barangay itself, including after a hardware failure or replacement.

## 5. Actors

| Actor | Permission family | Primary reason they use BMS |
|---|---|---|
| Barangay Captain / Officials | Read-only | Review resident/household records and dashboard reports; approve/sign issued documents. |
| Barangay Secretary / Admin | Full administrative access | Primary system operator: manage residents, households, staff accounts, licensing, backups, and audit logs. |
| Encoder | Data entry | Add/edit resident records and draft certificates under the Secretary's oversight. |
| Barangay Treasurer | Financial operations | Issue official receipts and view fee/financial ledgers once a fee-tracking module is licensed. |
| Vendor Support (BMS operator) | External, out-of-band | Issues signed license keys against a barangay's submitted Machine Hardware ID; never has runtime access to barangay data. |

## 6. Ownership Boundaries

- Each barangay's installation owns its resident, household, and certificate data exclusively. No product capability may write to another barangay's data.
- The BMS vendor (license issuer) owns the licensing registry and key-signing process, but has no runtime or network access to any barangay's operational data.
- A future municipal aggregator, if adopted, owns only a read-only mirrored cache built from barangay-exported data packages — it never becomes a second writer of resident data (see [Appendix A](appendix-a-sync-ready-schema-and-municipal-integration.md)).

## 7. Capability Horizons

### 7.1 Current

No capability is implemented yet — this document precedes Phase 1 development. The [Core Engine package](../core-engine/README.md) defines the first release contract.

### 7.2 Planned

- **Core Engine** — staff accounts & RBAC, resident registry, household registry, certificate generator with audit-grade control-number sequencing, offline licensing/key-management engine, immutable audit trail, and automated encrypted backups. This is the foundational release; every other module depends on its licensing engine for feature-gating. See [Core Engine PRD](../core-engine/PRD.core-engine.md).
- **Katarungang Pambarangay (KP Blotter) module** — dispute/incident logging and case lifecycle tracking, gated behind a paid license key, built on Core Engine's resident registry and audit trail. See [KP Blotter PRD](../kp-blotter/PRD.kp-blotter.md).
- **Barangay Treasury module** — fee collection ledger and official receipt tracking tied to certificates issued by Core Engine, gated behind a paid license key. See [Treasury PRD](../treasury/PRD.treasury.md).
- **Business Clearances & Permits module** — local business registry and annual permit/clearance tracking, gated behind a paid license key. See [Business Permits PRD](../business-permits/PRD.business-permits.md).

### 7.3 Exploratory

- **Municipal Data Aggregation & Syncing** — an encrypted, delta-based export routine (`.bmssync`) that would let a Municipal Hall aggregate read-only mirrored data from constituent barangays, if a Municipality ever adopts BMS across its barangays. Not scheduled, not committed, and must not be assumed by any Planned package's implementation. See [Appendix A](appendix-a-sync-ready-schema-and-municipal-integration.md) for the conceptual model. This item requires its own focused PRD before any delivery planning begins.
- **Multi-PC / Local Network Access (LAN Mode)** — letting one PC in a barangay hall act as the server for that installation's database while other PCs/laptops act as clients against it, so more than one staff member can work against a single barangay's data concurrently. Two connectivity shapes are envisioned: (1) over an existing local WiFi/router network, and (2) with no router or WiFi infrastructure present at all — e.g., a direct PC-to-PC link (Ethernet/USB networking) or a PC-hosted hotspot — using the same client/server model either way, never internet-dependent. Core Engine's NFR-05 (single logical write path through a backend data-access boundary) and [Appendix A](../core-engine/appendix-a-technical-architecture-direction.md) already keep this option open architecturally (an embedded local server such as `axum`) without implementing it; direct SQLite file-sharing over SMB/network drives remains explicitly ruled out as unsafe. Not scheduled, not committed, and must not be assumed by any Planned package's implementation. This item requires its own focused PRD before any delivery planning begins.

## 8. Cross-Cutting Requirements

These apply to the Core Engine and to every current or future add-on module:

- The product must operate with 100% functionality when fully disconnected from the internet and any network.
- The product must run acceptably on Windows 10/11 (64-bit), 4–8GB RAM, an Intel Core i3-class (or equivalent) processor, and a locally attached USB printer.
- The product must encrypt all resident personal data at rest, consistent with DPA 2012 expectations.
- Every module's licensing must be validated entirely offline via an asymmetric signature scheme tied to the installation's Machine Hardware ID; no module may phone home to validate a license.
- Every database table storing product-owned records must carry the sync-ready schema conventions (UUIDv4 identifiers, `barangay_code`, `updated_at`, `sync_status`) defined in [Appendix A](appendix-a-sync-ready-schema-and-municipal-integration.md), even while the Exploratory municipal sync capability remains unscheduled — retrofitting these later is materially more expensive than including them now.
- Monetization for every module follows the perpetual-license, pay-per-module model: one core license per installation, additional modules unlocked via separately purchased signed keys.

## 9. Product Boundaries

BMS, across all current and future packages, must not:

- require a persistent internet connection, cloud account, or hosted service for any day-to-day operation;
- support automatic over-the-internet software updates (distribution is via offline installer only);
- allow any system other than the originating barangay's own installation to create or edit that barangay's resident or household data;
- support multi-tenant hosting of more than one barangay's data in a single installation; or
- provide general-purpose accounting/GL functionality beyond the fee ledger scoped to the Treasury module.

## 10. Feature Map

| Package | Link | Product role | Lifecycle status |
|---|---|---|---|
| Core Engine | [core-engine/](../core-engine/README.md) | Foundational release: RBAC, resident/household registries, certificate generator, licensing engine, audit trail, backups | Planned — in scoping, precedes Phase 1 |
| KP Blotter | [kp-blotter/](../kp-blotter/README.md) | Add-on module: dispute/incident mediation tracking | Planned — depends on Core Engine |
| Treasury | [treasury/](../treasury/README.md) | Add-on module: fee ledger & official receipts | Planned — depends on Core Engine |
| Business Permits | [business-permits/](../business-permits/README.md) | Add-on module: business registry & clearance renewals | Planned — depends on Core Engine |
| Municipal Sync | *(no package yet)* | Exploratory: read-only municipal aggregation | Exploratory — not scheduled, no approved PRD |
| LAN Mode | *(no package yet)* | Exploratory: multi-PC/local-network access to one barangay's database, including no-router fallback | Exploratory — not scheduled, no approved PRD |
