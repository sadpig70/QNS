//! Noisy quantum simulator.
//!
//! Extends the state vector simulator with realistic noise models.

use qns_core::physics::{Matrix2x2, C64, ZERO};
use qns_core::prelude::*;
use rand::Rng;
use std::collections::HashMap;

use crate::noise::{DepolarizingChannel, MeasurementError, NoiseModel};
use crate::state_vector::StateVectorSimulator;
#[cfg(test)]
use qns_core::physics::ONE;

/// Quantum simulator with noise modeling.
///
/// Wraps a StateVectorSimulator and applies noise after each gate.
///
/// # Noise Model
///
/// The simulator applies three types of noise:
/// 1. **Thermal relaxation**: T1/T2 decoherence during gate execution
/// 2. **Gate errors**: Depolarizing channel after each gate
/// 3. **Measurement errors**: Readout errors during measurement
///
/// # Example
///
/// ```rust
/// use qns_simulator::{NoisySimulator, NoiseModel};
/// use qns_core::prelude::*;
///
/// // Create noisy simulator with default noise model
/// let mut sim = NoisySimulator::new(2, NoiseModel::new());
///
/// // Execute circuit
/// let mut circuit = CircuitGenome::new(2);
/// circuit.add_gate(Gate::H(0)).unwrap();
/// circuit.add_gate(Gate::CNOT(0, 1)).unwrap();
///
/// sim.execute(&circuit).unwrap();
///
/// // Fidelity will be less than 1 due to noise
/// let ideal_sim = qns_simulator::StateVectorSimulator::new(2);
/// // let fidelity = sim.fidelity_with_state(ideal_sim.statevector()).unwrap();
/// ```
pub struct NoisySimulator {
    /// Underlying state vector simulator
    inner: StateVectorSimulator,
    /// Noise model
    noise: NoiseModel,
    /// Random number generator
    rng: rand::rngs::ThreadRng,
    /// Accumulated simulation time (for tracking decoherence)
    elapsed_time_ns: f64,
    /// Gate count for statistics
    gate_count: usize,
    /// Error events recorded
    error_count: usize,
}

impl NoisySimulator {
    /// Creates a new noisy simulator.
    pub fn new(num_qubits: usize, noise: NoiseModel) -> Self {
        Self {
            inner: StateVectorSimulator::new(num_qubits),
            noise,
            rng: rand::thread_rng(),
            elapsed_time_ns: 0.0,
            gate_count: 0,
            error_count: 0,
        }
    }

    /// Creates a noisy simulator with ideal (no noise) model.
    pub fn ideal(num_qubits: usize) -> Self {
        Self::new(num_qubits, NoiseModel::ideal())
    }

    /// Creates from a NoiseVector.
    pub fn from_noise_vector(num_qubits: usize, nv: &NoiseVector) -> Self {
        Self::new(num_qubits, NoiseModel::from_noise_vector(nv))
    }

    /// Returns the number of qubits.
    pub fn num_qubits(&self) -> usize {
        self.inner.num_qubits()
    }

    /// Returns the noise model.
    pub fn noise_model(&self) -> &NoiseModel {
        &self.noise
    }

    /// Returns the elapsed simulation time in nanoseconds.
    pub fn elapsed_time(&self) -> f64 {
        self.elapsed_time_ns
    }

    /// Returns the total gate count.
    pub fn gate_count(&self) -> usize {
        self.gate_count
    }

    /// Returns the error event count.
    pub fn error_count(&self) -> usize {
        self.error_count
    }

    /// Resets the simulator to initial state |0...0⟩.
    pub fn reset(&mut self) {
        self.inner.reset();
        self.elapsed_time_ns = 0.0;
        self.gate_count = 0;
        self.error_count = 0;
    }

    /// Returns the state vector.
    pub fn statevector(&self) -> &[C64] {
        self.inner.statevector()
    }

    /// Returns probabilities for all basis states.
    pub fn probabilities(&self) -> Vec<f64> {
        self.inner.probabilities()
    }

    /// Returns the amplitude for a specific basis state.
    pub fn amplitude(&self, index: usize) -> C64 {
        self.inner.amplitude(index)
    }

