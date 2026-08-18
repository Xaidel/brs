//! The `BackupSnapshotWriter` outbound port (`tdd.phase-1` §8.6).

use async_trait::async_trait;

use crate::ports::errors::BackupError;
use crate::ports::records::{BackupDestination, BackupSnapshotLocation};

/// Writes one encrypted Backup Snapshot archive (`infra_backup`).
///
/// The adapter writes an archive whose file manifest (§4.7) lists
/// `bootstrap.json` (located via the injected `EncryptionCredentialGateway`)
/// and the current SQLCipher database file (via `infra_persistence`'s
/// connection, per ADR-0002's sanctioned edge). No scheduling, retention,
/// purge, or query capability is defined here.
#[async_trait]
pub trait BackupSnapshotWriter: Send + Sync {
    /// Takes one encrypted snapshot at `destination`.
    async fn take_snapshot(
        &self,
        destination: BackupDestination,
    ) -> Result<BackupSnapshotLocation, BackupError>;
}
