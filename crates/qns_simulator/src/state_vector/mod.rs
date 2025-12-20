//! State vector simulator module.
//!
//! This module provides a full state vector quantum simulator that:
//! - Maintains the complete quantum state as a complex amplitude vector
//! - Applies single and two-qubit gates using efficient indexing
//! - Performs measurements using Born rule sampling
//! - Calculates fidelity between states
//!
//! ## Physical Background
//!
//! The state vector |ψ⟩ is represented as a vector of 2^n complex amplitudes:
//! |ψ⟩ = Σᵢ αᵢ|i⟩ where i ranges from 0 to 2^n - 1.
//!
//! The index i corresponds to the binary representation of qubit states,
//! with qubit 0 being the least significant bit:
//! - |00⟩ = index 0
//! - |01⟩ = index 1 (qubit 0 = 1)
//! - |10⟩ = index 2 (qubit 1 = 1)
//! - |11⟩ = index 3
//!
//! ## Example
//!
//! ```rust
//! use qns_simulator::StateVectorSimulator;
//! use qns_core::prelude::*;
//!
//! // Create a 2-qubit simulator
//! let mut sim = StateVectorSimulator::new(2);
//!
//! // Create and execute a Bell state circuit
//! let mut circuit = CircuitGenome::new(2);
//! circuit.add_gate(Gate::H(0)).unwrap();
//! circuit.add_gate(Gate::CNOT(0, 1)).unwrap();
//!
//! sim.execute(&circuit).unwrap();
//!
//! // Measure multiple times
//! let results = sim.measure(1000).unwrap();
//! // Should get roughly equal |00⟩ and |11⟩
//! ```

use num_complex::Complex64;
use qns_core::physics::{
    rx, ry, rz, Matrix2x2, Matrix4x4, C64, CNOT, CZ, HADAMARD, ONE, PAULI_X, PAULI_Y, PAULI_Z,
    SWAP, S_GATE, T_GATE, ZERO,
};
use qns_core::prelude::*;
use rand::Rng;
use std::collections::HashMap;

/// State vector quantum simulator.
///
/// Simulates quantum circuits using full state vector representation.
/// Memory usage: O(2^n) complex numbers where n = number of qubits.
///
/// # Performance
///
/// - State vector size: 2^n * 16 bytes (Complex64)
/// - Single-qubit gate: O(2^n) operations
/// - Two-qubit gate: O(2^n) operations
/// - Measurement: O(2^n) for probability calculation, O(1) per sample
pub struct StateVectorSimulator {
    /// Number of qubits
    num_qubits: usize,
    /// State vector: amplitudes for each computational basis state
    state: Vec<C64>,
    /// Dimension (2^n)
    dimension: usize,
    /// Random number generator for measurements
    rng: rand::rngs::ThreadRng,
}

impl StateVectorSimulator {
    /// Creates a new simulator with the specified number of qubits.
    ///
    /// Initializes the state to |0...0⟩.
    ///
    /// # Panics
    ///
    /// Panics if num_qubits > 20 (would require > 16GB memory).
    pub fn new(num_qubits: usize) -> Self {
        assert!(
            num_qubits <= 20,
            "num_qubits {} exceeds limit 20",
            num_qubits
        );

        let dimension = 1 << num_qubits; // 2^n
        let mut state = vec![ZERO; dimension];
        state[0] = ONE; // |0...0⟩

        Self {
            num_qubits,
            state,
            dimension,
            rng: rand::thread_rng(),
        }
    }

    /// Returns the number of qubits.
    pub fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    /// Returns the dimension of the state space (2^n).
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Resets the simulator to initial state |0...0⟩.
    pub fn reset(&mut self) {
        self.state.fill(ZERO);
        self.state[0] = ONE;
    }

    /// Returns a reference to the state vector.
    pub fn statevector(&self) -> &[C64] {
        &self.state
    }

    /// Returns the amplitude for a specific basis state.
    pub fn amplitude(&self, index: usize) -> C64 {
        if index < self.dimension {
            self.state[index]
        } else {
            ZERO
        }
    }