    /// Applies a gate with noise.
    pub fn apply_gate(&mut self, gate: &Gate) -> Result<()> {
        // 1. Apply the ideal gate
        self.inner.apply_gate(gate)?;
        self.gate_count += 1;

        // Determine gate time and error rate
        // For 2Q gates, use per-edge error rate if available
        let (gate_time, error_rate) = match gate {
            Gate::H(_)
            | Gate::X(_)
            | Gate::Y(_)
            | Gate::Z(_)
            | Gate::S(_)
            | Gate::T(_)
            | Gate::Rx(_, _)
            | Gate::Ry(_, _)
            | Gate::Rz(_, _) => (self.noise.single_gate_time_ns, self.noise.single_gate_error),
            Gate::CNOT(q1, q2) | Gate::CZ(q1, q2) | Gate::SWAP(q1, q2) => {
                // Use edge-specific error rate if available
                let edge_error = self.noise.get_edge_error(*q1, *q2);
                (self.noise.two_gate_time_ns, edge_error)
            },
            Gate::Measure(_) => {
                return Ok(()); // Measurement handled separately
            },
        };

        // 2. Apply thermal relaxation (T1/T2 noise)
        if self.noise.thermal_relaxation {
            self.apply_thermal_relaxation(gate.qubits(), gate_time);
        }

        // 3. Apply gate error (depolarizing)
        if self.noise.gate_errors && error_rate > 0.0 {
            self.apply_depolarizing_error(gate.qubits(), error_rate);
        }

        // 4. Apply crosstalk error (Phase 2)
        self.apply_crosstalk_error(&gate.qubits());

        // Update elapsed time
        self.elapsed_time_ns += gate_time;

        Ok(())
    }

    /// Applies T1/T2 thermal relaxation to specified qubits.
    fn apply_thermal_relaxation(&mut self, qubits: Vec<usize>, time_ns: f64) {
        let gamma = self.noise.amplitude_damping_prob(time_ns);
        let lambda = self.noise.phase_damping_prob(time_ns);

        for &qubit in &qubits {
            // Apply amplitude damping (T1)
            if gamma > 1e-15 {
                self.apply_amplitude_damping(qubit, gamma);
            }

            // Apply phase damping (pure T2)
            if lambda > 1e-15 {
                self.apply_phase_damping(qubit, lambda);
            }
        }
    }

    /// Applies amplitude damping channel to a qubit.
    ///
    /// With probability γ, |1⟩ decays to |0⟩.
    fn apply_amplitude_damping(&mut self, qubit: usize, gamma: f64) {
        let mask = 1 << qubit;
        let dim = self.inner.dimension();

        // Simplified amplitude damping for state vector:
        // For each pair (|...0...⟩, |...1...⟩), apply damping
        let sqrt_1_gamma = (1.0 - gamma).sqrt();

        // We need mutable access, so work with raw state
        let state = self.inner.statevector().to_vec();
        let mut new_state = state.clone();

        for i in 0..dim {
            if (i & mask) == 0 {
                let j = i | mask; // State with qubit=1

                // K0: |0⟩ → |0⟩, |1⟩ → √(1-γ)|1⟩
                // K1: |1⟩ → √γ|0⟩
                //
                // For state vector, we use a probabilistic approach:
                // Sample whether decay happens based on |1⟩ probability
                let prob_1 = state[j].norm_sqr();

                if prob_1 > 1e-15 && self.rng.gen::<f64>() < gamma * prob_1 / (prob_1 + 1e-15) {
                    // Decay happened: transfer amplitude from |1⟩ to |0⟩
                    let transfer = state[j] * C64::new(gamma.sqrt(), 0.0);
                    new_state[i] += transfer;
                    new_state[j] = state[j] * C64::new(sqrt_1_gamma, 0.0);
                    self.error_count += 1;
                }
            }
        }

        // Renormalize
        let norm_sq: f64 = new_state.iter().map(|a| a.norm_sqr()).sum();
        if norm_sq > 1e-15 {
            let norm_factor = 1.0 / norm_sq.sqrt();
            for a in &mut new_state {
                *a *= norm_factor;
            }
        }

        self.inner.set_state(new_state).ok();
    }

    /// Applies phase damping channel to a qubit.
    fn apply_phase_damping(&mut self, qubit: usize, lambda: f64) {
        // Phase damping reduces off-diagonal coherence
        // For state vector: randomly apply phase flip with probability λ
        if self.rng.gen::<f64>() < lambda {
            self.apply_z_error(qubit);
            self.error_count += 1;
        }
    }

