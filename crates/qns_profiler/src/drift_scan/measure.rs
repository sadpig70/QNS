//! T1/T2 measurement simulation module.
//!
//! This module simulates the measurement of T1 (energy relaxation) and
//! T2 (dephasing) times using realistic noise models.
//!
//! ## Physical Background
//!
//! ### T1 (Longitudinal Relaxation)
//!
//! T1 is the time constant for energy relaxation, describing how quickly
//! an excited qubit (|1⟩) decays to its ground state (|0⟩).
//!
//! **Measurement Protocol**: Inversion Recovery
//! 1. Prepare qubit in |1⟩ state (X gate on |0⟩)
//! 2. Wait for time t
//! 3. Measure probability P(|1⟩)
//!
//! **Physics**: P(|1⟩) = exp(-t/T1)
//!
//! **Typical Values**:
//! - IBM Heron (2024): ~300 μs
//! - Google Sycamore: ~20 μs
//!
//! ### T2 (Transverse Relaxation / Dephasing)
//!
//! T2 is the time constant for coherence decay, describing how quickly
//! the phase information of a superposition is lost.
//!
//! **Measurement Protocol**: Ramsey Experiment
//! 1. Apply H gate (create superposition)
//! 2. Wait for time t (with small detuning δf)
//! 3. Apply H gate
//! 4. Measure probability P(|0⟩)
//!
//! **Physics**: P(|0⟩) = 0.5 * (1 + cos(2πδf·t) * exp(-t/T2))
//!
//! **Note**: This is T2, not T2*. T2* includes inhomogeneous dephasing and
//! is measured without echo pulses. T2 ≥ T2*.
//!
//! ### Physical Constraint
//!
//! **T2 ≤ 2·T1** (fundamental quantum mechanical limit)
//!
//! For superconducting qubits, typically T2/T1 ∈ [0.5, 2.0].
//! If T2 > 2·T1, the measurement indicates systematic errors.

use rand::Rng;
use rand_distr::{Distribution, Normal};

/// Result of T1 measurement simulation.
#[derive(Debug, Clone)]
pub struct T1Measurement {
    /// Measured T1 value (μs)
    pub t1: f64,
    /// Standard error of the measurement
    pub std_error: f64,
    /// Number of samples used
    pub num_samples: usize,
    /// Raw decay curve data points (time_us, probability)
    pub decay_curve: Vec<(f64, f64)>,
}

/// Result of T2 measurement simulation.
#[derive(Debug, Clone)]
pub struct T2Measurement {
    /// Measured T2 value (μs)
    pub t2: f64,
    /// Standard error of the measurement
    pub std_error: f64,
    /// Number of samples used
    pub num_samples: usize,
    /// Ramsey fringe data points (time_us, probability)
    pub ramsey_curve: Vec<(f64, f64)>,
}

/// Simulates T1 measurement using inversion recovery protocol.
///
/// # Arguments
/// * `t1_true` - The "true" T1 value to simulate (μs)
/// * `noise_level` - Relative noise level (0.0 = perfect, 0.1 = 10% noise)
/// * `num_samples` - Number of measurement samples
/// * `num_time_points` - Number of time points in the decay curve
///
/// # Returns
/// T1Measurement containing the estimated T1 and statistics
pub fn simulate_t1(
    t1_true: f64,
    noise_level: f64,
    num_samples: usize,
    num_time_points: usize,
) -> T1Measurement {
    let mut rng = rand::thread_rng();

    // Generate time points from 0 to 5*T1 (capture full decay)
    let t_max = 5.0 * t1_true;
    let dt = t_max / num_time_points as f64;

    let mut decay_curve = Vec::with_capacity(num_time_points);
    let mut log_probs = Vec::with_capacity(num_time_points);
    let mut times = Vec::with_capacity(num_time_points);

    // Noise distribution for measurement
    let noise_dist =
        Normal::new(0.0, noise_level).unwrap_or_else(|_| Normal::new(0.0, 0.01).unwrap());

    for i in 0..num_time_points {
        let t = (i as f64 + 0.5) * dt; // Center of time bin

        // True probability P(|1⟩) = exp(-t/T1)
        let p_true = (-t / t1_true).exp();

        // Add measurement noise (shot noise + readout error)
        let noise: f64 = noise_dist.sample(&mut rng);
        let p_measured = (p_true + noise).clamp(0.001, 0.999); // Avoid log(0)

        decay_curve.push((t, p_measured));

        // For linear fit: ln(P) = -t/T1
        log_probs.push(p_measured.ln());
        times.push(t);
    }

    // Linear regression to extract T1: ln(P) = -t/T1 + const
    // Using least squares: slope = -1/T1
    let (slope, _intercept, std_error) = linear_regression(&times, &log_probs);

    // T1 = -1/slope
    let t1_measured = if slope.abs() > 1e-10 {
        -1.0 / slope
    } else {
        t1_true // Fallback if slope is too small
    };

    // Ensure physical validity
    let t1_measured = t1_measured.max(0.1);

    T1Measurement {
        t1: t1_measured,
        std_error: std_error / (slope * slope).max(1e-10), // Propagate error
        num_samples,
        decay_curve,
    }
}

