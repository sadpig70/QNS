//! Noise vector data structure.
//!
//! This module provides the `NoiseVector` struct for representing qubit noise profiles,
//! including T1/T2 times, gate errors, and readout errors.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Source of noise data.
///
/// Indicates where the noise profile data originated from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum NoiseSource {
    /// Data from simulator noise model parameters (immediate access)
    #[default]
    Simulator,
    /// Data from hardware calibration API (1-4 hour refresh)
    Calibration,
    /// Data from stored calibration history (offline analysis)
    Historical,
    /// User-provided custom values
    Custom,
}

impl std::fmt::Display for NoiseSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NoiseSource::Simulator => write!(f, "Simulator"),
            NoiseSource::Calibration => write!(f, "Calibration"),
            NoiseSource::Historical => write!(f, "Historical"),
            NoiseSource::Custom => write!(f, "Custom"),
        }
    }
}

/// Noise profile vector for a qubit.
///
/// Contains T1/T2 statistics, gate errors, readout errors, and metadata.
///
/// # Data Sources
///
/// - **Simulator mode**: Values are read from NoiseModel parameters (μs latency)
/// - **Calibration mode**: Values are fetched from hardware calibration API (v2.0)
/// - **Historical mode**: Values are loaded from stored calibration records
///
/// # Example
///
/// ```
/// use qns_core::types::NoiseVector;
///
/// let mut nv = NoiseVector::new(0);
/// nv.t1_mean = 100.0;  // μs
/// nv.t2_mean = 80.0;   // μs
/// nv.gate_error_1q = 0.001;  // 0.1%
/// nv.gate_error_2q = 0.01;   // 1%
/// nv.readout_error = 0.02;   // 2%
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NoiseVector {
    // ===== Qubit Identification =====
    /// Qubit ID
    pub qubit_id: usize,

    // ===== T1/T2 Coherence Times =====
    /// Mean T1 relaxation time (μs)
    pub t1_mean: f64,
    /// Standard deviation of T1 (μs)
    pub t1_std: f64,
    /// Mean T2 dephasing time (μs)
    pub t2_mean: f64,
    /// Standard deviation of T2 (μs)
    pub t2_std: f64,

    // ===== Gate Errors (v2.0) =====
    /// Single-qubit gate error rate (0.0 - 1.0)
    pub gate_error_1q: f64,
    /// Two-qubit gate error rate (0.0 - 1.0)  
    pub gate_error_2q: f64,
    /// Readout (measurement) error rate (0.0 - 1.0)
    pub readout_error: f64,

    // ===== Drift Detection =====
    /// Drift rate (μs/hour)
    pub drift_rate: f64,
    /// Number of burst events detected
    pub burst_count: usize,

    // ===== Metadata =====
    /// Unix timestamp of data collection
    pub timestamp: u64,
    /// Number of samples collected (for statistics)
    pub sample_count: usize,
    /// Data source indicator
    pub source: NoiseSource,

    // ===== Optional Hardware Info =====
    /// Qubit frequency in GHz (if available)
    pub frequency: Option<f64>,
    /// Anharmonicity in MHz (if available)
    pub anharmonicity: Option<f64>,
}

impl NoiseVector {
    /// Creates a new NoiseVector for the specified qubit.
    pub fn new(qubit_id: usize) -> Self {
        Self {
            qubit_id,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            source: NoiseSource::Simulator,
            ..Default::default()
        }
    }

    /// Creates a NoiseVector with specified T1/T2 values.
    pub fn with_t1t2(qubit_id: usize, t1_mean: f64, t2_mean: f64) -> Self {
        Self {
            qubit_id,
            t1_mean,
            t2_mean,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            source: NoiseSource::Simulator,
            ..Default::default()
        }
    }

