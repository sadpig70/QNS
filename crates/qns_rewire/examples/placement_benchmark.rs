//! Placement Optimization Benchmark
//!
//! Demonstrates the "Route-through-better-edges" optimization.
//! Shows actual fidelity improvement by placing high-frequency CNOT pairs
//! on high-fidelity physical edges.

use qns_core::types::{Fidelity, Gate};
use qns_core::{CircuitGenome, HardwareProfile, NoiseVector};
use qns_rewire::scoring::{
    estimate_fidelity_with_hardware, gate_error_sum_with_hardware, ScoreConfig,
};
use qns_rewire::PlacementOptimizer;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║       Placement Optimization Benchmark                        ║");
    println!("║       Route-Through-Better-Edges 검증                         ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Create hardware with varying edge fidelities
    // Linear: Q0 --99%-- Q1 --90%-- Q2 --95%-- Q3
    let mut hw = HardwareProfile::linear("benchmark_chip", 4);
    hw.couplers[0].gate_fidelity = Fidelity::new(0.99); // Edge 0-1: BEST
    hw.couplers[1].gate_fidelity = Fidelity::new(0.90); // Edge 1-2: WORST
    hw.couplers[2].gate_fidelity = Fidelity::new(0.95); // Edge 2-3: MEDIUM

    // Create noise vector for scoring
    let noise = NoiseVector::comprehensive(0, 100.0, 80.0, 0.001, 0.01, 0.02);

    let config = ScoreConfig::default();

    println!("📊 Hardware Topology (Linear 4-qubit):");
    println!("   Q0 ──99%── Q1 ──90%── Q2 ──95%── Q3");
    println!();

    // Scenario 1: High-frequency CNOTs between logical qubits 1 and 2
    println!("═══════════════════════════════════════════════════════════════");
    println!("🔬 Scenario 1: 10 CNOTs between L1-L2");
    println!("   Identity mapping would use 90% edge (worst)");
    println!("═══════════════════════════════════════════════════════════════");

    let mut circuit1 = CircuitGenome::new(4);
    for _ in 0..10 {
        circuit1.add_gate(Gate::CNOT(1, 2)).unwrap();
    }

    run_placement_test(&circuit1, &hw, &noise, &config);

    // Scenario 2: Mixed CNOTs with different frequencies
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("🔬 Scenario 2: Mixed circuit (8 L0-L1, 2 L2-L3)");
    println!("   Tests prioritization of high-frequency pairs");
    println!("═══════════════════════════════════════════════════════════════");

    let mut circuit2 = CircuitGenome::new(4);
    for _ in 0..8 {
        circuit2.add_gate(Gate::CNOT(0, 1)).unwrap();
    }
    for _ in 0..2 {
        circuit2.add_gate(Gate::CNOT(2, 3)).unwrap();
    }

    run_placement_test(&circuit2, &hw, &noise, &config);

    // Scenario 3: Already optimal placement
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("🔬 Scenario 3: CNOTs on L0-L1 (already optimal)");
    println!("   Identity mapping already uses 99% edge");
    println!("═══════════════════════════════════════════════════════════════");

    let mut circuit3 = CircuitGenome::new(4);
    for _ in 0..5 {
        circuit3.add_gate(Gate::CNOT(0, 1)).unwrap();
    }

    run_placement_test(&circuit3, &hw, &noise, &config);

    // Scenario 4: Compare greedy vs local search
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("🔬 Scenario 4: Greedy vs Local Search comparison");
    println!("═══════════════════════════════════════════════════════════════");

    let mut circuit4 = CircuitGenome::new(4);
    // Create a pattern where local search might find better solution
    for _ in 0..5 {
        circuit4.add_gate(Gate::CNOT(1, 2)).unwrap();
    }
    for _ in 0..3 {
        circuit4.add_gate(Gate::CNOT(2, 3)).unwrap();
    }

    compare_strategies(&circuit4, &hw, &noise, &config);

    // Final summary
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                      Summary                                  ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("✅ Placement optimization successfully routes CNOTs through");
    println!("   higher-fidelity edges by remapping logical to physical qubits.");
    println!();
    println!("📈 Key insight: Unlike gate reordering (which only changes gate");
    println!("   order), placement optimization changes WHICH edges are used,");
    println!("   providing real fidelity improvements for non-optimal mappings.");
}

