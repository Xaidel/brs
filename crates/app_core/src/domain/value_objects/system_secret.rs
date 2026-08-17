//! The per-installation secret kept exclusively in the OS-protected credential
//! store (`tdd.phase-1` §6.1, ADR-0005). Debug output is redacted; it must
//! never leak its bytes.

use std::fmt;

/// The per-installation secret kept exclusively in the OS-protected credential
/// store (ADR-0005). Combined with the salt via PBKDF2-HMAC-SHA256 to derive
/// the SQLCipher database key in `infra_credentials`.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SystemSecret([u8; 32]);

impl SystemSecret {
    /// The fixed secret length in bytes.
    pub const LENGTH: usize = 32;

    /// Constructs a secret from raw bytes.
    pub const fn from_bytes(bytes: [u8; Self::LENGTH]) -> Self {
        Self(bytes)
    }

    /// The raw secret bytes (adapter-owned consumption for keyring storage).
    pub const fn as_bytes(&self) -> &[u8; Self::LENGTH] {
        &self.0
    }
}

impl fmt::Debug for SystemSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SystemSecret([redacted])")
    }
}
