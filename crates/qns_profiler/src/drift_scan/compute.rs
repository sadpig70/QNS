//! Statistical computation module for noise analysis.
//!
//! Provides functions for:
//! - Computing mean, variance, standard deviation
//! - Calculating drift rates
//! - Detecting anomalies using statistical methods

/// Statistics for a collection of measurements.
#[derive(Debug, Clone, Default)]
pub struct Statistics {
    /// Number of samples
    pub count: usize,
    /// Mean value
    pub mean: f64,
    /// Variance
    pub variance: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
}

impl Statistics {
    /// Computes statistics for a slice of values.
    ///
    /// Returns None if the slice is empty.
    pub fn compute(values: &[f64]) -> Option<Self> {
        if values.is_empty() {
            return None;
        }

        let count = values.len();
        let sum: f64 = values.iter().sum();
        let mean = sum / count as f64;

        let variance = if count > 1 {
            let sq_diff_sum: f64 = values.iter().map(|x| (x - mean).powi(2)).sum();
            sq_diff_sum / (count - 1) as f64 // Bessel's correction
        } else {
            0.0
        };

        let std_dev = variance.sqrt();

        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        Some(Self {
            count,
            mean,
            variance,
            std_dev,
            min,
            max,
        })
    }

    /// Returns the coefficient of variation (std_dev / mean).
    pub fn cv(&self) -> f64 {
        if self.mean.abs() > 1e-10 {
            self.std_dev / self.mean.abs()
        } else {
            f64::INFINITY
        }
    }

    /// Returns the range (max - min).
    pub fn range(&self) -> f64 {
        self.max - self.min
    }
}

/// Drift analysis result.
#[derive(Debug, Clone, Default)]
pub struct DriftAnalysis {
    /// Drift rate (units per hour)
    pub drift_rate: f64,
    /// Standard error of drift rate
    pub drift_rate_error: f64,
    /// Correlation coefficient (R²)
    pub r_squared: f64,
    /// Number of data points
    pub num_points: usize,
    /// Is drift statistically significant?
    pub is_significant: bool,
}

/// Calculates drift rate from time-series data.
///
/// # Arguments
/// * `timestamps` - Unix timestamps (seconds)
/// * `values` - Measured values
/// * `significance_threshold` - R² threshold for significance (default: 0.5)
///
/// # Returns
/// DriftAnalysis containing drift rate and statistics
pub fn calculate_drift_rate(
    timestamps: &[u64],
    values: &[f64],
    significance_threshold: f64,
) -> Option<DriftAnalysis> {
    if timestamps.len() != values.len() || timestamps.len() < 2 {
        return None;
    }

    let n = timestamps.len();

    // Convert timestamps to hours relative to first measurement
    let t0 = timestamps[0] as f64;
    let hours: Vec<f64> = timestamps
        .iter()
        .map(|&t| (t as f64 - t0) / 3600.0)
        .collect();

    // Linear regression: value = drift_rate * hours + intercept
    let sum_t: f64 = hours.iter().sum();
    let sum_v: f64 = values.iter().sum();
    let sum_tv: f64 = hours.iter().zip(values.iter()).map(|(t, v)| t * v).sum();
    let sum_t2: f64 = hours.iter().map(|t| t * t).sum();

    let n_f = n as f64;
    let denom = n_f * sum_t2 - sum_t * sum_t;

    if denom.abs() < 1e-20 {
        return Some(DriftAnalysis {
            drift_rate: 0.0,
            drift_rate_error: f64::INFINITY,
            r_squared: 0.0,
            num_points: n,
            is_significant: false,
        });
    }

    let drift_rate = (n_f * sum_tv - sum_t * sum_v) / denom;
    let intercept = (sum_v - drift_rate * sum_t) / n_f;

    // Calculate R²
    let v_mean = sum_v / n_f;
    let ss_tot: f64 = values.iter().map(|v| (v - v_mean).powi(2)).sum();
    let ss_res: f64 = hours
        .iter()
        .zip(values.iter())
        .map(|(t, v)| {
            let predicted = drift_rate * t + intercept;
            (v - predicted).powi(2)
        })
        .sum();

    let r_squared = if ss_tot > 1e-20 {
        1.0 - ss_res / ss_tot
    } else {
        0.0
    };

    // Standard error of slope
    let mse = ss_res / (n_f - 2.0).max(1.0);
    let se_slope = (mse / (sum_t2 - sum_t * sum_t / n_f).max(1e-20)).sqrt();

    Some(DriftAnalysis {
        drift_rate,
        drift_rate_error: se_slope,
        r_squared,
        num_points: n,
        is_significant: r_squared >= significance_threshold,
    })
}