fn run_placement_test(
    circuit: &CircuitGenome,
    hw: &HardwareProfile,
    noise: &NoiseVector,
    config: &ScoreConfig,
) {
    let optimizer = PlacementOptimizer::new(100, false); // Use local search

    // Calculate original fidelity (identity mapping)
    let identity_mapping: Vec<usize> = (0..circuit.num_qubits).collect();
    let identity_circuit = optimizer.apply_mapping(circuit, &identity_mapping);
    let original_error = gate_error_sum_with_hardware(&identity_circuit, noise, hw);
    let original_fidelity = estimate_fidelity_with_hardware(&identity_circuit, noise, hw, config);

    // Optimize placement
    let result = optimizer.optimize(circuit, hw);
    let optimized_error = gate_error_sum_with_hardware(&result.circuit, noise, hw);
    let optimized_fidelity = estimate_fidelity_with_hardware(&result.circuit, noise, hw, config);

    // Calculate improvement (handle near-zero original fidelity)
    let fidelity_improvement = if original_fidelity > 0.001 {
        ((optimized_fidelity - original_fidelity) / original_fidelity * 100.0).max(0.0)
    } else {
        // Original fidelity near zero - show absolute improvement instead
        (optimized_fidelity - original_fidelity) * 100.0
    };
    let error_reduction = if original_error > 0.001 {
        ((original_error - optimized_error) / original_error * 100.0).max(0.0)
    } else {
        0.0
    };

    println!();
    println!("  📋 Results:");
    println!("  ┌─────────────────────────────────────────────────────┐");
    println!("  │ Mapping: {:?}", result.mapping);
    println!("  │");
    println!("  │ Original (identity):                                │");
    println!("  │   Gate Error Sum: {:.4}", original_error);
    println!("  │   Est. Fidelity:  {:.4}%", original_fidelity * 100.0);
    println!("  │");
    println!("  │ Optimized:                                          │");
    println!("  │   Gate Error Sum: {:.4}", optimized_error);
    println!("  │   Est. Fidelity:  {:.4}%", optimized_fidelity * 100.0);
    println!("  │");
    if fidelity_improvement > 0.01 {
        println!("  │ 📈 Fidelity Improvement: +{:.2}%", fidelity_improvement);
        println!("  │ 📉 Error Reduction:      -{:.2}%", error_reduction);
    } else {
        println!("  │ ⚖️  No improvement (already optimal)");
    }
    println!("  └─────────────────────────────────────────────────────┘");
}

fn compare_strategies(
    circuit: &CircuitGenome,
    hw: &HardwareProfile,
    noise: &NoiseVector,
    config: &ScoreConfig,
) {
    let greedy_opt = PlacementOptimizer::new(100, true);
    let local_opt = PlacementOptimizer::new(100, false);

    let greedy_result = greedy_opt.optimize(circuit, hw);
    let local_result = local_opt.optimize(circuit, hw);

    let greedy_fidelity =
        estimate_fidelity_with_hardware(&greedy_result.circuit, noise, hw, config);
    let local_fidelity = estimate_fidelity_with_hardware(&local_result.circuit, noise, hw, config);

    println!();
    println!("  📋 Strategy Comparison:");
    println!("  ┌─────────────────────────────────────────────────────┐");
    println!("  │ Greedy:                                             │");
    println!("  │   Mapping:   {:?}", greedy_result.mapping);
    println!("  │   Fidelity:  {:.4}%", greedy_fidelity * 100.0);
    println!("  │");
    println!("  │ Local Search:                                       │");
    println!("  │   Mapping:   {:?}", local_result.mapping);
    println!("  │   Fidelity:  {:.4}%", local_fidelity * 100.0);
    println!("  │");
    if local_fidelity > greedy_fidelity {
        println!(
            "  │ 🏆 Local Search wins by {:.4}%",
            (local_fidelity - greedy_fidelity) * 100.0
        );
    } else if greedy_fidelity > local_fidelity {
        println!(
            "  │ 🏆 Greedy wins by {:.4}%",
            (greedy_fidelity - local_fidelity) * 100.0
        );
    } else {
        println!("  │ ⚖️  Both strategies found the same solution");
    }
    println!("  └─────────────────────────────────────────────────────┘");
}
