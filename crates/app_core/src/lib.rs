//! Core Engine framework-free core.
//!
//! `app_core` owns the licensing and key-management domain (value objects, the
//! `LicenseGrant` entity, the `LicenseGrantActivated` event, and their typed errors)
//! and the outbound port contracts adapters must satisfy. It depends on no
//! workspace crate and performs no I/O, crypto, or persistence — the capabilities
//! cross the boundary through [`ports`].
//!
//! Only [`ports`] is public; `domain` and (in later gates) `application` stay
//! private per `backend_arch_docs/architecture.md`.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

mod domain;

pub mod ports;
