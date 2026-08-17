//! The `LicenseGrantActivated` domain event (`tdd.phase-1` §6.3).

use crate::domain::value_objects::{FeatureFlag, LicenseGrantId, MachineHardwareId, Timestamp};

/// A License Key was validated, machine-bound, and persisted.
///
/// Invariant: carries no `SystemSecret`, `RecoveryCode`, or
/// `DatabaseEncryptionKey` — only non-secret identifiers, the granted flags, and
/// a timestamp. No consumer exists in Phase 1; the application layer drains
/// (discards) it for now (Gate 2 per the TDD's implementation plan §13).
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LicenseGrantActivated {
    /// The persisted grant's identifier.
    pub license_grant_id: LicenseGrantId,
    /// The Machine Hardware ID the grant is bound to.
    pub machine_hardware_id: MachineHardwareId,
    /// The granted Feature Flags.
    pub feature_flags: Vec<FeatureFlag>,
    /// When the grant was activated.
    pub activated_at: Timestamp,
}

#[allow(dead_code)]
impl LicenseGrantActivated {
    /// Records the activation of a grant.
    pub fn new(
        license_grant_id: LicenseGrantId,
        machine_hardware_id: MachineHardwareId,
        feature_flags: Vec<FeatureFlag>,
        activated_at: Timestamp,
    ) -> Self {
        Self {
            license_grant_id,
            machine_hardware_id,
            feature_flags,
            activated_at,
        }
    }
}
