//! Stable, non-leaking error classifications returned by outbound port calls.
//!
//! Adapters translate their own failure modes into these safe variants; they
//! must never leak internal error details across the boundary (HADR-0005's
//! boundary policy).

use thiserror::Error;

/// Failures computing the Machine Hardware ID (`tdd.phase-1` §8.1).
///
/// Implemented by `infra_hardware_id`; covers WMI query failure (e.g. an
/// unsupported virtualization environment) as a single safe variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum HardwareIdError {
    /// The Machine Hardware ID could not be computed in this environment.
    #[error("machine hardware id is unavailable in this environment")]
    Unavailable,
}

/// Failures verifying a License Key signature (`tdd.phase-1` §8.2).
///
/// Implemented by `infra_licensing` against its embedded Ed25519 public key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SignatureVerificationError {
    /// The signature did not verify against the embedded public key.
    #[error("license key signature is invalid")]
    InvalidSignature,
}

/// Failures persisting or reading license state (`tdd.phase-1` §8.3).
///
/// Implemented by `infra_persistence`. `save` is insert-only — no method
/// updates or deletes an existing `LicenseGrant` row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RepositoryError {
    /// The license grant store could not be read or written.
    #[error("license grant store is unavailable")]
    Unavailable,
}

/// Failures of the encryption-credential capability (`tdd.phase-1` §8.4).
///
/// Implemented by `infra_credentials`: OS keyring, `bootstrap.json` file I/O,
/// PBKDF2 derivation, AES-256-GCM wrap/unwrap, and secure random generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CredentialError {
    /// The OS credential store is unavailable or has no entry for this
    /// installation (e.g. OS reinstall without a Recovery Code recovery).
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

/// Failures writing a Backup Snapshot (`tdd.phase-1` §8.6).
///
/// Implemented by `infra_backup`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum BackupError {
    /// The snapshot archive could not be written.
    #[error("backup snapshot could not be written")]
    WriteFailed,
}
