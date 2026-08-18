//! `ActivateLicenseUseCase` (`tdd.phase-1` §7.2).

use std::sync::Arc;

use base64::Engine as _;
use thiserror::Error;

use crate::application::id::new_license_grant_id;
use crate::domain::entity::license_grant::LicenseGrant;
use crate::domain::errors::LicenseValidationError;
use crate::domain::events::license_grant_activated::LicenseGrantActivated;
use crate::domain::value_objects::{FeatureFlag, LicenseKeyPayload, MachineHardwareId};
use crate::ports::{
    Clock, HardwareIdError, LicenseGrantRecord, LicenseGrantRepository, LicenseSignatureVerifier,
    MachineHardwareIdSource, RepositoryError, SignatureVerificationError,
};

/// Activates a License Key pasted/uploaded by the Secretary: parse, read the
/// local Machine Hardware ID, verify the signature, construct and persist the
/// `LicenseGrant`, and return the granted Feature Flags.
///
/// Signature verification happens here — an application-layer port call, not
/// domain (§4.4) — before the grant is ever constructed, so a forged or
/// tampered key can never leave a `LicenseGrant` behind.
///
/// `#[allow(dead_code)]`: unreachable from `app_core`'s public surface until
/// the assembly/composition gate (HADR-0007 gates 4–5); exercised only by the
/// Gate 2 tests until then.
#[allow(dead_code)]
pub(crate) struct ActivateLicenseUseCase {
    machine_hardware_id_source: Arc<dyn MachineHardwareIdSource>,
    signature_verifier: Arc<dyn LicenseSignatureVerifier>,
    license_grant_repository: Arc<dyn LicenseGrantRepository>,
    clock: Arc<dyn Clock>,
}

/// Same `#[allow(dead_code)]` as the struct: unreachable until the
/// assembly/composition gate; exercised by the Gate 2 tests.
#[allow(dead_code)]
impl ActivateLicenseUseCase {
    /// Constructs the use case around its four outbound ports.
    pub(crate) fn new(
        machine_hardware_id_source: Arc<dyn MachineHardwareIdSource>,
        signature_verifier: Arc<dyn LicenseSignatureVerifier>,
        license_grant_repository: Arc<dyn LicenseGrantRepository>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            machine_hardware_id_source,
            signature_verifier,
            license_grant_repository,
            clock,
        }
    }

    /// Activates the raw License Key string and returns the granted flags.
    ///
    /// Sequence (§7.2): parse the envelope; read the local Machine Hardware ID;
    /// verify the signature; construct the `LicenseGrant` (machine-binding
    /// invariant); persist it insert-only; drain the `LicenseGrantActivated`
    /// event (§6.3 — no consumer exists in Phase 1).
    ///
    /// # Errors
    ///
    /// [`ActivateLicenseError`] — see its variants. `MachineHardwareMismatch`
    /// carries `current_machine_hardware_id` so the caller can render Appendix
    /// C's "Hardware Change Detected" message.
    pub(crate) async fn activate(
        &self,
        raw_key: &str,
    ) -> Result<Vec<FeatureFlag>, ActivateLicenseError> {
        let payload = LicenseKeyPayload::parse(raw_key)?;
        let local_machine_id = self.machine_hardware_id_source.current().await?;
        self.signature_verifier.verify(&payload).await?;
        let grant = LicenseGrant::activate(
            payload,
            &local_machine_id,
            new_license_grant_id(),
            self.clock.now(),
        )?;
        self.license_grant_repository
            .save(record_for(&grant))
            .await?;
        drain(LicenseGrantActivated::new(
            grant.id(),
            grant.machine_hardware_id().clone(),
            grant.feature_flags().to_vec(),
            grant.activated_at(),
        ));
        Ok(grant.feature_flags().to_vec())
    }
}

/// Maps a granted license onto the port-owned persisted record (§8.3).
///
/// Same `#[allow(dead_code)]` as the use case: exercised only via the Gate 2
/// tests until the composition gate.
#[allow(dead_code)]
fn record_for(grant: &LicenseGrant) -> LicenseGrantRecord {
    LicenseGrantRecord {
        id: grant.id().as_uuid().to_string(),
        machine_hardware_id: grant.machine_hardware_id().as_str().to_string(),
        feature_flags: grant
            .feature_flags()
            .iter()
            .map(|flag| flag.as_str().to_string())
            .collect(),
        signature: base64::engine::general_purpose::STANDARD.encode(grant.signature().as_bytes()),
        activated_at: grant.activated_at().to_iso8601(),
    }
}

