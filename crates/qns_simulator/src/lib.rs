//! # QNS Simulator
//!
//! Quantum simulation backend for QNS.
//!
//! ## Modules
//! - **StateVectorSimulator**: Exact full state vector simulation.
//! - **MpsSimulator**: Matrix Product State simulation for larger, lower-entanglement circuits.
//! - **NoisySimulator**: Simulation with noise models.
//! - **MockBackend**: Helper for testing and calibration mocking.

pub mod backend;
pub mod mock;
pub mod mps;
pub mod noise;
pub mod noisy;
pub mod state_vector;

pub use backend::SimulatorBackend;
pub use mock::{MockBackend, MockConfig};
pub use mps::MpsSimulator;
pub use noise::{DepolarizingChannel, KrausOperator, MeasurementError, NoiseModel};
pub use noisy::{estimate_circuit_fidelity, estimate_gate_fidelity, NoisySimulator};
pub use state_vector::StateVectorSimulator;
