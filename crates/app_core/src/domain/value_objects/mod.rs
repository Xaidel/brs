//! Value objects: validated, framework-free semantic values
//! (`tdd.phase-1` §6.1).

pub(crate) mod crockford;
pub(crate) mod database_encryption_key;
pub(crate) mod encryption_salt;
pub(crate) mod feature_flag;
pub(crate) mod license_grant_id;
pub(crate) mod license_key_payload;
pub(crate) mod license_signature;
pub(crate) mod machine_hardware_id;
pub(crate) mod recovery_code;
pub(crate) mod system_secret;
pub(crate) mod timestamp;

pub use database_encryption_key::DatabaseEncryptionKey;
pub use encryption_salt::EncryptionSalt;
pub use feature_flag::FeatureFlag;
pub use license_grant_id::LicenseGrantId;
pub use license_key_payload::LicenseKeyPayload;
pub use license_signature::LicenseSignature;
pub use machine_hardware_id::MachineHardwareId;
pub use recovery_code::RecoveryCode;
pub use system_secret::SystemSecret;
pub use timestamp::Timestamp;
