#![allow(clippy::needless_range_loop)]

use super::Router;
use qns_core::types::Gate;
use qns_core::{CircuitGenome, HardwareProfile, QnsError};
use std::collections::{HashMap, VecDeque};

pub struct BasicRouter;

impl Router for BasicRouter {
    fn route(
        &self,
        circuit: &CircuitGenome,
        hardware: &HardwareProfile,
    ) -> Result<CircuitGenome, QnsError> {
        let mut new_circuit = CircuitGenome::new(hardware.num_qubits);

        // Initial mapping: logical q -> physical q (trivial 1:1)
        // map[logical] = physical
        let mut logical_to_physical: Vec<usize> = (0..circuit.num_qubits).collect();
        // physical_to_logical[physical] = logical
        let mut physical_to_logical: Vec<usize> = (0..circuit.num_qubits).collect();

        if circuit.num_qubits > hardware.num_qubits {
            return Err(QnsError::InvalidQubit(
                circuit.num_qubits,
                hardware.num_qubits,
            ));
        }

        let mut gate_idx = 0;
        while gate_idx < circuit.gates.len() {
            let gate = &circuit.gates[gate_idx];

            match gate {
                Gate::CNOT(c, t) | Gate::CZ(c, t) | Gate::SWAP(c, t) => {
                    let phys_c = logical_to_physical[*c];
                    let phys_t = logical_to_physical[*t];

                    if hardware.are_connected(phys_c, phys_t) {
                        new_circuit.add_gate(gate.map_qubits(&logical_to_physical))?;
                        gate_idx += 1;
                    } else {
                        // Lookahead Strategy:
                        // 1. Find all possible SWAPs (edges adjacent to phys_c or phys_t on shortest path? Or just neighbors?)
                        // Heuristic: Try moving c towards t, or t towards c.
                        // Candidates: Neighbors of phys_c (on path to t) AND Neighbors of phys_t (on path to c).

                        // Simple approach: Get shortest path. The first edge (phys_c, next) is a candidate.
                        // Also consider moving target?
                        // Let's consider all neighbors of phys_c and phys_t that reduce distance?
                        // Or just evaluate ALL neighbors of phys_c and phys_t.

                        let mut best_swap = None;
                        let mut min_cost = usize::MAX;

                        // Candidates: neighbors of phys_c
                        for n in 0..hardware.num_qubits {
                            if hardware.are_connected(phys_c, n) {
                                // Try SWAP(phys_c, n)
                                let mut test_mapping = logical_to_physical.clone();
                                // Update mapping for simulation
                                // Find logical qubit at n
                                let log_c = *c;
                                let log_n = physical_to_logical[n];

                                test_mapping[log_c] = n;
                                test_mapping[log_n] = phys_c;

                                let cost = self.calculate_cost(
                                    &test_mapping,
                                    &circuit.gates[gate_idx..],
                                    hardware,
                                );
                                if cost < min_cost {
                                    min_cost = cost;
                                    best_swap = Some((phys_c, n));
                                }
                            }
                        }

                        // Candidates: neighbors of phys_t
                        for n in 0..hardware.num_qubits {
                            if hardware.are_connected(phys_t, n) {
                                // Try SWAP(phys_t, n)
                                let mut test_mapping = logical_to_physical.clone();
                                let log_t = *t;
                                let log_n = physical_to_logical[n];

                                test_mapping[log_t] = n;
                                test_mapping[log_n] = phys_t;

                                let cost = self.calculate_cost(
                                    &test_mapping,
                                    &circuit.gates[gate_idx..],
                                    hardware,
                                );
                                if cost < min_cost {
                                    min_cost = cost;
                                    best_swap = Some((phys_t, n));
                                }
                            }
                        }

                        if let Some((u, v)) = best_swap {
                            // Apply Best SWAP
                            new_circuit.add_gate(Gate::SWAP(u, v))?;

                            let log_u = physical_to_logical[u];
                            let log_v = physical_to_logical[v];

                            logical_to_physical[log_u] = v;
                            logical_to_physical[log_v] = u;
                            physical_to_logical[u] = log_v;
                            physical_to_logical[v] = log_u;

                            // Don't increment gate_idx, retry current gate with new mapping
                        } else {
                            return Err(QnsError::Rewire(format!(
                                "No beneficial SWAP found for {} and {}",
                                phys_c, phys_t
                            )));
                        }
                    }
                },
                _ => {
                    new_circuit.add_gate(gate.map_qubits(&logical_to_physical))?;
                    gate_idx += 1;
                },
            }
        }

        Ok(new_circuit)
    }
}

