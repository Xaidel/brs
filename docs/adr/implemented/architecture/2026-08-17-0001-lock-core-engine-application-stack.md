# ADR-0001: Lock the Core Engine application stack (Tauri v2 + bundled SQLCipher + React/shadcn)

<!-- Location: docs/adr/implemented/architecture/2026-08-17-0001-lock-core-engine-application-stack.md.
     The inline Status below must agree with {lifecycle}. -->

- **Date**: 2026-08-17
- **Status**: Implemented
- **Deciders**: Product and Engineering (solo)

## Context and Problem Statement

Core Engine [Appendix A](../../../specs/core-engine/appendix-a-technical-architecture-direction.md) records a technical architecture direction as *informative*: Tauri desktop, SQLite with SQLCipher via `rusqlite`, FTS5 search, and a "Vite + Vue 3 / React" frontend. The [Core Engine PRD](../../../specs/core-engine/PRD.core-engine.md) leaves password hashing open ("Argon2id or bcrypt", §9.1) and records key-derivation and recovery mechanics (PBKDF2-HMAC-SHA256, `keyring` for `system_secret`) only as *informative* [Appendix B](../../../specs/core-engine/appendix-b-key-derivation-and-recovery.md) guidance. The downstream crate-topology and technical-design tickets must finalize backend bindings, but they inherit an open set of choices instead of a fixed target.

The binding constraints are the Core Engine non-functional requirements: 100% offline operation with no runtime network dependency (NFR-04), encryption at rest recoverable independent of the Machine Hardware ID (PRD §9.6), audit-grade immutability (NFR-06), cold startup under 3s on an Intel i3/4GB PC (NFR-01), sub-200ms name search at 50,000 records (NFR-02), a ≤30MB installer (NFR-03), and a single logical write path through a Rust backend boundary (NFR-05).

## Decision Drivers

- **Offline-first, self-contained:** one installer, no runtime network dependency (NFR-03, NFR-04).
- **Encryption at rest that survives hardware change:** the DB key must not derive from the Machine Hardware ID, and a Recovery Code must remain an independent unlock path (PRD §9.6, Appendix B).
- **Single logical write path:** every DB read/write funnels through the Rust backend, never the frontend (NFR-05).
- **Low-spec Windows target:** idle RAM and cold start suited to an i3/4GB PC (NFR-01).
- **Search at scale:** sub-200ms at 50,000 resident records (NFR-02).
- **Memory-hard credential hashing:** Argon2id-class hardness for low-entropy PINs (PRD §9.1).
- **Asymmetric offline license validation:** Ed25519 against an embedded public key (PRD §9.5).
- **Framework-free domain:** keep `app_core` independent of deployment per HADR-0002/HADR-0003.

## Decision

We will build Core Engine (and the add-on modules that depend on it) on this normative stack:

