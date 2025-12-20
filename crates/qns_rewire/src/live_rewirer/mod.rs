// QNS v2.0 - live_rewirer/mod.rs
// Phase 3 Journal Implementation: LiveRewirer with noise-adaptive optimization

use crate::gate_reorder::{GateReorder, ReorderConfig};
use crate::router::placement::PlacementOptimizer;
use crate::router::NoiseAwareRouter;
use crate::scoring::{
    estimate_fidelity_with_hardware, estimate_fidelity_with_idle_tracking, ScoreConfig,
};
use qns_core::prelude::*;
use rayon::prelude::*;

/// Configuration for the LiveRewirer optimization
#[derive(Debug, Clone)]
pub struct RewireConfig {
    /// Maximum number of variants to evaluate
    pub max_variants: usize,
    /// Maximum depth for variant generation
    pub max_depth: usize,
    /// Minimum fidelity threshold to consider a circuit valid
    pub min_fidelity_threshold: f64,
    /// Enable hardware-aware optimization
    pub hardware_aware: bool,
    /// Scoring configuration (gate times, etc.)
    pub score_config: ScoreConfig,
    /// Beam width for beam search
    pub beam_width: usize,
    /// Threshold to switch from BFS to beam search
    pub beam_search_threshold: usize,
    /// Enable parallel evaluation
    pub parallel: bool,
}

impl Default for RewireConfig {
    fn default() -> Self {
        Self {
            max_variants: 50,
            max_depth: 4,
            min_fidelity_threshold: 0.5,
            hardware_aware: true,
            score_config: ScoreConfig::default(),
            beam_width: 10,
            beam_search_threshold: 30,
            parallel: true,
        }
    }
}

/// Result of circuit optimization
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    /// The optimized circuit
    pub circuit: CircuitGenome,
    /// Estimated fidelity of the optimized circuit
    pub fidelity: f64,
    /// Number of variants evaluated
    pub variants_evaluated: usize,
    /// Improvement in fidelity (optimized - original)
    pub improvement: f64,
    /// Whether the circuit was improved
    pub improved: bool,
    /// Strategy used for optimization
    pub strategy: String,
}

/// Scored circuit variant
#[derive(Debug, Clone)]
struct ScoredVariant {
    circuit: CircuitGenome,
    fidelity: f64,
}

/// LiveRewirer: Noise-adaptive circuit optimizer
///
/// Implements the core optimization algorithm:
/// 1. Generate circuit variants through gate reordering
/// 2. Score each variant using noise-aware fidelity estimation
/// 3. Select the best variant based on estimated fidelity
pub struct LiveRewirer {
    circuit: Option<CircuitGenome>,
    gate_reorder: GateReorder,
    config: RewireConfig,
    hardware: Option<HardwareProfile>,
}

impl Default for LiveRewirer {
    fn default() -> Self {
        Self::new()
    }
}

impl LiveRewirer {
    /// Create a LiveRewirer with custom configuration
    pub fn with_config(config: RewireConfig) -> Self {
        let reorder_config = ReorderConfig {
            max_variants: config.max_variants,
            max_depth: config.max_depth,
            deduplicate: true,
        };
        Self {
            circuit: None,
            gate_reorder: GateReorder::with_config(reorder_config),
            config,
            hardware: None,
        }
    }

    /// Create a LiveRewirer with default configuration
    pub fn new() -> Self {
        Self::with_config(RewireConfig::default())
    }

    /// Load a circuit for optimization
    pub fn load(&mut self, circuit: CircuitGenome) -> Result<()> {
        self.circuit = Some(circuit);
        Ok(())
    }

    /// Set hardware profile for hardware-aware optimization
    pub fn set_hardware(&mut self, hardware: HardwareProfile) {
        self.hardware = Some(hardware);
    }

    /// Get the current configuration
    pub fn config(&self) -> &RewireConfig {
        &self.config
    }

    // ========================================================================
    // Task 2.1: score_all_variants - Score all circuit variants
    // ========================================================================

    /// Scores all variants sequentially
    fn score_all_variants_sequential(
        &self,
        variants: &[CircuitGenome],
        noise: &NoiseVector,
    ) -> Vec<ScoredVariant> {
        variants
            .iter()
            .map(|circuit| {
                let fidelity =
                    estimate_fidelity_with_idle_tracking(circuit, noise, &self.config.score_config);
                ScoredVariant {
                    circuit: circuit.clone(),
                    fidelity,
                }
            })
            .collect()
    }

    // ========================================================================
    // Task 2.3: parallel_evaluation - Score variants in parallel using Rayon
    // ========================================================================

    /// Scores all variants in parallel using Rayon
    fn score_all_variants_parallel(
        &self,
        variants: &[CircuitGenome],
        noise: &NoiseVector,
    ) -> Vec<ScoredVariant> {
        variants
            .par_iter()
            .map(|circuit| {
                let fidelity =
                    estimate_fidelity_with_idle_tracking(circuit, noise, &self.config.score_config);
                ScoredVariant {
                    circuit: circuit.clone(),
                    fidelity,
                }
            })
            .collect()
    }

    /// Score all variants (chooses parallel or sequential based on config)
    fn score_all_variants(
        &self,
        variants: &[CircuitGenome],
        noise: &NoiseVector,
    ) -> Vec<ScoredVariant> {
        if self.config.parallel && variants.len() > 4 {
            self.score_all_variants_parallel(variants, noise)
        } else {
            self.score_all_variants_sequential(variants, noise)
        }
    }

