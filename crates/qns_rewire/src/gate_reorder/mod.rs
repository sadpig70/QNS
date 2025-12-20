//! Gate reordering module.
//!
//! This module provides algorithms for finding commuting gates and
//! generating reordered circuit variants for noise-aware optimization.
//!
//! ## Algorithm Overview
//!
//! The gate reordering process works as follows:
//!
//! 1. **Find Adjacent Commuting Pairs**: Identify pairs of adjacent gates
//!    that can be swapped without changing the circuit's quantum behavior.
//!
//! 2. **Generate Variants**: Use BFS to explore the space of possible
//!    reorderings, limited by max_depth and max_variants.
//!
//! 3. **Score Variants**: Each variant can be scored based on noise
//!    characteristics to find the optimal gate ordering.
//!
//! ## Physical Background
//!
//! Two gates A and B commute if AB = BA. When gates commute, their
//! relative order doesn't affect the final quantum state, but it
//! CAN affect the fidelity when hardware noise is considered.
//!
//! For example, if T1 is lower at time t1 than at time t2, we might
//! want to schedule noise-sensitive gates at t2.

use qns_core::prelude::*;
use std::collections::{HashSet, VecDeque};

/// Configuration for gate reordering.
#[derive(Debug, Clone)]
pub struct ReorderConfig {
    /// Maximum number of variants to generate
    pub max_variants: usize,
    /// Maximum BFS depth (number of consecutive swaps)
    pub max_depth: usize,
    /// Whether to remove duplicate circuits
    pub deduplicate: bool,
}

impl Default for ReorderConfig {
    fn default() -> Self {
        Self {
            max_variants: 100,
            max_depth: 5,
            deduplicate: true,
        }
    }
}

/// Configuration for Beam Search reordering.
///
/// Beam Search is more scalable than BFS for large circuits,
/// trading completeness for efficiency.
#[derive(Debug, Clone)]
pub struct BeamSearchConfig {
    /// Number of best candidates to keep at each iteration
    pub beam_width: usize,
    /// Maximum number of iterations
    pub max_iterations: usize,
    /// Whether to remove duplicate circuits
    pub deduplicate: bool,
    /// Early termination if no improvement for N iterations
    pub patience: usize,
}

impl Default for BeamSearchConfig {
    fn default() -> Self {
        Self {
            beam_width: 10,
            max_iterations: 50,
            deduplicate: true,
            patience: 5,
        }
    }
}

impl BeamSearchConfig {
    /// Creates a config optimized for speed.
    pub fn fast() -> Self {
        Self {
            beam_width: 5,
            max_iterations: 20,
            deduplicate: true,
            patience: 3,
        }
    }

    /// Creates a config optimized for thorough exploration.
    pub fn thorough() -> Self {
        Self {
            beam_width: 20,
            max_iterations: 100,
            deduplicate: true,
            patience: 10,
        }
    }
}

/// Information about a commuting pair of adjacent gates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommutingPair {
    /// Index of the first gate
    pub idx: usize,
    /// Both gates commute and can be swapped
    pub can_swap: bool,
}

/// Result of reordering analysis.
#[derive(Debug, Clone)]
pub struct ReorderAnalysis {
    /// Number of adjacent commuting pairs found
    pub num_commuting_pairs: usize,
    /// Indices of swappable positions
    pub swappable_positions: Vec<usize>,
    /// Estimated number of unique variants (upper bound)
    pub estimated_variants: usize,
}

/// Gate reordering operator.
///
/// Finds commuting gates and generates reordered circuit variants.
///
/// # Example
///
/// ```rust
/// use qns_rewire::GateReorder;
/// use qns_core::prelude::*;
///
/// let mut circuit = CircuitGenome::new(3);
/// circuit.add_gate(Gate::H(0)).unwrap();
/// circuit.add_gate(Gate::X(1)).unwrap();  // Commutes with H(0)
/// circuit.add_gate(Gate::CNOT(0, 1)).unwrap();
///
/// let reorder = GateReorder::default();
/// let variants = reorder.generate_reorderings(&circuit);
///
/// // Should generate at least 2 variants (original + swapped)
/// assert!(variants.len() >= 1);
/// ```
pub struct GateReorder {
    config: ReorderConfig,
}

