//! The PBKDF2 salt persisted in `bootstrap.json` (`tdd.phase-1` §6.1, ADR-0005).

/// Not secret — its role is uniqueness, not confidentiality (ADR-0005 §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EncryptionSalt([u8; 16]);

impl EncryptionSalt {
    /// The fixed salt length in bytes.
    pub const LENGTH: usize = 16;

    /// Constructs a salt from raw bytes.
    pub const fn from_bytes(bytes: [u8; Self::LENGTH]) -> Self {
        Self(bytes)
    }

    /// The raw salt bytes.
    pub const fn as_bytes(&self) -> &[u8; Self::LENGTH] {
        &self.0
    }
}
