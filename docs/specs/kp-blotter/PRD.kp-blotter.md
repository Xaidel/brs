# KP Blotter PRD

## Katarungang Pambarangay (KP Blotter) Module

Status: Proposed

Owner: Product and Engineering (solo)

Version: 2.1 (2026-08-04)
Release tag: Add-on module — depends on Core Engine

Related documents:

- [Master PRD](../master/PRD.md)
- [Core Engine PRD](../core-engine/PRD.core-engine.md) (dependency: licensing engine, resident registry, audit trail)
- [Package README](README.md)

---

## 1. Summary

The KP Blotter module adds Katarungang Pambarangay dispute/incident logging and case lifecycle tracking to a barangay's installation, unlocked via a paid License Key on top of the [Core Engine](../core-engine/PRD.core-engine.md). It depends entirely on Core Engine's resident registry, staff authentication, and audit trail — it introduces no independent identity or data-ownership model of its own.

## 2. Problem Statement

Barangay secretaries currently track dispute mediation (blotter entries, hearing notices, summons, case outcomes) on paper, with no structured link back to the complainant/respondent's resident record and no consistent way to produce the official DILG-prescribed forms a case requires.

## 3. Goals

- Staff must be able to log an incident with complainant(s), respondent(s), date/time, location, narration, and offense classification.
- Staff must be able to track a case through its lifecycle: Pending, Scheduled for Mediation, Settled, or Escalated to Court (Certificate to File Action issued).
- Staff must be able to generate the official KP forms (Notice of Hearing, Summons, Officer's Return) auto-filled from case data.

## 4. Non-Goals

- Any resident or household data entry — this module must reference Residents already created in Core Engine, never create its own resident records.
- Legal advice, mediation outcome determination, or court filing automation beyond producing the prescribed forms.
- Fee collection for blotter-related services — see the [Treasury package](../treasury/README.md) if applicable.

## 5. Target Users

Inherits Core Engine's roles ([Master PRD §5](../master/PRD.md#5-actors)). Primary users: Admin/Secretary and Encoder, for case entry and form generation; Read-Only/Captain for case status visibility.

## 6. Product Requirements

- The system must allow an authenticated staff member to create an incident record referencing one or more existing Resident records as complainant(s) and respondent(s).
- The system must capture incident date/time, location, narration, and offense classification for each incident.
- The system must track each case through the lifecycle states: Pending, Scheduled for Mediation, Settled, Escalated to Court.
- The system must generate official KP Form 7 (Notice of Hearing), Form 8 (Summons), and Form 9 (Officer's Return), auto-filled from the case and referenced Resident data.
- Generated KP forms must adhere to the template structures defined in the DILG Katarungang Pambarangay Law Implementing Rules and Regulations (IRR); form field accuracy must be validated against the current official IRR templates before release, not assumed from prior drafts.
- Every action against a case record must produce an Audit Log Entry, using the same immutable audit mechanism defined in [Core Engine PRD §9.7](../core-engine/PRD.core-engine.md#97-immutable-audit-trail).
- This module's Feature Flag must be gated by Core Engine's offline licensing engine ([Core Engine PRD §9.5](../core-engine/PRD.core-engine.md#95-offline-licensing--feature-gating)); its navigation and functionality must remain hidden until a valid License Key unlocks it.

## 7. Success Criteria

This release is successful when an authorized staff member can, entirely offline:

1. Log an incident against existing Resident records as complainant and respondent.
2. Move a case through its full lifecycle to a terminal state (Settled or Escalated to Court).
3. Generate Form 7, Form 8, and Form 9 auto-filled with case and resident data, matching current DILG IRR templates.
4. Confirm the module remains inaccessible without a valid License Key, and unlocks immediately once one is applied.