    /// Applies depolarizing error to specified qubits.
    fn apply_depolarizing_error(&mut self, qubits: Vec<usize>, error_rate: f64) {
        let channel = DepolarizingChannel::new(error_rate);

        for &qubit in &qubits {
            let pauli = channel.sample(&mut self.rng);
            if pauli != 0 {
                match pauli {
                    1 => self.apply_x_error(qubit),
                    2 => self.apply_y_error(qubit),
                    3 => self.apply_z_error(qubit),
                    _ => {},
                }
                self.error_count += 1;
            }
        }
    }

    /// Applies X error (bit flip) to a qubit.
    fn apply_x_error(&mut self, qubit: usize) {
        use qns_core::physics::PAULI_X;
        self.apply_pauli(qubit, &PAULI_X);
    }

    /// Applies Y error to a qubit.
    fn apply_y_error(&mut self, qubit: usize) {
        use qns_core::physics::PAULI_Y;
        self.apply_pauli(qubit, &PAULI_Y);
    }

    /// Applies Z error (phase flip) to a qubit.
    fn apply_z_error(&mut self, qubit: usize) {
        use qns_core::physics::PAULI_Z;
        self.apply_pauli(qubit, &PAULI_Z);
    }

    /// Applies a Pauli matrix to a qubit.
    fn apply_pauli(&mut self, qubit: usize, matrix: &Matrix2x2) {
        let mask = 1 << qubit;
        let dim = self.inner.dimension();

        let state = self.inner.statevector().to_vec();
        let mut new_state = vec![ZERO; dim];

        for i in 0..dim {
            if (i & mask) == 0 {
                let j = i | mask;

                let a0 = state[i];
                let a1 = state[j];

                new_state[i] = matrix[0][0] * a0 + matrix[0][1] * a1;
                new_state[j] = matrix[1][0] * a0 + matrix[1][1] * a1;
            }
        }

        self.inner.set_state(new_state).ok();
    }

    /// Applies crosstalk errors based on active qubits.
    fn apply_crosstalk_error(&mut self, active_qubits: &[usize]) {
        if let Some(crosstalk) = &self.noise.crosstalk {
            if crosstalk.is_empty() {
                return;
            }

            // Iterate over all defined interactions
            let mut errors_to_apply = Vec::new();

            for (&(q1, q2), &strength) in &crosstalk.interactions {
                let spectator = if active_qubits.contains(&q1) && !active_qubits.contains(&q2) {
                    Some(q2)
                } else if active_qubits.contains(&q2) && !active_qubits.contains(&q1) {
                    Some(q1)
                } else {
                    None
                };

                if let Some(target) = spectator {
                    // Apply error with probability proportional to strength
                    if strength > 0.0 {
                        errors_to_apply.push((target, strength));
                    }
                }
            }

            // Apply collected errors
            for (qubit, prob) in errors_to_apply {
                if self.rng.gen::<f64>() < prob {
                    self.apply_z_error(qubit);
                    self.error_count += 1;
                }
            }
        }
    }

    /// Executes a quantum circuit with noise.
    pub fn execute(&mut self, circuit: &CircuitGenome) -> Result<()> {
        if circuit.num_qubits != self.num_qubits() {
            return Err(QnsError::DimensionMismatch(
                self.num_qubits(),
                circuit.num_qubits,
            ));
        }

        for gate in &circuit.gates {
            self.apply_gate(gate)?;
        }

        Ok(())
    }

    /// Executes a circuit after resetting.
    pub fn run(&mut self, circuit: &CircuitGenome) -> Result<()> {
        self.reset();
        self.execute(circuit)
    }

    /// Performs measurement with optional readout error.
    pub fn measure(&mut self, shots: usize) -> Result<HashMap<String, usize>> {
        if self.noise.measurement_errors && self.noise.readout_error > 0.0 {
            self.measure_with_error(shots)
        } else {
            self.inner.measure(shots)
        }
    }

    /// Performs measurement with readout errors.
    fn measure_with_error(&mut self, shots: usize) -> Result<HashMap<String, usize>> {
        let probs = self.inner.probabilities();
        let me = MeasurementError::symmetric(self.noise.readout_error);
        let mut results: HashMap<String, usize> = HashMap::new();
        let n = self.num_qubits();

        for _ in 0..shots {
            // Sample ideal outcome
            let outcome = self.sample_outcome(&probs);

            // Apply readout error to each qubit
            let mut noisy_outcome = 0usize;
            for q in 0..n {
                let bit = ((outcome >> q) & 1) as u8;
                let noisy_bit = me.apply(bit, &mut self.rng);
                noisy_outcome |= (noisy_bit as usize) << q;
            }

            let bitstring = self.index_to_bitstring(noisy_outcome);
            *results.entry(bitstring).or_insert(0) += 1;
        }

        Ok(results)
    }