    /// Sets the state vector directly (for testing/initialization).
    ///
    /// # Errors
    ///
    /// Returns error if the state is not normalized or wrong dimension.
    pub fn set_state(&mut self, state: Vec<C64>) -> Result<()> {
        if state.len() != self.dimension {
            return Err(QnsError::DimensionMismatch(self.dimension, state.len()));
        }

        // Check normalization
        let norm_sq: f64 = state.iter().map(|a| a.norm_sqr()).sum();
        if (norm_sq - 1.0).abs() > 1e-6 {
            return Err(QnsError::InvalidState(format!(
                "State not normalized: |ψ|² = {}",
                norm_sq
            )));
        }

        self.state = state;
        Ok(())
    }

    /// Applies a single-qubit gate to the specified qubit.
    ///
    /// Uses efficient indexing to update only affected amplitudes.
    fn apply_single_qubit_gate(&mut self, qubit: usize, matrix: &Matrix2x2) {
        let mask = 1 << qubit;

        // Iterate over pairs of basis states differing only in target qubit
        for i in 0..self.dimension {
            if (i & mask) == 0 {
                // i has qubit=0, j has qubit=1
                let j = i | mask;

                let a0 = self.state[i];
                let a1 = self.state[j];

                // Apply 2x2 matrix: [new_a0, new_a1] = matrix * [a0, a1]
                self.state[i] = matrix[0][0] * a0 + matrix[0][1] * a1;
                self.state[j] = matrix[1][0] * a0 + matrix[1][1] * a1;
            }
        }
    }

    /// Applies a two-qubit gate to the specified qubits.
    ///
    /// qubit1 is the control (or first qubit), qubit2 is target (or second).
    fn apply_two_qubit_gate(&mut self, qubit1: usize, qubit2: usize, matrix: &Matrix4x4) {
        let mask1 = 1 << qubit1;
        let mask2 = 1 << qubit2;
        let mask_both = mask1 | mask2;

        // Iterate over groups of 4 basis states differing in both qubits
        for base in 0..self.dimension {
            if (base & mask_both) == 0 {
                // Indices for the 4 basis states
                let i00 = base; // qubit1=0, qubit2=0
                let i01 = base | mask2; // qubit1=0, qubit2=1
                let i10 = base | mask1; // qubit1=1, qubit2=0
                let i11 = base | mask_both; // qubit1=1, qubit2=1

                // Get current amplitudes
                let a00 = self.state[i00];
                let a01 = self.state[i01];
                let a10 = self.state[i10];
                let a11 = self.state[i11];

                // Apply 4x4 matrix
                // Note: Matrix ordering is [00, 01, 10, 11]
                self.state[i00] = matrix[0][0] * a00
                    + matrix[0][1] * a01
                    + matrix[0][2] * a10
                    + matrix[0][3] * a11;
                self.state[i01] = matrix[1][0] * a00
                    + matrix[1][1] * a01
                    + matrix[1][2] * a10
                    + matrix[1][3] * a11;
                self.state[i10] = matrix[2][0] * a00
                    + matrix[2][1] * a01
                    + matrix[2][2] * a10
                    + matrix[2][3] * a11;
                self.state[i11] = matrix[3][0] * a00
                    + matrix[3][1] * a01
                    + matrix[3][2] * a10
                    + matrix[3][3] * a11;
            }
        }
    }