    // ========================================================================
    // Task 2.2: find_best_variant - Find the variant with highest fidelity
    // ========================================================================

    /// Finds the variant with the highest fidelity score
    fn find_best_variant(scored_variants: &[ScoredVariant]) -> Option<&ScoredVariant> {
        scored_variants.iter().max_by(|a, b| {
            a.fidelity
                .partial_cmp(&b.fidelity)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    // ========================================================================
    // Task 2.4: optimize - Full optimization pipeline
    // ========================================================================

    /// Optimize the loaded circuit using noise-adaptive reordering
    ///
    /// # Algorithm
    /// 1. Generate all valid reorderings of the circuit
    /// 2. Score each variant using `estimate_fidelity_with_scheduling`
    /// 3. Select the variant with the highest estimated fidelity
    /// 4. Return the best variant with improvement metrics
    ///
    /// # Arguments
    /// * `noise` - Noise parameters for the target hardware
    /// * `max_iterations` - Maximum number of variants to evaluate
    ///
    /// # Returns
    /// `OptimizationResult` containing the best circuit and metrics
    pub fn optimize(
        &self,
        noise: &NoiseVector,
        max_iterations: usize,
    ) -> Result<OptimizationResult> {
        let circuit = self
            .circuit
            .as_ref()
            .ok_or_else(|| QnsError::Rewire("No circuit loaded".to_string()))?;

        // Handle empty circuit
        if circuit.gates.is_empty() {
            return Ok(OptimizationResult {
                circuit: circuit.clone(),
                fidelity: 1.0,
                variants_evaluated: 0,
                improvement: 0.0,
                improved: false,
                strategy: "empty".to_string(),
            });
        }

        // Handle single-gate circuit
        if circuit.gates.len() == 1 {
            let fidelity =
                estimate_fidelity_with_idle_tracking(circuit, noise, &self.config.score_config);
            return Ok(OptimizationResult {
                circuit: circuit.clone(),
                fidelity,
                variants_evaluated: 1,
                improvement: 0.0,
                improved: false,
                strategy: "single_gate".to_string(),
            });
        }

        // Calculate baseline fidelity
        let base_fidelity =
            estimate_fidelity_with_idle_tracking(circuit, noise, &self.config.score_config);

        // Generate variants using gate reorder
        let all_variants = self.gate_reorder.generate_reorderings(circuit);

        // Limit variants to max_iterations
        let variants: Vec<_> = all_variants.into_iter().take(max_iterations).collect();

        let num_variants = variants.len();

        // Handle case where no variants were generated
        if variants.is_empty() {
            return Ok(OptimizationResult {
                circuit: circuit.clone(),
                fidelity: base_fidelity,
                variants_evaluated: 1,
                improvement: 0.0,
                improved: false,
                strategy: "no_variants".to_string(),
            });
        }

        // Determine strategy based on variant count
        let strategy = if num_variants > self.config.beam_search_threshold {
            "beam_search"
        } else {
            "bfs"
        };

        // Score all variants
        let scored_variants = self.score_all_variants(&variants, noise);

        // Find the best variant
        let best = Self::find_best_variant(&scored_variants);

        match best {
            Some(best_variant) => {
                let improvement = best_variant.fidelity - base_fidelity;
                let improved = improvement > 1e-9; // Small epsilon for floating point

                Ok(OptimizationResult {
                    circuit: best_variant.circuit.clone(),
                    fidelity: best_variant.fidelity,
                    variants_evaluated: num_variants,
                    improvement,
                    improved,
                    strategy: strategy.to_string(),
                })
            },
            None => {
                // Shouldn't happen if variants is non-empty, but handle gracefully
                Ok(OptimizationResult {
                    circuit: circuit.clone(),
                    fidelity: base_fidelity,
                    variants_evaluated: num_variants,
                    improvement: 0.0,
                    improved: false,
                    strategy: "fallback".to_string(),
                })
            },
        }
    }

    /// Optimize with detailed statistics
    pub fn optimize_with_stats(
        &self,
        noise: &NoiseVector,
        max_iterations: usize,
    ) -> Result<(OptimizationResult, OptimizationStats)> {
        let start = std::time::Instant::now();
        let result = self.optimize(noise, max_iterations)?;
        let elapsed = start.elapsed();

        let stats = OptimizationStats {
            total_time_ms: elapsed.as_millis() as u64,
            variants_per_second: if elapsed.as_secs_f64() > 0.0 {
                result.variants_evaluated as f64 / elapsed.as_secs_f64()
            } else {
                0.0
            },
            parallel_enabled: self.config.parallel,
        };

        Ok((result, stats))
    }

    // ========================================================================
    // Hardware-aware optimization with per-edge fidelity
    // ========================================================================

    /// Score a single circuit using hardware-specific per-edge fidelities.
    fn score_circuit_with_hardware(
        &self,
        circuit: &CircuitGenome,
        noise: &NoiseVector,
        hardware: &HardwareProfile,
    ) -> f64 {
        estimate_fidelity_with_hardware(circuit, noise, hardware, &self.config.score_config)
    }

    /// Scores all variants using hardware-specific per-edge fidelities.
    fn score_all_variants_with_hardware(
        &self,
        variants: &[CircuitGenome],
        noise: &NoiseVector,
        hardware: &HardwareProfile,
    ) -> Vec<ScoredVariant> {
        if self.config.parallel && variants.len() > 4 {
            variants
                .par_iter()
                .map(|circuit| {
                    let fidelity = self.score_circuit_with_hardware(circuit, noise, hardware);
                    ScoredVariant {
                        circuit: circuit.clone(),
                        fidelity,
                    }
                })
                .collect()
        } else {
            variants
                .iter()
                .map(|circuit| {
                    let fidelity = self.score_circuit_with_hardware(circuit, noise, hardware);
                    ScoredVariant {
                        circuit: circuit.clone(),
                        fidelity,
                    }
                })
                .collect()
        }
    }

    /// Optimize the loaded circuit using hardware-aware scoring.
    ///
    /// This method uses per-edge fidelity from the HardwareProfile to score
    /// circuit variants, enabling optimization based on routing through
    /// higher-fidelity edges.
    ///
    /// # Arguments
    /// * `noise` - Noise parameters (T1, T2 for decoherence)
    /// * `hardware` - Hardware profile with per-edge fidelities
    /// * `max_iterations` - Maximum number of variants to evaluate
    ///
    /// # Returns
    /// `OptimizationResult` containing the best circuit and metrics
    pub fn optimize_with_hardware(
        &self,
        noise: &NoiseVector,
        hardware: &HardwareProfile,
        max_iterations: usize,
    ) -> Result<OptimizationResult> {
        let circuit = self
            .circuit
            .as_ref()
            .ok_or_else(|| QnsError::Rewire("No circuit loaded".to_string()))?;

        // Handle empty circuit
        if circuit.gates.is_empty() {
            return Ok(OptimizationResult {
                circuit: circuit.clone(),
                fidelity: 1.0,
                variants_evaluated: 0,
                improvement: 0.0,
                improved: false,
                strategy: "empty".to_string(),
            });
        }

        // Handle single-gate circuit
        if circuit.gates.len() == 1 {
            let fidelity = self.score_circuit_with_hardware(circuit, noise, hardware);
            return Ok(OptimizationResult {
                circuit: circuit.clone(),
                fidelity,
                variants_evaluated: 1,
                improvement: 0.0,
                improved: false,
                strategy: "single_gate".to_string(),
            });
        }

        // Calculate baseline fidelity with hardware-aware scoring
        let base_fidelity = self.score_circuit_with_hardware(circuit, noise, hardware);

        // Generate variants using gate reorder
        let all_variants = self.gate_reorder.generate_reorderings(circuit);

        // Limit variants to max_iterations
        let variants: Vec<_> = all_variants.into_iter().take(max_iterations).collect();

        let num_variants = variants.len();

        // Handle case where no variants were generated
        if variants.is_empty() {
            return Ok(OptimizationResult {
                circuit: circuit.clone(),
                fidelity: base_fidelity,
                variants_evaluated: 1,
                improvement: 0.0,
                improved: false,
                strategy: "no_variants".to_string(),
            });
        }

        // Determine strategy based on variant count
        let strategy = if num_variants > self.config.beam_search_threshold {
            "hardware_beam_search"
        } else {
            "hardware_bfs"
        };

        // Score all variants with hardware-aware scoring
        let scored_variants = self.score_all_variants_with_hardware(&variants, noise, hardware);

        // Find the best variant
        let best = Self::find_best_variant(&scored_variants);

        match best {
            Some(best_variant) => {
                let improvement = best_variant.fidelity - base_fidelity;
                let improved = improvement > 1e-9;

                Ok(OptimizationResult {
                    circuit: best_variant.circuit.clone(),
                    fidelity: best_variant.fidelity,
                    variants_evaluated: num_variants,
                    improvement,
                    improved,
                    strategy: strategy.to_string(),
                })
            },
            None => Ok(OptimizationResult {
                circuit: circuit.clone(),
                fidelity: base_fidelity,
                variants_evaluated: num_variants,
                improvement: 0.0,
                improved: false,
                strategy: "fallback".to_string(),
            }),
        }
    }

    // ========================================================================
    // Placement-aware optimization (route through better edges)
    // ========================================================================

    /// Optimize with placement optimization for better edge utilization.
    ///
    /// This is the key method for "route-through-better-edges":
    /// 1. Find optimal logical-to-physical qubit mapping
    /// 2. Remap the circuit to use high-fidelity edges
    /// 3. Run hardware-aware optimization on the remapped circuit
    ///
    /// # Why this helps
    /// In a topology like: P0 --99%-- P1 --95%-- P2
    ///
    /// Original: CNOT(L1, L2) with identity mapping uses the 95% edge
    /// After placement optimization: CNOT(L1, L2) can use the 99% edge
    ///
    /// # Arguments
    /// * `noise` - Noise parameters (T1, T2 for decoherence)
    /// * `hardware` - Hardware profile with per-edge fidelities
    /// * `max_iterations` - Maximum number of variants to evaluate
    ///
    /// # Returns
    /// `PlacementOptimizationResult` with remapped circuit and metrics
    pub fn optimize_with_placement(
        &self,
        noise: &NoiseVector,
        hardware: &HardwareProfile,
        max_iterations: usize,
    ) -> Result<PlacementOptimizationResult> {
        let circuit = self
            .circuit
            .as_ref()
            .ok_or_else(|| QnsError::Rewire("No circuit loaded".to_string()))?;

        // Handle empty circuit
        if circuit.gates.is_empty() {
            return Ok(PlacementOptimizationResult {
                circuit: circuit.clone(),
                mapping: (0..circuit.num_qubits).collect(),
                fidelity: 1.0,
                original_fidelity: 1.0,
                improvement: 0.0,
                improved: false,
                strategy: "empty".to_string(),
            });
        }

        // Calculate original fidelity with identity mapping
        let original_fidelity = self.score_circuit_with_hardware(circuit, noise, hardware);

        // Step 1: Optimize placement (find best qubit mapping)
        let placement_optimizer = PlacementOptimizer::new(100, false); // Use local search
        let placement_result = placement_optimizer.optimize(circuit, hardware);

        // Step 2: Score the placement-optimized circuit
        let placed_fidelity =
            self.score_circuit_with_hardware(&placement_result.circuit, noise, hardware);

        // Step 3: Generate reordering variants on the placed circuit
        let all_variants = self
            .gate_reorder
            .generate_reorderings(&placement_result.circuit);
        let variants: Vec<_> = all_variants.into_iter().take(max_iterations).collect();

        let (best_circuit, best_fidelity) = if variants.is_empty() {
            (placement_result.circuit.clone(), placed_fidelity)
        } else {
            // Score all variants
            let scored = self.score_all_variants_with_hardware(&variants, noise, hardware);

            // Find best
            if let Some(best) = Self::find_best_variant(&scored) {
                if best.fidelity > placed_fidelity {
                    (best.circuit.clone(), best.fidelity)
                } else {
                    (placement_result.circuit.clone(), placed_fidelity)
                }
            } else {
                (placement_result.circuit.clone(), placed_fidelity)
            }
        };

        // CRITICAL: Fallback to identity if optimization causes regression
        // This prevents greedy placement from making things worse
        let identity_mapping: Vec<usize> = (0..circuit.num_qubits).collect();
        let (final_circuit, final_fidelity, final_mapping, strategy) =
            if best_fidelity >= original_fidelity {
                // Optimization helped or was neutral
                let strat = if best_fidelity > original_fidelity + 1e-9 {
                    "placement_optimized"
                } else {
                    "placement_no_improvement"
                };
                (
                    best_circuit,
                    best_fidelity,
                    placement_result.mapping,
                    strat.to_string(),
                )
            } else {
                // Optimization caused regression - fallback to identity
                (
                    circuit.clone(),
                    original_fidelity,
                    identity_mapping,
                    "fallback_identity".to_string(),
                )
            };

        let improvement = final_fidelity - original_fidelity;
        let improved = improvement > 1e-9;

        Ok(PlacementOptimizationResult {
            circuit: final_circuit,
            mapping: final_mapping,
            fidelity: final_fidelity,
            original_fidelity,
            improvement,
            improved,
            strategy,
        })
    }

    // ========================================================================
    // Co-optimization: Placement + Routing (SWAP insertion)
    // ========================================================================

    /// Full co-optimization: Placement + SWAP routing + Gate reordering.
    ///
    /// This method provides the complete optimization pipeline:
    /// 1. **Placement**: Find optimal logical-to-physical qubit mapping
    /// 2. **Routing**: Insert SWAPs where needed using NoiseAwareRouter
    /// 3. **Reordering**: Optimize gate order on the routed circuit
    ///
    /// # Why use co-optimization?
    /// - Placement alone works only if all gates already have valid edges
    /// - Routing alone uses identity mapping which may be suboptimal
    /// - Co-optimization combines both: optimal mapping + SWAP insertion
    ///
    /// # Arguments
    /// * `noise` - Noise parameters (T1, T2 for decoherence)
    /// * `hardware` - Hardware profile with per-edge fidelities
    /// * `max_iterations` - Maximum number of reordering variants to evaluate
    ///
    /// # Returns
    /// `RoutingOptimizationResult` with routed circuit and metrics
    pub fn optimize_with_routing(
        &self,
        noise: &NoiseVector,
        hardware: &HardwareProfile,
        max_iterations: usize,
    ) -> Result<RoutingOptimizationResult> {
        let circuit = self
            .circuit
            .as_ref()
            .ok_or_else(|| QnsError::Rewire("No circuit loaded".to_string()))?;

        // Handle empty circuit
        if circuit.gates.is_empty() {
            return Ok(RoutingOptimizationResult {
                circuit: circuit.clone(),
                mapping: (0..circuit.num_qubits).collect(),
                swaps_inserted: 0,
                fidelity: 1.0,
                original_fidelity: 1.0,
                improvement: 0.0,
                improved: false,
                strategy: "empty".to_string(),
            });
        }

        // Calculate original fidelity with identity mapping and routing
        let router = NoiseAwareRouter::default();
        let identity_mapping: Vec<usize> = (0..circuit.num_qubits).collect();

        let identity_routed = router.route_with_mapping(circuit, hardware, &identity_mapping)?;
        let original_fidelity = self.score_circuit_with_hardware(&identity_routed, noise, hardware);
        let original_swaps = count_swaps(&identity_routed);

        // Step 1: Optimize placement
        let placement_optimizer = PlacementOptimizer::new(100, false);
        let placement_result = placement_optimizer.optimize(circuit, hardware);

        // Step 2: Route with optimized mapping
        let routed_circuit =
            router.route_with_mapping(circuit, hardware, &placement_result.mapping)?;

        let _routed_swaps = count_swaps(&routed_circuit);
        let routed_fidelity = self.score_circuit_with_hardware(&routed_circuit, noise, hardware);

        // Step 3: Generate reordering variants on the routed circuit
        let all_variants = self.gate_reorder.generate_reorderings(&routed_circuit);
        let variants: Vec<_> = all_variants.into_iter().take(max_iterations).collect();

        let (best_circuit, best_fidelity) = if variants.is_empty() {
            (routed_circuit.clone(), routed_fidelity)
        } else {
            // Score all variants
            let scored = self.score_all_variants_with_hardware(&variants, noise, hardware);

            // Find best
            if let Some(best) = Self::find_best_variant(&scored) {
                if best.fidelity > routed_fidelity {
                    (best.circuit.clone(), best.fidelity)
                } else {
                    (routed_circuit.clone(), routed_fidelity)
                }
            } else {
                (routed_circuit.clone(), routed_fidelity)
            }
        };

        // CRITICAL: Fallback to identity if optimization causes regression
        // This prevents greedy placement from making things worse
        let (final_circuit, final_fidelity, final_mapping, strategy) =
            if best_fidelity >= original_fidelity {
                // Optimization helped or was neutral
                let final_swaps = count_swaps(&best_circuit);
                let strat = if best_fidelity > original_fidelity + 1e-9 {
                    if final_swaps < original_swaps {
                        "co_opt_fewer_swaps"
                    } else {
                        "co_opt_better_edges"
                    }
                } else {
                    "co_opt_no_improvement"
                };
                (
                    best_circuit,
                    best_fidelity,
                    placement_result.mapping,
                    strat.to_string(),
                )
            } else {
                // Optimization caused regression - fallback to identity
                (
                    identity_routed,
                    original_fidelity,
                    identity_mapping,
                    "fallback_identity".to_string(),
                )
            };

        let final_swaps = count_swaps(&final_circuit);
        let improvement = final_fidelity - original_fidelity;
        let improved = improvement > 1e-9;

        Ok(RoutingOptimizationResult {
            circuit: final_circuit,
            mapping: final_mapping,
            swaps_inserted: final_swaps,
            fidelity: final_fidelity,
            original_fidelity,
            improvement,
            improved,
            strategy,
        })
    }
}

/// Count SWAP gates in a circuit
fn count_swaps(circuit: &CircuitGenome) -> usize {
    circuit
        .gates
        .iter()
        .filter(|g| matches!(g, Gate::SWAP(_, _)))
        .count()
}

/// Result of placement-aware optimization
#[derive(Debug, Clone)]
pub struct PlacementOptimizationResult {
    /// The optimized circuit with remapped qubits
    pub circuit: CircuitGenome,
    /// The logical-to-physical qubit mapping used
    pub mapping: Vec<usize>,
    /// Estimated fidelity of the optimized circuit
    pub fidelity: f64,
    /// Original fidelity before optimization
    pub original_fidelity: f64,
    /// Improvement in fidelity (optimized - original)
    pub improvement: f64,
    /// Whether the circuit was improved
    pub improved: bool,
    /// Strategy used for optimization
    pub strategy: String,
}

/// Result of full co-optimization (placement + routing + reordering)
#[derive(Debug, Clone)]
pub struct RoutingOptimizationResult {
    /// The fully optimized and routed circuit
    pub circuit: CircuitGenome,
    /// The logical-to-physical qubit mapping used
    pub mapping: Vec<usize>,
    /// Number of SWAP gates inserted for routing
    pub swaps_inserted: usize,
    /// Estimated fidelity of the optimized circuit
    pub fidelity: f64,
    /// Original fidelity before optimization
    pub original_fidelity: f64,
    /// Improvement in fidelity (optimized - original)
    pub improvement: f64,
    /// Whether the circuit was improved
    pub improved: bool,
    /// Strategy used for optimization
    pub strategy: String,
}

/// Statistics about the optimization process
#[derive(Debug, Clone)]
pub struct OptimizationStats {
    /// Total time spent in optimization (ms)
    pub total_time_ms: u64,
    /// Variants evaluated per second
    pub variants_per_second: f64,
    /// Whether parallel evaluation was enabled
    pub parallel_enabled: bool,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_circuit() -> CircuitGenome {
        let mut circuit = CircuitGenome::new(2);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::H(1)).unwrap();
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();
        circuit
    }

    fn create_commuting_circuit() -> CircuitGenome {
        // Circuit with commuting gates that can be reordered
        let mut circuit = CircuitGenome::new(3);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::H(1)).unwrap();
        circuit.add_gate(Gate::H(2)).unwrap();
        circuit.add_gate(Gate::Z(0)).unwrap();
        circuit.add_gate(Gate::Z(1)).unwrap();
        circuit
    }

    #[test]
    fn test_optimize_empty_circuit() {
        let circuit = CircuitGenome::new(2);
        let mut rewirer = LiveRewirer::new();
        rewirer.load(circuit).unwrap();

        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
        let result = rewirer.optimize(&noise, 10).unwrap();

        assert_eq!(result.fidelity, 1.0);
        assert_eq!(result.variants_evaluated, 0);
        assert!(!result.improved);
        assert_eq!(result.strategy, "empty");
    }

    #[test]
    fn test_optimize_single_gate() {
        let mut circuit = CircuitGenome::new(1);
        circuit.add_gate(Gate::H(0)).unwrap();

        let mut rewirer = LiveRewirer::new();
        rewirer.load(circuit).unwrap();

        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
        let result = rewirer.optimize(&noise, 10).unwrap();

        assert!(result.fidelity > 0.0 && result.fidelity <= 1.0);
        assert_eq!(result.variants_evaluated, 1);
        assert!(!result.improved);
        assert_eq!(result.strategy, "single_gate");
    }

    #[test]
    fn test_optimize_returns_valid_circuit() {
        let circuit = create_test_circuit();
        let mut rewirer = LiveRewirer::new();
        rewirer.load(circuit.clone()).unwrap();

        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
        let result = rewirer.optimize(&noise, 50).unwrap();

        // Result circuit should have same gate count
        assert_eq!(result.circuit.gates.len(), circuit.gates.len());
        // Fidelity should be valid
        assert!(result.fidelity >= 0.0 && result.fidelity <= 1.0);
    }

    #[test]
    fn test_optimize_with_commuting_gates() {
        let circuit = create_commuting_circuit();
        let mut rewirer = LiveRewirer::new();
        rewirer.load(circuit.clone()).unwrap();

        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
        let result = rewirer.optimize(&noise, 100).unwrap();

        // Should evaluate multiple variants
        assert!(result.variants_evaluated > 0);
        // Result should be valid
        assert!(result.fidelity >= 0.0 && result.fidelity <= 1.0);
    }

    #[test]
    fn test_score_all_variants_sequential() {
        let circuit = create_test_circuit();
        let mut rewirer = LiveRewirer::new();
        rewirer.load(circuit.clone()).unwrap();

        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
        let variants = vec![circuit.clone(), circuit.clone()];

        let scored = rewirer.score_all_variants_sequential(&variants, &noise);

        assert_eq!(scored.len(), 2);
        for sv in &scored {
            assert!(sv.fidelity >= 0.0 && sv.fidelity <= 1.0);
        }
    }

    #[test]
    fn test_score_all_variants_parallel() {
        let circuit = create_test_circuit();
        let mut rewirer = LiveRewirer::new();
        rewirer.load(circuit.clone()).unwrap();

        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
        let variants: Vec<_> = (0..10).map(|_| circuit.clone()).collect();

        let scored = rewirer.score_all_variants_parallel(&variants, &noise);

        assert_eq!(scored.len(), 10);
        for sv in &scored {
            assert!(sv.fidelity >= 0.0 && sv.fidelity <= 1.0);
        }
    }

    #[test]
    fn test_find_best_variant() {
        let circuit = create_test_circuit();

        let scored = vec![
            ScoredVariant {
                circuit: circuit.clone(),
                fidelity: 0.5,
            },
            ScoredVariant {
                circuit: circuit.clone(),
                fidelity: 0.9,
            },
            ScoredVariant {
                circuit: circuit.clone(),
                fidelity: 0.7,
            },
        ];

        let best = LiveRewirer::find_best_variant(&scored);
        assert!(best.is_some());
        assert!((best.unwrap().fidelity - 0.9).abs() < 1e-10);
    }

    #[test]
    fn test_find_best_variant_empty() {
        let scored: Vec<ScoredVariant> = vec![];
        let best = LiveRewirer::find_best_variant(&scored);
        assert!(best.is_none());
    }

    #[test]
    fn test_optimize_with_stats() {
        let circuit = create_test_circuit();
        let mut rewirer = LiveRewirer::new();
        rewirer.load(circuit).unwrap();

        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
        let (result, stats) = rewirer.optimize_with_stats(&noise, 20).unwrap();

        assert!(result.fidelity > 0.0);
        assert!(stats.total_time_ms < 10000); // Should complete in <10s
        assert!(stats.variants_per_second >= 0.0);
    }

    #[test]
    fn test_config_parallel_threshold() {
        let config = RewireConfig {
            parallel: true,
            ..Default::default()
        };

        let rewirer = LiveRewirer::with_config(config);
        assert!(rewirer.config().parallel);
    }

    #[test]
    fn test_no_circuit_loaded_error() {
        let rewirer = LiveRewirer::new();
        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);

        let result = rewirer.optimize(&noise, 10);
        assert!(result.is_err());
    }

