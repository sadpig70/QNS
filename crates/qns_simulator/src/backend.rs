//! QNS Simulator - backend.rs (skeleton)
//! NOTE: Paste full SimulatorBackend implementation here.

use crate::noise::NoiseModel;
use crate::noisy::NoisySimulator;
use qns_core::backend::{ExecutionResult, HardwareBackend};
use qns_core::prelude::*;
use std::collections::HashMap;
use std::time::Instant;

pub struct SimulatorBackend {
    name: String,
    simulator: NoisySimulator,
}

impl SimulatorBackend {
    pub fn new(num_qubits: usize) -> Self {
        Self::ideal(num_qubits)
    }

    pub fn ideal(num_qubits: usize) -> Self {
        Self {
            name: format!("simulator_ideal_{}q", num_qubits),
            simulator: NoisySimulator::ideal(num_qubits),
        }
    }

    pub fn with_noise(num_qubits: usize, noise: NoiseModel) -> Self {
        Self {
            name: format!("simulator_noisy_{}q", num_qubits),
            simulator: NoisySimulator::new(num_qubits, noise),
        }
    }
}

impl HardwareBackend for SimulatorBackend {
    fn name(&self) -> &str {
        &self.name
    }

    fn qubit_count(&self) -> usize {
        self.simulator.num_qubits()
    }

    fn get_topology(&self) -> Option<HardwareProfile> {
        // Assume linear topology for simulator default
        Some(HardwareProfile::linear(
            "simulator_linear",
            self.qubit_count(),
        ))
    }

    fn get_calibration(&self) -> Result<HashMap<usize, NoiseVector>> {
        let noise_model = self.simulator.noise_model();
        let mut calibration = HashMap::new();

        // Convert NoiseModel to NoiseVector (simplified)
        // Note: NoiseModel has single global parameters for simplicity in this version,
        // whereas NoiseVector is per-qubit. We broadcast parameters to all qubits.
        for q in 0..self.qubit_count() {
            let mut nv = NoiseVector::new(q);
            nv.t1_mean = 1.0 / noise_model.amplitude_damping_prob(1e3).max(1e-9) * 1e3; // Approx T1 from prob per us?
                                                                                        // Actually NoiseModel stores raw probs mostly or times?
                                                                                        // Checking NoiseModel: it has single_gate_time_ns, amplitude_damping_prob(time).
                                                                                        // Let's just mock reasonable values based heavily on assumption or use default if we can't reverse.
                                                                                        // Wait, NoiseVector has t1_mean (us).
                                                                                        // For now, let's just populate with dummy or what's in NoiseModel if feasible.
                                                                                        // Better: just fill with what we passed if possible, or Mock.
                                                                                        // Since this method is mainly for testing the PIPELINE (profiling step),
                                                                                        // and pipeline uses DriftScanner which usually SCANS.
                                                                                        // But validation uses this.
                                                                                        // Let's stick to returning what we can.

            // Use actual noise model parameters
            nv.t1_mean = noise_model.t1;
            nv.t2_mean = noise_model.t2;
            nv.gate_error_1q = noise_model.single_gate_error;
            nv.gate_error_2q = noise_model.two_gate_error;
            nv.readout_error = noise_model.readout_error;

            calibration.insert(q, nv);
        }

        Ok(calibration)
    }

    fn execute(&self, circuit: &CircuitGenome, shots: usize) -> Result<ExecutionResult> {
        let start = Instant::now();

        // We need a mutable simulator to execute.
        // But execute takes &self.
        // So we clone the simulator (state vector sim is clonable).
        let mut sim = self.simulator.clone();

        sim.execute(circuit)?;
        let counts = sim.measure(shots)?;

        let duration = start.elapsed();
        Ok(ExecutionResult::new(counts, shots, duration))
    }
}
