//! Prelude module for convenient imports.
//!
//! # Usage
//!
//! ```rust
//! use qns_core::prelude::*;
//! ```

pub use crate::backend::{ExecutionResult, HardwareBackend};
pub use crate::config::{ProfilerConfig, QnsConfig, RewireConfig, SimulatorConfig};
pub use crate::error::{QnsError, Result};
pub use crate::physics::{
    gate_errors, gate_times, t1_typical, t2_typical, GateType, Matrix2x2, Matrix4x4, C64,
};
pub use crate::types::{
    CircuitGenome, CircuitMetadata, CouplerProperties, Fidelity, Gate, HardwareProfile,
    NoiseSource, NoiseVector, QubitProperties, Topology,
};