    /// Creates a comprehensive NoiseVector with all error rates.
    ///
    /// # Arguments
    /// * `qubit_id` - Qubit identifier
    /// * `t1` - T1 relaxation time in μs
    /// * `t2` - T2 dephasing time in μs  
    /// * `gate_error_1q` - Single-qubit gate error rate (0.0 - 1.0)
    /// * `gate_error_2q` - Two-qubit gate error rate (0.0 - 1.0)
    /// * `readout_error` - Measurement error rate (0.0 - 1.0)
    pub fn comprehensive(
        qubit_id: usize,
        t1: f64,
        t2: f64,
        gate_error_1q: f64,
        gate_error_2q: f64,
        readout_error: f64,
    ) -> Self {
        Self {
            qubit_id,
            t1_mean: t1,
            t2_mean: t2,
            gate_error_1q,
            gate_error_2q,
            readout_error,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            source: NoiseSource::Simulator,
            ..Default::default()
        }
    }

    /// Sets the data source.
    pub fn with_source(mut self, source: NoiseSource) -> Self {
        self.source = source;
        self
    }

    /// Checks if the noise profile indicates an anomaly.
    ///
    /// An anomaly is detected when:
    /// - drift_rate exceeds threshold_sigma * t1_std, or
    /// - burst_count > 0
    pub fn is_anomaly(&self, threshold_sigma: f64) -> bool {
        if self.t1_std > 0.0 {
            self.drift_rate > threshold_sigma * self.t1_std || self.burst_count > 0
        } else {
            self.burst_count > 0
        }
    }

    /// Returns the T2/T1 ratio (dephasing quality).
    ///
    /// For physical qubits, this ratio should be ≤ 2.0.
    pub fn t2_t1_ratio(&self) -> f64 {
        if self.t1_mean > 0.0 {
            self.t2_mean / self.t1_mean
        } else {
            0.0
        }
    }

    /// Calculates pure dephasing time Tφ.
    ///
    /// Tφ is defined by: 1/T2 = 1/(2*T1) + 1/Tφ
    /// Therefore: 1/Tφ = 1/T2 - 1/(2*T1)
    ///
    /// Returns `None` if:
    /// - T1 or T2 is zero or negative
    /// - T2 ≥ 2*T1 (no pure dephasing, T2 limited only by T1)
    pub fn t_phi(&self) -> Option<f64> {
        if self.t1_mean <= 0.0 || self.t2_mean <= 0.0 {
            return None;
        }

        // T2 limit from T1 alone
        let t2_limit = 2.0 * self.t1_mean;

        // If T2 >= 2*T1, there's no pure dephasing contribution
        // (T2 is limited only by T1 relaxation)
        if self.t2_mean >= t2_limit - 1e-10 {
            return None;
        }

        let inv_t_phi = 1.0 / self.t2_mean - 1.0 / t2_limit;

        if inv_t_phi > 1e-15 {
            Some(1.0 / inv_t_phi)
        } else {
            None
        }
    }

    /// Estimates expected gate fidelity based on noise parameters.
    ///
    /// Uses the improved model: F = (1 - ε_g) * exp(-t/T1) * exp(-t/Tφ)
    ///
    /// # Arguments
    /// * `gate_time_ns` - Gate execution time in nanoseconds
    /// * `is_two_qubit` - Whether this is a two-qubit gate
    ///
    /// # Returns
    /// Estimated fidelity in range [0.0, 1.0]
    pub fn estimate_gate_fidelity(&self, gate_time_ns: f64, is_two_qubit: bool) -> f64 {
        // Convert ns to μs
        let gate_time_us = gate_time_ns / 1000.0;

        // Gate error contribution
        let gate_error = if is_two_qubit {
            self.gate_error_2q
        } else {
            self.gate_error_1q
        };
        let gate_fidelity = 1.0 - gate_error;

        // T1 decay contribution
        let t1_decay = if self.t1_mean > 0.0 {
            (-gate_time_us / self.t1_mean).exp()
        } else {
            1.0
        };

        // T2 (pure dephasing) contribution
        let t_phi_decay = if let Some(t_phi) = self.t_phi() {
            (-gate_time_us / t_phi).exp()
        } else if self.t2_mean > 0.0 {
            // Fallback: use T2 directly if Tφ calculation fails
            (-gate_time_us / self.t2_mean).exp()
        } else {
            1.0
        };

        gate_fidelity * t1_decay * t_phi_decay
    }

