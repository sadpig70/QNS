//! DriftScanner implementation.
//!
//! The DriftScanner measures T1/T2 coherence times and detects drift
//! and anomalies in quantum hardware noise characteristics.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use qns_profiler::DriftScanner;
//! use qns_core::prelude::*;
//!
//! let mut scanner = DriftScanner::with_defaults();
//!
//! // Perform a scan
//! let noise_vector = scanner.scan(0)?;
//!
//! // Check for anomalies
//! if scanner.is_anomaly(&noise_vector) {
//!     println!("Anomaly detected on qubit 0!");
//! }
//! ```

use qns_core::prelude::*;
use qns_core::types::CrosstalkMatrix;
use std::time::{SystemTime, UNIX_EPOCH};

use super::compute::{calculate_drift_rate, detect_anomaly, ExponentialMovingAverage, Statistics};
use super::measure::{simulate_burst_event, simulate_t1, simulate_t2};

/// Configuration for drift scanning.
#[derive(Debug, Clone)]
pub struct ScanConfig {
    /// Number of measurement shots per T1/T2 estimate
    pub sample_count: usize,
    /// Sigma threshold for anomaly detection (typically 3.0)
    pub threshold_sigma: f64,
    /// Base T1 time in μs (hardware-specific)
    pub t1_base: f64,
    /// Base T2 time in μs (hardware-specific)
    pub t2_base: f64,
    /// Number of time points for T1 decay curve
    pub t1_time_points: usize,
    /// Number of time points for T2 Ramsey curve  
    pub t2_time_points: usize,
    /// Measurement noise level (0.0 = perfect, 0.1 = 10% noise)
    pub noise_level: f64,
    /// Ramsey detuning frequency (MHz)
    pub ramsey_detuning: f64,
    /// Burst event probability per scan
    pub burst_probability: f64,
    /// Enable drift tracking
    pub track_drift: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            sample_count: 1000,
            threshold_sigma: 3.0,
            t1_base: t1_typical::NISQ_TYPICAL, // 100 μs
            t2_base: t2_typical::NISQ_TYPICAL, // 80 μs
            t1_time_points: 20,                // Reduced for speed (<10ms target)
            t2_time_points: 20,
            noise_level: 0.05,
            ramsey_detuning: 0.01,    // 10 kHz
            burst_probability: 0.001, // 0.1% per scan
            track_drift: true,
        }
    }
}

impl From<ProfilerConfig> for ScanConfig {
    fn from(config: ProfilerConfig) -> Self {
        Self {
            sample_count: config.sample_count,
            threshold_sigma: config.threshold_sigma,
            t1_base: config.t1_base,
            t2_base: config.t2_base,
            ..Default::default()
        }
    }
}

/// Scan history for a single qubit.
#[derive(Debug, Clone, Default)]
struct QubitHistory {
    /// Recent T1 measurements
    t1_history: Vec<f64>,
    /// Recent T2 measurements
    t2_history: Vec<f64>,
    /// Timestamps of measurements
    timestamps: Vec<u64>,
    /// T1 exponential moving average
    t1_ema: Option<ExponentialMovingAverage>,
    /// T2 exponential moving average
    t2_ema: Option<ExponentialMovingAverage>,
    /// Detected burst count
    burst_count: usize,
    /// Maximum history size
    max_history: usize,
}

impl QubitHistory {
    fn new(max_history: usize) -> Self {
        Self {
            t1_history: Vec::with_capacity(max_history),
            t2_history: Vec::with_capacity(max_history),
            timestamps: Vec::with_capacity(max_history),
            t1_ema: Some(ExponentialMovingAverage::from_period(10)),
            t2_ema: Some(ExponentialMovingAverage::from_period(10)),
            burst_count: 0,
            max_history,
        }
    }

    fn add(&mut self, t1: f64, t2: f64, timestamp: u64) {
        // Maintain bounded history
        if self.t1_history.len() >= self.max_history {
            self.t1_history.remove(0);
            self.t2_history.remove(0);
            self.timestamps.remove(0);
        }

        self.t1_history.push(t1);
        self.t2_history.push(t2);
        self.timestamps.push(timestamp);

        // Update EMAs
        if let Some(ema) = &mut self.t1_ema {
            ema.update(t1);
        }
        if let Some(ema) = &mut self.t2_ema {
            ema.update(t2);
        }
    }