    /// Applies a gate from the Gate enum.
    pub fn apply_gate(&mut self, gate: &Gate) -> Result<()> {
        match gate {
            // Single-qubit gates
            Gate::H(q) => {
                self.validate_qubit(*q)?;
                self.apply_single_qubit_gate(*q, &HADAMARD);
            },
            Gate::X(q) => {
                self.validate_qubit(*q)?;
                self.apply_single_qubit_gate(*q, &PAULI_X);
            },
            Gate::Y(q) => {
                self.validate_qubit(*q)?;
                self.apply_single_qubit_gate(*q, &PAULI_Y);
            },
            Gate::Z(q) => {
                self.validate_qubit(*q)?;
                self.apply_single_qubit_gate(*q, &PAULI_Z);
            },
            Gate::S(q) => {
                self.validate_qubit(*q)?;
                self.apply_single_qubit_gate(*q, &S_GATE);
            },
            Gate::T(q) => {
                self.validate_qubit(*q)?;
                self.apply_single_qubit_gate(*q, &T_GATE);
            },
            Gate::Rx(q, theta) => {
                self.validate_qubit(*q)?;
                self.apply_single_qubit_gate(*q, &rx(*theta));
            },
            Gate::Ry(q, theta) => {
                self.validate_qubit(*q)?;
                self.apply_single_qubit_gate(*q, &ry(*theta));
            },
            Gate::Rz(q, theta) => {
                self.validate_qubit(*q)?;
                self.apply_single_qubit_gate(*q, &rz(*theta));
            },

            // Two-qubit gates
            Gate::CNOT(ctrl, tgt) => {
                self.validate_qubit(*ctrl)?;
                self.validate_qubit(*tgt)?;
                self.apply_two_qubit_gate(*ctrl, *tgt, &CNOT);
            },
            Gate::CZ(q1, q2) => {
                self.validate_qubit(*q1)?;
                self.validate_qubit(*q2)?;
                self.apply_two_qubit_gate(*q1, *q2, &CZ);
            },
            Gate::SWAP(q1, q2) => {
                self.validate_qubit(*q1)?;
                self.validate_qubit(*q2)?;
                self.apply_two_qubit_gate(*q1, *q2, &SWAP);
            },

            // Measurement is handled separately
            Gate::Measure(_) => {
                // Measurement collapses state - handled in measure()
                // For now, we skip measurement gates during execution
            },
        }

        Ok(())
    }

    /// Validates that a qubit index is within range.
    fn validate_qubit(&self, qubit: usize) -> Result<()> {
        if qubit >= self.num_qubits {
            Err(QnsError::InvalidQubit(qubit, self.num_qubits))
        } else {
            Ok(())
        }
    }

    /// Executes a quantum circuit.
    pub fn execute(&mut self, circuit: &CircuitGenome) -> Result<()> {
        if circuit.num_qubits != self.num_qubits {
            return Err(QnsError::DimensionMismatch(
                self.num_qubits,
                circuit.num_qubits,
            ));
        }

        for gate in &circuit.gates {
            self.apply_gate(gate)?;
        }

        Ok(())
    }

    /// Executes a circuit after resetting to |0...0⟩.
    pub fn run(&mut self, circuit: &CircuitGenome) -> Result<()> {
        self.reset();
        self.execute(circuit)
    }

    /// Calculates the probability distribution over computational basis states.
    ///
    /// Returns a vector where probabilities[i] = |amplitude[i]|².
    pub fn probabilities(&self) -> Vec<f64> {
        self.state.iter().map(|a| a.norm_sqr()).collect()
    }

    /// Returns the probability of measuring a specific basis state.
    pub fn probability(&self, index: usize) -> f64 {
        if index < self.dimension {
            self.state[index].norm_sqr()
        } else {
            0.0
        }
    }

