//! Record-identifier generation (`tdd.phase-1` §6.2; follow-up Xaidel/brs#21
//! decision).
//!
//! Decision: ID generation lives in the application layer, not the domain
//! value object. [`LicenseGrantId`] stays a pure wrapper value type
//! (`from_uuid`/`as_uuid`); this module is the only place `app_core` touches
//! `Uuid::now_v7()`. Timestamps are still sourced exclusively through the
//! [`Clock`] port — the "no wall-clock reads" posture applies to domain
//! (`timestamp.rs`); record identifiers are an explicit application-layer
//! exception because UUIDv7 monotonicity is embedded at generation time, so
//! deriving the ID from the `Clock` port would forfeit the chronological
//! ordering `LicenseGrantRepository::find_current` relies on (`tdd.phase-1`
//! §8.3).
//!
//! [`Clock`]: crate::ports::Clock

use uuid::Uuid;

use crate::domain::value_objects::LicenseGrantId;

/// Generates a new monotonic UUIDv7 [`LicenseGrantId`].
///
/// The application layer is explicitly allowed to touch `Uuid::now_v7()`
/// (reads `SystemTime` + OS entropy); the domain is not
/// (`backend_arch_docs/dependency-rules.md`: "Domain modules perform no I/O").
///
/// `#[allow(dead_code)]`: unreachable from `app_core`'s public surface until
/// the assembly/composition gate (HADR-0007 gates 4–5); exercised only by the
/// tests below until then.
#[allow(dead_code)]
pub(crate) fn new_license_grant_id() -> LicenseGrantId {
    LicenseGrantId::from_uuid(Uuid::now_v7())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_distinct_monotonic_uuids() {
        let a = new_license_grant_id();
        let b = new_license_grant_id();
        assert_ne!(a, b);
        assert!(a.as_uuid() < b.as_uuid(), "UUIDv7 is monotonic per process");
    }
}
