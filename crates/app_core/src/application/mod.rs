//! Application layer: named use cases orchestrating domain behavior and
//! outbound port calls (`tdd.phase-1` §7).
//!
//! Stays private per `backend_arch_docs/architecture.md` — only
//! [`crate::ports`] is public, and a public assembly API is a future gate (the
//! composition root lives in `src-tauri`). The five Phase 1 use cases are
//! [`GetMachineHardwareIdUseCase`], [`ActivateLicenseUseCase`],
//! [`GetActiveLicenseUseCase`], [`EstablishEncryptionCredentialUseCase`], and
//! [`RecoverDatabaseEncryptionKeyUseCase`]. Day-to-day key loading (§7.6) is
//! deliberately not a use case — it is composition-root plumbing.
//!
//! [`ActivateLicenseUseCase`]: crate::application::activate_license::ActivateLicenseUseCase
//! [`EstablishEncryptionCredentialUseCase`]: crate::application::establish_encryption_credential::EstablishEncryptionCredentialUseCase
//! [`GetActiveLicenseUseCase`]: crate::application::get_active_license::GetActiveLicenseUseCase
//! [`GetMachineHardwareIdUseCase`]: crate::application::get_machine_hardware_id::GetMachineHardwareIdUseCase
//! [`RecoverDatabaseEncryptionKeyUseCase`]: crate::application::recover_database_encryption_key::RecoverDatabaseEncryptionKeyUseCase

pub(crate) mod activate_license;
pub(crate) mod establish_encryption_credential;
pub(crate) mod get_active_license;
pub(crate) mod get_machine_hardware_id;
pub(crate) mod id;
pub(crate) mod recover_database_encryption_key;
#[cfg(test)]
pub(crate) mod test_support;
