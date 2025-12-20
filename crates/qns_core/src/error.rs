//! Error types for QNS.

use thiserror::Error;

/// Unified error type for QNS operations.
#[derive(Error, Debug)]
pub enum QnsError {
    // ============ Profiler Errors ============
    /// Generic profiler error
    #[error("Profiler error: {0}")]
    Profiler(String),

    /// Invalid qubit index
    #[error("Invalid qubit index {0}, maximum is {1}")]
    InvalidQubit(usize, usize),

    /// Anomaly detected during profiling
    #[error("Anomaly detected: drift_rate={0:.4}")]
    AnomalyDetected(f64),

    /// Insufficient samples
    #[error("Insufficient samples: got {0}, need {1}")]
    InsufficientSamples(usize, usize),

    // ============ Rewire Errors ============
    /// Generic rewire error
    #[error("Rewire error: {0}")]
    Rewire(String),

    /// No circuit loaded
    #[error("No circuit loaded")]
    NoCircuitLoaded,

    /// No valid variants found
    #[error("No valid variants found after {0} attempts")]
    NoValidVariants(usize),

    /// Invalid gate sequence
    #[error("Invalid gate sequence: {0}")]
    InvalidGateSequence(String),

    // ============ Simulator Errors ============
    /// Generic simulator error
    #[error("Simulator error: {0}")]
    Simulator(String),

    /// State vector dimension mismatch
    #[error("Dimension mismatch: expected {0}, got {1}")]
    DimensionMismatch(usize, usize),

    /// Invalid quantum state
    #[error("Invalid quantum state: {0}")]
    InvalidState(String),

    // ============ Backend Errors (v2.0) ============
    /// Generic backend error
    #[error("Backend error: {0}")]
    Backend(String),

    /// Backend not available
    #[error("Backend '{0}' is not available")]
    BackendUnavailable(String),

    /// Calibration data stale or unavailable
    #[error("Calibration data stale: last updated {0} seconds ago")]
    CalibrationStale(u64),

    /// Circuit execution failed
    #[error("Circuit execution failed: {0}")]
    ExecutionFailed(String),

    /// Unsupported operation
    #[error("Unsupported operation: {0}")]
    Unsupported(String),

    // ============ Config Errors ============
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    // ============ I/O Errors ============
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type alias for QNS operations.
pub type Result<T> = std::result::Result<T, QnsError>;

impl QnsError {
    /// Creates a profiler error with the given message.
    pub fn profiler(msg: impl Into<String>) -> Self {
        Self::Profiler(msg.into())
    }

    /// Creates a rewire error with the given message.
    pub fn rewire(msg: impl Into<String>) -> Self {
        Self::Rewire(msg.into())
    }

    /// Creates a simulator error with the given message.
    pub fn simulator(msg: impl Into<String>) -> Self {
        Self::Simulator(msg.into())
    }

    /// Creates a config error with the given message.
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Creates a backend error with the given message.
    pub fn backend(msg: impl Into<String>) -> Self {
        Self::Backend(msg.into())
    }

    /// Creates an execution failed error.
    pub fn execution_failed(msg: impl Into<String>) -> Self {
        Self::ExecutionFailed(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = QnsError::InvalidQubit(5, 3);
        assert!(err.to_string().contains("5"));
        assert!(err.to_string().contains("3"));
    }

    #[test]
    fn test_result_type() {
        fn returns_ok() -> Result<i32> {
            Ok(42)
        }

        fn returns_err() -> Result<i32> {
            Err(QnsError::profiler("test error"))
        }

        assert!(returns_ok().is_ok());
        assert!(returns_err().is_err());
    }

    #[test]
    fn test_backend_errors() {
        let err = QnsError::backend("connection failed");
        assert!(err.to_string().contains("connection failed"));

        let err = QnsError::BackendUnavailable("ibmq_manila".to_string());
        assert!(err.to_string().contains("ibmq_manila"));

        let err = QnsError::CalibrationStale(3600);
        assert!(err.to_string().contains("3600"));
    }
}
