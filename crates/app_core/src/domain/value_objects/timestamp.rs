//! A framework-agnostic UTC instant, sourced only through the `Clock` port
//! (`tdd.phase-1` §6.1, §8.5). Domain code never reads the wall clock itself —
//! the one wall-clock touch in `app_core` is record-identifier generation,
//! which lives in the application layer (`application/id.rs`) per follow-up
//! Xaidel/brs#21.

use chrono::{DateTime, Utc};

/// A framework-agnostic UTC instant, sourced only through the `Clock` port
/// (`tdd.phase-1` §8.5). Domain code never reads the wall clock itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    /// Wraps a UTC instant.
    pub const fn new(instant: DateTime<Utc>) -> Self {
        Self(instant)
    }

    /// The wrapped UTC instant.
    pub const fn as_utc(&self) -> &DateTime<Utc> {
        &self.0
    }

    /// The instant as an ISO-8601 UTC timestamp (RFC 3339).
    pub fn to_iso8601(&self) -> String {
        self.0.to_rfc3339()
    }
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(instant: DateTime<Utc>) -> Self {
        Self::new(instant)
    }
}