/// Drains a domain event: Phase 1 has no consumer (the Audit Trail entity is
/// Phase 2, §4.5/§6.3), so the event is deliberately discarded here. This is
/// the insertion point where a future consumer would be invoked
/// (`architecture.md`: "domain events… may later be drained and logged/audited
/// by application code").
///
/// Same `#[allow(dead_code)]` as the use case: exercised only via the Gate 2
/// tests until the composition gate.
#[allow(dead_code)]
fn drain(event: LicenseGrantActivated) {
    drop(event);
}

/// Errors returned by [`ActivateLicenseUseCase`].
///
/// Wraps `LicenseValidationError` (as its two variants) plus `InvalidSignature`
/// plus the outbound-port failures, each mapped to a safe, non-leaking variant
/// per HADR-0005's boundary policy.
///
/// Each port failure folds into its own safe variant because this use case's
/// caller-facing surface is uniform across five distinct failure sources — the
/// deliberate contrast to `GetMachineHardwareIdUseCase`, which passes its
/// single port's error through as-is.
///
/// `#[allow(dead_code)]`: same as the use case — this is the caller-facing
/// error surface of a use case that is unreachable until the composition gate.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) enum ActivateLicenseError {
    /// The License Key string is not a decodable, well-formed envelope.
    #[error("license key is malformed or undecodable")]
    MalformedLicenseKey,
    /// The key is bound to a different machine; carries the local Machine
    /// Hardware ID for Appendix C's "Hardware Change Detected" message.
    #[error("machine hardware id mismatch: current machine is {current_machine_hardware_id}")]
    MachineHardwareMismatch {
        /// The local installation's machine hardware id.
        current_machine_hardware_id: MachineHardwareId,
    },
    /// The signature did not verify against the embedded public key; no
    /// `LicenseGrant` is ever constructed in this case.
    #[error("license key signature is invalid")]
    InvalidSignature,
    /// The Machine Hardware ID could not be computed in this environment.
    #[error("machine hardware id is unavailable in this environment")]
    HardwareIdUnavailable,
    /// The license grant store could not be written.
    #[error("license grant store is unavailable")]
    RepositoryUnavailable,
}

impl From<LicenseValidationError> for ActivateLicenseError {
    fn from(err: LicenseValidationError) -> Self {
        match err {
            LicenseValidationError::MalformedLicenseKey => Self::MalformedLicenseKey,
            LicenseValidationError::MachineHardwareMismatch {
                current_machine_hardware_id,
            } => Self::MachineHardwareMismatch {
                current_machine_hardware_id,
            },
        }
    }
}

impl From<HardwareIdError> for ActivateLicenseError {
    fn from(_: HardwareIdError) -> Self {
        Self::HardwareIdUnavailable
    }
}

impl From<SignatureVerificationError> for ActivateLicenseError {
    fn from(_: SignatureVerificationError) -> Self {
        Self::InvalidSignature
    }
}

