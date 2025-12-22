use qns_core::prelude::*;
use std::collections::HashMap;

/// Dependency Graph for Quantum Circuits.
/// Represents the circuit as a DAG where nodes are gate indices and edges represent dependency.
/// Used by SABRE to identify the "Front Layer" of executable gates.
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Adjacency list: node -> children (dependents)
    pub successors: Vec<Vec<usize>>,
    /// Predecessor count: node -> number of unsatisfied dependencies
    pub incoming_degree: Vec<usize>,
    /// Number of gates
    pub num_gates: usize,
}

impl DependencyGraph {
    pub fn new(circuit: &CircuitGenome) -> Self {
        let n = circuit.gates.len();
        let mut successors: Vec<Vec<usize>> = Vec::with_capacity(n);
        for _ in 0..n {
            successors.push(Vec::new());
        }
        let mut incoming_degree = vec![0; n];

        // Track the last gate index that acted on each qubit
        let mut last_gate_on_qubit = HashMap::new();

        for (idx, gate) in circuit.gates.iter().enumerate() {
            for &q in gate.qubits().iter() {
                if let Some(&prev_idx) = last_gate_on_qubit.get(&q) {
                    // Dependency: prev_idx -> idx
                    let deps: &mut Vec<usize> = &mut successors[prev_idx];
                    deps.push(idx);
                    incoming_degree[idx] += 1;
                }
                last_gate_on_qubit.insert(q, idx);
            }
        }

        Self {
            successors,
            incoming_degree,
            num_gates: n,
        }
    }

    /// Returns the initial front layer (gates with no dependencies).
    pub fn initial_front_layer(&self) -> Vec<usize> {
        self.incoming_degree
            .iter()
            .enumerate()
            .filter_map(|(idx, &deg)| if deg == 0 { Some(idx) } else { None })
            .collect::<Vec<usize>>() // Explicit type hint
    }
}
