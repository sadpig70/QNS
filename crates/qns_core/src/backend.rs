//! QNS Core - backend.rs (skeleton)
//! NOTE: Paste full HardwareBackend + ExecutionResult implementation here.

use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub counts: HashMap<String, usize>,
    pub shots: usize,
    pub duration: Duration,
    pub metadata: HashMap<String, String>,
}

impl ExecutionResult {
    pub fn new(counts: HashMap<String, usize>, shots: usize, duration: Duration) -> Self {
        Self {
            counts,
            shots,
            duration,
            metadata: HashMap::new(),
        }
    }

    pub fn probability(&self, outcome: &str) -> f64 {
        if self.shots == 0 {
            return 0.0;
        }
        *self.counts.get(outcome).unwrap_or(&0) as f64 / self.shots as f64
    }
}

pub trait HardwareBackend: Send + Sync {
    fn name(&self) -> &str;
    fn qubit_count(&self) -> usize;
    fn get_topology(&self) -> Option<HardwareProfile>;
    fn get_calibration(&self) -> Result<HashMap<usize, NoiseVector>>;
    fn execute(&self, circuit: &CircuitGenome, shots: usize) -> Result<ExecutionResult>;
}
