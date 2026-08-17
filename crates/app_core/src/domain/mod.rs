//! Domain layer: framework-free value objects, entities, domain events, and
//! typed domain errors. No I/O, ports, adapters, persistence, or API contract
//! concepts appear here (`tdd.phase-1` §6).

pub(crate) mod entity;
pub(crate) mod errors;
pub(crate) mod events;
pub(crate) mod value_objects;
