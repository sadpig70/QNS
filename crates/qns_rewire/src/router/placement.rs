//! Placement Optimizer - Finds optimal logical-to-physical qubit mapping
//!
//! This module implements the key insight for "route-through-better-edges":
//! By choosing which physical qubits to use for logical qubits, we can ensure
//! that high-frequency 2-qubit gates use high-fidelity edges.
//!
//! Example:
//! ```text
//! Physical: Q0 --99%-- Q1 --95%-- Q2
//! Logical circuit: CNOT(L0, L1)
//!
//! Mapping A: L0→P0, L1→P1 → uses 99% edge ✓
//! Mapping B: L0→P1, L1→P2 → uses 95% edge ✗
//! ```

use qns_core::types::Gate;
use qns_core::{CircuitGenome, HardwareProfile};
use std::collections::HashMap;

/// Result of placement optimization
#[derive(Debug, Clone)]
pub struct PlacementResult {
    /// Logical to physical qubit mapping
    pub mapping: Vec<usize>,
    /// The circuit with remapped qubits
    pub circuit: CircuitGenome,
    /// Estimated fidelity improvement from better placement
    pub improvement: f64,
}

/// Placement optimizer that finds the best qubit mapping
pub struct PlacementOptimizer {
    /// Maximum number of placement iterations
    pub max_iterations: usize,
    /// Use greedy algorithm (faster) vs exhaustive search (better)
    pub greedy: bool,
}

impl Default for PlacementOptimizer {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            greedy: true,
        }
    }
}

impl PlacementOptimizer {
    /// Creates a new PlacementOptimizer
    pub fn new(max_iterations: usize, greedy: bool) -> Self {
        Self {
            max_iterations,
            greedy,
        }
    }

    /// Analyzes the circuit to find interaction frequency between qubit pairs.
    ///
    /// Returns a map from (q1, q2) -> count of 2-qubit gates between them.
    pub fn analyze_interactions(&self, circuit: &CircuitGenome) -> HashMap<(usize, usize), usize> {
        let mut interactions: HashMap<(usize, usize), usize> = HashMap::new();

        for gate in &circuit.gates {
            if let Some((q1, q2)) = get_two_qubit_pair(gate) {
                let key = if q1 < q2 { (q1, q2) } else { (q2, q1) };
                *interactions.entry(key).or_insert(0) += 1;
            }
        }

        interactions
    }

    /// Ranks physical edges by fidelity (highest first).
    ///
    /// Returns list of ((p1, p2), fidelity) sorted by fidelity descending.
    pub fn rank_physical_edges(&self, hardware: &HardwareProfile) -> Vec<((usize, usize), f64)> {
        let mut edges: Vec<_> = hardware
            .couplers
            .iter()
            .map(|c| ((c.qubit1, c.qubit2), c.gate_fidelity.value()))
            .collect();

        // Sort by fidelity descending
        edges.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        edges
    }

    /// Builds a map of logical qubit neighbors (qubits that interact via 2-qubit gates).
    fn build_logical_neighbors(&self, circuit: &CircuitGenome) -> HashMap<usize, Vec<usize>> {
        let mut neighbors: HashMap<usize, Vec<usize>> = HashMap::new();

        for gate in &circuit.gates {
            if let Some((q1, q2)) = get_two_qubit_pair(gate) {
                neighbors.entry(q1).or_default().push(q2);
                neighbors.entry(q2).or_default().push(q1);
            }
        }

        // Deduplicate neighbors
        for v in neighbors.values_mut() {
            v.sort();
            v.dedup();
        }

        neighbors
    }

