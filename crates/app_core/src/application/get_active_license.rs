//! `GetActiveLicenseUseCase` (`tdd.phase-1` §7.3).

use std::sync::Arc;

use serde::Deserialize;
use serde::de::value::StrDeserializer;
use thiserror::Error;

use crate::domain::value_objects::FeatureFlag;
use crate::ports::{LicenseGrantRepository, RepositoryError};

/// Returns the Feature Flags unlocked by the most recently activated license.
///
/// Core Engine itself needs no flag (ADR-0003 §4), so "no license yet" and "a
/// license with no flags" both surface as an empty set.
///
/// Only the record's `feature_flags` are validated here; other corrupt record
/// fields are tolerated deliberately — this use case's scope is the flag set,
/// and the repository owns the record's persistence-level integrity.
///
/// `#[allow(dead_code)]`: unreachable from `app_core`'s public surface until
/// the assembly/composition gate (HADR-0007 gates 4–5); exercised only by the
/// Gate 2 tests until then.
#[allow(dead_code)]
pub(crate) struct GetActiveLicenseUseCase {
    license_grant_repository: Arc<dyn LicenseGrantRepository>,
}

/// Same `#[allow(dead_code)]` as the struct: unreachable until the
/// assembly/composition gate; exercised by the Gate 2 tests.
#[allow(dead_code)]
impl GetActiveLicenseUseCase {
    /// Constructs the use case around the `infra_persistence` implementation.
    pub(crate) fn new(license_grant_repository: Arc<dyn LicenseGrantRepository>) -> Self {
        Self {
            license_grant_repository,
        }
    }

    /// The most recently activated grant's Feature Flags, or an empty set if
    /// none exists yet.
    ///
    /// # Errors
    ///
    /// [`GetActiveLicenseError::Unavailable`] when the store cannot be read;
    /// [`GetActiveLicenseError::CorruptRecord`] when a stored record carries a
    /// flag string outside the ADR-0003 catalog.
    pub(crate) async fn current_feature_flags(
        &self,
    ) -> Result<Vec<FeatureFlag>, GetActiveLicenseError> {
        match self.license_grant_repository.find_current().await? {
            Some(record) => record
                .feature_flags
                .iter()
                .map(|flag| parse_feature_flag(flag))
                .collect(),
            None => Ok(Vec::new()),
        }
    }
}

/// Decodes a stored SCREAMING_SNAKE_CASE flag string (ADR-0003) back into a
/// [`FeatureFlag`]; a string outside the catalog is a corrupt record.
///
/// Same `#[allow(dead_code)]` as the use case: exercised only via the Gate 2
/// tests until the composition gate.
#[allow(dead_code)]
fn parse_feature_flag(flag: &str) -> Result<FeatureFlag, GetActiveLicenseError> {
    FeatureFlag::deserialize(StrDeserializer::<'_, serde::de::value::Error>::new(flag))
        .map_err(|_| GetActiveLicenseError::CorruptRecord)
}

/// Errors returned by [`GetActiveLicenseUseCase`].
///
/// `#[allow(dead_code)]`: same as the use case — this is the caller-facing
/// error surface of a use case that is unreachable until the composition gate.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub(crate) enum GetActiveLicenseError {
    /// The license grant store could not be read.
    #[error("license grant store is unavailable")]
    Unavailable,
    /// A stored record carries a feature-flag string outside the ADR-0003
    /// catalog.
    #[error("stored license grant record is corrupt")]
    CorruptRecord,
}

impl From<RepositoryError> for GetActiveLicenseError {
    fn from(_: RepositoryError) -> Self {
        Self::Unavailable
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::{FakeLicenseGrantRepository, block_on, machine_id};
    use crate::ports::LicenseGrantRecord;
    use base64::Engine as _;

    fn record_with_flags(flags: Vec<String>) -> LicenseGrantRecord {
        LicenseGrantRecord {
            id: "01960000-0000-7000-8000-000000000000".to_string(),
            machine_hardware_id: machine_id().as_str().to_string(),
            feature_flags: flags,
            signature: base64::engine::general_purpose::STANDARD.encode([7u8; 64]),
            activated_at: "2026-08-18T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn returns_flags_of_most_recent_grant() {
        let repository = Arc::new(FakeLicenseGrantRepository::with_find_result(Ok(Some(
            record_with_flags(vec!["KP_BLOTTER".to_string(), "TREASURY".to_string()]),
        ))));
        let use_case = GetActiveLicenseUseCase::new(repository);
        assert_eq!(
            block_on(use_case.current_feature_flags()).unwrap(),
            vec![FeatureFlag::KpBlotter, FeatureFlag::Treasury]
        );
    }

    #[test]
    fn returns_empty_set_when_no_grant_exists() {
        let repository = Arc::new(FakeLicenseGrantRepository::with_find_result(Ok(None)));
        let use_case = GetActiveLicenseUseCase::new(repository);
        assert_eq!(
            block_on(use_case.current_feature_flags()).unwrap(),
            Vec::<FeatureFlag>::new()
        );
    }

    #[test]
    fn repository_failure_maps_to_safe_variant() {
        let repository = Arc::new(FakeLicenseGrantRepository::with_find_result(Err(
            RepositoryError::Unavailable,
        )));
        let use_case = GetActiveLicenseUseCase::new(repository);
        assert_eq!(
            block_on(use_case.current_feature_flags()),
            Err(GetActiveLicenseError::Unavailable)
        );
    }

    #[test]
    fn record_with_empty_flags_returns_empty_set() {
        let repository = Arc::new(FakeLicenseGrantRepository::with_find_result(Ok(Some(
            record_with_flags(vec![]),
        ))));
        let use_case = GetActiveLicenseUseCase::new(repository);
        assert_eq!(
            block_on(use_case.current_feature_flags()).unwrap(),
            Vec::<FeatureFlag>::new()
        );
    }

    #[test]
    fn unknown_flag_string_is_a_corrupt_record() {
        let repository = Arc::new(FakeLicenseGrantRepository::with_find_result(Ok(Some(
            record_with_flags(vec!["BOGUS".to_string()]),
        ))));
        let use_case = GetActiveLicenseUseCase::new(repository);
        assert_eq!(
            block_on(use_case.current_feature_flags()),
            Err(GetActiveLicenseError::CorruptRecord)
        );
    }
}
