//! # QNS Core
//!
//! Core types and utilities for QNS (Quantum Noise Symbiote).
//!
//! This crate provides:
//! - `Gate`: Quantum gate enumeration (12 variants) with matrix representations
//! - `NoiseVector`: Noise profile data structure with gate errors and fidelity estimation
//! - `CircuitGenome`: Quantum circuit representation
//! - `HardwareProfile`: Quantum hardware topology and characteristics
//! - `HardwareBackend`: Hardware abstraction trait for simulators and real hardware
//! - `QnsError`: Unified error types
//! - Physical constants and gate matrices
//!
//! ## Example
//!
//! ```rust
//! use qns_core::prelude::*;
//!
//! let mut circuit = CircuitGenome::new(3);
//! circuit.add_gate(Gate::H(0)).unwrap();
//! circuit.add_gate(Gate::CNOT(0, 1)).unwrap();
//!
//! // Check commutativity for optimization
//! assert!(Gate::Z(0).commutes_with(&Gate::Rz(0, 0.5)));
//! ```

pub mod backend;
pub mod config;
pub mod error;
pub mod physics;
pub mod prelude;
pub mod types;

pub use backend::{ExecutionResult, HardwareBackend};
pub use error::{QnsError, Result};
pub use types::{CircuitGenome, CircuitMetadata, Fidelity, Gate, HardwareProfile, NoiseVector};
