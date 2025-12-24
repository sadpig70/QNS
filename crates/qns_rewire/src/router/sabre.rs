use crate::graph::DependencyGraph;
use qns_core::prelude::*;
use std::collections::HashSet;

/// SABRE: Swap-Based BidiREctional search router.
///
/// Addresses local minima in greedy search by using a heuristic cost function
/// that considers both forward and backward routing passes.
///
/// # Cost Function v2
/// Cost = W_dist * distance + W_err * error_rate + W_xtalk * crosstalk_penalty
pub struct SabreRouter {
    /// Lookahead weight (W)
    pub lookahead_weight: f64,
    /// Decay factor for heuristic
    pub decay_rate: f64,
    /// Max iterations for bidirectional passes
    pub max_iterations: usize,

    /// Weight for distance cost (swaps)
    pub dist_weight: f64,
    /// Weight for gate error cost
    pub error_weight: f64,
    /// Weight for crosstalk cost
    pub crosstalk_weight: f64,
}

impl Default for SabreRouter {
    fn default() -> Self {
        Self {
            lookahead_weight: 0.5,
            decay_rate: 0.001,
            max_iterations: 10,
            dist_weight: 1.0,
            error_weight: 0.5,     // Default: balanced error awareness
            crosstalk_weight: 0.5, // Default: balanced crosstalk awareness
        }
    }
}

impl SabreRouter {
    pub fn new(
        lookahead_weight: f64,
        decay_rate: f64,
        max_iterations: usize,
        crosstalk_weight: f64,
    ) -> Self {
        Self {
            lookahead_weight,
            decay_rate,
            max_iterations,
            dist_weight: 1.0,  // Standard distance weight
            error_weight: 0.5, // Default error weight
            crosstalk_weight,
        }
    }

    /// Route circuit using SABRE algorithm.
    pub fn route(
        &self,
        circuit: &CircuitGenome,
        hardware: &HardwareProfile,
    ) -> Result<(CircuitGenome, Vec<usize>)> {
        // Returns (routed_circuit, final_mapping)
        // 1. Initialize mapping (trivial for now, can be optimized)
        let mut mapping: Vec<usize> = (0..circuit.num_qubits).collect(); // Logical -> Physical

        // 2. Build Dependency Graph
        let dag = DependencyGraph::new(circuit);

        // 3. Routing (Simplified One-Pass for initial integration)
        // TODO: Implement full bidirectional sweep
        self.route_pass(circuit, &dag, hardware, &mut mapping)
    }

    fn route_pass(
        &self,
        circuit: &CircuitGenome,
        dag: &DependencyGraph,
        hardware: &HardwareProfile,
        mapping: &mut [usize], // Logical -> Physical
    ) -> Result<(CircuitGenome, Vec<usize>)> {
        let mut routed_gates = Vec::new();
        let mut incoming_degree = dag.incoming_degree.clone();

        // Front Layer: Gates with no dependencies
        let mut front_layer: Vec<usize> = dag.initial_front_layer();
        let mut executed_gates = HashSet::new();

        // While there are gates to execute
        while executed_gates.len() < circuit.gates.len() {
            let mut executable_gates = Vec::new();

            // 1. Check which gates in front layer are executable on current hardware
            // (i.e., qubits are adjacent)
            for &gate_idx in &front_layer {
                let gate = &circuit.gates[gate_idx];
                if self.is_executable(gate, mapping, hardware) {
                    executable_gates.push(gate_idx);
                }
            }

            if !executable_gates.is_empty() {
                // Execute gates
                for &gate_idx in &executable_gates {
                    let gate = &circuit.gates[gate_idx];

                    // Remap gate to physical qubits
                    let mapped_gate = self.remap_gate(gate, mapping);
                    routed_gates.push(mapped_gate);

                    executed_gates.insert(gate_idx);

                    // Update dependencies
                    for &child in &dag.successors[gate_idx] {
                        incoming_degree[child] -= 1;
                        if incoming_degree[child] == 0 {
                            front_layer.push(child);
                        }
                    }
                }
                // Remove executed gates from front_layer
                front_layer.retain(|idx| !executed_gates.contains(idx));
            } else {
                // No executable gates -> Insert SWAP
                // Heuristic: Choose SWAP that minimizes cost function
                let best_swap = self.find_best_swap(&front_layer, circuit, mapping, hardware);

                if let Some((p1, p2)) = best_swap {
                    // Update mapping (swap logical assignments)
                    // mapping[l] = p -> if we swap p1, p2, we need to find l1, l2 s.t. mapping[l1]=p1, mapping[l2]=p2
                    let l1 = mapping.iter().position(|&p| p == p1).unwrap();
                    let l2 = mapping.iter().position(|&p| p == p2).unwrap();
                    mapping.swap(l1, l2);

                    // Add SWAP gate
                    routed_gates.push(Gate::SWAP(p1, p2));
                } else {
                    return Err(QnsError::Rewire(
                        "Deadlock: No valid swap found".to_string(),
                    ));
                }
            }
        }

        let mut routed_circuit = CircuitGenome::new(circuit.num_qubits);
        for g in routed_gates {
            routed_circuit.add_gate(g)?;
        }

        Ok((routed_circuit, mapping.to_vec()))
    }

    /// Check if gate is executable on current mapping
    fn is_executable(&self, gate: &Gate, mapping: &[usize], hardware: &HardwareProfile) -> bool {
        match gate {
            Gate::CNOT(c, t) | Gate::CZ(c, t) | Gate::SWAP(c, t) => {
                let p_c = mapping[*c];
                let p_t = mapping[*t];
                hardware.are_connected(p_c, p_t)
            },
            _ => true, // Single qubit gates are always executable
        }
    }

