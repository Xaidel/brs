//! `GetMachineHardwareIdUseCase` (`tdd.phase-1` §7.1).
//!
//! `#![allow(dead_code)]`: the use case is unreachable from `app_core`'s
//! public surface until the assembly/composition gate (HADR-0007 gates 4–5);
//! until then it is exercised only by the Gate 2 tests below.

#![allow(dead_code)]

use std::sync::Arc;

use crate::domain::value_objects::MachineHardwareId;
use crate::ports::{HardwareIdError, MachineHardwareIdSource};

/// Returns this installation's Machine Hardware ID for the Settings display.
///
/// Thin passthrough: the capability (WMI reads, hashing, Crockford encoding)
/// is entirely `infra_hardware_id`'s (§4.4); this use case exists so
/// `src-tauri`'s Settings-display command has an inbound port to call rather
/// than reaching into an outbound port directly (runtime request flow,
/// `backend_arch_docs/architecture.md`).
///
/// The port's own error is returned as-is: `HardwareIdError` is already a
/// safe, non-leaking classification (HADR-0005), and this use case adds no
/// failure mode of its own, so a duplicate wrapper would be ceremony.
pub(crate) struct GetMachineHardwareIdUseCase {
    machine_hardware_id_source: Arc<dyn MachineHardwareIdSource>,
}

impl GetMachineHardwareIdUseCase {
    /// Constructs the use case around the `infra_hardware_id` implementation.
    pub(crate) fn new(machine_hardware_id_source: Arc<dyn MachineHardwareIdSource>) -> Self {
        Self {
            machine_hardware_id_source,
        }
    }

    /// The local Machine Hardware ID.
    pub(crate) async fn current_machine_hardware_id(
        &self,
    ) -> Result<MachineHardwareId, HardwareIdError> {
        self.machine_hardware_id_source.current().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::{FakeMachineHardwareIdSource, block_on, machine_id};

    #[test]
    fn returns_current_machine_hardware_id() {
        let local = machine_id();
        let source = Arc::new(FakeMachineHardwareIdSource::returning(Ok(local.clone())));
        let use_case = GetMachineHardwareIdUseCase::new(source.clone());
        assert_eq!(
            block_on(use_case.current_machine_hardware_id()).unwrap(),
            local
        );
        assert_eq!(source.call_count(), 1);
    }

    #[test]
    fn propagates_hardware_id_failure() {
        let source = Arc::new(FakeMachineHardwareIdSource::returning(Err(
            HardwareIdError::Unavailable,
        )));
        let use_case = GetMachineHardwareIdUseCase::new(source);
        assert_eq!(
            block_on(use_case.current_machine_hardware_id()),
            Err(HardwareIdError::Unavailable)
        );
    }
}
