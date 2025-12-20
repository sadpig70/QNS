//! # QNS CLI
//!
//! Command-line interface and integration library for QNS.
//!
//! This crate provides:
//! - QnsSystem: Unified pipeline integrating all QNS components
//! - CLI commands for profiling, optimization, and benchmarking
//!
//! ## Library Usage
//!
//! ```rust
//! use qns_cli::pipeline::{QnsSystem, PipelineConfig};
//! use qns_core::prelude::*;
//!
//! // Create a circuit
//! let mut circuit = CircuitGenome::new(2);
//! circuit.add_gate(Gate::H(0)).unwrap();
//! circuit.add_gate(Gate::CNOT(0, 1)).unwrap();
//!
//! // Optimize
//! let mut system = QnsSystem::new();
//! let result = system.optimize(circuit).unwrap();
//! ```

pub mod pipeline;

pub use pipeline::{BenchmarkResult, PipelineConfig, PipelineResult, PipelineTiming, QnsSystem};