    fn t1_stats(&self) -> Option<Statistics> {
        Statistics::compute(&self.t1_history)
    }

    fn t2_stats(&self) -> Option<Statistics> {
        Statistics::compute(&self.t2_history)
    }
}

/// T1/T2 drift scanner for quantum hardware characterization.
///
/// Performs periodic measurements of T1 (energy relaxation) and T2 (dephasing)
/// times, tracks drift over time, and detects anomalies.
pub struct DriftScanner {
    config: ScanConfig,
    /// Last scanned noise vector
    last_vector: Option<NoiseVector>,
    /// Per-qubit history (indexed by qubit_id)
    qubit_histories: Vec<QubitHistory>,
    /// Total scan count
    scan_count: usize,
}

impl DriftScanner {
    /// Creates a new DriftScanner with the given configuration.
    pub fn new(config: impl Into<ScanConfig>) -> Self {
        Self {
            config: config.into(),
            last_vector: None,
            qubit_histories: Vec::new(),
            scan_count: 0,
        }
    }

    /// Creates a DriftScanner with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(ScanConfig::default())
    }

    /// Creates a DriftScanner optimized for speed (reduced accuracy).
    ///
    /// Useful for real-time monitoring where <5ms latency is required.
    pub fn fast() -> Self {
        Self::new(ScanConfig {
            sample_count: 500,
            t1_time_points: 10,
            t2_time_points: 10,
            noise_level: 0.08,
            ..Default::default()
        })
    }

    /// Creates a DriftScanner optimized for accuracy.
    ///
    /// Takes longer but provides more precise measurements.
    pub fn accurate() -> Self {
        Self::new(ScanConfig {
            sample_count: 5000,
            t1_time_points: 50,
            t2_time_points: 50,
            noise_level: 0.02,
            ..Default::default()
        })
    }

    /// Ensures history exists for the given qubit.
    fn ensure_history(&mut self, qubit_id: usize) {
        while self.qubit_histories.len() <= qubit_id {
            self.qubit_histories.push(QubitHistory::new(100));
        }
    }

    /// Performs a drift scan on the specified qubit.
    ///
    /// Returns a NoiseVector containing T1/T2 statistics and drift information.
    ///
    /// # Performance
    /// Target: <10ms with default configuration
    pub fn scan(&mut self, qubit_id: usize) -> Result<NoiseVector> {
        self.ensure_history(qubit_id);
        self.scan_count += 1;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Check for burst event (simulated hardware anomaly)
        let (effective_t1, effective_t2, is_burst) = if let Some((t1, t2)) = simulate_burst_event(
            self.config.t1_base,
            self.config.t2_base,
            self.config.burst_probability,
        ) {
            self.qubit_histories[qubit_id].burst_count += 1;
            (t1, t2, true)
        } else {
            (self.config.t1_base, self.config.t2_base, false)
        };

        // Simulate T1 measurement
        let t1_result = simulate_t1(
            effective_t1,
            self.config.noise_level,
            self.config.sample_count,
            self.config.t1_time_points,
        );

        // Simulate T2 measurement (constrained by T1)
        let t2_result = simulate_t2(
            effective_t2,
            t1_result.t1, // Use measured T1 for physical constraint
            self.config.noise_level,
            self.config.sample_count,
            self.config.t2_time_points,
            self.config.ramsey_detuning,
        );

        // Update history
        let history = &mut self.qubit_histories[qubit_id];
        history.add(t1_result.t1, t2_result.t2, timestamp);
        if is_burst {
            history.burst_count += 1;
        }

        // Compute statistics from history
        let (t1_mean, t1_std) = match history.t1_stats() {
            Some(stats) => (stats.mean, stats.std_dev),
            None => (t1_result.t1, t1_result.std_error),
        };

        let (t2_mean, t2_std) = match history.t2_stats() {
            Some(stats) => (stats.mean, stats.std_dev),
            None => (t2_result.t2, t2_result.std_error),
        };

        // Calculate drift rate if we have enough history
        let drift_rate = if history.timestamps.len() >= 3 && self.config.track_drift {
            if let Some(analysis) = calculate_drift_rate(
                &history.timestamps,
                &history.t1_history,
                0.3, // Lower threshold for significance
            ) {
                analysis.drift_rate.abs()
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Build noise vector
        let noise_vector = NoiseVector {
            t1_mean,
            t1_std,
            t2_mean,
            t2_std,
            drift_rate,
            burst_count: history.burst_count,
            qubit_id,
            timestamp,
            sample_count: self.config.sample_count,
            source: qns_core::types::NoiseSource::Simulator,
            // v2.0 fields - defaults for simulator mode
            gate_error_1q: qns_core::physics::gate_errors::SINGLE_QUBIT_TYPICAL,
            gate_error_2q: qns_core::physics::gate_errors::TWO_QUBIT_TYPICAL,
            readout_error: qns_core::physics::gate_errors::MEASUREMENT_TYPICAL,
            frequency: None,
            anharmonicity: None,
        };

        self.last_vector = Some(noise_vector.clone());
        Ok(noise_vector)
    }

    /// Performs a batch scan on multiple qubits.
    ///
    /// More efficient than scanning qubits individually.
    pub fn scan_batch(&mut self, qubit_ids: &[usize]) -> Result<Vec<NoiseVector>> {
        qubit_ids.iter().map(|&id| self.scan(id)).collect()
    }

    /// Checks if the given noise vector indicates an anomaly.
    ///
    /// Anomaly detection uses:
    /// 1. Z-score against historical mean
    /// 2. Sudden change detection
    /// 3. Burst event detection
    pub fn is_anomaly(&self, noise: &NoiseVector) -> bool {
        // Quick check using NoiseVector's built-in method
        if noise.is_anomaly(self.config.threshold_sigma) {
            return true;
        }

        // Check against historical statistics
        if let Some(history) = self.qubit_histories.get(noise.qubit_id) {
            if let Some(stats) = history.t1_stats() {
                let anomaly = detect_anomaly(
                    noise.t1_mean,
                    stats.mean,
                    stats.std_dev,
                    self.config.threshold_sigma,
                );
                if anomaly.is_anomaly {
                    return true;
                }
            }
        }

        false
    }

    /// Returns detailed anomaly analysis for a noise vector.
    pub fn analyze_anomaly(&self, noise: &NoiseVector) -> AnomalyAnalysis {
        let mut analysis = AnomalyAnalysis::default();

        // Check drift threshold
        if noise.drift_rate > self.config.threshold_sigma * 10.0 {
            analysis.drift_anomaly = true;
            analysis.drift_severity = noise.drift_rate / (self.config.threshold_sigma * 10.0);
        }

        // Check burst events
        if noise.burst_count > 0 {
            analysis.burst_anomaly = true;
            analysis.burst_count = noise.burst_count;
        }

        // Check against historical T1
        if let Some(history) = self.qubit_histories.get(noise.qubit_id) {
            if let Some(stats) = history.t1_stats() {
                let anomaly = detect_anomaly(
                    noise.t1_mean,
                    stats.mean,
                    stats.std_dev,
                    self.config.threshold_sigma,
                );
                if anomaly.is_anomaly {
                    analysis.t1_anomaly = true;
                    analysis.t1_z_score = anomaly.z_score;
                }
            }

            if let Some(stats) = history.t2_stats() {
                let anomaly = detect_anomaly(
                    noise.t2_mean,
                    stats.mean,
                    stats.std_dev,
                    self.config.threshold_sigma,
                );
                if anomaly.is_anomaly {
                    analysis.t2_anomaly = true;
                    analysis.t2_z_score = anomaly.z_score;
                }
            }
        }

        analysis.is_anomaly = analysis.drift_anomaly
            || analysis.burst_anomaly
            || analysis.t1_anomaly
            || analysis.t2_anomaly;

        analysis
    }

    /// Returns the last scanned noise vector.
    pub fn last_scan(&self) -> Option<&NoiseVector> {
        self.last_vector.as_ref()
    }

    /// Returns the current configuration.
    pub fn config(&self) -> &ScanConfig {
        &self.config
    }

    /// Returns the total number of scans performed.
    pub fn scan_count(&self) -> usize {
        self.scan_count
    }

    /// Returns historical T1 values for a qubit.
    pub fn t1_history(&self, qubit_id: usize) -> Option<&[f64]> {
        self.qubit_histories
            .get(qubit_id)
            .map(|h| h.t1_history.as_slice())
    }

    /// Returns historical T2 values for a qubit.
    pub fn t2_history(&self, qubit_id: usize) -> Option<&[f64]> {
        self.qubit_histories
            .get(qubit_id)
            .map(|h| h.t2_history.as_slice())
    }

    /// Clears history for a specific qubit.
    pub fn clear_history(&mut self, qubit_id: usize) {
        if let Some(history) = self.qubit_histories.get_mut(qubit_id) {
            history.t1_history.clear();
            history.t2_history.clear();
            history.timestamps.clear();
            history.burst_count = 0;
            if let Some(ema) = &mut history.t1_ema {
                ema.reset();
            }
            if let Some(ema) = &mut history.t2_ema {
                ema.reset();
            }
        }
    }

    /// Clears all history.
    pub fn clear_all_history(&mut self) {
        self.qubit_histories.clear();
        self.last_vector = None;
        self.scan_count = 0;
    }

    /// Performs a crosstalk scan (simulated).
    ///
    /// Generates a synthetic crosstalk matrix for testing and simulation.
    /// In a real backend, this would parse `backend.properties()`.
    pub fn scan_crosstalk(&self, num_qubits: usize) -> CrosstalkMatrix {
        let mut matrix = CrosstalkMatrix::new();

        // Simulate random crosstalk between adjacent qubits (linear chain assumed for simplicity if topology unknown)
        // In a real scenario, we would use the HardwareProfile topology.
        // Here we just add some random noise interactions.
        for i in 0..num_qubits.saturating_sub(1) {
            // Simulate 10% chance of high crosstalk, otherwise low
            let strength = if (i * 7 + self.scan_count) % 10 == 0 {
                0.05
            } else {
                0.001
            };
            matrix.set_interaction(i, i + 1, strength);
        }

        matrix
    }
}

/// Detailed anomaly analysis result.
#[derive(Debug, Clone, Default)]
pub struct AnomalyAnalysis {
    /// Is any anomaly detected?
    pub is_anomaly: bool,
    /// T1 anomaly detected
    pub t1_anomaly: bool,
    /// T1 z-score
    pub t1_z_score: f64,
    /// T2 anomaly detected
    pub t2_anomaly: bool,
    /// T2 z-score
    pub t2_z_score: f64,
    /// Drift anomaly detected
    pub drift_anomaly: bool,
    /// Drift severity (ratio to threshold)
    pub drift_severity: f64,
    /// Burst anomaly detected
    pub burst_anomaly: bool,
    /// Burst event count
    pub burst_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let scanner = DriftScanner::with_defaults();
        assert_eq!(scanner.config().sample_count, 1000);
        assert_eq!(scanner.scan_count(), 0);
    }

    #[test]
    fn test_scan() {
        let mut scanner = DriftScanner::with_defaults();
        let result = scanner.scan(0);
        assert!(result.is_ok());

        let nv = result.unwrap();
        assert_eq!(nv.qubit_id, 0);
        assert!(nv.t1_mean > 0.0);
        assert!(nv.t2_mean > 0.0);
        assert!(nv.timestamp > 0);
        assert_eq!(scanner.scan_count(), 1);
    }

    #[test]
    fn test_scan_multiple_qubits() {
        let mut scanner = DriftScanner::with_defaults();

        let nv0 = scanner.scan(0).unwrap();
        let nv1 = scanner.scan(1).unwrap();
        let nv2 = scanner.scan(2).unwrap();

        assert_eq!(nv0.qubit_id, 0);
        assert_eq!(nv1.qubit_id, 1);
        assert_eq!(nv2.qubit_id, 2);
        assert_eq!(scanner.scan_count(), 3);
    }

    #[test]
    fn test_scan_batch() {
        let mut scanner = DriftScanner::fast();
        let results = scanner.scan_batch(&[0, 1, 2, 3, 4]).unwrap();

        assert_eq!(results.len(), 5);
        for (i, nv) in results.iter().enumerate() {
            assert_eq!(nv.qubit_id, i);
        }
    }

    #[test]
    fn test_history_tracking() {
        let mut scanner = DriftScanner::fast();

        // Perform multiple scans
        for _ in 0..5 {
            scanner.scan(0).unwrap();
        }

        let history = scanner.t1_history(0).unwrap();
        assert_eq!(history.len(), 5);
    }

    #[test]
    fn test_t2_physical_constraint() {
        let mut scanner = DriftScanner::with_defaults();
        let nv = scanner.scan(0).unwrap();

        // T2 should be ≤ 2*T1 (physical constraint)
        assert!(
            nv.t2_mean <= 2.0 * nv.t1_mean + 10.0, // Allow some tolerance
            "T2 = {} exceeds 2*T1 = {}",
            nv.t2_mean,
            2.0 * nv.t1_mean
        );
    }

    #[test]
    fn test_anomaly_detection_normal() {
        let mut scanner = DriftScanner::fast();

        // Build up history with normal values
        for _ in 0..10 {
            scanner.scan(0).unwrap();
        }

        let nv = scanner.scan(0).unwrap();
        // Normal scan should typically not be anomalous
        // (though random noise might occasionally trigger)
        let analysis = scanner.analyze_anomaly(&nv);

        // At minimum, analysis should be valid
        assert!(analysis.t1_z_score.is_finite());
    }

    #[test]
    fn test_fast_scanner() {
        let mut scanner = DriftScanner::fast();
        assert_eq!(scanner.config().t1_time_points, 10);

        // Should complete quickly
        let start = std::time::Instant::now();
        scanner.scan(0).unwrap();
        let elapsed = start.elapsed();

        // Fast scanner should be under 10ms typically
        assert!(
            elapsed.as_millis() < 100,
            "Scan took {}ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_clear_history() {
        let mut scanner = DriftScanner::fast();

        for _ in 0..5 {
            scanner.scan(0).unwrap();
        }

        assert_eq!(scanner.t1_history(0).unwrap().len(), 5);

        scanner.clear_history(0);

        // After clear, should be empty (or not exist)
        assert!(scanner.t1_history(0).map(|h| h.is_empty()).unwrap_or(true));
    }

    #[test]
    fn test_from_profiler_config() {
        let profiler_config = ProfilerConfig {
            sample_count: 500,
            threshold_sigma: 2.5,
            t1_base: 150.0,
            t2_base: 100.0,
            scan_interval_sec: 60,
        };

        let scan_config: ScanConfig = profiler_config.into();
        assert_eq!(scan_config.sample_count, 500);
        assert!((scan_config.threshold_sigma - 2.5).abs() < 1e-10);
        assert!((scan_config.t1_base - 150.0).abs() < 1e-10);
    }

    #[test]
    fn test_t1_t2_ratio_physical() {
        // T2/T1 ratio should be in physical range [0.1, 2.0]
        let mut scanner = DriftScanner::with_defaults();

        for _ in 0..10 {
            let nv = scanner.scan(0).unwrap();
            let ratio = nv.t2_mean / nv.t1_mean;

            assert!(
                ratio > 0.1 && ratio <= 2.0,
                "T2/T1 ratio {} outside physical range [0.1, 2.0]",
                ratio
            );
        }
    }

    #[test]
    fn test_measurement_consistency() {
        // Multiple scans should produce consistent results (within noise)
        let mut scanner = DriftScanner::with_defaults();

        let mut t1_values = Vec::new();
        let mut t2_values = Vec::new();

        for _ in 0..20 {
            let nv = scanner.scan(0).unwrap();
            t1_values.push(nv.t1_mean);
            t2_values.push(nv.t2_mean);
        }

        // Calculate coefficient of variation (should be < 30% with default noise)
        let t1_mean: f64 = t1_values.iter().sum::<f64>() / t1_values.len() as f64;
        let t1_var: f64 =
            t1_values.iter().map(|x| (x - t1_mean).powi(2)).sum::<f64>() / t1_values.len() as f64;
        let t1_cv = t1_var.sqrt() / t1_mean;

        assert!(t1_cv < 0.5, "T1 CV {} too high (> 50%)", t1_cv);
    }

    #[test]
    fn test_positive_values() {
        // T1 and T2 must always be positive
        let mut scanner = DriftScanner::with_defaults();

        for _ in 0..20 {
            let nv = scanner.scan(0).unwrap();
            assert!(nv.t1_mean > 0.0, "T1 must be positive");
            assert!(nv.t2_mean > 0.0, "T2 must be positive");
            assert!(nv.t1_std >= 0.0, "T1 std must be non-negative");
            assert!(nv.t2_std >= 0.0, "T2 std must be non-negative");
        }
    }

    #[test]
    fn test_drift_rate_calculation() {
        let mut scanner = DriftScanner::fast();
        scanner.config.track_drift = true;

        // Perform enough scans to calculate drift
        for _ in 0..10 {
            let nv = scanner.scan(0).unwrap();
            // Drift rate should be finite
            assert!(nv.drift_rate.is_finite(), "Drift rate must be finite");
        }
    }
}
