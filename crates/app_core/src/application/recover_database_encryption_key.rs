//! `RecoverDatabaseEncryptionKeyUseCase` (`tdd.phase-1` §7.5).
//!
//! `#![allow(dead_code)]`: the use case is unreachable from `app_core`'s
//! public surface until the assembly/composition gate (HADR-0007 gates 4–5);
//! until then it is exercised only by the Gate 2 tests below.

#![allow(dead_code)]

use std::sync::Arc;

use thiserror::Error;

use crate::domain::value_objects::{DatabaseEncryptionKey, RecoveryCode};
use crate::ports::{CredentialError, EncryptionCredentialGateway};

/// Recovers the SQLCipher `DatabaseEncryptionKey` from a Recovery Code.
///
/// The input is already checksum-validated by [`RecoveryCode`]'s smart
/// constructor (§6.1), so a mistyped code never reaches this use case; a
/// wrong-but-well-formed code fails at the AES-GCM unwrap inside
/// `infra_credentials` and surfaces as
/// [`RecoverDatabaseEncryptionKeyError::RecoveryCodeMismatch`].
pub(crate) struct RecoverDatabaseEncryptionKeyUseCase {
    encryption_credential_gateway: Arc<dyn EncryptionCredentialGateway>,
}

impl RecoverDatabaseEncryptionKeyUseCase {
    /// Constructs the use case around the `infra_credentials` implementation.
    pub(crate) fn new(encryption_credential_gateway: Arc<dyn EncryptionCredentialGateway>) -> Self {
        Self {
            encryption_credential_gateway,
        }
    }

    /// Recovers the `DatabaseEncryptionKey` for the composition root (not a
    /// UI-facing value, §9.5).
    ///
    /// # Errors
    ///
    /// [`RecoverDatabaseEncryptionKeyError`] — the safe variants mirror the
    /// port's classification: a wrong-but-well-formed code, a missing/corrupt
    /// `bootstrap.json`, or an unavailable OS credential store.
    pub(crate) async fn recover_database_key(
        &self,
        recovery_code: &RecoveryCode,
    ) -> Result<DatabaseEncryptionKey, RecoverDatabaseEncryptionKeyError> {
        Ok(self
            .encryption_credential_gateway
            .recover_database_key(recovery_code)
            .await?)
    }
}

/// Errors returned by [`RecoverDatabaseEncryptionKeyUseCase`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub(crate) enum RecoverDatabaseEncryptionKeyError {
    /// The OS credential store is unavailable (e.g. OS reinstall without a
    /// Recovery Code recovery).
    #[error("OS credential store is unavailable")]
    KeyringUnavailable,
    /// `bootstrap.json` is missing or corrupt.
    #[error("bootstrap state is missing or corrupt")]
    BootstrapUnavailable,
    /// A well-formed Recovery Code failed to unwrap the stored `system_secret`
    /// (wrong-but-well-formed code).
    #[error("recovery code does not match the stored bootstrap state")]
    RecoveryCodeMismatch,
}

impl From<CredentialError> for RecoverDatabaseEncryptionKeyError {
    fn from(err: CredentialError) -> Self {
        match err {
            CredentialError::KeyringUnavailable => Self::KeyringUnavailable,
            CredentialError::BootstrapUnavailable => Self::BootstrapUnavailable,
            CredentialError::RecoveryCodeMismatch => Self::RecoveryCodeMismatch,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::{
        FakeEncryptionCredentialGateway, block_on, database_key, recovery_code,
    };

    #[test]
    fn returns_recovered_database_key() {
        let key = database_key();
        let gateway = Arc::new(FakeEncryptionCredentialGateway::with_recover_result(Ok(
            key,
        )));
        let use_case = RecoverDatabaseEncryptionKeyUseCase::new(gateway.clone());
        let code = recovery_code();
        assert_eq!(block_on(use_case.recover_database_key(&code)).unwrap(), key);
        assert_eq!(
            gateway.received_recovery_code().as_deref(),
            Some(code.as_str()),
            "the transcribed code reaches the gateway unchanged"
        );
    }

    #[test]
    fn wrong_but_well_formed_code_maps_to_recovery_code_mismatch() {
        let gateway = Arc::new(FakeEncryptionCredentialGateway::with_recover_result(Err(
            CredentialError::RecoveryCodeMismatch,
        )));
        let use_case = RecoverDatabaseEncryptionKeyUseCase::new(gateway);
        let err = block_on(use_case.recover_database_key(&recovery_code())).unwrap_err();
        assert_eq!(err, RecoverDatabaseEncryptionKeyError::RecoveryCodeMismatch);
    }

    #[test]
    fn missing_or_corrupt_bootstrap_maps_to_bootstrap_unavailable() {
        let gateway = Arc::new(FakeEncryptionCredentialGateway::with_recover_result(Err(
            CredentialError::BootstrapUnavailable,
        )));
        let use_case = RecoverDatabaseEncryptionKeyUseCase::new(gateway);
        let err = block_on(use_case.recover_database_key(&recovery_code())).unwrap_err();
        assert_eq!(err, RecoverDatabaseEncryptionKeyError::BootstrapUnavailable);
    }

    #[test]
    fn keyring_failure_maps_to_safe_variant() {
        let gateway = Arc::new(FakeEncryptionCredentialGateway::with_recover_result(Err(
            CredentialError::KeyringUnavailable,
        )));
        let use_case = RecoverDatabaseEncryptionKeyUseCase::new(gateway);
        let err = block_on(use_case.recover_database_key(&recovery_code())).unwrap_err();
        assert_eq!(err, RecoverDatabaseEncryptionKeyError::KeyringUnavailable);
    }
}
