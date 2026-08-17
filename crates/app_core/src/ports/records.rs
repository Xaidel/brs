//! Port-owned boundary records and DTOs (`tdd.phase-1` §8, HADR-0005).
//!
//! Persistence and adapters exchange these complete state records, never the
//! private `LicenseGrant` entity.

/// The persisted shape of a `LicenseGrant` as it crosses the
/// `LicenseGrantRepository` port.
///
/// Mapped to/from the private entity by the application layer, never exposed as
/// the entity itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LicenseGrantRecord {
    /// The grant's identifier (UUIDv7, hyphenated string form).
    pub id: String,
    /// The bound Machine Hardware ID's canonical grouped string.
    pub machine_hardware_id: String,
    /// The granted Feature Flags as SCREAMING_SNAKE_CASE strings (ADR-0003).
    pub feature_flags: Vec<String>,
    /// The Ed25519 signature, Base64-encoded.
    pub signature: String,
    /// When the grant was activated (ISO-8601 UTC).
    pub activated_at: String,
}

/// Where a Backup Snapshot archive is written (`tdd.phase-1` §8.6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackupDestination {
    /// The default backup directory (`%APPDATA%\BarangayMS\backups\`); the
    /// adapter resolves the concrete path.
    Default,
    /// An explicit path (Phase 3's manual USB export).
    Explicit(String),
}

/// Where a Backup Snapshot was written (`tdd.phase-1` §8.6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupSnapshotLocation {
    /// Absolute path to the archive.
    pub path: String,
    /// When the snapshot was taken (ISO-8601 UTC).
    pub created_at: String,
}
