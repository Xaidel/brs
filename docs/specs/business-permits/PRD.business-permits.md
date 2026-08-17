# Business Permits PRD

## Business Clearances & Permits Module

Status: Proposed

Owner: Product and Engineering (solo)

Version: 2.1 (2026-08-04)
Release tag: Add-on module — depends on Core Engine

Related documents:

- [Master PRD](../master/PRD.md)
- [Core Engine PRD](../core-engine/PRD.core-engine.md) (dependency: licensing engine, audit trail)
- [Package README](README.md)

---

## 1. Summary

The Business Permits module adds a local business entity registry and annual clearance renewal tracking to a barangay's installation, unlocked via a paid License Key on top of the [Core Engine](../core-engine/PRD.core-engine.md).

## 2. Problem Statement

Barangays currently track registered local businesses and their annual clearance renewals on paper, with no structured way to see which businesses are due for renewal or to issue a business clearance consistently.

## 3. Goals

- Staff must be able to register a local business entity (business name, owner, line of business, address/Purok).
- Staff must be able to track annual clearance renewal status and assessment fees per business.
- Staff must be able to issue a Barangay Business Clearance auto-filled from the business record.

## 4. Non-Goals

- Business tax computation or general revenue assessment beyond the clearance fee itself.
- Any resident/household data entry — this module references business owners; it does not duplicate resident demographic fields already owned by Core Engine.
- Integration with any external national business registry (e.g., DTI, SEC, BIR) — out of scope for this release.

## 5. Target Users

Inherits Core Engine's roles ([Master PRD §5](../master/PRD.md#5-actors)). Primary users: Admin/Secretary and Encoder, for business registry maintenance and clearance issuance.

## 6. Product Requirements

- The system must allow an authenticated staff member to register a local business entity, capturing business name, owner, line of business, and address/Purok.
- The system must track each business's annual clearance renewal status and associated assessment fee.
- The system must generate a Barangay Business Clearance document auto-filled from the business record.
- If a business owner corresponds to an existing Resident record, the system should reference that Resident rather than duplicating demographic fields.
- Every action against a business or clearance record must produce an Audit Log Entry, using the same immutable audit mechanism defined in [Core Engine PRD §9.7](../core-engine/PRD.core-engine.md#97-immutable-audit-trail).
- This module's Feature Flag must be gated by Core Engine's offline licensing engine ([Core Engine PRD §9.5](../core-engine/PRD.core-engine.md#95-offline-licensing--feature-gating)); its navigation and functionality must remain hidden until a valid License Key unlocks it.

## 7. Success Criteria

This release is successful when an authorized staff member can, entirely offline:

1. Register a local business entity and record its annual clearance renewal status and fee.
2. Issue a Barangay Business Clearance auto-filled from the business record.
3. Confirm the module remains inaccessible without a valid License Key, and unlocks immediately once one is applied.
