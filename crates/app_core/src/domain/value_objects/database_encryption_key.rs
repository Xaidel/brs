//! The derived SQLCipher key material (`PBKDF2-HMAC-SHA256(system_secret,
//! salt)`), returned only by `EncryptionCredentialGateway` (`tdd.phase-1` §6.1,
//! ADR-0005). Output-only; never constructed directly by `app_core`. Debug
//! output is redacted.

use std::fmt;

/// The derived SQLCipher key material, `PBKDF2-HMAC-SHA256(system_secret,
/// salt)`, returned only by `EncryptionCredentialGateway` (ADR-0005).
/// Output-only; never constructed directly by `app_core`.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct DatabaseEncryptionKey([u8; 32]);

impl DatabaseEncryptionKey {
    /// The fixed key length in bytes.
    pub const LENGTH: usize = 32;

    /// Constructs the key from raw derived bytes (adapter-owned).
    pub const fn from_bytes(bytes: [u8; Self::LENGTH]) -> Self {
        Self(bytes)
    }

    /// The raw key bytes (adapter-owned consumption by `infra_persistence`).
    pub const fn as_bytes(&self) -> &[u8; Self::LENGTH] {
        &self.0
    }
}

impl fmt::Debug for DatabaseEncryptionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DatabaseEncryptionKey([redacted])")
    }
}