/// Simulates T2 measurement using Ramsey experiment.
///
/// # Arguments
/// * `t2_true` - The "true" T2 value to simulate (μs)
/// * `t1_true` - The T1 value (for physical constraint T2 ≤ 2*T1)
/// * `noise_level` - Relative noise level
/// * `num_samples` - Number of measurement samples
/// * `num_time_points` - Number of time points in Ramsey curve
/// * `detuning` - Detuning frequency for Ramsey oscillation (MHz)
///
/// # Returns
/// T2Measurement containing the estimated T2 and statistics
pub fn simulate_t2(
    t2_true: f64,
    t1_true: f64,
    noise_level: f64,
    num_samples: usize,
    num_time_points: usize,
    detuning: f64,
) -> T2Measurement {
    let mut rng = rand::thread_rng();

    // Physical constraint: T2 ≤ 2*T1
    let t2_physical = t2_true.min(2.0 * t1_true);

    // Generate time points
    let t_max = 5.0 * t2_physical;
    let dt = t_max / num_time_points as f64;

    let mut ramsey_curve = Vec::with_capacity(num_time_points);
    let mut envelopes = Vec::with_capacity(num_time_points);
    let mut times = Vec::with_capacity(num_time_points);

    let noise_dist =
        Normal::new(0.0, noise_level).unwrap_or_else(|_| Normal::new(0.0, 0.01).unwrap());

    for i in 0..num_time_points {
        let t = (i as f64 + 0.5) * dt;

        // Ramsey signal: P = 0.5 * (1 + cos(2π*f*t) * exp(-t/T2))
        let envelope = (-t / t2_physical).exp();
        let oscillation = (2.0 * std::f64::consts::PI * detuning * t).cos();
        let p_true = 0.5 * (1.0 + oscillation * envelope);

        // Add noise
        let noise: f64 = noise_dist.sample(&mut rng);
        let p_measured = (p_true + noise).clamp(0.001, 0.999);

        ramsey_curve.push((t, p_measured));

        // Extract envelope for T2 fitting
        // |P - 0.5| / 0.5 ≈ envelope (when oscillation = ±1)
        let env_estimate = ((p_measured - 0.5).abs() * 2.0).clamp(0.001, 1.0);
        envelopes.push(env_estimate.ln());
        times.push(t);
    }

    // Fit envelope decay: ln(envelope) = -t/T2
    let (slope, _intercept, std_error) = linear_regression(&times, &envelopes);

    let t2_measured = if slope.abs() > 1e-10 {
        -1.0 / slope
    } else {
        t2_physical
    };

    // Ensure physical validity: T2 > 0 and T2 ≤ 2*T1
    let t2_measured = t2_measured.clamp(0.1, 2.0 * t1_true);

    T2Measurement {
        t2: t2_measured,
        std_error: std_error / (slope * slope).max(1e-10),
        num_samples,
        ramsey_curve,
    }
}

