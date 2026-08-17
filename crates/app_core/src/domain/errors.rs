//! Typed domain errors (`tdd.phase-1` §6.4).
//!
//! The two named errors are `LicenseValidationError` and `RecoveryCodeError`.
//! `MachineHardwareIdError` is added for the `MachineHardwareId` smart
//! constructor's format rejection; without it the value object cannot fail
//! construction, and `LicenseKeyPayload::parse` needs a distinct source to map
//! into `MalformedLicenseKey`.

use crate::domain::value_objects::MachineHardwareId;
use thiserror::Error;

/// Errors validating a submitted License Key or constructing a [`LicenseGrant`].
///
/// [`LicenseGrant`]: crate::domain::entity::license_grant::LicenseGrant
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LicenseValidationError {
    /// The License Key string is not a decodable, well-formed
    /// [`LicenseKeyPayload`].
    ///
    /// [`LicenseKeyPayload`]: crate::domain::value_objects::license_key_payload::LicenseKeyPayload
    #[error("license key is malformed or undecodable")]
    MalformedLicenseKey,

    /// The payload's bound machine hardware id disagrees with this installation.
    #[error("machine hardware id mismatch: current machine is {current_machine_hardware_id}")]
    MachineHardwareMismatch {
        /// The local installation's machine hardware id.
        current_machine_hardware_id: MachineHardwareId,
    },
}

/// Errors validating a [`RecoveryCode`] smart-constructor input.
///
/// [`RecoveryCode`]: crate::domain::value_objects::recovery_code::RecoveryCode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RecoveryCodeError {
    /// The input was a well-formed Crockford Base32 code whose checksum did
    /// not match its data symbols.
    #[error("recovery code checksum is invalid")]
    InvalidChecksum,

    /// The input was not 28 data symbols plus one check symbol, contained a
    /// data symbol outside the Crockford Base32 alphabet, or a check symbol
    /// outside the 37-symbol Crockford check alphabet.
    #[error("recovery code format is malformed")]
    MalformedFormat,
}

/// Errors rejecting a [`MachineHardwareId`] construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum MachineHardwareIdError {
    /// The input was not exactly 52 Crockford Base32 data characters (13 groups
    /// of 4).
    #[error("machine hardware id must be 52 Crockford Base32 characters in 13 groups of 4")]
    InvalidFormat,
}