    /// Remap gate from logical to physical qubits
    fn remap_gate(&self, gate: &Gate, mapping: &[usize]) -> Gate {
        match gate {
            Gate::H(q) => Gate::H(mapping[*q]),
            Gate::X(q) => Gate::X(mapping[*q]),
            Gate::Y(q) => Gate::Y(mapping[*q]),
            Gate::Z(q) => Gate::Z(mapping[*q]),
            Gate::S(q) => Gate::S(mapping[*q]),
            Gate::T(q) => Gate::T(mapping[*q]),
            Gate::Rx(q, t) => Gate::Rx(mapping[*q], *t),
            Gate::Ry(q, t) => Gate::Ry(mapping[*q], *t),
            Gate::Rz(q, t) => Gate::Rz(mapping[*q], *t),
            Gate::CNOT(c, t) => Gate::CNOT(mapping[*c], mapping[*t]),
            Gate::CZ(c, t) => Gate::CZ(mapping[*c], mapping[*t]),
            Gate::SWAP(c, t) => Gate::SWAP(mapping[*c], mapping[*t]),
            Gate::Measure(q) => Gate::Measure(mapping[*q]),
        }
    }

    /// Find best SWAP to reduce heuristic cost
    fn find_best_swap(
        &self,
        front_layer: &[usize],
        circuit: &CircuitGenome,
        mapping: &[usize],
        hardware: &HardwareProfile,
    ) -> Option<(usize, usize)> {
        // (p1, p2)
        let mut best_score = f64::INFINITY;
        let mut best_swap = None;

        // Consider all physical edges as candidate SWAPs
        for coupler in hardware.coupling_map() {
            let p1 = coupler.qubit1;
            let p2 = coupler.qubit2;

            // Calculate score if we perform this SWAP
            // Note: Score assumes SWAP is performed, so we temporarily swap mapping
            let mut temp_mapping = mapping.to_vec();
            let l1 = temp_mapping.iter().position(|&p| p == p1);
            let l2 = temp_mapping.iter().position(|&p| p == p2);

            // If these physical qubits map to logical qubits active in front layer, it's relevant
            // Optimization: Only swap if it affects qubits in the front layer (basic heuristic)
            // But standard Sabre tries all edges to escape local minima. We stick to standard all-edge trial for now.
            if let (Some(idx1), Some(idx2)) = (l1, l2) {
                temp_mapping.swap(idx1, idx2);
                let score = self.heuristic_score(front_layer, circuit, &temp_mapping, hardware);
                if score < best_score {
                    best_score = score;
                    best_swap = Some((p1, p2));
                }
            }
        }
        best_swap
    }

    /// Calculate H-score (Weighted sum of Distance + Error + Crosstalk)
    fn heuristic_score(
        &self,
        front_layer: &[usize],
        circuit: &CircuitGenome,
        mapping: &[usize],
        hardware: &HardwareProfile,
    ) -> f64 {
        let mut total_dist = 0.0;
        let mut total_error = 0.0;
        let mut total_xtalk = 0.0;

        // Collect active physical edges for crosstalk check
        let mut active_edges = Vec::new();

        for &gate_idx in front_layer {
            let gate = &circuit.gates[gate_idx];
            match gate {
                Gate::CNOT(c, t) | Gate::CZ(c, t) | Gate::SWAP(c, t) => {
                    let p_c = mapping[*c];
                    let p_t = mapping[*t];

                    // 1. Distance Cost
                    let dist = hardware.shortest_path_distance(p_c, p_t).unwrap_or(100) as f64;
                    total_dist += dist;

                    // 2. Gate Error Cost (if adjacent)
                    // If dist == 1, we can check the actual edge error
                    if dist <= 1.5 {
                        if let Some(coupler) = hardware.get_coupler(p_c, p_t) {
                            total_error += coupler.gate_fidelity.error_rate();
                        }
                        active_edges.push((p_c, p_t));
                    }
                },
                _ => {},
            }
        }

        // 3. Crosstalk Cost
        // Calculate interaction between all pairs of active edges
        if self.crosstalk_weight > 0.0 && !active_edges.is_empty() {
            for i in 0..active_edges.len() {
                for j in (i + 1)..active_edges.len() {
                    let (q1, q2) = active_edges[i];
                    let (q3, q4) = active_edges[j];

                    // Check interactions: (q1, q3), (q1, q4), (q2, q3), (q2, q4)
                    // CrosstalkMatrix stores interactions between pairs.
                    // We verify if activating edge (q1,q2) affects edge (q3,q4).
                    // Simplified model: sum of all pairwise qubit interactions between the two edges.

                    if let Some(s) = hardware.crosstalk.get_interaction(q1, q3) {
                        total_xtalk += s;
                    }
                    if let Some(s) = hardware.crosstalk.get_interaction(q1, q4) {
                        total_xtalk += s;
                    }
                    if let Some(s) = hardware.crosstalk.get_interaction(q2, q3) {
                        total_xtalk += s;
                    }
                    if let Some(s) = hardware.crosstalk.get_interaction(q2, q4) {
                        total_xtalk += s;
                    }
                }
            }
        }

        // Weighted Sum
        self.dist_weight * total_dist +
        self.error_weight * total_error * 10.0 + // Scale error to be comparable to distance (0.01 vs 1.0)
        self.crosstalk_weight * total_xtalk * 100.0 // Scale crosstalk (0.001 ranges)
    }
}