    /// Estimates circuit fidelity based on gate sequence.
    ///
    /// # Arguments
    /// * `num_1q_gates` - Number of single-qubit gates
    /// * `num_2q_gates` - Number of two-qubit gates
    /// * `num_measurements` - Number of measurements
    /// * `gate_time_1q_ns` - Single-qubit gate time (ns)
    /// * `gate_time_2q_ns` - Two-qubit gate time (ns)
    ///
    /// # Returns
    /// Estimated circuit fidelity in range [0.0, 1.0]
    pub fn estimate_circuit_fidelity(
        &self,
        num_1q_gates: usize,
        num_2q_gates: usize,
        num_measurements: usize,
        gate_time_1q_ns: f64,
        gate_time_2q_ns: f64,
    ) -> f64 {
        // Gate fidelity contributions
        let f_1q = (1.0 - self.gate_error_1q).powi(num_1q_gates as i32);
        let f_2q = (1.0 - self.gate_error_2q).powi(num_2q_gates as i32);
        let f_ro = (1.0 - self.readout_error).powi(num_measurements as i32);

        // Total circuit time (μs)
        let total_time_us = (num_1q_gates as f64 * gate_time_1q_ns
            + num_2q_gates as f64 * gate_time_2q_ns)
            / 1000.0;

        // Coherence decay
        let t1_decay = if self.t1_mean > 0.0 {
            (-total_time_us / self.t1_mean).exp()
        } else {
            1.0
        };

        let t_phi_decay = if let Some(t_phi) = self.t_phi() {
            (-total_time_us / t_phi).exp()
        } else if self.t2_mean > 0.0 {
            (-total_time_us / self.t2_mean).exp()
        } else {
            1.0
        };

        f_1q * f_2q * f_ro * t1_decay * t_phi_decay
    }

