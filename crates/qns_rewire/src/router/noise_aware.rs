//! Noise-Aware Router - Routes circuits through higher-fidelity edges
//!
//! Unlike BasicRouter which minimizes distance (SWAP count), this router
//! considers per-edge fidelity to minimize total error.

use super::Router;
use qns_core::types::Gate;
use qns_core::{CircuitGenome, HardwareProfile, QnsError};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

/// Router that considers per-edge fidelity when making routing decisions.
///
/// # Algorithm
/// Uses a cost function that combines:
/// 1. Distance cost (number of SWAPs needed)
/// 2. Edge fidelity cost (prefer higher-fidelity edges)
///
/// Cost = α * distance + β * (1 - edge_fidelity)
/// where α balances SWAP overhead vs edge quality
pub struct NoiseAwareRouter {
    /// Weight for distance component (SWAP count)
    pub distance_weight: f64,
    /// Weight for edge fidelity component
    pub fidelity_weight: f64,
    /// Lookahead window for cost calculation
    pub lookahead: usize,
}

impl Default for NoiseAwareRouter {
    fn default() -> Self {
        Self {
            distance_weight: 1.0,
            fidelity_weight: 0.5,
            lookahead: 5,
        }
    }
}

/// State for Dijkstra-like path finding with fidelity awareness
#[derive(Clone)]
struct PathState {
    node: usize,
    cost: f64,
    path: Vec<usize>,
}

impl PartialEq for PathState {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Eq for PathState {}

impl PartialOrd for PathState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathState {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap behavior
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

impl NoiseAwareRouter {
    /// Creates a new NoiseAwareRouter with custom weights.
    pub fn new(distance_weight: f64, fidelity_weight: f64) -> Self {
        Self {
            distance_weight,
            fidelity_weight,
            lookahead: 5,
        }
    }

    /// Finds the minimum-cost path considering edge fidelities.
    ///
    /// Uses Dijkstra's algorithm where edge cost = (1 - fidelity) * fidelity_weight
    fn find_fidelity_aware_path(
        &self,
        start: usize,
        end: usize,
        hardware: &HardwareProfile,
    ) -> Option<Vec<usize>> {
        if start == end {
            return Some(vec![start]);
        }

        let mut heap = BinaryHeap::new();
        let mut best_cost: HashMap<usize, f64> = HashMap::new();

        heap.push(PathState {
            node: start,
            cost: 0.0,
            path: vec![start],
        });
        best_cost.insert(start, 0.0);

        while let Some(PathState { node, cost, path }) = heap.pop() {
            if node == end {
                return Some(path);
            }

            // Skip if we've found a better path to this node
            if let Some(&best) = best_cost.get(&node) {
                if cost > best {
                    continue;
                }
            }

            // Explore neighbors
            for neighbor in 0..hardware.num_qubits {
                if hardware.are_connected(node, neighbor) {
                    let edge_cost = if let Some(coupler) = hardware.get_coupler(node, neighbor) {
                        // Cost based on error rate (1 - fidelity)
                        self.distance_weight
                            + self.fidelity_weight * coupler.gate_fidelity.error_rate()
                    } else {
                        self.distance_weight + self.fidelity_weight * 0.01 // Default 1% error
                    };

                    let new_cost = cost + edge_cost;

                    if !best_cost.contains_key(&neighbor) || new_cost < best_cost[&neighbor] {
                        best_cost.insert(neighbor, new_cost);
                        let mut new_path = path.clone();
                        new_path.push(neighbor);
                        heap.push(PathState {
                            node: neighbor,
                            cost: new_cost,
                            path: new_path,
                        });
                    }
                }
            }
        }

        None // No path found
    }

    /// Calculates the routing cost considering both distance and fidelity.
    fn calculate_routing_cost(
        &self,
        mapping: &[usize],
        future_gates: &[Gate],
        hardware: &HardwareProfile,
    ) -> f64 {
        let mut cost = 0.0;
        let limit = future_gates.len().min(self.lookahead);

        for gate in future_gates.iter().take(limit) {
            match gate {
                Gate::CNOT(c, t) | Gate::CZ(c, t) | Gate::SWAP(c, t) => {
                    let phys_c = mapping[*c];
                    let phys_t = mapping[*t];

                    if hardware.are_connected(phys_c, phys_t) {
                        // Direct connection: only edge fidelity cost
                        if let Some(coupler) = hardware.get_coupler(phys_c, phys_t) {
                            cost += self.fidelity_weight * coupler.gate_fidelity.error_rate();
                        }
                    } else {
                        // Need routing: estimate path cost
                        if let Some(path) = self.find_fidelity_aware_path(phys_c, phys_t, hardware)
                        {
                            // Each edge in path contributes to cost
                            let swaps_needed = path.len().saturating_sub(2);
                            cost += self.distance_weight * swaps_needed as f64;

                            // Add fidelity cost for final edge
                            if let Some(coupler) =
                                hardware.get_coupler(path[path.len() - 2], path[path.len() - 1])
                            {
                                cost += self.fidelity_weight * coupler.gate_fidelity.error_rate();
                            }
                        } else {
                            cost += 100.0; // Penalty for unreachable
                        }
                    }
                },
                _ => {}, // Single qubit gates have no routing cost
            }
        }

        cost
    }

