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