impl From<RepositoryError> for ActivateLicenseError {
    fn from(_: RepositoryError) -> Self {
        Self::RepositoryUnavailable
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::{
        FakeClock, FakeLicenseGrantRepository, FakeLicenseSignatureVerifier,
        FakeMachineHardwareIdSource, block_on, machine_id, raw_license_key, timestamp,
    };
    use crate::domain::value_objects::test_support::grouped;
    use uuid::Uuid;

    fn harness(
        source: FakeMachineHardwareIdSource,
        verifier: FakeLicenseSignatureVerifier,
        repository: FakeLicenseGrantRepository,
    ) -> (
        ActivateLicenseUseCase,
        Arc<FakeMachineHardwareIdSource>,
        Arc<FakeLicenseSignatureVerifier>,
        Arc<FakeLicenseGrantRepository>,
    ) {
        let source = Arc::new(source);
        let verifier = Arc::new(verifier);
        let repository = Arc::new(repository);
        let use_case = ActivateLicenseUseCase::new(
            source.clone(),
            verifier.clone(),
            repository.clone(),
            Arc::new(FakeClock::new(timestamp())),
        );
        (use_case, source, verifier, repository)
    }

    #[test]
    fn activates_and_persists_valid_license() {
        let local = machine_id();
        let (use_case, _source, _verifier, repository) = harness(
            FakeMachineHardwareIdSource::returning(Ok(local.clone())),
            FakeLicenseSignatureVerifier::succeeding(),
            FakeLicenseGrantRepository::new(),
        );
        let flags = block_on(use_case.activate(&raw_license_key(
            local.as_str(),
            &["KP_BLOTTER", "TREASURY"],
        )))
        .unwrap();
        assert_eq!(flags, vec![FeatureFlag::KpBlotter, FeatureFlag::Treasury]);

        let saved = repository.saved_records();
        assert_eq!(saved.len(), 1);
        let record = &saved[0];
        let id = Uuid::parse_str(&record.id).unwrap();
        assert_eq!(id.get_version_num(), 7, "record id is a UUIDv7 string");
        assert_eq!(record.machine_hardware_id, local.as_str());
        assert_eq!(
            record.feature_flags,
            vec!["KP_BLOTTER".to_string(), "TREASURY".to_string()]
        );
        assert_eq!(
            record.signature,
            base64::engine::general_purpose::STANDARD.encode([7u8; 64])
        );
        assert_eq!(record.activated_at, timestamp().to_iso8601());
    }

    #[test]
    fn invalid_signature_constructs_no_grant_and_persists_nothing() {
        let local = machine_id();
        let (use_case, _source, verifier, repository) = harness(
            FakeMachineHardwareIdSource::returning(Ok(local.clone())),
            FakeLicenseSignatureVerifier::failing(),
            FakeLicenseGrantRepository::new(),
        );
        let err = block_on(use_case.activate(&raw_license_key(local.as_str(), &["KP_BLOTTER"])))
            .unwrap_err();
        assert_eq!(err, ActivateLicenseError::InvalidSignature);
        assert_eq!(verifier.call_count(), 1);
        assert!(
            repository.saved_records().is_empty(),
            "verification failure must never construct or persist a LicenseGrant (§14)"
        );
    }

    #[test]
    fn empty_flags_key_validates_and_persists_grant_with_no_flags() {
        let local = machine_id();
        let (use_case, _source, _verifier, repository) = harness(
            FakeMachineHardwareIdSource::returning(Ok(local.clone())),
            FakeLicenseSignatureVerifier::succeeding(),
            FakeLicenseGrantRepository::new(),
        );
        let flags = block_on(use_case.activate(&raw_license_key(local.as_str(), &[]))).unwrap();
        assert!(flags.is_empty());
        let saved = repository.saved_records();
        assert_eq!(saved.len(), 1);
        assert!(saved[0].feature_flags.is_empty());
    }

    #[test]
    fn malformed_key_fails_before_any_port_call() {
        let (use_case, source, verifier, _repository) = harness(
            FakeMachineHardwareIdSource::returning(Ok(machine_id())),
            FakeLicenseSignatureVerifier::succeeding(),
            FakeLicenseGrantRepository::new(),
        );
        let err = block_on(use_case.activate("!!!not base64!!!")).unwrap_err();
        assert_eq!(err, ActivateLicenseError::MalformedLicenseKey);
        assert_eq!(source.call_count(), 0, "parse fails before any port call");
        assert_eq!(verifier.call_count(), 0);
    }

    #[test]
    fn machine_mismatch_carries_the_current_hardware_id() {
        let local = machine_id();
        let other =
            MachineHardwareId::parse(&grouped(13).replace('A', "B").replace('B', "C")).unwrap();
        assert_ne!(other, local);
        let (use_case, _source, _verifier, repository) = harness(
            FakeMachineHardwareIdSource::returning(Ok(local.clone())),
            FakeLicenseSignatureVerifier::succeeding(),
            FakeLicenseGrantRepository::new(),
        );
        let err = block_on(use_case.activate(&raw_license_key(other.as_str(), &["KP_BLOTTER"])))
            .unwrap_err();
        assert_eq!(
            err,
            ActivateLicenseError::MachineHardwareMismatch {
                current_machine_hardware_id: local.clone(),
            }
        );
        assert!(repository.saved_records().is_empty());
    }

    #[test]
    fn hardware_id_failure_maps_to_safe_variant_and_skips_verification() {
        let (use_case, _source, verifier, repository) = harness(
            FakeMachineHardwareIdSource::returning(Err(HardwareIdError::Unavailable)),
            FakeLicenseSignatureVerifier::succeeding(),
            FakeLicenseGrantRepository::new(),
        );
        let err =
            block_on(use_case.activate(&raw_license_key(machine_id().as_str(), &["KP_BLOTTER"])))
                .unwrap_err();
        assert_eq!(err, ActivateLicenseError::HardwareIdUnavailable);
        assert_eq!(
            verifier.call_count(),
            0,
            "verification runs only after the local machine id is read (§7.2)"
        );
        assert!(repository.saved_records().is_empty());
    }

    #[test]
    fn repository_failure_maps_to_safe_variant() {
        let local = machine_id();
        let (use_case, _source, _verifier, _repository) = harness(
            FakeMachineHardwareIdSource::returning(Ok(local.clone())),
            FakeLicenseSignatureVerifier::succeeding(),
            FakeLicenseGrantRepository::with_save_result(Err(RepositoryError::Unavailable)),
        );
        let err = block_on(use_case.activate(&raw_license_key(local.as_str(), &["KP_BLOTTER"])))
            .unwrap_err();
        assert_eq!(err, ActivateLicenseError::RepositoryUnavailable);
    }
}
