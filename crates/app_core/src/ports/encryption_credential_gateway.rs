//! The `EncryptionCredentialGateway` outbound port (`tdd.phase-1` §8.4).

use async_trait::async_trait;

use crate::domain::value_objects::{DatabaseEncryptionKey, RecoveryCode};
use crate::ports::errors::CredentialError;

/// One cohesive, capability-shaped credential port (`infra_credentials`).
///
/// Deliberately not split into separate keyring/file/crypto ports: no
/// `app_core` caller ever needs those primitives independently (§4.4). The
/// adapter owns the OS keyring (`keyring`) for `system_secret`, `bootstrap.json`
/// file I/O, PBKDF2-HMAC-SHA256 derivation, AES-256-GCM wrap/unwrap, and secure
/// random generation.
#[async_trait]
pub trait EncryptionCredentialGateway: Send + Sync {
    /// First-run only: generates `system_secret`, salt, and a Recovery Code;
    /// persists the wrapped bootstrap state and the OS-keyring copy of
    /// `system_secret`; returns the `RecoveryCode` for one-time display.
    ///
    /// Safely re-invocable (overwrites state; nothing shown to the user yet).
    async fn establish(&self) -> Result<RecoveryCode, CredentialError>;

    /// Day-to-day: derives the SQLCipher key without the Recovery Code, using
    /// the keyring-stored `system_secret` and the `bootstrap.json` salt.
    async fn load_database_key(&self) -> Result<DatabaseEncryptionKey, CredentialError>;

    /// Recovery: unwraps `system_secret` from `bootstrap.json` using the given
    /// Recovery Code, best-effort re-stores it into the local keyring, and
    /// derives the SQLCipher key. A wrong-but-well-formed code returns
    /// `CredentialError::RecoveryCodeMismatch` (distinct from the domain-layer
    /// `RecoveryCodeError::InvalidChecksum` checked earlier at entry).
    async fn recover_database_key(
        &self,
        recovery_code: &RecoveryCode,
    ) -> Result<DatabaseEncryptionKey, CredentialError>;

    /// The absolute path to `bootstrap.json`, for `infra_backup`'s
    /// `BackupSnapshotWriter` to locate through the port that owns it (§4.7).
    /// Not `async` (a path query, like `Clock`).
    fn bootstrap_file_path(&self) -> String;
}
