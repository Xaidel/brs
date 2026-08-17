# Naming Conventions

- Workspace crate names use snake_case in Cargo, directories, and Rust imports.
- The framework-free crate is `app_core`; deployment crates use `infra_{target}`.
- Types and traits use PascalCase; modules, functions, fields, and variables use snake_case.
- Name inbound traits after capabilities, such as `CreateNote` and `ListNotes`.
- Name concrete orchestration types with the `UseCase` suffix.
- Name adapters by technology and responsibility, such as `InMemoryNoteRepository`.
- Name domain events as past-tense business facts rather than CRUD notifications.
- Name domain services after their business rule; avoid generic manager containers.
- Keep transport DTO names private. Ports own operation-level commands and results.
- Outbound-port traits normally end in an `{Noun}`-shaped suffix (`{Noun}Repository` / `{Noun}Source` / `{Noun}Gateway`). Accepted capability-shaped exceptions, mandated by `tdd.phase-1` §8 and not to be re-litigated by later gates: `LicenseSignatureVerifier`, `BackupSnapshotWriter`, and `Clock`.