    /// Performs measurement and returns outcome statistics.
    ///
    /// Samples from the probability distribution `shots` times.
    /// Does NOT collapse the state (non-destructive measurement).
    ///
    /// # Returns
    ///
    /// HashMap mapping bit strings (e.g., "01", "10") to count of occurrences.
    pub fn measure(&mut self, shots: usize) -> Result<HashMap<String, usize>> {
        let probs = self.probabilities();
        let mut results: HashMap<String, usize> = HashMap::new();

        for _ in 0..shots {
            let outcome = self.sample_outcome(&probs);
            let bitstring = self.index_to_bitstring(outcome);
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

        // Fallback (should not reach here for normalized states)
        self.dimension - 1
    }

    /// Converts a basis state index to a bit string.
    ///
    /// Qubit 0 is the rightmost (least significant) bit.
    fn index_to_bitstring(&self, index: usize) -> String {
        (0..self.num_qubits)
            .rev()
            .map(|q| if (index >> q) & 1 == 1 { '1' } else { '0' })
            .collect()
    }

    /// Converts a bit string to a basis state index.
    pub fn bitstring_to_index(&self, bitstring: &str) -> Option<usize> {
        if bitstring.len() != self.num_qubits {
            return None;
        }

        let mut index = 0;
        for (i, c) in bitstring.chars().enumerate() {
            match c {
                '1' => index |= 1 << (self.num_qubits - 1 - i),
                '0' => {},
                _ => return None,
            }
        }

        Some(index)
    }

    /// Performs a single projective measurement and collapses the state.
    ///
    /// Returns the measured bit string.
    pub fn measure_and_collapse(&mut self) -> String {
        let probs = self.probabilities();
        let outcome = self.sample_outcome(&probs);

        // Collapse to measured state
        self.state.fill(ZERO);
        self.state[outcome] = ONE;

        self.index_to_bitstring(outcome)
    }

    /// Measures a single qubit and collapses the state.
    ///
    /// Returns the measurement result (0 or 1).
    pub fn measure_qubit(&mut self, qubit: usize) -> Result<u8> {
        self.validate_qubit(qubit)?;

        let mask = 1 << qubit;

        // Calculate probability of measuring 0
        let prob_0: f64 = self
            .state
            .iter()
            .enumerate()
            .filter(|(i, _)| (i & mask) == 0)
            .map(|(_, a)| a.norm_sqr())
            .sum();

        let result = if self.rng.gen::<f64>() < prob_0 { 0 } else { 1 };

        // Collapse and renormalize
        let mut new_norm_sq = 0.0;
        for i in 0..self.dimension {
            if ((i >> qubit) & 1) != result as usize {
                self.state[i] = ZERO;
            } else {
                new_norm_sq += self.state[i].norm_sqr();
            }
        }

        // Renormalize
        if new_norm_sq > 1e-15 {
            let norm_factor = Complex64::new(1.0 / new_norm_sq.sqrt(), 0.0);
            for a in &mut self.state {
                *a *= norm_factor;
            }
        }

        Ok(result)
    }

    /// Calculates the fidelity between the current state and a target state.
    ///
    /// Fidelity F = |⟨ψ|φ⟩|² where ψ is current state and φ is target.
    pub fn fidelity(&self, target: &[C64]) -> Result<f64> {
        if target.len() != self.dimension {
            return Err(QnsError::DimensionMismatch(self.dimension, target.len()));
        }

        // Inner product ⟨ψ|φ⟩ = Σ ψᵢ* φᵢ
        let inner: C64 = self
            .state
            .iter()
            .zip(target.iter())
            .map(|(a, b)| a.conj() * b)
            .sum();

        Ok(inner.norm_sqr())
    }

    /// Calculates fidelity with another simulator's state.
    pub fn fidelity_with(&self, other: &Self) -> Result<f64> {
        if self.dimension != other.dimension {
            return Err(QnsError::DimensionMismatch(self.dimension, other.dimension));
        }
        self.fidelity(&other.state)
    }

    /// Creates a Bell state (|00⟩ + |11⟩)/√2 on qubits 0 and 1.
    pub fn prepare_bell_state(&mut self) -> Result<()> {
        if self.num_qubits < 2 {
            return Err(QnsError::InvalidQubit(1, self.num_qubits));
        }

        self.reset();
        self.apply_single_qubit_gate(0, &HADAMARD);
        self.apply_two_qubit_gate(0, 1, &CNOT);

        Ok(())
    }

    /// Creates a GHZ state (|00...0⟩ + |11...1⟩)/√2 on all qubits.
    pub fn prepare_ghz_state(&mut self) -> Result<()> {
        self.reset();
        self.apply_single_qubit_gate(0, &HADAMARD);

        for q in 1..self.num_qubits {
            self.apply_two_qubit_gate(0, q, &CNOT);
        }

        Ok(())
    }

    /// Checks if the state is normalized (|ψ|² ≈ 1).
    pub fn is_normalized(&self) -> bool {
        let norm_sq: f64 = self.state.iter().map(|a| a.norm_sqr()).sum();
        (norm_sq - 1.0).abs() < 1e-10
    }

    /// Returns the expectation value of Z operator on specified qubit.
    ///
    /// ⟨Z⟩ = P(0) - P(1)
    pub fn expectation_z(&self, qubit: usize) -> Result<f64> {
        self.validate_qubit(qubit)?;

        let mask = 1 << qubit;
        let mut exp_val = 0.0;

        for (i, a) in self.state.iter().enumerate() {
            let prob = a.norm_sqr();
            if (i & mask) == 0 {
                exp_val += prob; // qubit is |0⟩, eigenvalue +1
            } else {
                exp_val -= prob; // qubit is |1⟩, eigenvalue -1
            }
        }

        Ok(exp_val)
    }
}

impl Clone for StateVectorSimulator {
    fn clone(&self) -> Self {
        Self {
            num_qubits: self.num_qubits,
            state: self.state.clone(),
            dimension: self.dimension,
            rng: rand::thread_rng(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    const TOLERANCE: f64 = 1e-10;

    #[test]
    fn test_new() {
        let sim = StateVectorSimulator::new(3);
        assert_eq!(sim.num_qubits(), 3);
        assert_eq!(sim.dimension(), 8);

        // Initial state should be |000⟩
        assert!((sim.amplitude(0) - ONE).norm() < TOLERANCE);
        for i in 1..8 {
            assert!(sim.amplitude(i).norm() < TOLERANCE);
        }
    }

    #[test]
    fn test_reset() {
        let mut sim = StateVectorSimulator::new(2);
        sim.apply_gate(&Gate::X(0)).unwrap();

        // State should be |01⟩ (index 1)
        assert!(sim.amplitude(1).norm() > 0.9);

        sim.reset();

        // Should be back to |00⟩
        assert!((sim.amplitude(0) - ONE).norm() < TOLERANCE);
    }

    #[test]
    fn test_hadamard() {
        let mut sim = StateVectorSimulator::new(1);
        sim.apply_gate(&Gate::H(0)).unwrap();

        // State should be (|0⟩ + |1⟩)/√2
        let expected = Complex64::new(std::f64::consts::FRAC_1_SQRT_2, 0.0);
        assert!((sim.amplitude(0) - expected).norm() < TOLERANCE);
        assert!((sim.amplitude(1) - expected).norm() < TOLERANCE);
    }

    #[test]
    fn test_pauli_x() {
        let mut sim = StateVectorSimulator::new(1);
        sim.apply_gate(&Gate::X(0)).unwrap();

        // |0⟩ -> |1⟩
        assert!(sim.amplitude(0).norm() < TOLERANCE);
        assert!((sim.amplitude(1) - ONE).norm() < TOLERANCE);
    }

    #[test]
    fn test_pauli_z() {
        let mut sim = StateVectorSimulator::new(1);

        // Put in superposition first
        sim.apply_gate(&Gate::H(0)).unwrap();
        sim.apply_gate(&Gate::Z(0)).unwrap();

        // Z|+⟩ = |−⟩ = (|0⟩ - |1⟩)/√2
        let expected_0 = Complex64::new(std::f64::consts::FRAC_1_SQRT_2, 0.0);
        let expected_1 = Complex64::new(-std::f64::consts::FRAC_1_SQRT_2, 0.0);

        assert!((sim.amplitude(0) - expected_0).norm() < TOLERANCE);
        assert!((sim.amplitude(1) - expected_1).norm() < TOLERANCE);
    }

    #[test]
    fn test_cnot() {
        // Test CNOT creates entanglement: H|0⟩ ⊗ |0⟩ -> Bell state
        let mut sim = StateVectorSimulator::new(2);
        sim.apply_gate(&Gate::H(0)).unwrap();
        sim.apply_gate(&Gate::CNOT(0, 1)).unwrap();

        // Bell state: (|00⟩ + |11⟩)/√2
        let expected = Complex64::new(std::f64::consts::FRAC_1_SQRT_2, 0.0);

        assert!((sim.amplitude(0) - expected).norm() < TOLERANCE); // |00⟩
        assert!(sim.amplitude(1).norm() < TOLERANCE); // |01⟩
        assert!(sim.amplitude(2).norm() < TOLERANCE); // |10⟩
        assert!((sim.amplitude(3) - expected).norm() < TOLERANCE); // |11⟩
    }

    #[test]
    fn test_bell_state() {
        let mut sim = StateVectorSimulator::new(2);
        sim.prepare_bell_state().unwrap();

        let probs = sim.probabilities();

        // Should have equal probability for |00⟩ and |11⟩
        assert!((probs[0] - 0.5).abs() < TOLERANCE); // |00⟩
        assert!(probs[1] < TOLERANCE); // |01⟩
        assert!(probs[2] < TOLERANCE); // |10⟩
        assert!((probs[3] - 0.5).abs() < TOLERANCE); // |11⟩
    }

    #[test]
    fn test_ghz_state() {
        let mut sim = StateVectorSimulator::new(3);
        sim.prepare_ghz_state().unwrap();

        let probs = sim.probabilities();

        // Should have equal probability for |000⟩ and |111⟩
        assert!((probs[0] - 0.5).abs() < TOLERANCE); // |000⟩
        assert!((probs[7] - 0.5).abs() < TOLERANCE); // |111⟩

        // All others should be zero
        for prob in probs.iter().take(7).skip(1) {
            assert!(*prob < TOLERANCE);
        }
    }

    #[test]
    fn test_rotation_gates() {
        // Rx(π) = -iX (up to global phase, acts like X)
        let mut sim = StateVectorSimulator::new(1);
        sim.apply_gate(&Gate::Rx(0, PI)).unwrap();

        // |0⟩ -> -i|1⟩
        assert!(sim.amplitude(0).norm() < TOLERANCE);
        assert!((sim.amplitude(1).norm() - 1.0).abs() < TOLERANCE);

        // Rz(π) = -iZ
        let mut sim2 = StateVectorSimulator::new(1);
        sim2.apply_gate(&Gate::H(0)).unwrap();
        sim2.apply_gate(&Gate::Rz(0, PI)).unwrap();

        // Should be normalized
        assert!(sim2.is_normalized());
    }

    #[test]
    fn test_execute() {
        let mut sim = StateVectorSimulator::new(2);
        let mut circuit = CircuitGenome::new(2);

        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();

        sim.execute(&circuit).unwrap();

        // Should be Bell state
        let probs = sim.probabilities();
        assert!((probs[0] - 0.5).abs() < TOLERANCE);
        assert!((probs[3] - 0.5).abs() < TOLERANCE);
    }

    #[test]
    fn test_dimension_mismatch() {
        let mut sim = StateVectorSimulator::new(2);
        let circuit = CircuitGenome::new(3);

        assert!(sim.execute(&circuit).is_err());
    }

    #[test]
    fn test_measurement_statistics() {
        let mut sim = StateVectorSimulator::new(1);
        sim.apply_gate(&Gate::H(0)).unwrap();

        // Measure many times
        let results = sim.measure(10000).unwrap();

        // Should get roughly equal 0s and 1s
        let count_0 = results.get("0").copied().unwrap_or(0);
        let count_1 = results.get("1").copied().unwrap_or(0);

        // Within 5% of expected (very loose for statistical test)
        assert!(count_0 > 4000 && count_0 < 6000);
        assert!(count_1 > 4000 && count_1 < 6000);
    }

    #[test]
    fn test_fidelity_same_state() {
        let sim = StateVectorSimulator::new(2);
        let target = sim.statevector().to_vec();

        let fidelity = sim.fidelity(&target).unwrap();
        assert!((fidelity - 1.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_fidelity_orthogonal() {
        let sim = StateVectorSimulator::new(1);

        // |0⟩ and |1⟩ are orthogonal
        let target = vec![ZERO, ONE];

        let fidelity = sim.fidelity(&target).unwrap();
        assert!(fidelity < TOLERANCE);
    }

    #[test]
    fn test_bitstring_conversion() {
        let sim = StateVectorSimulator::new(3);

        // Test round-trip
        for i in 0..8 {
            let bs = sim.index_to_bitstring(i);
            let idx = sim.bitstring_to_index(&bs).unwrap();
            assert_eq!(i, idx);
        }

        // Specific test: |101⟩ = qubit2=1, qubit1=0, qubit0=1 = index 5
        assert_eq!(sim.bitstring_to_index("101"), Some(5));
    }

    #[test]
    fn test_measure_qubit() {
        let mut sim = StateVectorSimulator::new(2);
        sim.prepare_bell_state().unwrap();

        // Measure qubit 0
        let result = sim.measure_qubit(0).unwrap();

        // After measurement, state should be collapsed
        // If result is 0, state is |00⟩; if 1, state is |11⟩
        if result == 0 {
            assert!((sim.amplitude(0) - ONE).norm() < TOLERANCE);
        } else {
            assert!((sim.amplitude(3) - ONE).norm() < TOLERANCE);
        }
    }

    #[test]
    fn test_expectation_z() {
        let sim = StateVectorSimulator::new(1);

        // |0⟩ has ⟨Z⟩ = +1
        let exp = sim.expectation_z(0).unwrap();
        assert!((exp - 1.0).abs() < TOLERANCE);

        // |1⟩ has ⟨Z⟩ = -1
        let mut sim2 = StateVectorSimulator::new(1);
        sim2.apply_gate(&Gate::X(0)).unwrap();
        let exp2 = sim2.expectation_z(0).unwrap();
        assert!((exp2 - (-1.0)).abs() < TOLERANCE);

        // |+⟩ has ⟨Z⟩ = 0
        let mut sim3 = StateVectorSimulator::new(1);
        sim3.apply_gate(&Gate::H(0)).unwrap();
        let exp3 = sim3.expectation_z(0).unwrap();
        assert!(exp3.abs() < TOLERANCE);
    }

    #[test]
    fn test_swap_gate() {
        let mut sim = StateVectorSimulator::new(2);

        // Prepare |01⟩ (qubit0=1, qubit1=0)
        sim.apply_gate(&Gate::X(0)).unwrap();

        // SWAP should give |10⟩
        sim.apply_gate(&Gate::SWAP(0, 1)).unwrap();

        assert!((sim.amplitude(2) - ONE).norm() < TOLERANCE); // |10⟩ = index 2
    }

    #[test]
    fn test_cz_gate() {
        let mut sim = StateVectorSimulator::new(2);

        // Prepare |11⟩
        sim.apply_gate(&Gate::X(0)).unwrap();
        sim.apply_gate(&Gate::X(1)).unwrap();

        // CZ adds phase -1 to |11⟩
        sim.apply_gate(&Gate::CZ(0, 1)).unwrap();

        let expected = Complex64::new(-1.0, 0.0);
        assert!((sim.amplitude(3) - expected).norm() < TOLERANCE);
    }

    #[test]
    fn test_normalization_preserved() {
        let mut sim = StateVectorSimulator::new(3);

        // Apply many gates
        for _ in 0..10 {
            sim.apply_gate(&Gate::H(0)).unwrap();
            sim.apply_gate(&Gate::CNOT(0, 1)).unwrap();
            sim.apply_gate(&Gate::T(2)).unwrap();
            sim.apply_gate(&Gate::Rz(1, 0.5)).unwrap();
        }

        assert!(sim.is_normalized());
    }

    #[test]
    fn test_clone() {
        let mut sim = StateVectorSimulator::new(2);
        sim.prepare_bell_state().unwrap();

        let sim2 = sim.clone();

        let fidelity = sim.fidelity_with(&sim2).unwrap();
        assert!((fidelity - 1.0).abs() < TOLERANCE);
    }
}