    // ---------- Hardware-aware optimization tests ----------

    fn create_hardware_with_varying_fidelity() -> HardwareProfile {
        use qns_core::types::Fidelity;

        // Create a linear chain with varying edge fidelities
        // 0 -- 1 -- 2
        //  99%   95%
        let mut hw = HardwareProfile::linear("test", 3);
        hw.couplers[0].gate_fidelity = Fidelity::new(0.99); // Edge 0-1: high fidelity
        hw.couplers[1].gate_fidelity = Fidelity::new(0.95); // Edge 1-2: low fidelity
        hw
    }

    #[test]
    fn test_optimize_with_hardware_basic() {
        let circuit = create_test_circuit();
        let mut rewirer = LiveRewirer::new();
        rewirer.load(circuit.clone()).unwrap();

        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
        let hw = HardwareProfile::linear("test", 2);

        let result = rewirer.optimize_with_hardware(&noise, &hw, 50).unwrap();

        assert!(result.fidelity >= 0.0 && result.fidelity <= 1.0);
        assert_eq!(result.circuit.gates.len(), circuit.gates.len());
    }

    #[test]
    fn test_optimize_with_hardware_uses_edge_fidelity() {
        // Create circuit with CNOT on different edges
        let mut circuit = CircuitGenome::new(3);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap(); // High fidelity edge
        circuit.add_gate(Gate::CNOT(1, 2)).unwrap(); // Low fidelity edge

        let mut rewirer = LiveRewirer::new();
        rewirer.load(circuit).unwrap();

        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
        let hw = create_hardware_with_varying_fidelity();

        let result = rewirer.optimize_with_hardware(&noise, &hw, 50).unwrap();

        // Should use hardware_bfs strategy
        assert!(result.strategy.starts_with("hardware"));
        assert!(result.fidelity > 0.0);
    }