impl GateReorder {
    /// Creates a new GateReorder with the specified maximum variants.
    pub fn new(max_variants: usize) -> Self {
        Self {
            config: ReorderConfig {
                max_variants,
                ..Default::default()
            },
        }
    }

    /// Creates a GateReorder with custom configuration.
    pub fn with_config(config: ReorderConfig) -> Self {
        Self { config }
    }

    /// Returns the current configuration.
    pub fn config(&self) -> &ReorderConfig {
        &self.config
    }

    /// Analyzes the circuit for reordering opportunities.
    pub fn analyze(&self, circuit: &CircuitGenome) -> ReorderAnalysis {
        let swappable = self.find_adjacent_commuting_pairs(circuit);
        let num_pairs = swappable.len();

        // Estimate upper bound of variants: 2^n where n = number of swap positions
        // But capped by max_variants and actual BFS exploration
        let estimated = (1usize << num_pairs.min(10)).min(self.config.max_variants);

        ReorderAnalysis {
            num_commuting_pairs: num_pairs,
            swappable_positions: swappable,
            estimated_variants: estimated,
        }
    }

    /// Finds pairs of ADJACENT gates that can be swapped.
    ///
    /// Only adjacent gates (at positions i and i+1) are considered,
    /// as non-adjacent swaps would require multiple operations.
    ///
    /// Returns a list of indices where gates[idx] and gates[idx+1] commute.
    pub fn find_adjacent_commuting_pairs(&self, circuit: &CircuitGenome) -> Vec<usize> {
        let gates = &circuit.gates;
        let mut swappable = Vec::new();

        for i in 0..gates.len().saturating_sub(1) {
            if gates[i].commutes_with(&gates[i + 1]) {
                swappable.push(i);
            }
        }

        swappable
    }

    /// Finds ALL pairs of gates that commute (not just adjacent).
    ///
    /// This is useful for analysis but not directly for reordering,
    /// as non-adjacent swaps require intermediate steps.
    pub fn find_commuting_pairs(&self, circuit: &CircuitGenome) -> Vec<(usize, usize)> {
        let mut pairs = Vec::new();
        let gates = &circuit.gates;

        for i in 0..gates.len() {
            for j in (i + 1)..gates.len() {
                if gates[i].commutes_with(&gates[j]) {
                    pairs.push((i, j));
                }
            }
        }

        pairs
    }

    /// Generates reordered circuit variants using BFS.
    ///
    /// The algorithm explores possible gate orderings by swapping
    /// adjacent commuting gates, up to max_depth swaps.
    ///
    /// # Performance
    ///
    /// Time complexity: O(max_variants * gates.len())
    /// Space complexity: O(max_variants * gates.len())
    pub fn generate_reorderings(&self, circuit: &CircuitGenome) -> Vec<CircuitGenome> {
        if circuit.gates.is_empty() {
            return vec![circuit.clone()];
        }

        let mut variants: Vec<CircuitGenome> = Vec::new();
        let mut visited: HashSet<u64> = HashSet::new();
        let mut queue: VecDeque<(CircuitGenome, usize)> = VecDeque::new();

        // Start with original circuit
        let original_hash = self.circuit_hash(circuit);
        visited.insert(original_hash);
        variants.push(circuit.clone());
        queue.push_back((circuit.clone(), 0));

        // BFS exploration
        while let Some((current, depth)) = queue.pop_front() {
            if depth >= self.config.max_depth {
                continue;
            }

            if variants.len() >= self.config.max_variants {
                break;
            }

            // Find swappable positions in current circuit
            let swappable = self.find_adjacent_commuting_pairs(&current);

            for &swap_idx in &swappable {
                // Create variant by swapping gates at swap_idx and swap_idx+1
                let variant = self.swap_gates(&current, swap_idx);
                let variant_hash = self.circuit_hash(&variant);

                // Check if we've seen this variant before
                if self.config.deduplicate && visited.contains(&variant_hash) {
                    continue;
                }

                visited.insert(variant_hash);
                variants.push(variant.clone());

                if variants.len() >= self.config.max_variants {
                    break;
                }

                // Add to queue for further exploration
                queue.push_back((variant, depth + 1));
            }
        }

        variants
    }

