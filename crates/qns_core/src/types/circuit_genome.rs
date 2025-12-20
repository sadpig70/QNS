//! Circuit genome data structure.

use super::Gate;
use crate::error::{QnsError, Result};
use serde::{Deserialize, Serialize};

/// Metadata for a circuit genome.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CircuitMetadata {
    /// Species ID (if part of species bank)
    pub species_id: Option<String>,
    /// Generation number in evolution
    pub generation: usize,
    /// Fitness score (0.0 - 1.0)
    pub fitness_score: f64,
    /// Parent circuit ID
    pub parent_id: Option<String>,
}

/// Quantum circuit representation.
///
/// A circuit genome contains the gate sequence and metadata
/// for evolutionary optimization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitGenome {
    /// Number of qubits in the circuit
    pub num_qubits: usize,
    /// Sequence of quantum gates
    pub gates: Vec<Gate>,
    /// Circuit metadata
    pub metadata: CircuitMetadata,
}

impl CircuitGenome {
    /// Creates a new empty circuit with the specified number of qubits.
    pub fn new(num_qubits: usize) -> Self {
        Self {
            num_qubits,
            gates: Vec::new(),
            metadata: CircuitMetadata::default(),
        }
    }

    /// Creates a circuit with pre-allocated gate capacity.
    pub fn with_capacity(num_qubits: usize, gate_capacity: usize) -> Self {
        Self {
            num_qubits,
            gates: Vec::with_capacity(gate_capacity),
            metadata: CircuitMetadata::default(),
        }
    }

    /// Adds a gate to the circuit.
    ///
    /// Returns an error if the gate operates on invalid qubit indices.
    pub fn add_gate(&mut self, gate: Gate) -> Result<()> {
        for q in gate.qubits() {
            if q >= self.num_qubits {
                return Err(QnsError::InvalidQubit(q, self.num_qubits));
            }
        }
        self.gates.push(gate);
        Ok(())
    }

    /// Adds multiple gates to the circuit.
    pub fn add_gates(&mut self, gates: impl IntoIterator<Item = Gate>) -> Result<()> {
        for gate in gates {
            self.add_gate(gate)?;
        }
        Ok(())
    }

    /// Returns the circuit depth (critical path length).
    ///
    /// The depth is the maximum number of gates that must be
    /// executed sequentially on any qubit.
    pub fn depth(&self) -> usize {
        if self.gates.is_empty() {
            return 0;
        }

        let mut qubit_depths = vec![0usize; self.num_qubits];

        for gate in &self.gates {
            let qs = gate.qubits();
            let max_depth = qs.iter().map(|&q| qubit_depths[q]).max().unwrap_or(0);
            for &q in &qs {
                qubit_depths[q] = max_depth + 1;
            }
        }

        qubit_depths.into_iter().max().unwrap_or(0)
    }

    /// Returns the total gate count.
    pub fn gate_count(&self) -> usize {
        self.gates.len()
    }

    /// Returns the number of two-qubit gates.
    pub fn two_qubit_gate_count(&self) -> usize {
        self.gates.iter().filter(|g| g.is_two_qubit()).count()
    }

    /// Clears all gates from the circuit.
    pub fn clear(&mut self) {
        self.gates.clear();
    }

    /// Creates a deep clone with new metadata.
    pub fn clone_with_new_metadata(&self) -> Self {
        Self {
            num_qubits: self.num_qubits,
            gates: self.gates.clone(),
            metadata: CircuitMetadata::default(),
        }
    }
}

impl Default for CircuitGenome {
    fn default() -> Self {
        Self::new(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let circuit = CircuitGenome::new(3);
        assert_eq!(circuit.num_qubits, 3);
        assert!(circuit.gates.is_empty());
    }

    #[test]
    fn test_add_gate() {
        let mut circuit = CircuitGenome::new(3);
        assert!(circuit.add_gate(Gate::H(0)).is_ok());
        assert!(circuit.add_gate(Gate::CNOT(0, 1)).is_ok());
        assert_eq!(circuit.gate_count(), 2);
    }

    #[test]
    fn test_add_gate_invalid_qubit() {
        let mut circuit = CircuitGenome::new(2);
        assert!(circuit.add_gate(Gate::H(2)).is_err());
        assert!(circuit.add_gate(Gate::CNOT(0, 5)).is_err());
    }

    #[test]
    fn test_depth() {
        let mut circuit = CircuitGenome::new(3);
        // Empty circuit has depth 0
        assert_eq!(circuit.depth(), 0);

        // H(0) -> depth 1
        circuit.add_gate(Gate::H(0)).unwrap();
        assert_eq!(circuit.depth(), 1);

        // H(0), H(1) -> depth 1 (parallel)
        circuit.add_gate(Gate::H(1)).unwrap();
        assert_eq!(circuit.depth(), 1);

        // H(0), H(1), CNOT(0,1) -> depth 2
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();
        assert_eq!(circuit.depth(), 2);

        // Add another gate on qubit 0 -> depth 3
        circuit.add_gate(Gate::X(0)).unwrap();
        assert_eq!(circuit.depth(), 3);
    }

    #[test]
    fn test_two_qubit_gate_count() {
        let mut circuit = CircuitGenome::new(3);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();
        circuit.add_gate(Gate::CZ(1, 2)).unwrap();
        circuit.add_gate(Gate::X(2)).unwrap();

        assert_eq!(circuit.two_qubit_gate_count(), 2);
    }
}
