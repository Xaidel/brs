//! The parsed, pre-verification structure of a submitted License Key string
//! (`tdd.phase-1` §6.1, §11).

use base64::Engine as _;

use super::feature_flag::FeatureFlag;
use super::license_signature::LicenseSignature;
use super::machine_hardware_id::MachineHardwareId;
use crate::domain::errors::LicenseValidationError;

/// Smart constructor; malformed input fails before any port call is attempted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LicenseKeyPayload {
    machine_hardware_id: MachineHardwareId,
    feature_flags: Vec<FeatureFlag>,
    signature: LicenseSignature,
}

/// The wire envelope (`tdd.phase-1` §11): whole-object Base64 of this JSON.
#[derive(serde::Deserialize)]
struct LicenseKeyEnvelope {
    machine_hardware_id: String,
    feature_flags: Vec<FeatureFlag>,
    signature: String,
}

impl LicenseKeyPayload {
    /// Parses the whole-object Base64 License Key envelope (§11).
    ///
    /// Fails fast with `LicenseValidationError::MalformedLicenseKey` on bad
    /// Base64, non-JSON content, an unrecognized feature-flag string, a
    /// malformed machine hardware id, or a signature that is not exactly
    /// 64 bytes after decoding.
    pub fn parse(raw_key: &str) -> Result<Self, LicenseValidationError> {
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(raw_key.trim())
            .map_err(|_| LicenseValidationError::MalformedLicenseKey)?;
        let envelope: LicenseKeyEnvelope = serde_json::from_slice(&decoded)
            .map_err(|_| LicenseValidationError::MalformedLicenseKey)?;
        let machine_hardware_id = MachineHardwareId::parse(&envelope.machine_hardware_id)
            .map_err(|_| LicenseValidationError::MalformedLicenseKey)?;
        let signature_bytes = base64::engine::general_purpose::STANDARD
            .decode(envelope.signature.as_bytes())
            .map_err(|_| LicenseValidationError::MalformedLicenseKey)?;
        let signature: [u8; LicenseSignature::LENGTH] = signature_bytes
            .try_into()
            .map_err(|_| LicenseValidationError::MalformedLicenseKey)?;
        Ok(Self {
            machine_hardware_id,
            feature_flags: envelope.feature_flags,
            signature: LicenseSignature::from_bytes(signature),
        })
    }

    /// The payload's bound Machine Hardware ID.
    pub fn machine_hardware_id(&self) -> &MachineHardwareId {
        &self.machine_hardware_id
    }

    /// The granted Feature Flags, in payload order.
    pub fn feature_flags(&self) -> &[FeatureFlag] {
        &self.feature_flags
    }

    /// The Ed25519 signature over the canonical pre-signature payload (§11).
    pub fn signature(&self) -> &LicenseSignature {
        &self.signature
    }

    /// Consumes the payload into its three constituent values, for the
    /// application/entity layer to move into a persisted `LicenseGrant`.
    #[allow(clippy::type_complexity)]
    pub fn into_parts(self) -> (MachineHardwareId, Vec<FeatureFlag>, LicenseSignature) {
        (self.machine_hardware_id, self.feature_flags, self.signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn machine_id() -> String {
        "4ZQK-4ZQK-4ZQK-4ZQK-4ZQK-4ZQK-4ZQK-4ZQK-4ZQK-4ZQK-4ZQK-4ZQK-4ZQK".to_string()
    }

    fn envelope(machine_hardware_id: &str, flags: &[&str], signature: &[u8]) -> String {
        let obj = json!({
            "machine_hardware_id": machine_hardware_id,
            "feature_flags": flags,
            "signature": base64::engine::general_purpose::STANDARD.encode(signature),
        });
        base64::engine::general_purpose::STANDARD.encode(obj.to_string().as_bytes())
    }

    #[test]
    fn parses_valid_envelope() {
        let signature = [7u8; 64];
        let raw = envelope(&machine_id(), &["KP_BLOTTER", "TREASURY"], &signature);
        let payload = LicenseKeyPayload::parse(&raw).unwrap();
        assert_eq!(payload.machine_hardware_id().as_str(), machine_id());
        assert_eq!(
            payload.feature_flags(),
            &[FeatureFlag::KpBlotter, FeatureFlag::Treasury]
        );
        assert_eq!(payload.signature().as_bytes(), &signature);
    }

    #[test]
    fn rejects_bad_base64() {
        let err = LicenseKeyPayload::parse("!!!not base64!!!").unwrap_err();
        assert_eq!(err, LicenseValidationError::MalformedLicenseKey);
    }

    #[test]
    fn rejects_non_json_content() {
        let raw = base64::engine::general_purpose::STANDARD.encode(b"not json");
        let err = LicenseKeyPayload::parse(&raw).unwrap_err();
        assert_eq!(err, LicenseValidationError::MalformedLicenseKey);
    }

    #[test]
    fn rejects_unrecognized_feature_flag() {
        let raw = envelope(&machine_id(), &["BOGUS"], &[7u8; 64]);
        let err = LicenseKeyPayload::parse(&raw).unwrap_err();
        assert_eq!(err, LicenseValidationError::MalformedLicenseKey);
    }

    #[test]
    fn rejects_wrong_length_signature() {
        let raw = envelope(&machine_id(), &["KP_BLOTTER"], &[7u8; 32]);
        let err = LicenseKeyPayload::parse(&raw).unwrap_err();
        assert_eq!(err, LicenseValidationError::MalformedLicenseKey);
    }

    #[test]
    fn rejects_malformed_machine_id() {
        let raw = envelope("BAD", &["KP_BLOTTER"], &[7u8; 64]);
        let err = LicenseKeyPayload::parse(&raw).unwrap_err();
        assert_eq!(err, LicenseValidationError::MalformedLicenseKey);
    }
}