/// Anomaly detection result.
#[derive(Debug, Clone)]
pub struct AnomalyResult {
    /// Is the value anomalous?
    pub is_anomaly: bool,
    /// Z-score of the value
    pub z_score: f64,
    /// Type of anomaly (if detected)
    pub anomaly_type: Option<AnomalyType>,
}

/// Types of detected anomalies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnomalyType {
    /// Value exceeds upper threshold
    HighOutlier,
    /// Value below lower threshold
    LowOutlier,
    /// Sudden change from previous value
    SuddenChange,
    /// Burst event (temporary degradation)
    BurstEvent,
}

/// Detects if a value is anomalous based on z-score.
///
/// # Arguments
/// * `value` - The value to check
/// * `mean` - Historical mean
/// * `std_dev` - Historical standard deviation
/// * `threshold_sigma` - Number of standard deviations for anomaly (typically 3)
pub fn detect_anomaly(value: f64, mean: f64, std_dev: f64, threshold_sigma: f64) -> AnomalyResult {
    let z_score = if std_dev > 1e-10 {
        (value - mean) / std_dev
    } else {
        0.0
    };

    let is_anomaly = z_score.abs() > threshold_sigma;

    let anomaly_type = if is_anomaly {
        if z_score > 0.0 {
            Some(AnomalyType::HighOutlier)
        } else {
            Some(AnomalyType::LowOutlier)
        }
    } else {
        None
    };

    AnomalyResult {
        is_anomaly,
        z_score,
        anomaly_type,
    }
}

/// Detects sudden change between consecutive measurements.
///
/// # Arguments
/// * `current` - Current value
/// * `previous` - Previous value
/// * `std_dev` - Historical standard deviation
/// * `change_threshold` - Threshold in units of std_dev (typically 2-3)
pub fn detect_sudden_change(
    current: f64,
    previous: f64,
    std_dev: f64,
    change_threshold: f64,
) -> AnomalyResult {
    let change = (current - previous).abs();
    let normalized_change = if std_dev > 1e-10 {
        change / std_dev
    } else {
        0.0
    };

    let is_anomaly = normalized_change > change_threshold;

    AnomalyResult {
        is_anomaly,
        z_score: normalized_change,
        anomaly_type: if is_anomaly {
            Some(AnomalyType::SuddenChange)
        } else {
            None
        },
    }
}

/// Exponential moving average for drift tracking.
#[derive(Debug, Clone)]
pub struct ExponentialMovingAverage {
    /// Current EMA value
    value: Option<f64>,
    /// Smoothing factor (0 < α ≤ 1)
    alpha: f64,
}

impl ExponentialMovingAverage {
    /// Creates a new EMA with the given smoothing factor.
    ///
    /// Alpha closer to 1 = more weight on recent values.
    /// Alpha closer to 0 = more weight on historical values.
    pub fn new(alpha: f64) -> Self {
        Self {
            value: None,
            alpha: alpha.clamp(0.01, 1.0),
        }
    }

    /// Creates an EMA from a period (number of samples).
    ///
    /// Alpha = 2 / (period + 1)
    pub fn from_period(period: usize) -> Self {
        let alpha = 2.0 / (period as f64 + 1.0);
        Self::new(alpha)
    }

