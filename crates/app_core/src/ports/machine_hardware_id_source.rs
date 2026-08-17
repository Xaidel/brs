//! The `MachineHardwareIdSource` outbound port (`tdd.phase-1` §8.1).

use async_trait::async_trait;

use crate::domain::value_objects::MachineHardwareId;
use crate::ports::errors::HardwareIdError;

/// Supplies this installation's Machine Hardware ID end-to-end: three WMI
/// queries (`Win32_Processor.ProcessorId`, `Win32_BaseBoard.SerialNumber`,
/// `Win32_ComputerSystemProduct.UUID`), concatenation, SHA-256, Crockford
/// Base32 encoding, and grouping (ADR-0005 §1).
///
/// The capability is entirely infra-side (`infra_hardware_id`); `app_core`
/// receives the finished value object through this port.
#[async_trait]
pub trait MachineHardwareIdSource: Send + Sync {
    /// Reads and derives the local Machine Hardware ID.
    async fn current(&self) -> Result<MachineHardwareId, HardwareIdError>;
}