    /// Generates variants with scoring based on a fitness function.
    ///
    /// Returns variants sorted by score (highest first).
    pub fn generate_scored_reorderings<F>(
        &self,
        circuit: &CircuitGenome,
        score_fn: F,
    ) -> Vec<(CircuitGenome, f64)>
    where
        F: Fn(&CircuitGenome) -> f64,
    {
        let variants = self.generate_reorderings(circuit);
        let mut scored: Vec<(CircuitGenome, f64)> = variants
            .into_iter()
            .map(|c| {
                let score = score_fn(&c);
                (c, score)
            })
            .collect();

        // Sort by score descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored
    }

    /// Swaps gates at positions idx and idx+1.
    fn swap_gates(&self, circuit: &CircuitGenome, idx: usize) -> CircuitGenome {
        let mut new_circuit = circuit.clone();

        if idx + 1 < new_circuit.gates.len() {
            new_circuit.gates.swap(idx, idx + 1);
        }

        new_circuit
    }

    /// Generates reordered circuit variants using Beam Search.
    ///
    /// Beam Search keeps only the top-k candidates at each iteration,
    /// making it more scalable than BFS for large circuits.
    ///
    /// # Algorithm
    ///
    /// 1. Start with original circuit in beam
    /// 2. For each iteration:
    ///    a. For each circuit in beam, generate all swap variants
    ///    b. Score all candidates using the provided scoring function
    ///    c. Keep top beam_width candidates
    /// 3. Return best circuit found
    ///
    /// # Arguments
    /// * `circuit` - The circuit to optimize
    /// * `config` - Beam search configuration
    /// * `score_fn` - Scoring function (higher = better)
    ///
    /// # Performance
    ///
    /// Time complexity: O(iterations * beam_width * gates * score_cost)
    /// Space complexity: O(beam_width * gates)
    ///
    /// # Example
    ///
    /// ```rust
    /// use qns_rewire::{GateReorder, BeamSearchConfig, score_circuit_variant};
    /// use qns_core::prelude::*;
    ///
    /// let mut circuit = CircuitGenome::new(3);
    /// circuit.add_gate(Gate::H(0)).unwrap();
    /// circuit.add_gate(Gate::X(1)).unwrap();
    /// circuit.add_gate(Gate::CNOT(0, 1)).unwrap();
    ///
    /// let reorder = GateReorder::default();
    /// let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
    ///
    /// let (best, score) = reorder.beam_search_reorderings(
    ///     &circuit,
    ///     BeamSearchConfig::default(),
    ///     |c| score_circuit_variant(c, &noise),
    /// );
    /// ```
    pub fn beam_search_reorderings<F>(
        &self,
        circuit: &CircuitGenome,
        config: BeamSearchConfig,
        score_fn: F,
    ) -> (CircuitGenome, f64)
    where
        F: Fn(&CircuitGenome) -> f64,
    {
        if circuit.gates.is_empty() {
            return (circuit.clone(), score_fn(circuit));
        }

        let mut visited: HashSet<u64> = HashSet::new();

        // Initialize beam with original circuit
        let original_score = score_fn(circuit);
        let original_hash = self.circuit_hash(circuit);
        visited.insert(original_hash);

        let mut beam: Vec<(CircuitGenome, f64)> = vec![(circuit.clone(), original_score)];
        let mut best_circuit = circuit.clone();
        let mut best_score = original_score;
        let mut no_improvement_count = 0;

        for _iteration in 0..config.max_iterations {
            let mut candidates: Vec<(CircuitGenome, f64)> = Vec::new();

            // Expand each circuit in the beam
            for (current, _current_score) in &beam {
                // Find swappable positions
                let swappable = self.find_adjacent_commuting_pairs(current);

                // Generate all swap variants
                for &swap_idx in &swappable {
                    let variant = self.swap_gates(current, swap_idx);
                    let variant_hash = self.circuit_hash(&variant);

                    // Skip duplicates if configured
                    if config.deduplicate && visited.contains(&variant_hash) {
                        continue;
                    }

                    visited.insert(variant_hash);
                    let variant_score = score_fn(&variant);
                    candidates.push((variant, variant_score));
                }
            }

            // If no new candidates, we've exhausted the search space
            if candidates.is_empty() {
                break;
            }

            // Sort by score descending
            candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            // Keep top beam_width candidates
            candidates.truncate(config.beam_width);

            // Update best if improved
            if let Some((top_circuit, top_score)) = candidates.first() {
                if *top_score > best_score {
                    best_circuit = top_circuit.clone();
                    best_score = *top_score;
                    no_improvement_count = 0;
                } else {
                    no_improvement_count += 1;
                }
            }

            // Early termination if no improvement
            if no_improvement_count >= config.patience {
                break;
            }

            // Update beam for next iteration
            beam = candidates;
        }

        (best_circuit, best_score)
    }

