# Treasury PRD

## Barangay Treasury & Fee Tracking Module

Status: Proposed

Owner: Product and Engineering (solo)

Version: 2.1 (2026-08-04)
Release tag: Add-on module — depends on Core Engine

Related documents:

- [Master PRD](../master/PRD.md)
- [Core Engine PRD](../core-engine/PRD.core-engine.md) (dependency: certificate generator, licensing engine, audit trail)
- [Package README](README.md)

---

## 1. Summary

The Treasury module adds fee collection ledger tracking and official receipt issuance to a barangay's installation, unlocked via a paid License Key on top of the [Core Engine](../core-engine/PRD.core-engine.md). It ties directly to certificates already issued by Core Engine rather than introducing an independent document-issuance path.

## 2. Problem Statement

Barangay treasurers currently track certificate/clearance fee collections and issue official receipts manually, with no structured tie back to which certificate was paid for, and no simple way to produce the daily/monthly collection summaries expected for financial reporting.

## 3. Goals

- Staff must be able to record a fee payment tied to a specific certificate issued in Core Engine.
- Staff must be able to generate an Official Receipt (OR) with its own audit-grade sequence number.
- Staff must be able to export daily and monthly collection summaries to CSV/Excel.

## 4. Non-Goals

- General ledger or full accounting functionality — this module is a fee/collection ledger only, not a chart-of-accounts bookkeeping system.
- Certificate generation itself — Certificates are created in Core Engine; this module only records payment against them.
- Payroll, budgeting, or any financial function beyond fee collection for issued certificates and permits.

## 5. Target Users

Inherits Core Engine's roles ([Master PRD §5](../master/PRD.md#5-actors)). Primary user: Treasurer, for OR issuance and ledger review; Admin/Secretary for oversight and summary export.

## 6. Product Requirements

- The system must allow an authenticated Treasurer to record a fee payment against a specific Certificate issued in Core Engine.
- The system must generate an Official Receipt with a unique, monotonically increasing sequence number, using the same atomic-transaction sequencing guarantee defined in [Core Engine PRD §9.4](../core-engine/PRD.core-engine.md#94-certificate-generation--control-number-sequencing).
- The system must record, per Official Receipt: cashier (Staff Account), sequence number, amount, and issued date.
- The system must produce a daily and a monthly collection summary, exportable as CSV or Excel.
- Every action against a fee or receipt record must produce an Audit Log Entry, using the same immutable audit mechanism defined in [Core Engine PRD §9.7](../core-engine/PRD.core-engine.md#97-immutable-audit-trail).
- This module's Feature Flag must be gated by Core Engine's offline licensing engine ([Core Engine PRD §9.5](../core-engine/PRD.core-engine.md#95-offline-licensing--feature-gating)); its navigation and functionality must remain hidden until a valid License Key unlocks it.

## 7. Success Criteria

This release is successful when an authorized Treasurer can, entirely offline:

1. Record a fee payment against an existing Certificate and issue an Official Receipt with a gapless sequence number.
2. Export a daily and a monthly collection summary to CSV or Excel.
3. Confirm the module remains inaccessible without a valid License Key, and unlocks immediately once one is applied.
