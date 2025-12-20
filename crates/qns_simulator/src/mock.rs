//! QNS Simulator - mock.rs (skeleton)
//! NOTE: Paste full MockBackend + MockConfig implementation here.

use crate::noise::NoiseModel;
use qns_core::backend::{ExecutionResult, HardwareBackend};
use qns_core::prelude::*;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct MockConfig {
    pub num_qubits: usize,
    pub latency_ms: u64,
    pub calibration_drift: f64,
    pub base_noise: NoiseModel,
}

impl Default for MockConfig {
    fn default() -> Self {
        Self {
            num_qubits: 5,
            latency_ms: 100,
            calibration_drift: 0.02,
            base_noise: NoiseModel::default(),
        }
    }
}

pub struct MockBackend {
    name: String,
    config: MockConfig,
}

impl MockBackend {
    pub fn new(name: &str, config: MockConfig) -> Self {
        Self {
            name: name.to_string(),
            config,
        }
    }
}

impl HardwareBackend for MockBackend {
    fn name(&self) -> &str {
        &self.name
    }
    fn qubit_count(&self) -> usize {
        self.config.num_qubits
    }
    fn get_topology(&self) -> Option<HardwareProfile> {
        Some(HardwareProfile::linear(&self.name, self.config.num_qubits))
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