impl BasicRouter {
    /// BFS to find shortest path distance between two qubits
    fn get_distance(&self, start: usize, end: usize, hardware: &HardwareProfile) -> usize {
        if start == end {
            return 0;
        }
        let mut visited = vec![false; hardware.num_qubits];
        let mut queue = VecDeque::new();
        queue.push_back((start, 0));
        visited[start] = true;

        while let Some((curr, dist)) = queue.pop_front() {
            if curr == end {
                return dist;
            }

            for next in 0..hardware.num_qubits {
                if !visited[next] && hardware.are_connected(curr, next) {
                    visited[next] = true;
                    queue.push_back((next, dist + 1));
                }
            }
        }
        usize::MAX // Should not happen if graph is connected
    }

    /// Calculate cost of a candidate mapping based on future gates
    fn calculate_cost(
        &self,
        mapping: &[usize], // logical -> physical
        future_gates: &[Gate],
        hardware: &HardwareProfile,
    ) -> usize {
        let mut cost = 0;
        let lookahead_window = 5; // Look at next 5 gates
        let limit = std::cmp::min(future_gates.len(), lookahead_window);

        for i in 0..limit {
            match &future_gates[i] {
                Gate::CNOT(c, t) | Gate::CZ(c, t) | Gate::SWAP(c, t) => {
                    let phys_c = mapping[*c];
                    let phys_t = mapping[*t];
                    let dist = self.get_distance(phys_c, phys_t, hardware);
                    // Weight closer gates more heavily? For now, uniform weight.
                    cost += dist;
                },
                _ => {}, // Single qubit gates have 0 routing cost
            }
        }
        cost
    }

    #[allow(dead_code)]
    fn find_path(
        &self,
        start: usize,
        end: usize,
        hardware: &HardwareProfile,
    ) -> Option<Vec<usize>> {
        // Keep existing BFS for fallback or simple path finding if needed
        let mut visited = vec![false; hardware.num_qubits];
        let mut queue = VecDeque::new();
        let mut parent = HashMap::new();

        visited[start] = true;
        queue.push_back(start);

        while let Some(curr) = queue.pop_front() {
            if curr == end {
                let mut path = vec![end];
                let mut node = end;
                while let Some(&p) = parent.get(&node) {
                    path.push(p);
                    node = p;
                }
                path.reverse();
                return Some(path);
            }

            for next in 0..hardware.num_qubits {
                if !visited[next] && hardware.are_connected(curr, next) {
                    visited[next] = true;
                    parent.insert(next, curr);
                    queue.push_back(next);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_routing_linear() {
        // Linear chain: 0-1-2
        let hw = HardwareProfile::linear("test", 3);
        let router = BasicRouter;

        // Circuit: CNOT(0, 2) - not connected
        let mut circuit = CircuitGenome::new(3);
        circuit.add_gate(Gate::CNOT(0, 2)).unwrap();

        let routed = router.route(&circuit, &hw).unwrap();

        // Check if all gates are valid on hardware
        assert!(hw.is_circuit_valid(&routed));

        // Should have at least one SWAP and one CNOT
        assert!(routed.gates.len() >= 2);
    }

    #[test]
    fn test_basic_routing_heavy_hex() {
        // Heavy-hex 2x3 topology
        let hw = HardwareProfile::heavy_hex("test_hh", 2, 3);
        let router = BasicRouter;

        // Circuit with non-adjacent qubits (depends on topology)
        // In 2x3 heavy-hex: qubits 0,1,2 in row 0, bridges at 3,4, qubits 5,6,7 in row 1
        let mut circuit = CircuitGenome::new(hw.num_qubits);

        // CNOT between qubits that may not be directly connected
        // Let's try 0 and 7 (far apart)
        if hw.num_qubits >= 8 {
            circuit.add_gate(Gate::CNOT(0, 7)).unwrap();

            let routed = router.route(&circuit, &hw).unwrap();

            // All gates should be valid on hardware
            assert!(hw.is_circuit_valid(&routed));

            // Should have SWAPs inserted
            println!("Heavy-hex routed circuit: {} gates", routed.gates.len());
            for g in &routed.gates {
                println!("  {:?}", g);
            }
        }
    }

    #[test]
    fn test_basic_routing_grid() {
        // 2x2 grid topology
        let hw = HardwareProfile::grid("test_grid", 2, 2);
        let router = BasicRouter;

        // In 2x2 grid: 0-1 (top), 2-3 (bottom), 0-2, 1-3 (vertical)
        // CNOT(0, 3) requires routing (not directly connected)
        let mut circuit = CircuitGenome::new(4);
        circuit.add_gate(Gate::CNOT(0, 3)).unwrap();

        let routed = router.route(&circuit, &hw).unwrap();

        assert!(hw.is_circuit_valid(&routed));
        assert!(routed.gates.len() >= 2); // At least 1 SWAP + 1 CNOT

        println!("Grid routed circuit: {} gates", routed.gates.len());
    }
}