    /// Finds the optimal placement using greedy matching.
    ///
    /// Algorithm:
    /// 1. Sort logical pairs by interaction count (descending)
    /// 2. Sort physical edges by fidelity (descending)
    /// 3. Greedily assign logical pairs to physical edges
    pub fn optimize_greedy(
        &self,
        circuit: &CircuitGenome,
        hardware: &HardwareProfile,
    ) -> PlacementResult {
        let interactions = self.analyze_interactions(circuit);
        let physical_edges = self.rank_physical_edges(hardware);

        // Sort logical pairs by interaction count (most frequent first)
        let mut logical_pairs: Vec<_> = interactions.into_iter().collect();
        logical_pairs.sort_by(|a, b| b.1.cmp(&a.1));

        // Track which logical and physical qubits are assigned
        let mut mapping: Vec<usize> = (0..circuit.num_qubits).collect();
        let mut assigned_physical: Vec<bool> = vec![false; hardware.num_qubits];
        let mut assigned_logical: Vec<bool> = vec![false; circuit.num_qubits];

        // Greedy assignment: match frequent logical pairs to high-fidelity physical edges
        for ((l1, l2), _count) in &logical_pairs {
            // Skip if either logical qubit is already assigned
            if assigned_logical[*l1] || assigned_logical[*l2] {
                continue;
            }

            // Find the best available physical edge for this logical pair
            for ((p1, p2), _fidelity) in &physical_edges {
                // Check if both physical qubits are available
                if !assigned_physical[*p1] && !assigned_physical[*p2] {
                    // Assign logical to physical
                    if *l1 < mapping.len() && *l2 < mapping.len() {
                        mapping[*l1] = *p1;
                        mapping[*l2] = *p2;
                        assigned_physical[*p1] = true;
                        assigned_physical[*p2] = true;
                        assigned_logical[*l1] = true;
                        assigned_logical[*l2] = true;
                    }
                    break;
                }
            }
        }

        // Assign remaining logical qubits with connectivity awareness
        // For each unassigned logical qubit, find a physical qubit that:
        // 1. Is available
        // 2. Is connected to physical qubits of its logical neighbors (if any are assigned)
        let logical_neighbors = self.build_logical_neighbors(circuit);

        // Sort unassigned qubits by number of assigned neighbors (descending)
        // This ensures qubits with more connectivity constraints are placed first
        let mut unassigned: Vec<usize> = (0..circuit.num_qubits)
            .filter(|&l| !assigned_logical[l])
            .collect();

        unassigned.sort_by(|&a, &b| {
            let a_assigned_neighbors = logical_neighbors
                .get(&a)
                .map(|n| n.iter().filter(|&&x| assigned_logical[x]).count())
                .unwrap_or(0);
            let b_assigned_neighbors = logical_neighbors
                .get(&b)
                .map(|n| n.iter().filter(|&&x| assigned_logical[x]).count())
                .unwrap_or(0);
            b_assigned_neighbors.cmp(&a_assigned_neighbors)
        });

        for l in unassigned {
            if assigned_logical[l] {
                continue;
            }

            // Find physical qubits that logical neighbors are mapped to
            let neighbor_physicals: Vec<usize> = logical_neighbors
                .get(&l)
                .map(|neighbors| {
                    neighbors
                        .iter()
                        .filter(|&&n| assigned_logical[n])
                        .map(|&n| mapping[n])
                        .collect()
                })
                .unwrap_or_default();

            // Try to find a physical qubit connected to any assigned neighbor
            let mut best_physical: Option<usize> = None;

            if !neighbor_physicals.is_empty() {
                // Prefer physical qubits connected to neighbors
                for &np in &neighbor_physicals {
                    for coupler in &hardware.couplers {
                        let candidate = if coupler.qubit1 == np {
                            coupler.qubit2
                        } else if coupler.qubit2 == np {
                            coupler.qubit1
                        } else {
                            continue;
                        };

                        if !assigned_physical[candidate] {
                            best_physical = Some(candidate);
                            break;
                        }
                    }
                    if best_physical.is_some() {
                        break;
                    }
                }
            }

            // Fallback: use any available physical qubit
            let physical = best_physical.unwrap_or_else(|| {
                (0..hardware.num_qubits)
                    .find(|&p| !assigned_physical[p])
                    .unwrap_or(l) // Last resort: identity
            });

            if physical < hardware.num_qubits {
                mapping[l] = physical;
                assigned_physical[physical] = true;
                assigned_logical[l] = true;
            }
        }

        // Apply mapping to circuit
        let remapped_circuit = self.apply_mapping(circuit, &mapping);

        PlacementResult {
            mapping,
            circuit: remapped_circuit,
            improvement: 0.0, // Will be calculated by caller
        }
    }