/// Performs simple linear regression y = slope * x + intercept.
///
/// Returns (slope, intercept, standard_error_of_slope)
fn linear_regression(x: &[f64], y: &[f64]) -> (f64, f64, f64) {
    let n = x.len() as f64;
    if n < 2.0 {
        return (0.0, 0.0, f64::INFINITY);
    }

    let sum_x: f64 = x.iter().sum();
    let sum_y: f64 = y.iter().sum();
    let sum_xy: f64 = x.iter().zip(y.iter()).map(|(xi, yi)| xi * yi).sum();
    let sum_x2: f64 = x.iter().map(|xi| xi * xi).sum();

    let denom = n * sum_x2 - sum_x * sum_x;
    if denom.abs() < 1e-20 {
        return (0.0, sum_y / n, f64::INFINITY);
    }

    let slope = (n * sum_xy - sum_x * sum_y) / denom;
    let intercept = (sum_y - slope * sum_x) / n;

    // Calculate standard error of slope
    let y_pred: Vec<f64> = x.iter().map(|xi| slope * xi + intercept).collect();
    let residuals: f64 = y
        .iter()
        .zip(y_pred.iter())
        .map(|(yi, ypi)| (yi - ypi).powi(2))
        .sum();

    let mse = residuals / (n - 2.0).max(1.0);
    let se_slope = (mse / (sum_x2 - sum_x * sum_x / n).max(1e-20)).sqrt();

    (slope, intercept, se_slope)
}

/// Simulates a burst noise event.
///
/// Burst events are sudden, temporary degradations in T1/T2
/// caused by environmental factors (cosmic rays, TLS fluctuators, etc.)
pub fn simulate_burst_event(
    base_t1: f64,
    base_t2: f64,
    burst_probability: f64,
) -> Option<(f64, f64)> {
    let mut rng = rand::thread_rng();

    if rng.gen::<f64>() < burst_probability {
        // Burst reduces T1/T2 by 20-80%
        let reduction = rng.gen_range(0.2..0.8);
        Some((base_t1 * reduction, base_t2 * reduction))
    } else {
        None
    }
}

/// Simulates slow drift in T1/T2 values.
///
/// Drift is modeled as a random walk with mean-reverting tendency.
pub fn simulate_drift(current_t1: f64, base_t1: f64, drift_rate: f64, dt_hours: f64) -> f64 {
    let mut rng = rand::thread_rng();

    // Ornstein-Uhlenbeck process: mean-reverting random walk
    // dT1 = θ(μ - T1)dt + σdW
    let theta = 0.1; // Mean reversion rate
    let sigma = drift_rate;

    let mean_reversion = theta * (base_t1 - current_t1) * dt_hours;
    let random_walk = sigma * rng.gen_range(-1.0..1.0) * dt_hours.sqrt();

    (current_t1 + mean_reversion + random_walk).max(base_t1 * 0.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulate_t1() {
        let result = simulate_t1(100.0, 0.05, 1000, 50);

        // T1 should be within 30% of true value with moderate noise
        // (Allowing wider range due to stochastic nature)
        assert!(
            result.t1 > 70.0 && result.t1 < 130.0,
            "T1 = {} not in expected range [70, 130]",
            result.t1
        );
        assert_eq!(result.num_samples, 1000);
        assert_eq!(result.decay_curve.len(), 50);
    }

    #[test]
    fn test_simulate_t2() {
        let result = simulate_t2(80.0, 100.0, 0.05, 1000, 50, 0.01);

        // T2 should be within reasonable range
        // Physical constraint: T2 ≤ 2*T1 = 200
        assert!(
            result.t2 > 40.0 && result.t2 <= 200.0,
            "T2 = {} not in expected range [40, 200]",
            result.t2
        );
    }

    #[test]
    fn test_t2_physical_constraint() {
        // If T2_true > 2*T1, the measured T2 should be clamped
        let result = simulate_t2(300.0, 100.0, 0.01, 1000, 50, 0.01);
        assert!(result.t2 <= 200.0, "T2 = {} exceeds 2*T1 = 200", result.t2);
    }

    #[test]
    fn test_linear_regression() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0]; // y = 2x

        let (slope, intercept, _) = linear_regression(&x, &y);
        assert!((slope - 2.0).abs() < 0.01);
        assert!(intercept.abs() < 0.01);
    }

    #[test]
    fn test_burst_event() {
        // With probability 1.0, should always get a burst
        let result = simulate_burst_event(100.0, 80.0, 1.0);
        assert!(result.is_some());

        let (t1, t2) = result.unwrap();
        assert!(t1 < 100.0 && t1 > 20.0);
        assert!(t2 < 80.0 && t2 > 16.0);
    }

    #[test]
    fn test_no_burst_event() {
        // With probability 0.0, should never get a burst
        let result = simulate_burst_event(100.0, 80.0, 0.0);
        assert!(result.is_none());
    }
}
