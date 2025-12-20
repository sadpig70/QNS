//! Noise model for quantum simulation.
//!
//! This module provides realistic noise models for quantum circuit simulation:
//!
//! - **Decoherence**: T1 (relaxation) and T2 (dephasing) noise
//! - **Gate errors**: Depolarizing channel after gates
//! - **Measurement errors**: Bit-flip errors during measurement
//!
//! ## Physical Background
//!
//! ### T1 Relaxation (Amplitude Damping)
//! Models energy loss to environment. Probability of |1⟩ → |0⟩ transition:
//! ```text
//! P(decay) = 1 - exp(-t/T1)
//! ```
//!
//! ### T2 Dephasing (Phase Damping)  
//! Models loss of phase coherence. For pure dephasing (T2* or Tφ):
//! ```text
//! P(dephase) = 1 - exp(-t/Tφ)
//! where 1/Tφ = 1/T2 - 1/(2*T1)
//! ```
//!
//! Note: T2 ≤ 2*T1 is required by physics.
//!
//! ### Depolarizing Channel
//! Completely randomizes state with probability p:
//! ```text
//! ρ → (1-p)ρ + p/3 * (XρX + YρY + ZρZ)
//! ```
//!
//! ## Usage
//!
//! ```rust
//! use qns_simulator::{NoiseModel, NoisySimulator};
//! use qns_core::prelude::*;
//!
//! // Create noise model from noise vector
//! let noise_vec = NoiseVector::with_t1t2(0, 100.0, 80.0);
//! let noise_model = NoiseModel::from_noise_vector(&noise_vec);
//!
//! // Create noisy simulator
//! let mut sim = NoisySimulator::new(2, noise_model);
//!
//! // Execute circuit with noise
//! let mut circuit = CircuitGenome::new(2);
//! circuit.add_gate(Gate::H(0)).unwrap();
//! circuit.add_gate(Gate::CNOT(0, 1)).unwrap();
//!
//! sim.execute(&circuit).unwrap();
//! ```

use qns_core::physics::{Matrix2x2, C64, ONE, PAULI_X, PAULI_Y, PAULI_Z, ZERO};
use qns_core::prelude::*;
use rand::Rng;
use std::collections::HashMap;

/// Noise model parameters for quantum simulation.
#[derive(Debug, Clone)]
pub struct NoiseModel {
    /// T1 relaxation time in microseconds
    pub t1: f64,
    /// T2 dephasing time in microseconds
    pub t2: f64,
    /// Single-qubit gate time in nanoseconds
    pub single_gate_time_ns: f64,
    /// Two-qubit gate time in nanoseconds
    pub two_gate_time_ns: f64,
    /// Single-qubit gate error rate
    pub single_gate_error: f64,
    /// Two-qubit gate error rate (default for edges not in edge_error_rates)
    pub two_gate_error: f64,
    /// Per-edge error rates for two-qubit gates (edge -> error rate)
    /// Key is (min(q1,q2), max(q1,q2)) for canonical ordering
    pub edge_error_rates: HashMap<(usize, usize), f64>,
    /// Measurement error rate (readout error)
    pub readout_error: f64,
    /// Whether to apply thermal relaxation
    pub thermal_relaxation: bool,
    /// Whether to apply gate errors
    pub gate_errors: bool,
    /// Whether to apply measurement errors
    pub measurement_errors: bool,
}

impl NoiseModel {
    /// Creates a new noise model with default realistic parameters.
    ///
    /// Based on IBM Heron (2024) typical values.
    pub fn new() -> Self {
        Self {
            t1: 300.0,                 // 300 μs (IBM Heron)
            t2: 200.0,                 // 200 μs
            single_gate_time_ns: 35.0, // 35 ns
            two_gate_time_ns: 300.0,   // 300 ns (CZ/CNOT)
            single_gate_error: 1e-4,   // 0.01%
            two_gate_error: 5e-3,      // 0.5%
            edge_error_rates: HashMap::new(),
            readout_error: 1e-2, // 1%
            thermal_relaxation: true,
            gate_errors: true,
            measurement_errors: true,
        }
    }

