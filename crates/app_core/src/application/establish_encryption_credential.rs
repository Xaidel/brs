//! `EstablishEncryptionCredentialUseCase` (`tdd.phase-1` §7.4, §4.6).
//!
//! `#![allow(dead_code)]`: the use case is unreachable from `app_core`'s
//! public surface until the assembly/composition gate (HADR-0007 gates 4–5);
//! until then it is exercised only by the Gate 2 tests below.

#![allow(dead_code)]

use std::sync::Arc;

use thiserror::Error;

use crate::domain::value_objects::RecoveryCode;
use crate::ports::{
    BackupDestination, BackupError, BackupSnapshotWriter, CredentialError,
    EncryptionCredentialGateway,
};

/// First-run credential establishment, gated on one written Backup Snapshot
/// (§4.6).
///
/// Sequence: `establish()` generates and persists `system_secret`, salt, and
/// the Recovery Code; `take_snapshot()` captures `bootstrap.json` and the
/// current database file into one encrypted archive. Only on success of both
/// does the Recovery Code reach the Secretary for one-time display.
pub(crate) struct EstablishEncryptionCredentialUseCase {
    encryption_credential_gateway: Arc<dyn EncryptionCredentialGateway>,
    backup_snapshot_writer: Arc<dyn BackupSnapshotWriter>,
}

impl EstablishEncryptionCredentialUseCase {
    /// Constructs the use case around the `infra_credentials` and
    /// `infra_backup` implementations.
    pub(crate) fn new(
        encryption_credential_gateway: Arc<dyn EncryptionCredentialGateway>,
        backup_snapshot_writer: Arc<dyn BackupSnapshotWriter>,
    ) -> Self {
        Self {
            encryption_credential_gateway,
            backup_snapshot_writer,
        }
    }

    /// Runs first-run setup and returns the Recovery Code for one-time display.
    ///
    /// # Errors
    ///
    /// [`EstablishEncryptionCredentialError::EstablishmentFailed`] when
    /// credential establishment fails;
    /// [`EstablishEncryptionCredentialError::InitialBackupFailed`] when the
    /// mandatory initial Backup Snapshot cannot be written — the Recovery Code
    /// is withheld and the caller must retry the whole use case (§4.6:
    /// `establish()` is safely re-invocable; nothing has been shown to the
    /// Secretary yet).
    pub(crate) async fn establish(
        &self,
    ) -> Result<RecoveryCode, EstablishEncryptionCredentialError> {
        let recovery_code = self.encryption_credential_gateway.establish().await?;
        self.backup_snapshot_writer
            .take_snapshot(BackupDestination::Default)
            .await?;
        Ok(recovery_code)
    }
}

/// Errors returned by [`EstablishEncryptionCredentialUseCase`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub(crate) enum EstablishEncryptionCredentialError {
    /// Credential establishment (`establish()`) failed; the safe variants of
    /// `CredentialError` are folded into one non-leaking case.
    #[error("encryption credentials could not be established")]
    EstablishmentFailed,
    /// The mandatory initial Backup Snapshot could not be written. No
    /// `RecoveryCode` is ever returned in this case (§4.6).
    #[error("initial backup snapshot failed; retry first-run setup")]
    InitialBackupFailed,
}

impl From<CredentialError> for EstablishEncryptionCredentialError {
    fn from(_: CredentialError) -> Self {
        Self::EstablishmentFailed
    }
}

impl From<BackupError> for EstablishEncryptionCredentialError {
    fn from(_: BackupError) -> Self {
        Self::InitialBackupFailed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::{
        FakeBackupSnapshotWriter, FakeEncryptionCredentialGateway, block_on, recovery_code,
    };

    #[test]
    fn returns_recovery_code_when_establish_and_snapshot_succeed() {
        let code = recovery_code();
        let gateway = Arc::new(FakeEncryptionCredentialGateway::with_establish_result(Ok(
            code.clone(),
        )));
        let writer = Arc::new(FakeBackupSnapshotWriter::succeeding());
        let use_case = EstablishEncryptionCredentialUseCase::new(gateway.clone(), writer.clone());
        assert_eq!(block_on(use_case.establish()).unwrap(), code);
        assert_eq!(gateway.establish_count(), 1);
        assert_eq!(writer.call_count(), 1);
        assert_eq!(
            writer.destinations(),
            vec![BackupDestination::Default],
            "Phase 1 snapshots go to the default backup directory (§4.6)"
        );
    }

    #[test]
    fn withholds_recovery_code_when_initial_snapshot_fails() {
        // §4.6: the Secretary never sees a Recovery Code unless the Backup
        // Snapshot was confirmed written. `establish()` succeeded and
        // `take_snapshot()` failed, so the assertion surface holds no
        // `RecoveryCode` at all.
        let gateway = Arc::new(FakeEncryptionCredentialGateway::with_establish_result(Ok(
            recovery_code(),
        )));
        let writer = Arc::new(FakeBackupSnapshotWriter::failing());
        let use_case = EstablishEncryptionCredentialUseCase::new(gateway.clone(), writer.clone());
        let err = block_on(use_case.establish()).unwrap_err();
        assert_eq!(err, EstablishEncryptionCredentialError::InitialBackupFailed);
        assert_eq!(gateway.establish_count(), 1);
        assert_eq!(writer.call_count(), 1);
    }

    #[test]
    fn establishment_failure_skips_the_snapshot_gate() {
        let gateway = Arc::new(FakeEncryptionCredentialGateway::with_establish_result(Err(
            CredentialError::KeyringUnavailable,
        )));
        let writer = Arc::new(FakeBackupSnapshotWriter::succeeding());
        let use_case = EstablishEncryptionCredentialUseCase::new(gateway.clone(), writer.clone());
        let err = block_on(use_case.establish()).unwrap_err();
        assert_eq!(err, EstablishEncryptionCredentialError::EstablishmentFailed);
        assert_eq!(
            writer.call_count(),
            0,
            "the snapshot gate runs only after establish() succeeds"
        );
    }
}