    /// Samples a single measurement outcome.
    fn sample_outcome(&mut self, probs: &[f64]) -> usize {
        let r: f64 = self.rng.gen();
        let mut cumulative = 0.0;

        for (i, &p) in probs.iter().enumerate() {
            cumulative += p;
            if r < cumulative {
                return i;
            }
        }

        probs.len() - 1
    }

    /// Converts index to bitstring.
    fn index_to_bitstring(&self, index: usize) -> String {
        let n = self.num_qubits();
        (0..n)
            .rev()
            .map(|q| if (index >> q) & 1 == 1 { '1' } else { '0' })
            .collect()
    }

    /// Calculates fidelity with a target state.
    pub fn fidelity(&self, target: &[C64]) -> Result<f64> {
        self.inner.fidelity(target)
    }

    /// Calculates fidelity with another simulator.
    pub fn fidelity_with(&self, other: &StateVectorSimulator) -> Result<f64> {
        self.inner.fidelity_with(other)
    }

    /// Checks if state is normalized.
    pub fn is_normalized(&self) -> bool {
        self.inner.is_normalized()
    }
}

impl Clone for NoisySimulator {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            noise: self.noise.clone(),
            rng: rand::thread_rng(),
            elapsed_time_ns: self.elapsed_time_ns,
            gate_count: self.gate_count,
            error_count: self.error_count,
        }
    }
}

/// Calculates the process fidelity of a noisy gate.
///
/// Runs the circuit multiple times and compares to ideal.
pub fn estimate_gate_fidelity(gate: &Gate, noise: &NoiseModel, samples: usize) -> f64 {
    let num_qubits = gate.qubits().iter().max().map(|q| q + 1).unwrap_or(1);

    let mut total_fidelity = 0.0;

    for _ in 0..samples {
        // Create ideal result
        let mut ideal = StateVectorSimulator::new(num_qubits);
        ideal.apply_gate(gate).ok();

        // Create noisy result
        let mut noisy = NoisySimulator::new(num_qubits, noise.clone());
        noisy.apply_gate(gate).ok();

        if let Ok(f) = noisy.fidelity_with(&ideal) {
            total_fidelity += f;
        }
    }

    total_fidelity / samples as f64
}

