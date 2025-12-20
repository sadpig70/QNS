//! T1/T2 drift scanning module.
//!
//! This module provides tools for measuring and tracking quantum
//! coherence times (T1, T2) and detecting drift and anomalies.
//!
//! ## Modules
//!
//! - `measure`: T1/T2 measurement simulation
//! - `compute`: Statistical analysis and drift calculation
//! - `scanner`: Main DriftScanner implementation

pub mod compute;
pub mod measure;
mod scanner;

pub use compute::{
    AnomalyResult, AnomalyType, DriftAnalysis, ExponentialMovingAverage, Statistics,
};
pub use measure::{T1Measurement, T2Measurement};
pub use scanner::{AnomalyAnalysis, DriftScanner, ScanConfig};
