//! QNS Simulator - lib.rs (skeleton)
//! NOTE: Replace with full module declarations as needed.

pub mod backend;
pub mod mock;
pub mod noise;
pub mod noisy;
pub mod state_vector;

pub use backend::SimulatorBackend;
pub use mock::{MockBackend, MockConfig};
pub use noise::{DepolarizingChannel, KrausOperator, MeasurementError, NoiseModel};
pub use noisy::{estimate_circuit_fidelity, estimate_gate_fidelity, NoisySimulator};
pub use state_vector::StateVectorSimulator;