    /// Gets the best SWAP to apply for a given two-qubit gate.
    fn find_best_swap(
        &self,
        phys_c: usize,
        phys_t: usize,
        logical_to_physical: &[usize],
        physical_to_logical: &[usize],
        future_gates: &[Gate],
        hardware: &HardwareProfile,
    ) -> Option<(usize, usize, f64)> {
        let mut best_swap = None;
        let mut min_cost = f64::INFINITY;

        // Try SWAPs on neighbors of control qubit
        for n in 0..hardware.num_qubits {
            if hardware.are_connected(phys_c, n) {
                let swap_fidelity_cost = hardware
                    .get_coupler(phys_c, n)
                    .map(|c| c.gate_fidelity.error_rate())
                    .unwrap_or(0.01);

                // Simulate the SWAP
                let mut test_mapping = logical_to_physical.to_vec();
                let log_c = physical_to_logical[phys_c];
                let log_n = physical_to_logical[n];
                test_mapping[log_c] = n;
                test_mapping[log_n] = phys_c;

                let future_cost =
                    self.calculate_routing_cost(&test_mapping, future_gates, hardware);
                let total_cost = swap_fidelity_cost * self.fidelity_weight + future_cost;

                if total_cost < min_cost {
                    min_cost = total_cost;
                    best_swap = Some((phys_c, n, total_cost));
                }
            }
        }

        // Try SWAPs on neighbors of target qubit
        for n in 0..hardware.num_qubits {
            if hardware.are_connected(phys_t, n) {
                let swap_fidelity_cost = hardware
                    .get_coupler(phys_t, n)
                    .map(|c| c.gate_fidelity.error_rate())
                    .unwrap_or(0.01);

                let mut test_mapping = logical_to_physical.to_vec();
                let log_t = physical_to_logical[phys_t];
                let log_n = physical_to_logical[n];
                test_mapping[log_t] = n;
                test_mapping[log_n] = phys_t;

                let future_cost =
                    self.calculate_routing_cost(&test_mapping, future_gates, hardware);
                let total_cost = swap_fidelity_cost * self.fidelity_weight + future_cost;

                if total_cost < min_cost {
                    min_cost = total_cost;
                    best_swap = Some((phys_t, n, total_cost));
                }
            }
        }

        best_swap
    }
}

impl NoiseAwareRouter {
    /// Routes the circuit with a custom initial mapping.
    ///
    /// This enables co-optimization with PlacementOptimizer:
    /// 1. PlacementOptimizer finds optimal logical-to-physical mapping
    /// 2. NoiseAwareRouter routes with this mapping as starting point
    ///
    /// # Arguments
    /// * `circuit` - The circuit to route
    /// * `hardware` - Hardware topology with edge fidelities
    /// * `initial_mapping` - Logical-to-physical qubit mapping to start from
    ///
    /// # Returns
    /// Routed circuit with all gates on valid edges
    pub fn route_with_mapping(
        &self,
        circuit: &CircuitGenome,
        hardware: &HardwareProfile,
        initial_mapping: &[usize],
    ) -> Result<CircuitGenome, QnsError> {
        let mut new_circuit = CircuitGenome::new(hardware.num_qubits);

        // Use provided initial mapping
        let mut logical_to_physical: Vec<usize> = initial_mapping.to_vec();

        // Build reverse mapping
        let mut physical_to_logical: Vec<usize> = vec![usize::MAX; hardware.num_qubits];
        for (logical, &physical) in logical_to_physical.iter().enumerate() {
            if physical < hardware.num_qubits {
                physical_to_logical[physical] = logical;
            }
        }

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
                        // Direct connection available
                        new_circuit.add_gate(gate.map_qubits(&logical_to_physical))?;
                        gate_idx += 1;
                    } else {
                        // Need to route
                        if let Some((u, v, _cost)) = self.find_best_swap(
                            phys_c,
                            phys_t,
                            &logical_to_physical,
                            &physical_to_logical,
                            &circuit.gates[gate_idx..],
                            hardware,
                        ) {
                            // Apply SWAP
                            new_circuit.add_gate(Gate::SWAP(u, v))?;

                            // Update mappings
                            let log_u = physical_to_logical[u];
                            let log_v = physical_to_logical[v];

                            logical_to_physical[log_u] = v;
                            logical_to_physical[log_v] = u;
                            physical_to_logical[u] = log_v;
                            physical_to_logical[v] = log_u;
                        } else {
                            return Err(QnsError::Rewire(format!(
                                "No beneficial SWAP found for qubits {} and {}",
                                phys_c, phys_t
                            )));
                        }
                    }
                },
                _ => {
                    // Single-qubit gate
                    new_circuit.add_gate(gate.map_qubits(&logical_to_physical))?;
                    gate_idx += 1;
                },
            }
        }

        Ok(new_circuit)
    }
}

