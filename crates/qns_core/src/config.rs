//! Configuration management for QNS.

use serde::{Deserialize, Serialize};

/// Global configuration for QNS system.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QnsConfig {
    /// Profiler configuration
    pub profiler: ProfilerConfig,
    /// Rewire configuration
    pub rewire: RewireConfig,
    /// Simulator configuration
    pub simulator: SimulatorConfig,
}

/// Configuration for the noise profiler.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilerConfig {
    /// Number of samples to collect per measurement
    pub sample_count: usize,
    /// Sigma threshold for anomaly detection
    pub threshold_sigma: f64,
    /// Base T1 time (μs) for simulation
    pub t1_base: f64,
    /// Base T2 time (μs) for simulation
    pub t2_base: f64,
    /// Scan interval in seconds
    pub scan_interval_sec: u64,
}

/// Configuration for the circuit rewirer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewireConfig {
    /// Maximum number of variants to generate
    pub max_variants: usize,
    /// Minimum fitness threshold for acceptance
    pub fitness_threshold: f64,
    /// Enable parallel variant generation
    pub parallel: bool,
}

/// Configuration for the quantum simulator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatorConfig {
    /// Maximum number of qubits
    pub max_qubits: usize,
    /// Number of measurement shots
    pub default_shots: usize,
    /// Enable noise simulation
    pub enable_noise: bool,
}

impl Default for ProfilerConfig {
    fn default() -> Self {
        Self {
            sample_count: 1000,
            threshold_sigma: 3.0,
            t1_base: 100.0,
            t2_base: 80.0,
            scan_interval_sec: 300,
        }
    }
}

impl Default for RewireConfig {
    fn default() -> Self {
        Self {
            max_variants: 10,
            fitness_threshold: 0.9,
            parallel: true,
        }
    }
}

impl Default for SimulatorConfig {
    fn default() -> Self {
        Self {
            max_qubits: 10,
            default_shots: 1000,
            enable_noise: false,
        }
    }
}

impl QnsConfig {
    /// Creates a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads configuration from a JSON string.
    pub fn from_json(json: &str) -> crate::Result<Self> {
        serde_json::from_str(json).map_err(Into::into)
    }

    /// Serializes configuration to a JSON string.
    pub fn to_json(&self) -> crate::Result<String> {
        serde_json::to_string_pretty(self).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = QnsConfig::default();
        assert_eq!(config.profiler.sample_count, 1000);
        assert_eq!(config.rewire.max_variants, 10);
        assert_eq!(config.simulator.max_qubits, 10);
    }

    #[test]
    fn test_json_roundtrip() {
        let config = QnsConfig::default();
        let json = config.to_json().unwrap();
        let parsed = QnsConfig::from_json(&json).unwrap();

        assert_eq!(config.profiler.sample_count, parsed.profiler.sample_count);
    }
}