    /// Updates the EMA with a new value and returns the new EMA.
    pub fn update(&mut self, value: f64) -> f64 {
        let new_ema = match self.value {
            Some(prev) => self.alpha * value + (1.0 - self.alpha) * prev,
            None => value,
        };
        self.value = Some(new_ema);
        new_ema
    }

    /// Returns the current EMA value.
    pub fn value(&self) -> Option<f64> {
        self.value
    }

    /// Resets the EMA.
    pub fn reset(&mut self) {
        self.value = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistics_basic() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = Statistics::compute(&values).unwrap();

        assert_eq!(stats.count, 5);
        assert!((stats.mean - 3.0).abs() < 1e-10);
        assert!((stats.variance - 2.5).abs() < 1e-10); // Sample variance
        assert!((stats.min - 1.0).abs() < 1e-10);
        assert!((stats.max - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_statistics_empty() {
        let values: Vec<f64> = vec![];
        assert!(Statistics::compute(&values).is_none());
    }

    #[test]
    fn test_statistics_single() {
        let values = vec![42.0];
        let stats = Statistics::compute(&values).unwrap();

        assert_eq!(stats.count, 1);
        assert!((stats.mean - 42.0).abs() < 1e-10);
        assert!((stats.variance - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_drift_rate_no_drift() {
        let timestamps = vec![0, 3600, 7200, 10800, 14400]; // Every hour
        let values = vec![100.0, 100.0, 100.0, 100.0, 100.0]; // Constant

        let analysis = calculate_drift_rate(&timestamps, &values, 0.5).unwrap();
        assert!(analysis.drift_rate.abs() < 0.01);
        assert!(!analysis.is_significant);
    }

    #[test]
    fn test_drift_rate_linear() {
        let timestamps = vec![0, 3600, 7200, 10800, 14400]; // Every hour
        let values = vec![100.0, 99.0, 98.0, 97.0, 96.0]; // -1 per hour

        let analysis = calculate_drift_rate(&timestamps, &values, 0.5).unwrap();
        assert!((analysis.drift_rate - (-1.0)).abs() < 0.01);
        assert!(analysis.r_squared > 0.99);
        assert!(analysis.is_significant);
    }

    #[test]
    fn test_anomaly_detection() {
        let result = detect_anomaly(110.0, 100.0, 5.0, 3.0);
        assert!(!result.is_anomaly); // z = 2, not anomalous at 3σ

        let result = detect_anomaly(120.0, 100.0, 5.0, 3.0);
        assert!(result.is_anomaly); // z = 4, anomalous at 3σ
        assert_eq!(result.anomaly_type, Some(AnomalyType::HighOutlier));
    }

    #[test]
    fn test_sudden_change_detection() {
        let result = detect_sudden_change(105.0, 100.0, 5.0, 2.0);
        assert!(!result.is_anomaly); // Change = 1σ

        let result = detect_sudden_change(120.0, 100.0, 5.0, 2.0);
        assert!(result.is_anomaly); // Change = 4σ
    }

    #[test]
    fn test_ema() {
        let mut ema = ExponentialMovingAverage::new(0.5);

        // First value is the EMA
        assert!((ema.update(100.0) - 100.0).abs() < 1e-10);

        // Second value: EMA = 0.5 * 110 + 0.5 * 100 = 105
        assert!((ema.update(110.0) - 105.0).abs() < 1e-10);

        // Third value: EMA = 0.5 * 100 + 0.5 * 105 = 102.5
        assert!((ema.update(100.0) - 102.5).abs() < 1e-10);
    }

    #[test]
    fn test_ema_from_period() {
        let ema = ExponentialMovingAverage::from_period(9);
        // Alpha = 2 / (9 + 1) = 0.2
        assert!((ema.alpha - 0.2).abs() < 1e-10);
    }
}