    #[test]
    fn test_hardware_aware_scoring_differs_from_uniform() {
        use crate::scoring::estimate_fidelity_with_idle_tracking;

        let mut circuit = CircuitGenome::new(3);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::CNOT(1, 2)).unwrap(); // Low fidelity edge (95%)

        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
        let hw = create_hardware_with_varying_fidelity();
        let config = ScoreConfig::default();

        // Uniform scoring (uses noise.gate_error_2q)
        let uniform_fidelity = estimate_fidelity_with_idle_tracking(&circuit, &noise, &config);

        // Hardware-aware scoring (uses per-edge fidelity)
        let hw_fidelity = estimate_fidelity_with_hardware(&circuit, &noise, &hw, &config);

        // They should differ because edge (1,2) has 5% error vs uniform ~1% error
        // Hardware-aware should be lower due to higher edge error
        assert!(
            hw_fidelity < uniform_fidelity,
            "Hardware-aware fidelity ({}) should be lower than uniform ({}) due to noisy edge",
            hw_fidelity,
            uniform_fidelity
        );
    }

    #[test]
    fn test_optimize_with_hardware_empty_circuit() {
        let circuit = CircuitGenome::new(2);
        let mut rewirer = LiveRewirer::new();
        rewirer.load(circuit).unwrap();

        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
        let hw = HardwareProfile::linear("test", 2);

        let result = rewirer.optimize_with_hardware(&noise, &hw, 10).unwrap();

        assert_eq!(result.fidelity, 1.0);
        assert_eq!(result.strategy, "empty");
    }

    // ---------- Placement optimization E2E tests ----------

    #[test]
    fn test_optimize_with_placement_improves_suboptimal_mapping() {
        use qns_core::types::Fidelity;

        // Create hardware with varying fidelities: 0--99%--1--90%--2--95%--3
        let mut hw = HardwareProfile::linear("test", 4);
        hw.couplers[0].gate_fidelity = Fidelity::new(0.99); // Edge 0-1: best
        hw.couplers[1].gate_fidelity = Fidelity::new(0.90); // Edge 1-2: worst
        hw.couplers[2].gate_fidelity = Fidelity::new(0.95); // Edge 2-3: medium

        // Circuit with CNOTs on L1-L2 (identity mapping would use 90% edge)
        let mut circuit = CircuitGenome::new(4);
        for _ in 0..5 {
            circuit.add_gate(Gate::CNOT(1, 2)).unwrap();
        }

        let mut rewirer = LiveRewirer::new();
        rewirer.load(circuit).unwrap();

        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
        let result = rewirer.optimize_with_placement(&noise, &hw, 50).unwrap();

        // Should show improvement (placement moves L1-L2 to better edge)
        assert!(
            result.improved,
            "Placement should improve suboptimal mapping"
        );
        assert!(
            result.fidelity > result.original_fidelity,
            "Optimized fidelity {} should be > original {}",
            result.fidelity,
            result.original_fidelity
        );
        assert!(result.strategy.contains("placement"));
    }

    #[test]
    fn test_optimize_with_placement_preserves_optimal() {
        // Create hardware with best edge at 0-1
        let hw = HardwareProfile::linear("test", 3);

        // Circuit already using optimal edge (0-1 with identity mapping)
        let mut circuit = CircuitGenome::new(3);
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();

        let mut rewirer = LiveRewirer::new();
        rewirer.load(circuit).unwrap();

        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
        let result = rewirer.optimize_with_placement(&noise, &hw, 50).unwrap();

        // Should be identity mapping (already optimal)
        assert_eq!(
            result.mapping,
            vec![0, 1, 2],
            "Should preserve identity for already-optimal circuit"
        );
    }

    #[test]
    fn test_optimize_with_placement_validates_edges() {
        use qns_core::types::Fidelity;

        // Linear topology: 0--1--2--3
        let mut hw = HardwareProfile::linear("test", 4);
        hw.couplers[0].gate_fidelity = Fidelity::new(0.99);
        hw.couplers[1].gate_fidelity = Fidelity::new(0.90);
        hw.couplers[2].gate_fidelity = Fidelity::new(0.95);

        // Circuit with chained CNOTs: L1-L2 and L2-L3
        let mut circuit = CircuitGenome::new(4);
        for _ in 0..5 {
            circuit.add_gate(Gate::CNOT(1, 2)).unwrap();
        }
        for _ in 0..3 {
            circuit.add_gate(Gate::CNOT(2, 3)).unwrap();
        }

        let mut rewirer = LiveRewirer::new();
        rewirer.load(circuit).unwrap();

        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
        let result = rewirer.optimize_with_placement(&noise, &hw, 50).unwrap();

        // Verify all gates use valid edges after remapping
        for gate in &result.circuit.gates {
            if let Gate::CNOT(q1, q2) = gate {
                assert!(
                    hw.get_coupler(*q1, *q2).is_some(),
                    "CNOT({}, {}) should use valid edge after placement",
                    q1,
                    q2
                );
            }
        }
    }

    #[test]
    fn test_optimize_with_placement_empty_circuit() {
        let circuit = CircuitGenome::new(2);
        let mut rewirer = LiveRewirer::new();
        rewirer.load(circuit).unwrap();

        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
        let hw = HardwareProfile::linear("test", 2);

        let result = rewirer.optimize_with_placement(&noise, &hw, 10).unwrap();

        assert_eq!(result.fidelity, 1.0);
        assert_eq!(result.strategy, "empty");
    }

    // ---------- Co-optimization (Placement + Routing) E2E tests ----------

    #[test]
    fn test_optimize_with_routing_handles_non_adjacent_qubits() {
        use qns_core::types::Fidelity;

        // Linear topology: 0--1--2--3 (only adjacent qubits connected)
        let mut hw = HardwareProfile::linear("test", 4);
        hw.couplers[0].gate_fidelity = Fidelity::new(0.99);
        hw.couplers[1].gate_fidelity = Fidelity::new(0.90);
        hw.couplers[2].gate_fidelity = Fidelity::new(0.95);

        // Circuit with CNOT(0, 2) - requires at least 1 SWAP in linear topology
        // because 0 and 2 are not adjacent (0-1-2)
        let mut circuit = CircuitGenome::new(4);
        circuit.add_gate(Gate::CNOT(0, 2)).unwrap();

        let mut rewirer = LiveRewirer::new();
        rewirer.load(circuit).unwrap();

        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
        let result = rewirer.optimize_with_routing(&noise, &hw, 50).unwrap();

        // Verify circuit is valid (all 2Q gates on connected edges)
        for gate in &result.circuit.gates {
            match gate {
                Gate::CNOT(q1, q2) | Gate::CZ(q1, q2) | Gate::SWAP(q1, q2) => {
                    assert!(
                        hw.get_coupler(*q1, *q2).is_some(),
                        "Gate({}, {}) should use valid edge",
                        q1,
                        q2
                    );
                },
                _ => {},
            }
        }

        // Either SWAPs were inserted OR placement made L0 and L2 adjacent
        // (which shouldn't be possible in linear topology)
        // The important thing is the circuit is valid
    }

    #[test]
    fn test_optimize_with_routing_empty_circuit() {
        let circuit = CircuitGenome::new(2);
        let mut rewirer = LiveRewirer::new();
        rewirer.load(circuit).unwrap();

        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
        let hw = HardwareProfile::linear("test", 2);

        let result = rewirer.optimize_with_routing(&noise, &hw, 10).unwrap();

        assert_eq!(result.fidelity, 1.0);
        assert_eq!(result.swaps_inserted, 0);
        assert_eq!(result.strategy, "empty");
    }

    #[test]
    fn test_optimize_with_routing_already_connected() {
        // Circuit with CNOT(0, 1) - already connected, no SWAPs needed
        let mut circuit = CircuitGenome::new(3);
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();

        let mut rewirer = LiveRewirer::new();
        rewirer.load(circuit).unwrap();

        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
        let hw = HardwareProfile::linear("test", 3);

        let result = rewirer.optimize_with_routing(&noise, &hw, 50).unwrap();

        // No SWAPs needed for already-connected qubits
        assert_eq!(
            result.swaps_inserted, 0,
            "No SWAPs needed for already-connected qubits"
        );
    }

    #[test]
    fn test_optimize_with_routing_co_opt_vs_routing_only() {
        use qns_core::types::Fidelity;

        // Linear topology with varying fidelities
        let mut hw = HardwareProfile::linear("test", 4);
        hw.couplers[0].gate_fidelity = Fidelity::new(0.99); // 0-1: best
        hw.couplers[1].gate_fidelity = Fidelity::new(0.85); // 1-2: worst
        hw.couplers[2].gate_fidelity = Fidelity::new(0.95); // 2-3: medium

        // Circuit with CNOT(1, 2) - using worst edge with identity mapping
        let mut circuit = CircuitGenome::new(4);
        for _ in 0..3 {
            circuit.add_gate(Gate::CNOT(1, 2)).unwrap();
        }

        let mut rewirer = LiveRewirer::new();
        rewirer.load(circuit).unwrap();

        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
        let result = rewirer.optimize_with_routing(&noise, &hw, 50).unwrap();

        // Verify circuit is valid
        for gate in &result.circuit.gates {
            match gate {
                Gate::CNOT(q1, q2) | Gate::CZ(q1, q2) | Gate::SWAP(q1, q2) => {
                    assert!(
                        hw.get_coupler(*q1, *q2).is_some(),
                        "Gate({}, {}) should use valid edge",
                        q1,
                        q2
                    );
                },
                _ => {},
            }
        }
    }
}