    /// Generates multiple good variants using Beam Search.
    ///
    /// Unlike `beam_search_reorderings` which returns only the best,
    /// this returns multiple top candidates.
    pub fn beam_search_top_k<F>(
        &self,
        circuit: &CircuitGenome,
        config: BeamSearchConfig,
        score_fn: F,
        k: usize,
    ) -> Vec<(CircuitGenome, f64)>
    where
        F: Fn(&CircuitGenome) -> f64,
    {
        if circuit.gates.is_empty() {
            return vec![(circuit.clone(), score_fn(circuit))];
        }

        let mut visited: HashSet<u64> = HashSet::new();
        let mut all_variants: Vec<(CircuitGenome, f64)> = Vec::new();

        // Initialize
        let original_score = score_fn(circuit);
        let original_hash = self.circuit_hash(circuit);
        visited.insert(original_hash);
        all_variants.push((circuit.clone(), original_score));

        let mut beam: Vec<(CircuitGenome, f64)> = vec![(circuit.clone(), original_score)];

        for _iteration in 0..config.max_iterations {
            let mut candidates: Vec<(CircuitGenome, f64)> = Vec::new();

            for (current, _) in &beam {
                let swappable = self.find_adjacent_commuting_pairs(current);

                for &swap_idx in &swappable {
                    let variant = self.swap_gates(current, swap_idx);
                    let variant_hash = self.circuit_hash(&variant);

                    if config.deduplicate && visited.contains(&variant_hash) {
                        continue;
                    }

                    visited.insert(variant_hash);
                    let variant_score = score_fn(&variant);
                    candidates.push((variant.clone(), variant_score));
                    all_variants.push((variant, variant_score));
                }
            }

            if candidates.is_empty() {
                break;
            }

            candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            candidates.truncate(config.beam_width);
            beam = candidates;
        }

        // Sort all variants and return top k
        all_variants.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        all_variants.truncate(k);
        all_variants
    }

