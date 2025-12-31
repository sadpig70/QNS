//! MPS Simulator Backend

use crate::noise::NoiseModel;
use qns_core::backend::{ExecutionResult, HardwareBackend};
use qns_core::prelude::*;
use qns_tensor::TensorNetwork;
use std::collections::HashMap;
use std::time::Instant;

/// MPS-based Quantum Simulator
pub struct MpsSimulator {
    name: String,
    num_qubits: usize,
    max_bond_dim: usize,
    noise_model: Option<NoiseModel>,
}

impl MpsSimulator {
    /// Create a new MpsSimulator
    pub fn new(num_qubits: usize) -> Self {
        Self {
            name: format!("mps_simulator_{}q", num_qubits),
            num_qubits,
            max_bond_dim: 16, // Default max bond dimension
            noise_model: None,
        }
    }

    /// Set maximum bond dimension
    pub fn with_bond_dim(mut self, bond_dim: usize) -> Self {
        self.max_bond_dim = bond_dim;
        self
    }

    /// Set noise model
    pub fn with_noise(mut self, noise_model: NoiseModel) -> Self {
        self.noise_model = Some(noise_model);
        self
    }
}

impl HardwareBackend for MpsSimulator {
    fn name(&self) -> &str {
        &self.name
    }

    fn qubit_count(&self) -> usize {
        self.num_qubits
    }

    fn get_topology(&self) -> Option<HardwareProfile> {
        None
    }

    fn get_calibration(&self) -> Result<HashMap<usize, NoiseVector>> {
        Ok(HashMap::new())
    }

    fn execute(&self, circuit: &CircuitGenome, shots: usize) -> Result<ExecutionResult> {
        let start = Instant::now();

        // Initialize Tensor Network
        let mut tn = TensorNetwork::new(self.num_qubits, self.max_bond_dim);

        // Apply gates
        for gate in &circuit.gates {
            tn.apply_gate(gate)?;
        }

        // Measure
        let counts = tn.measure(shots)?;

        // Duration
        let duration = start.elapsed();

        Ok(ExecutionResult::new(counts, shots, duration))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mps_simulator_ghz() {
        // GHZ State: H(0) -> CX(0, 1) -> CX(1, 2)
        // Result should be 50% |000> and 50% |111>
        let mut circuit = CircuitGenome::new(3);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();
        circuit.add_gate(Gate::CNOT(1, 2)).unwrap();

        let backend = MpsSimulator::new(3);
        let result = backend.execute(&circuit, 1000).unwrap();

        let counts = result.counts;
        let p000 = *counts.get("000").unwrap_or(&0) as f64 / 1000.0;
        let p111 = *counts.get("111").unwrap_or(&0) as f64 / 1000.0;

        println!("MPS GHZ Result: |000>: {:.2}, |111>: {:.2}", p000, p111);

        assert!((p000 - 0.5).abs() < 0.1);
        assert!((p111 - 0.5).abs() < 0.1);
        assert!((p000 + p111) > 0.9);
    }
}
