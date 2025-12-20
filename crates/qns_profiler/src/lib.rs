//! # QNS Profiler
//!
//! Noise profiling module for QNS (Quantum Noise Symbiote).
//!
//! ## Features
//!
//! - `DriftScanner`: T1/T2 drift measurement and analysis
//! - Anomaly detection with configurable thresholds
//! - Historical tracking and drift rate calculation
//!
//! ## Example
//!
//! ```rust
//! use qns_profiler::{DriftScanner, ScanConfig};
//! use qns_core::prelude::*;
//!
//! let mut scanner = DriftScanner::with_defaults();
//!
//! // Perform a scan on qubit 0
//! let noise_vector = scanner.scan(0).unwrap();
//!
//! println!("T1 = {:.1} μs, T2 = {:.1} μs",
//!     noise_vector.t1_mean, noise_vector.t2_mean);
//!
//! // Check for anomalies
//! if scanner.is_anomaly(&noise_vector) {
//!     println!("Warning: Anomaly detected!");
//! }
//! ```
//!
//! ## Performance
//!
//! The default `DriftScanner` targets <10ms per scan.
//! Use `DriftScanner::fast()` for <5ms latency.
//! Use `DriftScanner::accurate()` for higher precision.

pub mod drift_scan;

// Re-export main types
pub use drift_scan::{
    AnomalyAnalysis, AnomalyResult, AnomalyType, DriftAnalysis, DriftScanner,
    ExponentialMovingAverage, ScanConfig, Statistics, T1Measurement, T2Measurement,
};
