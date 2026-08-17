# Dependency Rules

The workspace manifests permit this graph:

```text
app_core -> no workspace crates
infra_*  -> app_core
```

The reference contains `crates/app_core` and `crates/infra_local`. A shared adapter
crate requires demonstrated reuse and an accepted HADR.

| Importer | Allowed project dependencies | Enforcement |
| --- | --- | --- |
| Domain value objects | none | review and module privacy |
| Domain events | value objects | review and module privacy |
| Domain entities | value objects and events | review and module privacy |
| Domain services | entities and value objects | review and module privacy |
| Ports | public boundary values and errors | review and module privacy |
| Application | domain and ports | review and module privacy |
| Core assembly | application and ports | review and module privacy |
| `infra_*` | public `app_core` API | Cargo and Rust privacy |

Domain modules perform no I/O and never depend on application or infrastructure.
Application coordinates through ports without knowing transport or persistence.
Deployment code cannot access private core modules. Tests may span public boundaries
for integration coverage without changing the production graph.

A new workspace crate, shared adapter, public core boundary, or dependency edge
requires a HADR and synchronized updates to `AGENTS.md` and this document. Do not add
a partial dependency scanner and present it as enforcement of semantic ownership.