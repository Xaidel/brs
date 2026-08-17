# Appendix A

## Technical Architecture Direction

Back to the owning PRD: [Core Engine PRD](PRD.core-engine.md)

The owning PRD remains authoritative for product scope. This appendix clarifies supporting detail and must not broaden or contradict that scope. The specific technology choices below are informative direction for the downstream TDD, not product requirements in themselves — the binding product constraints are NFR-01 through NFR-06 in the main PRD.

---

## 1. Purpose

Record the technology direction chosen to meet this release's non-functional requirements (startup time, installer size, offline operation, single logical write path), and the feasibility risk that must be retired early.

## 2. Recommended Stack

| Architecture Layer | Chosen Technology | Rationale |
|---|---|---|
| Desktop Framework | Tauri (Rust + Web Frontend) | Idle RAM footprint ~30MB–50MB; cold startup under 1 second — supports NFR-01. |
| Embedded DB & Encryption | SQLite with SQLCipher (via `rusqlite`) | Zero background CPU usage; encrypts data at rest, supporting the encryption requirement in [PRD §9.6](PRD.core-engine.md#96-data-protection--key-management). |
| Search Engine | SQLite FTS5 (Full-Text Search) | Supports the sub-200ms search requirement in NFR-02 at 50,000+ records. |
| Frontend Framework | Vite + Vue 3 / React (Tailwind CSS) | Lightweight DOM rendering for fluid UI on dual-core, low-RAM hardware. |

## 3. Backend Command Service Boundary

This diagram illustrates one way to satisfy NFR-05 (single logical write path):

```
+-----------------------------------------------------------------------------------+
|                            TAURI FRONTEND (UI LAYER)                              |
+-----------------------------------------------------------------------------------+
                                          |
                        Tauri IPC Commands (Strict Boundary)
                                          v
+-----------------------------------------------------------------------------------+
|                        RUST BACKEND COMMAND SERVICE LAYER                         |
|  • Single Data Access Service (all DB reads/writes funnel through Rust logic)     |
|  • Local Staff Authentication & Session State                                     |
|  • Cryptographic Licensing Engine (Ed25519) & Key Derivation                      |
|  • Transactional Control Number Sequencing Engine                                 |
+-----------------------------------------------------------------------------------+
                                          |
                                    SQLCipher IPC
                                          v
+-----------------------------------------------------------------------------------+
|                           ENCRYPTED LOCAL SQLITE DB                               |
|                     (Core Engine + Pre-initialized Plugin Tables)                 |
+-----------------------------------------------------------------------------------+
```

Direct database queries from frontend components are prohibited by NFR-05, not merely discouraged. This boundary is what would allow a later release to expose the same service over a local network (e.g., via an embedded `axum` server) to support multi-PC use within one barangay hall, without rewriting business logic. SQLite file-sharing over SMB/network drives is unreliable on Windows and risks corruption — it must never be the answer to multi-PC access, in this release or any future one.

## 4. Mandatory Technical Spike (Phase 1)

Before finalizing backend bindings, run a spike to verify:

- Static linking of `sqlcipher` in `rusqlite` alongside the `fts5` feature compiles cleanly.
- Full-text search performance is preserved with both features enabled simultaneously.
- The resulting installer binary size stays under the 30MB budget in NFR-03.

If any of these fail, revisit the stack choice before committing further implementation time — do not assume compatibility.

## 5. Usage Guidance

The downstream Phase 1 TDD should treat this appendix as its starting technical direction, subject to the spike's outcome, and should cite [PRD §9.5–9.6](PRD.core-engine.md#95-offline-licensing--feature-gating) and [Appendix B](appendix-b-key-derivation-and-recovery.md) for the licensing and key-management requirements this architecture must satisfy.