impl Router for NoiseAwareRouter {
    fn route(
        &self,
        circuit: &CircuitGenome,
        hardware: &HardwareProfile,
    ) -> Result<CircuitGenome, QnsError> {
        // Default: use identity mapping
        let identity_mapping: Vec<usize> = (0..circuit.num_qubits).collect();
        self.route_with_mapping(circuit, hardware, &identity_mapping)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qns_core::types::Fidelity;

    fn create_hardware_with_varying_fidelity() -> HardwareProfile {
        // Create a linear chain with varying edge fidelities
        // 0 -- 1 -- 2 -- 3
        //  99%   95%   99%
        let mut hw = HardwareProfile::linear("test", 4);

        // Set different fidelities for each edge
        hw.couplers[0].gate_fidelity = Fidelity::new(0.99); // Edge 0-1: high fidelity
        hw.couplers[1].gate_fidelity = Fidelity::new(0.95); // Edge 1-2: low fidelity
        hw.couplers[2].gate_fidelity = Fidelity::new(0.99); // Edge 2-3: high fidelity

        hw
    }

    #[test]
    fn test_noise_aware_router_basic() {
        let hw = HardwareProfile::linear("test", 3);
        let router = NoiseAwareRouter::default();

        let mut circuit = CircuitGenome::new(3);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap(); // Connected

        let routed = router.route(&circuit, &hw).unwrap();
        assert!(hw.is_circuit_valid(&routed));
    }

    #[test]
    fn test_noise_aware_router_requires_swap() {
        let hw = HardwareProfile::linear("test", 3);
        let router = NoiseAwareRouter::default();

        // CNOT(0,2) requires routing in linear topology
        let mut circuit = CircuitGenome::new(3);
        circuit.add_gate(Gate::CNOT(0, 2)).unwrap();

        let routed = router.route(&circuit, &hw).unwrap();
        assert!(hw.is_circuit_valid(&routed));
        assert!(routed.gates.len() >= 2, "Should have at least one SWAP");
    }

    #[test]
    fn test_fidelity_aware_path_finding() {
        let hw = create_hardware_with_varying_fidelity();
        let router = NoiseAwareRouter::new(0.5, 1.0); // High fidelity weight

        // Path from 0 to 3 should prefer high-fidelity edges
        let path = router.find_fidelity_aware_path(0, 3, &hw);
        assert!(path.is_some());

        let path = path.unwrap();
        assert_eq!(path.first(), Some(&0));
        assert_eq!(path.last(), Some(&3));
    }

    #[test]
    fn test_routing_prefers_high_fidelity_edges() {
        let hw = create_hardware_with_varying_fidelity();

        // With high fidelity weight, router should consider edge quality
        let router_fidelity_aware = NoiseAwareRouter::new(1.0, 2.0);

        let mut circuit = CircuitGenome::new(4);
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap(); // Uses high-fidelity edge

        let routed = router_fidelity_aware.route(&circuit, &hw).unwrap();
        assert!(hw.is_circuit_valid(&routed));
    }

    #[test]
    fn test_cost_calculation() {
        let hw = create_hardware_with_varying_fidelity();
        let router = NoiseAwareRouter::new(1.0, 1.0);

        let mapping: Vec<usize> = (0..4).collect();
        let gates = vec![Gate::CNOT(0, 1)];

        let cost = router.calculate_routing_cost(&mapping, &gates, &hw);

        // Cost should include edge fidelity (0.01 error for 99% fidelity)
        assert!(cost > 0.0);
        assert!(cost < 1.0);
    }
}
