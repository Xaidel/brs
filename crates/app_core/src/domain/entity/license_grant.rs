//! The `LicenseGrant` entity (PRD §11, `tdd.phase-1` §6.2).

use crate::domain::errors::LicenseValidationError;
use crate::domain::value_objects::{
    FeatureFlag, LicenseGrantId, LicenseKeyPayload, LicenseSignature, MachineHardwareId, Timestamp,
};

/// The persisted record of a validated License Key's effect on an installation:
/// which Feature Flags it unlocked, for which Machine Hardware ID, with what
/// signature metadata.
///
/// Construction invariant: the only public constructor is
/// [`LicenseGrant::activate`], which enforces that the payload's bound Machine
/// Hardware ID equals the local installation's. "A `LicenseGrant` exists" and
/// "its bound machine ID matches this installation" are therefore the same fact
/// by construction, satisfying PRD §9.5 bullet 3's ordering requirement without
/// requiring domain to hold crypto material — signature verification happens
/// earlier, in the application layer behind `LicenseSignatureVerifier` (§4.4).
///
/// The entity is not yet constructed by `app_core`'s lib code in this slice —
/// its application-layer use cases (`ActivateLicenseUseCase` et al.) are Gate 2
/// per the TDD's implementation plan (§13). It is domain-tested here, per
/// HADR-0006 Gate 1.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LicenseGrant {
    id: LicenseGrantId,
    machine_hardware_id: MachineHardwareId,
    feature_flags: Vec<FeatureFlag>,
    signature: LicenseSignature,
    activated_at: Timestamp,
}

#[allow(dead_code)]
impl LicenseGrant {
    /// Activates a license: constructs the grant iff the payload is bound to the
    /// local machine.
    ///
    /// Signature verification is assumed to have already happened in the
    /// application layer via `LicenseSignatureVerifier` before this call; this
    /// constructor only enforces the machine-binding invariant.
    ///
    /// # Errors
    ///
    /// `LicenseValidationError::MachineHardwareMismatch { current_machine_hardware_id }`
    /// when the payload's bound Machine Hardware ID disagrees with
    /// `local_machine_id`.
    pub fn activate(
        payload: LicenseKeyPayload,
        local_machine_id: &MachineHardwareId,
        id: LicenseGrantId,
        activated_at: Timestamp,
    ) -> Result<Self, LicenseValidationError> {
        let (machine_hardware_id, feature_flags, signature) = payload.into_parts();
        if machine_hardware_id != *local_machine_id {
            return Err(LicenseValidationError::MachineHardwareMismatch {
                current_machine_hardware_id: local_machine_id.clone(),
            });
        }
        Ok(Self {
            id,
            machine_hardware_id,
            feature_flags,
            signature,
            activated_at,
        })
    }

    /// The grant's record identifier (UUIDv7).
    pub const fn id(&self) -> LicenseGrantId {
        self.id
    }

    /// The Machine Hardware ID this grant is bound to.
    pub const fn machine_hardware_id(&self) -> &MachineHardwareId {
        &self.machine_hardware_id
    }

    /// The granted Feature Flags.
    pub fn feature_flags(&self) -> &[FeatureFlag] {
        &self.feature_flags
    }

    /// The retained signature (for audit/support inspection).
    pub const fn signature(&self) -> &LicenseSignature {
        &self.signature
    }

    /// When the grant was activated.
    pub const fn activated_at(&self) -> Timestamp {
        self.activated_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::test_support::grouped;
    use base64::Engine as _;
    use chrono::{TimeZone, Utc};

    fn machine_id() -> MachineHardwareId {
        MachineHardwareId::parse(&grouped(13)).unwrap()
    }

    fn payload_for(mid: &MachineHardwareId) -> LicenseKeyPayload {
        let obj = serde_json::json!({
            "machine_hardware_id": mid.as_str(),
            "feature_flags": ["KP_BLOTTER", "TREASURY"],
            "signature": base64::engine::general_purpose::STANDARD.encode([7u8; 64]),
        });
        let raw = base64::engine::general_purpose::STANDARD.encode(obj.to_string().as_bytes());
        LicenseKeyPayload::parse(&raw).unwrap()
    }

    fn timestamp() -> Timestamp {
        Timestamp::new(Utc.timestamp_opt(1_700_000_000, 0).unwrap())
    }

    #[test]
    fn activate_succeeds_when_machine_id_matches() {
        let local = machine_id();
        let grant = LicenseGrant::activate(
            payload_for(&local),
            &local,
            LicenseGrantId::generate(),
            timestamp(),
        )
        .unwrap();
        assert_eq!(grant.machine_hardware_id(), &local);
        assert_eq!(
            grant.feature_flags(),
            &[FeatureFlag::KpBlotter, FeatureFlag::Treasury]
        );
        assert_eq!(grant.signature().as_bytes(), &[7u8; 64]);
        assert_eq!(grant.activated_at(), timestamp());
    }

    #[test]
    fn activate_rejects_mismatched_machine_id() {
        let local = machine_id();
        let other =
            MachineHardwareId::parse(&grouped(13).replace('A', "B").replace('B', "C")).unwrap();
        assert_ne!(other, local);
        let err = LicenseGrant::activate(
            payload_for(&other),
            &local,
            LicenseGrantId::generate(),
            timestamp(),
        )
        .unwrap_err();
        assert_eq!(
            err,
            LicenseValidationError::MachineHardwareMismatch {
                current_machine_hardware_id: local.clone(),
            }
        );
    }
}