- **Desktop shell:** Tauri v2 — Rust backend + web frontend.
- **Embedded database:** SQLite with **SQLCipher compiled bundled** (statically linked into the binary via `rusqlite`'s bundled SQLCipher build), with **FTS5** enabled for full-text search. No system SQLite/SQLCipher dependency at install time.
- **Frontend:** React + Tailwind CSS + shadcn/ui, built with Vite.
- **Offline licensing:** Ed25519 signature verification against a public key embedded in the binary.
- **Password hashing:** Argon2id (not bcrypt).
- **Database key derivation:** PBKDF2-HMAC-SHA256 from an installation `system_secret`, deliberately excluding the Machine Hardware ID.
- **Credential storage:** the OS protected credential store (Windows Credential Manager) via the `keyring` crate, with the human-transcribable Recovery Code as the independent fallback unlock path.

Concrete crate selection and versions (`rusqlite`, `ed25519-dalek`, `argon2`, `pbkdf2`/`hmac`/`sha2`, `keyring`, and the frontend package set) are **deferred** to the crate-topology ticket and the Core Engine technical design, and will be pinned in `Cargo.toml` / `package.json` there. This ADR locks technologies, not crate versions.

The frontend must not query the database directly (NFR-05): all reads and writes flow through Tauri IPC into the Rust backend boundary.

## Alternatives Considered

### Option A: Keep Appendix A informative (do nothing)

Leave the stack as informative direction and resolve per-layer choices inside the technical design.

- Benefits: defers all commitment; no record to maintain.
- Costs and risks: the crate-topology and technical-design tickets inherit an open choice set, so "finalizing backend bindings" (Appendix A §4) has no fixed target to finalize against; the frontend framework (Vue vs React) and password KDF (Argon2id vs bcrypt) remain unresolved.

### Option B: Electron + plain SQLite

The default web-technology fallback: Electron shell with plain SQLite (no at-rest encryption).

- Benefits: largest ecosystem; plain SQLite is trivial to embed.
- Costs and risks: Electron's idle RAM and cold start miss the low-spec target (NFR-01); larger installer (NFR-03); plain SQLite does not encrypt at rest, failing PRD §9.6 and the DPA 2012 expectation.

### Option C: Tauri + system-linked SQLCipher

Same shell as the chosen option but link against a system-installed SQLCipher rather than bundling it.

- Benefits: smaller build-time dependency tree.
- Costs and risks: the barangay PC must carry a compatible SQLCipher DLL, contradicting the self-contained single-installer model (NFR-03, NFR-04) and inviting DLL-mismatch corruption.

### Option D (chosen): Tauri v2 + bundled SQLCipher + FTS5 + React/shadcn

- Benefits: meets every driver — offline and self-contained, encrypted at rest, single write path, low-spec friendly, FTS5 search, Argon2id + PBKDF2 + Ed25519 crypto profile.
- Costs and risks: bundled SQLCipher + FTS5 co-compilation is an unverified feasibility risk (retired by the stack spike ticket); Tauri v2 and the Rust crypto crates must be pinned at build time (deferred to the technical design).

## Consequences

### Positive

- Later tickets (crate topology, technical design, builds) inherit one fixed stack instead of open choices.
- The single-installer, offline, encrypted-at-rest, single-write-path properties are locked as the target architecture.
- The frontend framework (React + shadcn/ui) and password KDF (Argon2id) ambiguities are resolved.

### Negative

- Bundled SQLCipher adds build complexity and a compile-time dependency the project must own (mitigated by the spike, not eliminated).
- Deferring crate pinning means the technical design still carries that residual selection work before Phase 1 build.

### Neutral / Risks

- The stack's feasibility (SQLCipher + FTS5 static link, FTS latency, ≤30MB installer) remains unverified until the stack spike ticket passes; a failure triggers a revisit of this ADR rather than a workaround.

## Confirmation

- The stack spike ticket (issue #10) passes: bundled SQLCipher + FTS5 compile statically, FTS search is sub-200ms at 50k records, and the installer is ≤30MB (NFR-03).
- The Cargo workspace shows `app_core` with no Tauri, web, or database dependencies (HADR-0002/HADR-0003 conformance).
- No frontend code path queries the database directly; all access goes through the Tauri IPC → Rust backend boundary (NFR-05 review check).
- Password hashing uses Argon2id; the DB key derives via PBKDF2-HMAC-SHA256 excluding the Machine Hardware ID (code/dependency review).

## Relationships and References

- Retains [HADR-0002 (hexon architecture)](../../../../backend_arch_docs/adr/HADR-0002-adopt-rust-hexon-architecture.md) and [HADR-0003 (crate responsibilities)](../../../../backend_arch_docs/adr/HADR-0003-define-crate-and-module-responsibilities.md).
- Promotes [Core Engine Appendix A](../../../specs/core-engine/appendix-a-technical-architecture-direction.md) from informative to normative; refines [Appendix B](../../../specs/core-engine/appendix-b-key-derivation-and-recovery.md) key-derivation guidance to normative.
- Owning spec: [Core Engine PRD](../../../specs/core-engine/PRD.core-engine.md) (NFR-01…NFR-06, §9.1, §9.5, §9.6).
- Supporting issue: [Lock the application stack (Tauri + SQLCipher + crypto + React/shadcn)](https://github.com/Xaidel/brs/issues/2).
- Stack spike: [Run the Core Engine stack spike (bundled SQLCipher + FTS5, search perf, installer size)](https://github.com/Xaidel/brs/issues/10).
