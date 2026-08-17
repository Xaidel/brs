# Appendix A

## Sync-Ready Schema & Municipal Integration Concepts

Back to the owning PRD: [Master PRD](PRD.md)

The owning PRD remains authoritative for product scope. This appendix clarifies supporting detail and must not broaden or contradict that scope. Municipal aggregation itself remains Exploratory and unscheduled; nothing here authorizes building it.

---

## 1. Purpose

Explain why every current and future BMS package must reuse the same identifier and change-tracking conventions, and describe the conceptual (non-committed) municipal integration model those conventions exist to support.

## 2. Why This Matters Now, Even Though Sync Is Exploratory

It is currently unknown whether any Municipality will mandate or adopt BMS across its constituent barangays. Regardless, every barangay installation runs as a fully standalone unit with zero runtime dependency on a municipal system. The reason to standardize identifiers and change-tracking columns today, before any sync capability is built, is that retrofitting them onto an already-populated production database (real resident records, in real barangays) is far more disruptive than including them from each package's first schema migration.

## 3. Shared Schema Conventions

Every product-owned table, in every package, should include:

- **`id`** — a UUIDv4 (or ULID) string primary key, not a local auto-incrementing integer. This guarantees that if many barangays run independent installations for years, their records never collide if ever imported into a shared destination.
- **`barangay_code`** — the Philippine Standard Geographic Code (PSGC) identifying which barangay owns the record.
- **`updated_at`** — an ISO-8601 UTC timestamp of last modification.
- **`sync_status`** — `PENDING` or `SYNCED`, tracking whether a record has been included in an export package.

## 4. Conceptual Municipal Hub Model (Informative Only — Not Committed)

If a Municipality ever adopts BMS, the intended shape (subject to its own future PRD before any implementation) is a two-tier model:

```
+------------------------------------+           +------------------------------------+
|         BARANGAY POBLACION         |           |         BARANGAY SAN JOSE          |
|    (Full Data Owner & Creator)     |           |    (Full Data Owner & Creator)     |
+-----------------+------------------+           +-----------------+------------------+
                  |                                                |
                  | Encrypted Delta Exports                        | Encrypted Delta Exports
                  | (.bmssync via USB/Email)                       | (.bmssync via USB/Email)
                  v                                                v
    +--------------------------------------------------------------------+
    |                     MUNICIPAL HUB AGGREGATOR                        |
    |                                                                      |
    |  • Reads imported .bmssync files into a mirrored database            |
    |  • Municipal 'residents' table is READ-ONLY (no direct creation)     |
    |  • Municipal services (e.g., MSWDO/Ayuda) link to Resident UUID      |
    +--------------------------------------------------------------------+
```

Each barangay would remain the sole authoritative owner of its demographic data. A `.bmssync` package would be a compressed, digitally signed, encrypted export of records where `sync_status = PENDING`, transported out-of-band (USB drive, email) — never a live network connection. The Municipal Hub's mirrored copy would be strictly read-only; only the originating barangay's own installation could ever create or edit that data.

## 5. Usage Guidance

Downstream focused PRDs and TDDs should cite this appendix when defining a table's conceptual data model, rather than re-explaining the rationale each time. If a future Municipal Sync PRD is written, it supersedes the informative model described here and becomes the normative source for that capability.
