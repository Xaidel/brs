//! The `LicenseSignatureVerifier` outbound port (`tdd.phase-1` §8.2).

use async_trait::async_trait;

use crate::domain::value_objects::LicenseKeyPayload;
use crate::ports::errors::SignatureVerificationError;

/// Verifies a License Key payload's Ed25519 signature against the embedded
/// public key (`infra_licensing`).
///
/// The adapter holds the public key as its own baked-in configuration, not a
/// port parameter, and verifies `payload`'s signature over the canonical signed
/// bytes (§11). Declared `async` per `architecture.md`'s "every outbound port…
/// declared async" rule, although verification itself is synchronous,
/// in-process, I/O-free.
#[async_trait]
pub trait LicenseSignatureVerifier: Send + Sync {
    /// Verifies `payload`'s signature. Fails with
    /// `SignatureVerificationError::InvalidSignature` on a bad signature.
    async fn verify(&self, payload: &LicenseKeyPayload) -> Result<(), SignatureVerificationError>;
}