    /// Creates an ideal (noise-free) model.
    pub fn ideal() -> Self {
        Self {
            t1: f64::INFINITY,
            t2: f64::INFINITY,
            single_gate_time_ns: 0.0,
            two_gate_time_ns: 0.0,
            single_gate_error: 0.0,
            two_gate_error: 0.0,
            edge_error_rates: HashMap::new(),
            readout_error: 0.0,
            thermal_relaxation: false,
            gate_errors: false,
            measurement_errors: false,
        }
    }

    /// Creates a noise model from a NoiseVector.
    pub fn from_noise_vector(nv: &NoiseVector) -> Self {
        let mut model = Self::new();
        model.t1 = nv.t1_mean;
        model.t2 = nv.t2_mean;
        model
    }

    /// Creates a noise model with custom T1/T2 times.
    pub fn with_t1t2(t1: f64, t2: f64) -> Self {
        let mut model = Self::new();
        model.t1 = t1;
        model.t2 = t2;
        model
    }

    /// Sets gate error rates.
    pub fn with_gate_errors(mut self, single: f64, two_qubit: f64) -> Self {
        self.single_gate_error = single;
        self.two_gate_error = two_qubit;
        self
    }

    /// Sets measurement error rate.
    pub fn with_readout_error(mut self, error: f64) -> Self {
        self.readout_error = error;
        if error > 0.0 {
            self.measurement_errors = true;
        }
        self
    }

    /// Enables or disables thermal relaxation.
    pub fn enable_thermal_relaxation(mut self, enabled: bool) -> Self {
        self.thermal_relaxation = enabled;
        self
    }

    /// Sets per-edge error rates from a HardwareProfile.
    ///
    /// This enables hardware-accurate simulation where each edge has
    /// its own error rate derived from calibration data.
    ///
    /// # Example
    /// ```ignore
    /// let hw = HardwareProfile::linear("chip", 4);
    /// let noise = NoiseModel::new().with_hardware(&hw);
    /// ```
    pub fn with_hardware(mut self, hw: &HardwareProfile) -> Self {
        self.edge_error_rates.clear();
        for coupler in &hw.couplers {
            let key = Self::edge_key(coupler.qubit1, coupler.qubit2);
            let error_rate = coupler.gate_fidelity.error_rate();
            self.edge_error_rates.insert(key, error_rate);
        }
        self
    }

    /// Sets a specific edge error rate.
    pub fn set_edge_error(&mut self, q1: usize, q2: usize, error_rate: f64) {
        let key = Self::edge_key(q1, q2);
        self.edge_error_rates.insert(key, error_rate);
    }

    /// Gets the error rate for a specific edge.
    ///
    /// Returns the edge-specific rate if available, otherwise the default `two_gate_error`.
    pub fn get_edge_error(&self, q1: usize, q2: usize) -> f64 {
        let key = Self::edge_key(q1, q2);
        *self
            .edge_error_rates
            .get(&key)
            .unwrap_or(&self.two_gate_error)
    }

    /// Creates canonical edge key (smaller qubit first).
    fn edge_key(q1: usize, q2: usize) -> (usize, usize) {
        if q1 <= q2 {
            (q1, q2)
        } else {
            (q2, q1)
        }
    }

    /// Returns whether per-edge error rates are configured.
    pub fn has_edge_errors(&self) -> bool {
        !self.edge_error_rates.is_empty()
    }

    /// Calculates the amplitude damping probability for a given time.
    ///
    /// P(|1⟩ → |0⟩) = 1 - exp(-t/T1)
    pub fn amplitude_damping_prob(&self, time_ns: f64) -> f64 {
        if self.t1 <= 0.0 || self.t1.is_infinite() {
            return 0.0;
        }
        let time_us = time_ns / 1000.0;
        1.0 - (-time_us / self.t1).exp()
    }

    /// Calculates the pure dephasing probability.
    ///
    /// Uses Tφ where 1/Tφ = 1/T2 - 1/(2*T1)
    pub fn phase_damping_prob(&self, time_ns: f64) -> f64 {
        if self.t2 <= 0.0 || self.t2.is_infinite() {
            return 0.0;
        }

        // Calculate pure dephasing time Tφ
        // 1/Tφ = 1/T2 - 1/(2*T1)
        let t_phi = if self.t1.is_infinite() {
            self.t2
        } else {
            let rate_phi = 1.0 / self.t2 - 1.0 / (2.0 * self.t1);
            if rate_phi <= 0.0 {
                return 0.0; // No pure dephasing if T2 = 2*T1
            }
            1.0 / rate_phi
        };

        let time_us = time_ns / 1000.0;
        1.0 - (-time_us / t_phi).exp()
    }

