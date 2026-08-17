# Application ADRs

Application-specific architecture decisions for the Barangay Management System (BMS).
Baseline Hexon decisions live separately as HADRs in
[`backend_arch_docs/adr/`](../../backend_arch_docs/adr/README.md); application-only
decisions live here. A decision belongs here when it is application-specific rather
than reusable across every derived service.

## What warrants an ADR

A decision that materially affects application structure, boundaries, dependencies,
data ownership, security, delivery, or construction technique — and whose rationale
will matter after the discussion ends. Routine local choices belong in a technical
design (TDD) or in code.

## Naming and status

- Path: `docs/adr/{lifecycle}/{class}/yyyy-mm-dd-nnnn-short-kebab-title.md`
- `{lifecycle}`: `proposed | implemented | rejected | archived` — must agree with the
  inline `Status:` field in the record.
- `{class}`: `feature | bug-fix | simplification | architecture | process | testing`.
- `nnnn`: four-digit, zero-padded, **global monotonic across all of `docs/adr/`**, never reused.
- Date (`yyyy-mm-dd`): the date the decision was recorded.

Historical ADRs are retained; change an implemented decision with a new ADR and
explicit relationship links rather than rewriting the old record.

## Index

| ADR | Title | Status | Class | Relationships |
| --- | --- | --- | --- | --- |
| [ADR-0001](implemented/architecture/2026-08-17-0001-lock-core-engine-application-stack.md) | Lock the Core Engine application stack (Tauri v2 + bundled SQLCipher + React/shadcn) | Implemented | architecture | Retains HADR-0002, HADR-0003 |
| [ADR-0002](implemented/architecture/2026-08-17-0002-fix-core-engine-workspace-crate-topology.md) | Fix the Core Engine workspace crate topology (app_core + infra_* + src-tauri) | Implemented | architecture | Retains HADR-0002, HADR-0003; refines ADR-0001 |
| [ADR-0003](implemented/architecture/2026-08-17-0003-define-feature-flag-taxonomy.md) | Define the Feature Flag taxonomy | Implemented | architecture | Refines ADR-0002 |
| [ADR-0004](implemented/architecture/2026-08-17-0004-define-rbac-permission-taxonomy.md) | Define the RBAC Permission taxonomy and seed roles | Implemented | architecture | Refines ADR-0002; follows ADR-0003 precedent |
| [ADR-0005](implemented/architecture/2026-08-17-0005-lock-core-engine-licensing-key-management-mechanics.md) | Lock the Core Engine licensing & key-management mechanics (Hardware ID, bootstrap storage, Recovery Code) | Implemented | architecture | Refines ADR-0002; retains ADR-0001; normalizes Appendix B/C |