    /// Finds optimal placement using local search (swap-based improvement).
    ///
    /// Starts from identity or greedy solution, then tries swapping qubit assignments.
    pub fn optimize_local_search(
        &self,
        circuit: &CircuitGenome,
        hardware: &HardwareProfile,
    ) -> PlacementResult {
        // Start with greedy solution
        let mut best = self.optimize_greedy(circuit, hardware);
        let mut best_score = self.calculate_placement_score(&best.circuit, hardware);

        // Local search: try swapping pairs
        for _ in 0..self.max_iterations {
            let mut improved = false;

            for i in 0..best.mapping.len() {
                for j in (i + 1)..best.mapping.len() {
                    // Try swapping mapping[i] and mapping[j]
                    let mut new_mapping = best.mapping.clone();
                    new_mapping.swap(i, j);

                    let new_circuit = self.apply_mapping(circuit, &new_mapping);
                    let new_score = self.calculate_placement_score(&new_circuit, hardware);

                    if new_score > best_score {
                        best.mapping = new_mapping;
                        best.circuit = new_circuit;
                        best_score = new_score;
                        improved = true;
                    }
                }
            }

            if !improved {
                break; // No improvement found, stop early
            }
        }

        best.improvement = best_score;
        best
    }

    /// Applies a logical-to-physical mapping to a circuit.
    pub fn apply_mapping(&self, circuit: &CircuitGenome, mapping: &[usize]) -> CircuitGenome {
        let max_physical = *mapping.iter().max().unwrap_or(&0) + 1;
        let mut new_circuit = CircuitGenome::new(max_physical.max(circuit.num_qubits));

        for gate in &circuit.gates {
            let mapped_gate = gate.map_qubits(mapping);
            let _ = new_circuit.add_gate(mapped_gate);
        }

        new_circuit
    }

    /// Calculates a placement score (higher is better).
    ///
    /// Uses multiplicative fidelity model: score = product of edge fidelities.
    /// This matches the evaluation function used externally.
    fn calculate_placement_score(
        &self,
        circuit: &CircuitGenome,
        hardware: &HardwareProfile,
    ) -> f64 {
        let mut score = 1.0;

        for gate in &circuit.gates {
            if let Some((q1, q2)) = get_two_qubit_pair(gate) {
                if let Some(coupler) = hardware.get_coupler(q1, q2) {
                    // Multiplicative: fidelity compounds
                    score *= coupler.gate_fidelity.value();
                } else {
                    // Non-connected qubits: severe penalty (0.5 per gate)
                    score *= 0.5;
                }
            }
        }

        score
    }

    /// Main optimization entry point.
    pub fn optimize(&self, circuit: &CircuitGenome, hardware: &HardwareProfile) -> PlacementResult {
        if circuit.num_qubits > hardware.num_qubits {
            // Can't fit circuit on hardware, return identity
            return PlacementResult {
                mapping: (0..circuit.num_qubits).collect(),
                circuit: circuit.clone(),
                improvement: 0.0,
            };
        }

        if self.greedy {
            self.optimize_greedy(circuit, hardware)
        } else {
            self.optimize_local_search(circuit, hardware)
        }
    }
}