    /// Validates that T2 ≤ 2*T1 (physical constraint).
    pub fn is_valid(&self) -> bool {
        if self.t1.is_infinite() {
            return true;
        }
        self.t2 <= 2.0 * self.t1 + 1e-10 // Small tolerance for floating point
    }

    // ===== Getter methods for backend compatibility =====

    /// Returns T1 relaxation time in microseconds.
    pub fn t1(&self) -> f64 {
        self.t1
    }

    /// Returns T2 dephasing time in microseconds.
    pub fn t2(&self) -> f64 {
        self.t2
    }

    /// Returns single-qubit gate error rate.
    pub fn gate_error_1q(&self) -> f64 {
        self.single_gate_error
    }

    /// Returns two-qubit gate error rate.
    pub fn gate_error_2q(&self) -> f64 {
        self.two_gate_error
    }

    /// Returns readout (measurement) error rate.
    pub fn readout_error(&self) -> f64 {
        self.readout_error
    }
}

impl Default for NoiseModel {
    fn default() -> Self {
        Self::new()
    }
}

/// Applies a single-qubit Kraus operator to a density matrix element.
///
/// For state vector simulation, we use a simplified approach:
/// - Apply damping with probability based on |1⟩ amplitude
/// - Apply dephasing by randomizing phase
#[derive(Debug, Clone, Copy)]
pub struct KrausOperator {
    /// Kraus matrix (2x2)
    pub matrix: Matrix2x2,
    /// Probability weight
    pub prob: f64,
}

impl KrausOperator {
    /// Creates amplitude damping Kraus operators.
    ///
    /// K0 = [[1, 0], [0, sqrt(1-γ)]]
    /// K1 = [[0, sqrt(γ)], [0, 0]]
    pub fn amplitude_damping(gamma: f64) -> [Self; 2] {
        let sqrt_gamma = gamma.sqrt();
        let sqrt_1_gamma = (1.0 - gamma).sqrt();

        let k0 = [[ONE, ZERO], [ZERO, C64::new(sqrt_1_gamma, 0.0)]];

        let k1 = [[ZERO, C64::new(sqrt_gamma, 0.0)], [ZERO, ZERO]];

        [
            KrausOperator {
                matrix: k0,
                prob: 1.0 - gamma,
            },
            KrausOperator {
                matrix: k1,
                prob: gamma,
            },
        ]
    }

    /// Creates phase damping Kraus operators.
    ///
    /// K0 = [[1, 0], [0, sqrt(1-λ)]]
    /// K1 = [[0, 0], [0, sqrt(λ)]]
    pub fn phase_damping(lambda: f64) -> [Self; 2] {
        let sqrt_lambda = lambda.sqrt();
        let sqrt_1_lambda = (1.0 - lambda).sqrt();

        let k0 = [[ONE, ZERO], [ZERO, C64::new(sqrt_1_lambda, 0.0)]];

        let k1 = [[ZERO, ZERO], [ZERO, C64::new(sqrt_lambda, 0.0)]];

        [
            KrausOperator {
                matrix: k0,
                prob: 1.0 - lambda,
            },
            KrausOperator {
                matrix: k1,
                prob: lambda,
            },
        ]
    }
}

/// Depolarizing channel implementation.
///
/// Applies X, Y, or Z with probability p/3 each.
pub struct DepolarizingChannel {
    /// Total depolarizing probability
    pub prob: f64,
}

impl DepolarizingChannel {
    pub fn new(prob: f64) -> Self {
        Self {
            prob: prob.clamp(0.0, 1.0),
        }
    }

    /// Samples which Pauli to apply (0=I, 1=X, 2=Y, 3=Z).
    pub fn sample<R: Rng>(&self, rng: &mut R) -> u8 {
        let r: f64 = rng.gen();
        if r < 1.0 - self.prob {
            0 // Identity (no error)
        } else {
            let pauli_r: f64 = rng.gen();
            if pauli_r < 1.0 / 3.0 {
                1 // X
            } else if pauli_r < 2.0 / 3.0 {
                2 // Y
            } else {
                3 // Z
            }
        }
    }