    /// Validates the physical constraints of the noise parameters.
    ///
    /// Checks:
    /// - T2 ≤ 2*T1 (physical requirement)
    /// - All error rates are in [0, 1]
    /// - T1, T2 are non-negative
    pub fn validate(&self) -> Result<(), String> {
        if self.t1_mean < 0.0 {
            return Err(format!("T1 must be non-negative, got {}", self.t1_mean));
        }
        if self.t2_mean < 0.0 {
            return Err(format!("T2 must be non-negative, got {}", self.t2_mean));
        }
        if self.t1_mean > 0.0 && self.t2_mean > 2.0 * self.t1_mean {
            return Err(format!(
                "T2 ({}) must be ≤ 2*T1 ({})",
                self.t2_mean,
                2.0 * self.t1_mean
            ));
        }
        if !(0.0..=1.0).contains(&self.gate_error_1q) {
            return Err(format!(
                "gate_error_1q must be in [0,1], got {}",
                self.gate_error_1q
            ));
        }
        if !(0.0..=1.0).contains(&self.gate_error_2q) {
            return Err(format!(
                "gate_error_2q must be in [0,1], got {}",
                self.gate_error_2q
            ));
        }
        if !(0.0..=1.0).contains(&self.readout_error) {
            return Err(format!(
                "readout_error must be in [0,1], got {}",
                self.readout_error
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let nv = NoiseVector::new(0);
        assert_eq!(nv.qubit_id, 0);
        assert!(nv.timestamp > 0);
        assert_eq!(nv.source, NoiseSource::Simulator);
    }

    #[test]
    fn test_with_t1t2() {
        let nv = NoiseVector::with_t1t2(1, 100.0, 80.0);
        assert_eq!(nv.qubit_id, 1);
        assert_eq!(nv.t1_mean, 100.0);
        assert_eq!(nv.t2_mean, 80.0);
    }

    #[test]
    fn test_comprehensive() {
        let nv = NoiseVector::comprehensive(0, 100.0, 80.0, 0.001, 0.01, 0.02);
        assert_eq!(nv.t1_mean, 100.0);
        assert_eq!(nv.t2_mean, 80.0);
        assert_eq!(nv.gate_error_1q, 0.001);
        assert_eq!(nv.gate_error_2q, 0.01);
        assert_eq!(nv.readout_error, 0.02);
    }

    #[test]
    fn test_with_source() {
        let nv = NoiseVector::new(0).with_source(NoiseSource::Calibration);
        assert_eq!(nv.source, NoiseSource::Calibration);
    }

    #[test]
    fn test_is_anomaly() {
        let mut nv = NoiseVector::new(0);
        nv.t1_std = 10.0;
        nv.drift_rate = 25.0;

        // drift_rate (25) < 3 * t1_std (30), no burst
        assert!(!nv.is_anomaly(3.0));

        // drift_rate (25) > 2 * t1_std (20)
        assert!(nv.is_anomaly(2.0));

        // burst detected
        nv.burst_count = 1;
        assert!(nv.is_anomaly(10.0));
    }

    #[test]
    fn test_t2_t1_ratio() {
        let nv = NoiseVector::with_t1t2(0, 100.0, 80.0);
        assert!((nv.t2_t1_ratio() - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_t_phi_calculation() {
        // T1 = 100 μs, T2 = 80 μs
        // 1/Tφ = 1/T2 - 1/(2*T1) = 1/80 - 1/200 = 0.0125 - 0.005 = 0.0075
        // Tφ = 1/0.0075 ≈ 133.33 μs
        let nv = NoiseVector::with_t1t2(0, 100.0, 80.0);
        let t_phi = nv.t_phi().unwrap();
        assert!((t_phi - 133.333).abs() < 0.01);
    }

    #[test]
    fn test_t_phi_limit_case() {
        // When T2 = 2*T1, Tφ should be infinite (returns None in our impl)
        let nv = NoiseVector::with_t1t2(0, 100.0, 200.0);
        // 1/Tφ = 1/200 - 1/200 = 0, so Tφ = ∞
        // Our implementation returns None for this edge case
        assert!(nv.t_phi().is_none());
    }

    #[test]
    fn test_estimate_gate_fidelity() {
        let nv = NoiseVector::comprehensive(0, 100.0, 80.0, 0.001, 0.01, 0.02);

        // Single-qubit gate (35 ns)
        let f_1q = nv.estimate_gate_fidelity(35.0, false);
        assert!(f_1q > 0.99); // Should be very high for short gate

        // Two-qubit gate (300 ns)
        let f_2q = nv.estimate_gate_fidelity(300.0, true);
        assert!(f_2q > 0.98 && f_2q < 0.995); // Lower due to higher gate error
    }

    #[test]
    fn test_estimate_circuit_fidelity() {
        let nv = NoiseVector::comprehensive(0, 100.0, 80.0, 0.001, 0.01, 0.02);

        // Simple circuit: 5 1Q gates, 2 2Q gates, 1 measurement
        let fidelity = nv.estimate_circuit_fidelity(5, 2, 1, 35.0, 300.0);

        // Should be reasonable fidelity for short circuit
        assert!(fidelity > 0.9 && fidelity < 1.0);
    }

    #[test]
    fn test_validate_success() {
        let nv = NoiseVector::comprehensive(0, 100.0, 80.0, 0.001, 0.01, 0.02);
        assert!(nv.validate().is_ok());
    }

    #[test]
    fn test_validate_t2_constraint() {
        // T2 > 2*T1 violates physical constraint
        let nv = NoiseVector::with_t1t2(0, 100.0, 250.0);
        assert!(nv.validate().is_err());
    }

    #[test]
    fn test_validate_error_bounds() {
        let mut nv = NoiseVector::new(0);
        nv.gate_error_1q = 1.5; // Invalid: > 1.0
        assert!(nv.validate().is_err());
    }

    #[test]
    fn test_noise_source_display() {
        assert_eq!(format!("{}", NoiseSource::Simulator), "Simulator");
        assert_eq!(format!("{}", NoiseSource::Calibration), "Calibration");
    }
}
