pub mod channels;
pub mod error;

pub use channels::*;
pub use error::*;

use qns_core::types::Gate;

/// A trait for quantum noise channels.
pub trait NoiseChannel {
    /// Apply the noise channel to a gate or a set of qubits.
    /// Returns a list of possible error gates and their probabilities.
    /// Note: This is a simplified stochastic model.
    fn apply(&self, gate: &Gate) -> Vec<(f64, Gate)>;

    /// Returns the name of the channel.
    fn name(&self) -> &str;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum NoiseModel {
    Ideal,
    Depolarizing { p: f64 },
    BitFlip { p: f64 },
    PhaseFlip { p: f64 },
}