    /// Selects the best algorithm based on circuit size.
    ///
    /// - Small circuits (<50 gates): BFS (complete exploration)
    /// - Large circuits (≥50 gates): Beam Search (scalable)
    pub fn auto_reorder<F>(&self, circuit: &CircuitGenome, score_fn: F) -> (CircuitGenome, f64)
    where
        F: Fn(&CircuitGenome) -> f64,
    {
        const BFS_THRESHOLD: usize = 50;

        if circuit.gates.len() < BFS_THRESHOLD {
            // Use BFS for small circuits
            let variants = self.generate_scored_reorderings(circuit, &score_fn);
            variants
                .into_iter()
                .next()
                .unwrap_or_else(|| (circuit.clone(), score_fn(circuit)))
        } else {
            // Use Beam Search for large circuits
            self.beam_search_reorderings(circuit, BeamSearchConfig::default(), score_fn)
        }
    }

    /// Computes a hash for the circuit's gate sequence.
    ///
    /// Used for deduplication of equivalent circuits.
    fn circuit_hash(&self, circuit: &CircuitGenome) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Hash the gate sequence
        for gate in &circuit.gates {
            // Hash gate type and qubits
            std::mem::discriminant(gate).hash(&mut hasher);
            gate.qubits().hash(&mut hasher);

            // Hash rotation angle if present (discretized)
            if let Some(angle) = gate.rotation_angle() {
                let discretized = (angle * 1000.0).round() as i64;
                discretized.hash(&mut hasher);
            }
        }

        hasher.finish()
    }

    /// Checks if two circuits are equivalent (same gate sequence).
    pub fn circuits_equivalent(&self, c1: &CircuitGenome, c2: &CircuitGenome) -> bool {
        self.circuit_hash(c1) == self.circuit_hash(c2)
    }
}

impl Default for GateReorder {
    fn default() -> Self {
        Self::new(100)
    }
}

/// Estimates the expected error rate for a circuit given noise parameters.
///
/// This is a simplified model that considers:
/// - Single-qubit gate errors
/// - Two-qubit gate errors  
/// - Decoherence during gate execution
///
/// # Arguments
/// * `circuit` - The circuit to evaluate
/// * `noise` - Noise characteristics
/// * `gate_time_ns` - Approximate gate time in nanoseconds
///
/// # Returns
/// Estimated error probability (0.0 = perfect, 1.0 = completely random)
pub fn estimate_circuit_error(
    circuit: &CircuitGenome,
    noise: &NoiseVector,
    gate_time_ns: f64,
) -> f64 {
    let mut total_error = 0.0;
    let mut current_time_ns = 0.0;

    for gate in &circuit.gates {
        // Gate-specific error
        let gate_error = gate.estimated_error();

        // Decoherence error based on current time
        // Error increases as we approach T1/T2
        let t1_us = noise.t1_mean;
        let _t2_us = noise.t2_mean; // Reserved for future T2-based dephasing model

        // Convert current time to microseconds
        let t_us = current_time_ns / 1000.0;

        // Decoherence contribution (exponential decay model)
        // P(error) ≈ 1 - exp(-t/T1) for energy relaxation
        let decoherence_error = if t1_us > 0.0 {
            1.0 - (-t_us / t1_us).exp()
        } else {
            0.0
        };

        // Combined error (assuming independent errors)
        // P(total) ≈ P(gate) + P(decoherence) for small errors
        total_error += gate_error + decoherence_error * 0.1; // Scale decoherence contribution

        // Update time
        current_time_ns += gate_time_ns;
    }

    // Clamp to valid probability range
    total_error.clamp(0.0, 1.0)
}