/// Extracts the qubit pair from a two-qubit gate.
fn get_two_qubit_pair(gate: &Gate) -> Option<(usize, usize)> {
    match gate {
        Gate::CNOT(q1, q2) | Gate::CZ(q1, q2) | Gate::SWAP(q1, q2) => Some((*q1, *q2)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qns_core::types::Fidelity;

    fn create_hardware_with_varying_fidelity() -> HardwareProfile {
        // Linear: 0 --99%-- 1 --95%-- 2 --98%-- 3
        let mut hw = HardwareProfile::linear("test", 4);
        hw.couplers[0].gate_fidelity = Fidelity::new(0.99); // Edge 0-1: best
        hw.couplers[1].gate_fidelity = Fidelity::new(0.95); // Edge 1-2: worst
        hw.couplers[2].gate_fidelity = Fidelity::new(0.98); // Edge 2-3: good
        hw
    }

    #[test]
    fn test_analyze_interactions() {
        let mut circuit = CircuitGenome::new(3);
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();
        circuit.add_gate(Gate::CNOT(1, 2)).unwrap();

        let optimizer = PlacementOptimizer::default();
        let interactions = optimizer.analyze_interactions(&circuit);

        assert_eq!(interactions.get(&(0, 1)), Some(&2));
        assert_eq!(interactions.get(&(1, 2)), Some(&1));
    }

    #[test]
    fn test_rank_physical_edges() {
        let hw = create_hardware_with_varying_fidelity();
        let optimizer = PlacementOptimizer::default();
        let edges = optimizer.rank_physical_edges(&hw);

        // Should be sorted by fidelity descending
        assert_eq!(edges[0].0, (0, 1)); // 99%
        assert_eq!(edges[1].0, (2, 3)); // 98%
        assert_eq!(edges[2].0, (1, 2)); // 95%
    }

    #[test]
    fn test_greedy_placement_prefers_high_fidelity() {
        let hw = create_hardware_with_varying_fidelity();

        // Circuit with one CNOT between logical qubits 0 and 1
        let mut circuit = CircuitGenome::new(4);
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();

        let optimizer = PlacementOptimizer::new(100, true);
        let result = optimizer.optimize_greedy(&circuit, &hw);

        // The mapping should place logical 0,1 on physical 0,1 (99% edge)
        // because that's the highest fidelity edge
        let p0 = result.mapping[0];
        let p1 = result.mapping[1];

        // Either (0,1) or (1,0) mapping to physical 0-1 edge
        assert!(
            (p0 == 0 && p1 == 1) || (p0 == 1 && p1 == 0),
            "Should map to high-fidelity edge 0-1, got ({}, {})",
            p0,
            p1
        );
    }

    #[test]
    fn test_placement_improves_fidelity() {
        let hw = create_hardware_with_varying_fidelity();

        // Create a circuit where placement matters
        let mut circuit = CircuitGenome::new(4);
        // Lots of CNOTs between logical 0 and 1
        for _ in 0..5 {
            circuit.add_gate(Gate::CNOT(0, 1)).unwrap();
        }
        // One CNOT between logical 2 and 3
        circuit.add_gate(Gate::CNOT(2, 3)).unwrap();

        let optimizer = PlacementOptimizer::new(100, false);

        // Calculate score with identity mapping
        let identity_circuit = optimizer.apply_mapping(&circuit, &[0, 1, 2, 3]);
        let identity_score = optimizer.calculate_placement_score(&identity_circuit, &hw);

        // Optimize placement
        let result = optimizer.optimize(&circuit, &hw);
        let optimized_score = optimizer.calculate_placement_score(&result.circuit, &hw);

        // Optimized should be >= identity (greedy should at least match or improve)
        assert!(
            optimized_score >= identity_score,
            "Optimized score {} should be >= identity score {}",
            optimized_score,
            identity_score
        );
    }

    #[test]
    fn test_apply_mapping() {
        let mut circuit = CircuitGenome::new(3);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();
        circuit.add_gate(Gate::X(2)).unwrap();

        let optimizer = PlacementOptimizer::default();
        let mapping = vec![2, 0, 1]; // logical 0→physical 2, logical 1→physical 0, etc.

        let remapped = optimizer.apply_mapping(&circuit, &mapping);

        // Check gates are remapped correctly
        assert_eq!(remapped.gates[0], Gate::H(2)); // H(0) → H(2)
        assert_eq!(remapped.gates[1], Gate::CNOT(2, 0)); // CNOT(0,1) → CNOT(2,0)
        assert_eq!(remapped.gates[2], Gate::X(1)); // X(2) → X(1)
    }

    #[test]
    fn test_local_search_finds_better_mapping() {
        let hw = create_hardware_with_varying_fidelity();

        // Create circuit with specific interaction pattern
        let mut circuit = CircuitGenome::new(4);
        // Many CNOTs between logical 1 and 2 (if identity, uses 95% edge)
        for _ in 0..10 {
            circuit.add_gate(Gate::CNOT(1, 2)).unwrap();
        }

        let optimizer = PlacementOptimizer::new(100, false);
        let result = optimizer.optimize_local_search(&circuit, &hw);

        // Local search should find a mapping that puts 1,2 on a better edge
        // Best edge is 0-1 (99%), so logical 1,2 should map to physical 0,1
        let p1 = result.mapping[1];
        let p2 = result.mapping[2];

        // The logical pair (1,2) should be on the best available edge
        if let Some(coupler) = hw.get_coupler(p1, p2) {
            assert!(
                coupler.gate_fidelity.value() >= 0.98,
                "Should use high-fidelity edge, got {}% for physical ({}, {})",
                coupler.gate_fidelity.value() * 100.0,
                p1,
                p2
            );
        }
    }
}
