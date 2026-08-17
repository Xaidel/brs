//! Port contracts and boundary values (`tdd.phase-1` §8).
//!
//! This is the crate's only public module and the supported external core API
//! (`backend_arch_docs/architecture.md`). It re-exports the domain value objects
//! and errors that cross the boundary, and defines the outbound port traits,
//! port-owned records, and stable port error classifications that
//! `infra_hardware_id`, `infra_licensing`, `infra_credentials`,
//! `infra_persistence`, and `infra_backup` must satisfy.

mod backup_snapshot_writer;
mod clock;
mod encryption_credential_gateway;
mod errors;
mod license_grant_repository;
mod license_signature_verifier;
mod machine_hardware_id_source;
mod records;

pub use crate::domain::errors::{
    LicenseValidationError, MachineHardwareIdError, RecoveryCodeError,
};
pub use crate::domain::value_objects::{
    DatabaseEncryptionKey, EncryptionSalt, FeatureFlag, LicenseGrantId, LicenseKeyPayload,
    LicenseSignature, MachineHardwareId, RecoveryCode, SystemSecret, Timestamp,
};
pub use backup_snapshot_writer::BackupSnapshotWriter;
pub use clock::Clock;
pub use encryption_credential_gateway::EncryptionCredentialGateway;
pub use errors::{
    BackupError, CredentialError, HardwareIdError, RepositoryError, SignatureVerificationError,
};
pub use license_grant_repository::LicenseGrantRepository;
pub use license_signature_verifier::LicenseSignatureVerifier;
pub use machine_hardware_id_source::MachineHardwareIdSource;
pub use records::{BackupDestination, BackupSnapshotLocation, LicenseGrantRecord};
