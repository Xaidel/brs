//! The `Clock` outbound port (`tdd.phase-1` §8.5, ADR-0006).

use crate::domain::value_objects::Timestamp;

/// Supplies the current UTC time, stamping `created_at`/`updated_at` and
/// activation timestamps via `app_core`, never SQLite `DEFAULT`/triggers
/// (ADR-0006). Concrete adapter `SystemClock` lives in `src-tauri` (§4.5).
/// Not `async` — reading the system clock is not a suspension point.
pub trait Clock: Send + Sync {
    /// The current UTC instant.
    fn now(&self) -> Timestamp;
}
