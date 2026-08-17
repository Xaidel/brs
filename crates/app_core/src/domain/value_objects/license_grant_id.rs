//! The `LicenseGrant` record identifier (`tdd.phase-1` §6.1, §6.2). Generated
//! as a monotonic UUIDv7 via the `uuid` crate's `now_v7()` per the map's
//! record-identifier decision (ticket #8).

use std::fmt;

use uuid::Uuid;

/// The `LicenseGrant` record identifier (`tdd.phase-1` §6.2). Generated as a
/// monotonic UUIDv7 via the `uuid` crate's `now_v7()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LicenseGrantId(Uuid);

impl LicenseGrantId {
    /// Generates a new monotonic UUIDv7 identifier.
    pub fn generate() -> Self {
        Self(Uuid::now_v7())
    }

    /// Wraps an existing UUID.
    pub const fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// The underlying UUID.
    pub const fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for LicenseGrantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_distinct_monotonic_uuids() {
        let a = LicenseGrantId::generate();
        let b = LicenseGrantId::generate();
        assert_ne!(a, b);
        assert!(a.as_uuid() < b.as_uuid(), "UUIDv7 is monotonic per process");
    }
}
