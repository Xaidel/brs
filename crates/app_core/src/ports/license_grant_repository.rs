//! The `LicenseGrantRepository` outbound port (`tdd.phase-1` §8.3).

use async_trait::async_trait;

use crate::ports::errors::RepositoryError;
use crate::ports::records::LicenseGrantRecord;

/// Persists and reads license state (`infra_persistence`).
///
/// `save` is insert-only: reissuance (Appendix C) always inserts a new row; no
/// port method updates or deletes an existing `LicenseGrant` row. `find_current`
/// returns the row with the highest `id` (UUIDv7 sorts chronologically), i.e.
/// the most recently activated grant. This is `infra_persistence`'s sole writer
/// for license state (NFR-05).
#[async_trait]
pub trait LicenseGrantRepository: Send + Sync {
    /// Appends a license grant record (insert-only).
    async fn save(&self, record: LicenseGrantRecord) -> Result<(), RepositoryError>;

    /// Returns the most recently activated grant, if any.
    async fn find_current(&self) -> Result<Option<LicenseGrantRecord>, RepositoryError>;
}