/// Calculates the circuit fidelity under noise.
pub fn estimate_circuit_fidelity(
    circuit: &CircuitGenome,
    noise: &NoiseModel,
    samples: usize,
) -> f64 {
    let mut total_fidelity = 0.0;

    for _ in 0..samples {
        // Create ideal result
        let mut ideal = StateVectorSimulator::new(circuit.num_qubits);
        ideal.execute(circuit).ok();

        // Create noisy result
        let mut noisy = NoisySimulator::new(circuit.num_qubits, noise.clone());
        noisy.execute(circuit).ok();

        if let Ok(f) = noisy.fidelity_with(&ideal) {
            total_fidelity += f;
        }
    }

    total_fidelity / samples as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOLERANCE: f64 = 1e-6;

    #[test]
    fn test_noisy_simulator_new() {
        let sim = NoisySimulator::new(2, NoiseModel::new());
        assert_eq!(sim.num_qubits(), 2);
        assert_eq!(sim.gate_count(), 0);
    }

    #[test]
    fn test_ideal_simulator() {
        let mut sim = NoisySimulator::ideal(2);

        // Create Bell state
        sim.apply_gate(&Gate::H(0)).unwrap();
        sim.apply_gate(&Gate::CNOT(0, 1)).unwrap();

        // Should be perfect Bell state
        let probs = sim.probabilities();
        assert!((probs[0] - 0.5).abs() < TOLERANCE);
        assert!((probs[3] - 0.5).abs() < TOLERANCE);
    }

    #[test]
    fn test_noisy_reduces_fidelity() {
        let noise = NoiseModel::with_t1t2(50.0, 40.0) // Short T1/T2 for visible noise
            .with_gate_errors(0.01, 0.05);

        // Create ideal reference
        let mut ideal = StateVectorSimulator::new(2);
        ideal.apply_gate(&Gate::H(0)).unwrap();
        ideal.apply_gate(&Gate::CNOT(0, 1)).unwrap();

        // Run noisy simulation multiple times
        let mut total_fidelity = 0.0;
        let samples = 100;

        for _ in 0..samples {
            let mut noisy = NoisySimulator::new(2, noise.clone());
            noisy.apply_gate(&Gate::H(0)).unwrap();
            noisy.apply_gate(&Gate::CNOT(0, 1)).unwrap();

            if let Ok(f) = noisy.fidelity_with(&ideal) {
                total_fidelity += f;
            }
        }

        let avg_fidelity = total_fidelity / samples as f64;

        // With significant noise, fidelity should be less than 1
        // but still reasonably high for a simple circuit
        assert!(avg_fidelity < 1.0);
        assert!(avg_fidelity > 0.5); // Shouldn't be completely random
    }

    #[test]
    fn test_measurement_error() {
        let noise = NoiseModel::ideal().with_readout_error(0.1);

        let mut sim = NoisySimulator::new(1, noise);
        // State is |0⟩

        // Measure many times
        let results = sim.measure(10000).unwrap();

        // Should have mostly 0s but some 1s due to readout error
        let count_0 = results.get("0").copied().unwrap_or(0);
        let count_1 = results.get("1").copied().unwrap_or(0);

        // Expect ~10% readout error (allow larger statistical variance)
        let error_rate = count_1 as f64 / (count_0 + count_1) as f64;
        assert!(
            (error_rate - 0.1).abs() < 0.05,
            "error_rate = {}",
            error_rate
        );
    }

    #[test]
    fn test_estimate_circuit_fidelity() {
        // Use higher noise to ensure fidelity is measurably less than 1
        let noise = NoiseModel::with_t1t2(30.0, 25.0) // Short T1/T2 for visible noise
            .with_gate_errors(0.01, 0.05);

        let mut circuit = CircuitGenome::new(2);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();

        let fidelity = estimate_circuit_fidelity(&circuit, &noise, 50);

        // Should be reasonably high but noticeably less than perfect
        assert!(fidelity > 0.5, "fidelity too low: {}", fidelity);
        assert!(
            fidelity < 0.999,
            "fidelity too high (noise not applied?): {}",
            fidelity
        );
    }

    #[test]
    fn test_gate_count_tracking() {
        let mut sim = NoisySimulator::ideal(2);

        sim.apply_gate(&Gate::H(0)).unwrap();
        sim.apply_gate(&Gate::X(1)).unwrap();
        sim.apply_gate(&Gate::CNOT(0, 1)).unwrap();

        assert_eq!(sim.gate_count(), 3);
    }

    #[test]
    fn test_elapsed_time() {
        let noise = NoiseModel::new();
        let single_time = noise.single_gate_time_ns;
        let two_time = noise.two_gate_time_ns;

        let mut sim = NoisySimulator::new(2, noise);

        sim.apply_gate(&Gate::H(0)).unwrap();
        sim.apply_gate(&Gate::CNOT(0, 1)).unwrap();

        let expected = single_time + two_time;
        assert!((sim.elapsed_time() - expected).abs() < 0.01);
    }

    #[test]
    fn test_execute_circuit() {
        let mut sim = NoisySimulator::ideal(2);

        let mut circuit = CircuitGenome::new(2);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();

        sim.execute(&circuit).unwrap();

        let probs = sim.probabilities();
        assert!((probs[0] - 0.5).abs() < TOLERANCE);
        assert!((probs[3] - 0.5).abs() < TOLERANCE);
    }

    #[test]
    fn test_reset() {
        let mut sim = NoisySimulator::ideal(2);

        sim.apply_gate(&Gate::X(0)).unwrap();
        sim.apply_gate(&Gate::X(1)).unwrap();

        sim.reset();

        // Should be back to |00⟩
        assert!((sim.amplitude(0) - ONE).norm() < TOLERANCE);
        assert_eq!(sim.gate_count(), 0);
        assert!((sim.elapsed_time()).abs() < TOLERANCE);
    }

    #[test]
    fn test_dimension_mismatch() {
        let mut sim = NoisySimulator::ideal(2);
        let circuit = CircuitGenome::new(3);

        assert!(sim.execute(&circuit).is_err());
    }
}
