//! Core Engine framework-free core.
//!
//! `app_core` owns the licensing and key-management domain (value objects, the
//! `LicenseGrant` entity, the `LicenseGrantActivated` event, and their typed
//! errors), the Phase 1 application layer (the five named use cases), and the
//! outbound port contracts adapters must satisfy. It depends on no workspace
//! crate and performs no crypto, persistence, or adapter I/O — those
//! capabilities cross the boundary through [`ports`].
//!
//! Only [`ports`] is public; `domain` and `application` stay private per
//! `backend_arch_docs/architecture.md` (a public assembly API is a future
//! gate).
//!
//! The one deliberate exception to "no wall-clock reads": the application
//! layer generates record identifiers directly via `Uuid::now_v7()`
//! (`application/id.rs`, follow-up Xaidel/brs#21); domain timestamps are still
//! sourced only through the `Clock` port.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

mod application;
mod domain;

pub mod ports;
