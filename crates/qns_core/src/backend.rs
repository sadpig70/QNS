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
}

pub trait HardwareBackend: Send + Sync {
    fn name(&self) -> &str;
    fn qubit_count(&self) -> usize;
    fn get_topology(&self) -> Option<HardwareProfile>;
    fn get_calibration(&self) -> Result<HashMap<usize, NoiseVector>>;
    fn execute(&self, circuit: &CircuitGenome, shots: usize) -> Result<ExecutionResult>;
}