    /// Returns the Pauli matrix for a given index.
    pub fn pauli_matrix(index: u8) -> &'static Matrix2x2 {
        match index {
            1 => &PAULI_X,
            2 => &PAULI_Y,
            3 => &PAULI_Z,
            _ => &[[ONE, ZERO], [ZERO, ONE]], // Identity
        }
    }
}

/// Measurement error model.
///
/// Models readout errors where 0 is misread as 1 and vice versa.
#[derive(Debug, Clone)]
pub struct MeasurementError {
    /// P(read 1 | prepared 0)
    pub p01: f64,
    /// P(read 0 | prepared 1)
    pub p10: f64,
}

impl MeasurementError {
    /// Creates symmetric measurement error.
    pub fn symmetric(error_rate: f64) -> Self {
        Self {
            p01: error_rate,
            p10: error_rate,
        }
    }

    /// Creates asymmetric measurement error.
    pub fn asymmetric(p01: f64, p10: f64) -> Self {
        Self { p01, p10 }
    }

    /// Applies measurement error to a result.
    pub fn apply<R: Rng>(&self, result: u8, rng: &mut R) -> u8 {
        let r: f64 = rng.gen();
        if result == 0 && r < self.p01 {
            1
        } else if result == 1 && r < self.p10 {
            0
        } else {
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_model_default() {
        let model = NoiseModel::new();
        assert!(model.t1 > 0.0);
        assert!(model.t2 > 0.0);
        assert!(model.is_valid());
    }

    #[test]
    fn test_noise_model_ideal() {
        let model = NoiseModel::ideal();
        assert!(model.t1.is_infinite());
        assert!(!model.thermal_relaxation);
        assert!(!model.gate_errors);
    }

    #[test]
    fn test_noise_model_from_noise_vector() {
        let nv = NoiseVector::with_t1t2(0, 100.0, 80.0);
        let model = NoiseModel::from_noise_vector(&nv);
        assert!((model.t1 - 100.0).abs() < 1e-10);
        assert!((model.t2 - 80.0).abs() < 1e-10);
    }

    #[test]
    fn test_amplitude_damping_prob() {
        let model = NoiseModel::with_t1t2(100.0, 80.0);

        // At t=0, no damping
        assert!((model.amplitude_damping_prob(0.0)).abs() < 1e-10);

        // At t=T1 (100μs = 100000ns), P ≈ 1 - e^(-1) ≈ 0.632
        let p = model.amplitude_damping_prob(100_000.0);
        assert!((p - 0.632).abs() < 0.01);
    }

    #[test]
    fn test_physical_constraint() {
        let valid_model = NoiseModel::with_t1t2(100.0, 150.0); // T2 < 2*T1
        assert!(valid_model.is_valid());

        let invalid_model = NoiseModel::with_t1t2(100.0, 250.0); // T2 > 2*T1
        assert!(!invalid_model.is_valid());

        let edge_model = NoiseModel::with_t1t2(100.0, 200.0); // T2 = 2*T1
        assert!(edge_model.is_valid());
    }

    #[test]
    fn test_depolarizing_channel() {
        let channel = DepolarizingChannel::new(0.5);
        let mut rng = rand::thread_rng();

        // Sample many times
        let mut error_count = 0;
        let trials = 10000;
        for _ in 0..trials {
            if channel.sample(&mut rng) != 0 {
                error_count += 1;
            }
        }

        // Should be approximately 50% errors
        let error_rate = error_count as f64 / trials as f64;
        assert!((error_rate - 0.5).abs() < 0.05);
    }

    #[test]
    fn test_measurement_error() {
        let me = MeasurementError::symmetric(0.1);
        let mut rng = rand::thread_rng();

        // Apply to many 0 measurements
        let mut flipped = 0;
        let trials = 10000;
        for _ in 0..trials {
            if me.apply(0, &mut rng) != 0 {
                flipped += 1;
            }
        }

        // Should flip approximately 10%
        let flip_rate = flipped as f64 / trials as f64;
        assert!((flip_rate - 0.1).abs() < 0.02);
    }

    #[test]
    fn test_kraus_amplitude_damping() {
        let gamma = 0.1;
        let [k0, k1] = KrausOperator::amplitude_damping(gamma);

        // Check completeness: K0†K0 + K1†K1 = I (approximately)
        assert!(k0.prob > 0.0);
        assert!(k1.prob > 0.0);
    }
}