/// Scores a circuit variant based on noise-aware heuristics.
///
/// Higher scores indicate better expected fidelity.
pub fn score_circuit_variant(circuit: &CircuitGenome, noise: &NoiseVector) -> f64 {
    // Estimate error rate
    let error = estimate_circuit_error(circuit, noise, 35.0);

    // Convert to fidelity-like score (1.0 = perfect)
    1.0 - error
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_circuit() -> CircuitGenome {
        let mut circuit = CircuitGenome::new(3);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::X(1)).unwrap(); // Commutes with H(0) - different qubits
        circuit.add_gate(Gate::Z(0)).unwrap(); // Commutes with X(1) - different qubits
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();
        circuit
    }

    #[test]
    fn test_find_adjacent_commuting_pairs() {
        let circuit = create_test_circuit();
        let reorder = GateReorder::default();

        let pairs = reorder.find_adjacent_commuting_pairs(&circuit);

        // H(0) and X(1) commute (different qubits)
        // X(1) and Z(0) commute (different qubits)
        assert!(pairs.contains(&0), "H(0) and X(1) should commute");
        assert!(pairs.contains(&1), "X(1) and Z(0) should commute");
    }

    #[test]
    fn test_find_commuting_pairs() {
        let mut circuit = CircuitGenome::new(3);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::X(1)).unwrap();
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();

        let reorder = GateReorder::new(10);
        let pairs = reorder.find_commuting_pairs(&circuit);

        // H(0) and X(1) should commute
        assert!(pairs.contains(&(0, 1)));
    }

    #[test]
    fn test_generate_reorderings() {
        let circuit = create_test_circuit();
        let reorder = GateReorder::new(20);

        let variants = reorder.generate_reorderings(&circuit);

        // Should generate multiple variants
        assert!(
            variants.len() >= 2,
            "Expected at least 2 variants, got {}",
            variants.len()
        );

        // All variants should have same number of gates
        for variant in &variants {
            assert_eq!(variant.gates.len(), circuit.gates.len());
        }
    }

    #[test]
    fn test_generate_reorderings_respects_max() {
        let circuit = create_test_circuit();
        let reorder = GateReorder::new(3);

        let variants = reorder.generate_reorderings(&circuit);

        assert!(variants.len() <= 3);
    }

    #[test]
    fn test_swap_gates() {
        let mut circuit = CircuitGenome::new(2);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::X(1)).unwrap();

        let reorder = GateReorder::default();
        let swapped = reorder.swap_gates(&circuit, 0);

        // Gates should be swapped
        assert!(matches!(swapped.gates[0], Gate::X(1)));
        assert!(matches!(swapped.gates[1], Gate::H(0)));
    }

    #[test]
    fn test_circuit_hash_different() {
        let mut c1 = CircuitGenome::new(2);
        c1.add_gate(Gate::H(0)).unwrap();
        c1.add_gate(Gate::X(1)).unwrap();

        let mut c2 = CircuitGenome::new(2);
        c2.add_gate(Gate::X(1)).unwrap();
        c2.add_gate(Gate::H(0)).unwrap();

        let reorder = GateReorder::default();
        assert_ne!(reorder.circuit_hash(&c1), reorder.circuit_hash(&c2));
    }

    #[test]
    fn test_circuit_hash_same() {
        let mut c1 = CircuitGenome::new(2);
        c1.add_gate(Gate::H(0)).unwrap();
        c1.add_gate(Gate::CNOT(0, 1)).unwrap();

        let c2 = c1.clone();

        let reorder = GateReorder::default();
        assert_eq!(reorder.circuit_hash(&c1), reorder.circuit_hash(&c2));
    }

    #[test]
    fn test_no_commuting_pairs() {
        let mut circuit = CircuitGenome::new(2);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::X(0)).unwrap(); // Same qubit, doesn't commute
        circuit.add_gate(Gate::Measure(0)).unwrap();

        let reorder = GateReorder::default();
        let pairs = reorder.find_adjacent_commuting_pairs(&circuit);

        // H and X on same qubit don't commute
        // X and Measure don't commute
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_analyze() {
        let circuit = create_test_circuit();
        let reorder = GateReorder::default();

        let analysis = reorder.analyze(&circuit);

        assert!(analysis.num_commuting_pairs >= 2);
        assert!(analysis.estimated_variants >= 2);
    }

    #[test]
    fn test_diagonal_gate_reordering() {
        // Diagonal gates on same qubit should commute
        let mut circuit = CircuitGenome::new(1);
        circuit.add_gate(Gate::Z(0)).unwrap();
        circuit.add_gate(Gate::S(0)).unwrap();
        circuit.add_gate(Gate::T(0)).unwrap();

        let reorder = GateReorder::default();
        let pairs = reorder.find_adjacent_commuting_pairs(&circuit);

        // Z and S commute, S and T commute
        assert!(pairs.contains(&0), "Z and S should commute");
        assert!(pairs.contains(&1), "S and T should commute");
    }

    #[test]
    fn test_scored_reorderings() {
        let circuit = create_test_circuit();
        let reorder = GateReorder::new(10);

        // Simple scoring: prefer fewer two-qubit gates at the end
        let scored = reorder.generate_scored_reorderings(&circuit, |c| {
            let last_is_two_qubit = c.gates.last().map(|g| g.is_two_qubit()).unwrap_or(false);
            if last_is_two_qubit {
                0.5
            } else {
                1.0
            }
        });

        // Should have scored variants
        assert!(!scored.is_empty());

        // Scores should be valid
        for (_, score) in &scored {
            assert!(*score >= 0.0 && *score <= 1.0);
        }
    }

    #[test]
    fn test_estimate_circuit_error() {
        let mut circuit = CircuitGenome::new(2);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();

        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);

        let error = estimate_circuit_error(&circuit, &noise, 35.0);

        // Error should be in valid range
        assert!((0.0..=1.0).contains(&error));
        // Should have some error (not zero)
        assert!(error > 0.0);
    }

    #[test]
    fn test_empty_circuit() {
        let circuit = CircuitGenome::new(2);
        let reorder = GateReorder::default();

        let variants = reorder.generate_reorderings(&circuit);

        assert_eq!(variants.len(), 1);
        assert!(variants[0].gates.is_empty());
    }

    #[test]
    fn test_single_gate_circuit() {
        let mut circuit = CircuitGenome::new(1);
        circuit.add_gate(Gate::H(0)).unwrap();

        let reorder = GateReorder::default();
        let variants = reorder.generate_reorderings(&circuit);

        // Only original circuit possible
        assert_eq!(variants.len(), 1);
    }

    #[test]
    fn test_deduplication() {
        // Circuit where multiple swap paths lead to same result
        let mut circuit = CircuitGenome::new(3);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::X(1)).unwrap();
        circuit.add_gate(Gate::Y(2)).unwrap();

        let reorder = GateReorder::with_config(ReorderConfig {
            max_variants: 100,
            max_depth: 10,
            deduplicate: true,
        });

        let variants = reorder.generate_reorderings(&circuit);

        // With 3 gates all on different qubits, we have 3! = 6 permutations
        // All should be unique
        let mut hashes: HashSet<u64> = HashSet::new();
        for v in &variants {
            let h = reorder.circuit_hash(v);
            assert!(hashes.insert(h), "Duplicate circuit found!");
        }
    }

    // ==================== Beam Search Tests ====================

    #[test]
    fn test_beam_search_basic() {
        let circuit = create_test_circuit();
        let reorder = GateReorder::default();
        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);

        let (best, score) =
            reorder.beam_search_reorderings(&circuit, BeamSearchConfig::default(), |c| {
                score_circuit_variant(c, &noise)
            });

        // Should return a valid circuit
        assert_eq!(best.gates.len(), circuit.gates.len());
        // Score should be valid
        assert!((0.0..=1.0).contains(&score));
    }

    #[test]
    fn test_beam_search_improves_or_maintains() {
        let circuit = create_test_circuit();
        let reorder = GateReorder::default();
        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);

        let original_score = score_circuit_variant(&circuit, &noise);
        let (_, best_score) =
            reorder.beam_search_reorderings(&circuit, BeamSearchConfig::default(), |c| {
                score_circuit_variant(c, &noise)
            });

        // Best score should be at least as good as original
        assert!(
            best_score >= original_score - 1e-10,
            "Beam search should not worsen score: {} vs {}",
            best_score,
            original_score
        );
    }

    #[test]
    fn test_beam_search_config_fast() {
        let circuit = create_test_circuit();
        let reorder = GateReorder::default();
        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);

        let start = std::time::Instant::now();
        let (_best, _score) =
            reorder.beam_search_reorderings(&circuit, BeamSearchConfig::fast(), |c| {
                score_circuit_variant(c, &noise)
            });
        let elapsed = start.elapsed();

        // Fast config should complete quickly
        assert!(
            elapsed.as_millis() < 100,
            "Fast beam search took {}ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_beam_search_top_k() {
        let circuit = create_test_circuit();
        let reorder = GateReorder::default();
        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);

        let top_k = reorder.beam_search_top_k(
            &circuit,
            BeamSearchConfig::default(),
            |c| score_circuit_variant(c, &noise),
            5,
        );

        // Should return up to k variants
        assert!(!top_k.is_empty());
        assert!(top_k.len() <= 5);

        // Should be sorted by score descending
        for i in 1..top_k.len() {
            assert!(
                top_k[i - 1].1 >= top_k[i].1,
                "Results should be sorted by score descending"
            );
        }
    }

    #[test]
    fn test_beam_search_empty_circuit() {
        let circuit = CircuitGenome::new(2);
        let reorder = GateReorder::default();
        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);

        let (best, _score) =
            reorder.beam_search_reorderings(&circuit, BeamSearchConfig::default(), |c| {
                score_circuit_variant(c, &noise)
            });

        assert!(best.gates.is_empty());
    }

    #[test]
    fn test_beam_search_single_gate() {
        let mut circuit = CircuitGenome::new(1);
        circuit.add_gate(Gate::H(0)).unwrap();

        let reorder = GateReorder::default();
        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);

        let (best, _score) =
            reorder.beam_search_reorderings(&circuit, BeamSearchConfig::default(), |c| {
                score_circuit_variant(c, &noise)
            });

        assert_eq!(best.gates.len(), 1);
    }

    #[test]
    fn test_beam_search_patience() {
        let circuit = create_test_circuit();
        let reorder = GateReorder::default();
        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);

        // Very low patience - should terminate early
        let config = BeamSearchConfig {
            beam_width: 5,
            max_iterations: 100,
            deduplicate: true,
            patience: 1,
        };

        let start = std::time::Instant::now();
        let _ =
            reorder.beam_search_reorderings(&circuit, config, |c| score_circuit_variant(c, &noise));
        let elapsed = start.elapsed();

        // Should terminate early due to patience
        assert!(
            elapsed.as_millis() < 50,
            "Early termination failed: {}ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_auto_reorder_small_circuit() {
        let circuit = create_test_circuit(); // < 50 gates
        let reorder = GateReorder::default();
        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);

        let (best, score) = reorder.auto_reorder(&circuit, |c| score_circuit_variant(c, &noise));

        assert_eq!(best.gates.len(), circuit.gates.len());
        assert!((0.0..=1.0).contains(&score));
    }

    #[test]
    fn test_beam_search_large_circuit() {
        // Create a larger circuit to test scalability
        let mut circuit = CircuitGenome::new(10);
        for i in 0..10 {
            circuit.add_gate(Gate::H(i % 10)).unwrap();
            circuit.add_gate(Gate::X((i + 1) % 10)).unwrap();
        }

        let reorder = GateReorder::default();
        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);

        let start = std::time::Instant::now();
        let (best, score) =
            reorder.beam_search_reorderings(&circuit, BeamSearchConfig::fast(), |c| {
                score_circuit_variant(c, &noise)
            });
        let elapsed = start.elapsed();

        assert_eq!(best.gates.len(), circuit.gates.len());
        assert!(score >= 0.0);
        // Should complete in reasonable time
        assert!(
            elapsed.as_millis() < 500,
            "Large circuit took {}ms",
            elapsed.as_millis()
        );
    }
}
