//! A fixed-length Ed25519 signature (64 bytes), structurally validated at
//! construction (`tdd.phase-1` §6.1).

/// Verification itself happens behind `LicenseSignatureVerifier` in
/// `infra_licensing`, not in domain (§4.4); this type only enforces the shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LicenseSignature([u8; 64]);

impl LicenseSignature {
    /// The fixed encoded length of an Ed25519 signature.
    pub const LENGTH: usize = 64;

    /// Constructs a signature from its raw 64 bytes.
    pub const fn from_bytes(bytes: [u8; Self::LENGTH]) -> Self {
        Self(bytes)
    }

    /// The raw signature bytes.
    pub const fn as_bytes(&self) -> &[u8; Self::LENGTH] {
        &self.0
    }
}
