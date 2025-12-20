//! QNS Simulator - backend.rs (skeleton)
//! NOTE: Paste full SimulatorBackend implementation here.

use crate::noise::NoiseModel;
use qns_core::backend::{ExecutionResult, HardwareBackend};
use qns_core::prelude::*;
use std::collections::HashMap;
use std::time::Instant;

pub struct SimulatorBackend {
    name: String,
    num_qubits: usize,
    #[allow(dead_code)]
    noise_model: Option<NoiseModel>,
}

impl SimulatorBackend {
    pub fn new(num_qubits: usize) -> Self {
        Self {
            name: format!("simulator_ideal_{}q", num_qubits),
            num_qubits,
            noise_model: None,
        }
    }
}

impl HardwareBackend for SimulatorBackend {
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
    fn execute(&self, _circuit: &CircuitGenome, _shots: usize) -> Result<ExecutionResult> {
        let counts = HashMap::new();
        let duration = Instant::now().elapsed();
        Ok(ExecutionResult::new(counts, 0, duration))
    }
}
