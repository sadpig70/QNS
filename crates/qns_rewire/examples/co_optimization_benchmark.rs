//! Co-Optimization Benchmark
//!
//! Compares all optimization strategies:
//! 1. Identity (no optimization)
//! 2. Placement only (qubit remapping)
//! 3. Routing only (SWAP insertion with identity mapping)
//! 4. Co-optimization (placement + routing + reordering)

use qns_core::types::{Fidelity, Gate};
use qns_core::{CircuitGenome, HardwareProfile, NoiseVector};
use qns_rewire::scoring::{estimate_fidelity_with_hardware, ScoreConfig};
use qns_rewire::{LiveRewirer, NoiseAwareRouter, PlacementOptimizer};

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║         Co-Optimization Strategy Comparison                  ║");
    println!("║         Placement + Routing + Reordering                     ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Create hardware with varying edge fidelities
    // Linear: Q0 --99%-- Q1 --85%-- Q2 --95%-- Q3
    let mut hw = HardwareProfile::linear("benchmark_chip", 4);
    hw.couplers[0].gate_fidelity = Fidelity::new(0.99); // Edge 0-1: BEST
    hw.couplers[1].gate_fidelity = Fidelity::new(0.85); // Edge 1-2: WORST
    hw.couplers[2].gate_fidelity = Fidelity::new(0.95); // Edge 2-3: MEDIUM

    let noise = NoiseVector::comprehensive(0, 100.0, 80.0, 0.001, 0.01, 0.02);
    let config = ScoreConfig::default();

    println!("📊 Hardware Topology (Linear 4-qubit):");
    println!("   Q0 ──99%── Q1 ──85%── Q2 ──95%── Q3");
    println!();

    // Test Case 1: CNOTs on worst edge (L1-L2)
    println!("═══════════════════════════════════════════════════════════════");
    println!("🔬 Test Case 1: 5 CNOTs on L1-L2 (worst edge with identity)");
    println!("═══════════════════════════════════════════════════════════════");

    let mut circuit1 = CircuitGenome::new(4);
    for _ in 0..5 {
        circuit1.add_gate(Gate::CNOT(1, 2)).unwrap();
    }
    run_strategy_comparison(&circuit1, &hw, &noise, &config);

    // Test Case 2: Non-adjacent qubits (requires SWAP)
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("🔬 Test Case 2: CNOT(0, 2) - Non-adjacent, requires routing");
    println!("═══════════════════════════════════════════════════════════════");

    let mut circuit2 = CircuitGenome::new(4);
    circuit2.add_gate(Gate::CNOT(0, 2)).unwrap();
    run_strategy_comparison(&circuit2, &hw, &noise, &config);

    // Test Case 3: Long-range CNOT (0, 3)
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("🔬 Test Case 3: CNOT(0, 3) - Long-range, multiple SWAPs needed");
    println!("═══════════════════════════════════════════════════════════════");

    let mut circuit3 = CircuitGenome::new(4);
    circuit3.add_gate(Gate::CNOT(0, 3)).unwrap();
    run_strategy_comparison(&circuit3, &hw, &noise, &config);

    // Test Case 4: Mixed circuit
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("🔬 Test Case 4: Mixed circuit (multiple pairs)");
    println!("═══════════════════════════════════════════════════════════════");

    let mut circuit4 = CircuitGenome::new(4);
    circuit4.add_gate(Gate::H(0)).unwrap();
    circuit4.add_gate(Gate::H(1)).unwrap();
    for _ in 0..3 {
        circuit4.add_gate(Gate::CNOT(1, 2)).unwrap();
    }
    for _ in 0..2 {
        circuit4.add_gate(Gate::CNOT(2, 3)).unwrap();
    }
    circuit4.add_gate(Gate::CNOT(0, 1)).unwrap();
    run_strategy_comparison(&circuit4, &hw, &noise, &config);

    // Summary
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                         Summary                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("✅ Co-optimization combines three techniques:");
    println!("   1. Placement: Route frequent pairs through high-fidelity edges");
    println!("   2. Routing: Insert SWAPs for non-adjacent qubits");
    println!("   3. Reordering: Minimize idle time decoherence");
    println!();
    println!("📈 Key insight: Co-optimization often matches or beats");
    println!("   individual strategies, especially for complex circuits.");
}

fn run_strategy_comparison(
    circuit: &CircuitGenome,
    hw: &HardwareProfile,
    noise: &NoiseVector,
    config: &ScoreConfig,
) {
    let router = NoiseAwareRouter::default();
    let placement_opt = PlacementOptimizer::new(100, false);

    // Strategy 1: Identity (no optimization)
    let identity_mapping: Vec<usize> = (0..circuit.num_qubits).collect();
    let identity_fidelity = estimate_fidelity_with_hardware(circuit, noise, hw, config);

    // Strategy 2: Placement only
    let placement_result = placement_opt.optimize(circuit, hw);
    let placement_fidelity =
        estimate_fidelity_with_hardware(&placement_result.circuit, noise, hw, config);

    // Strategy 3: Routing only (identity mapping + SWAP insertion)
    let routing_only = router.route_with_mapping(circuit, hw, &identity_mapping);
    let routing_fidelity = match &routing_only {
        Ok(routed) => estimate_fidelity_with_hardware(routed, noise, hw, config),
        Err(_) => 0.0,
    };
    let routing_swaps = routing_only.as_ref().map(count_swaps).unwrap_or(0);

    // Strategy 4: Co-optimization (placement + routing)
    let mut rewirer = LiveRewirer::new();
    rewirer.load(circuit.clone()).unwrap();
    let co_opt_result = rewirer.optimize_with_routing(noise, hw, 50);
    let (co_opt_fidelity, co_opt_swaps, co_opt_mapping) = match &co_opt_result {
        Ok(result) => (
            result.fidelity,
            result.swaps_inserted,
            result.mapping.clone(),
        ),
        Err(_) => (0.0, 0, vec![]),
    };

    // Find best strategy
    let strategies = [
        ("Identity", identity_fidelity),
        ("Placement", placement_fidelity),
        ("Routing", routing_fidelity),
        ("Co-opt", co_opt_fidelity),
    ];
    let best = strategies
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .unwrap();

    println!();
    println!("  📋 Strategy Comparison:");
    println!("  ┌───────────────┬────────────┬────────┬─────────────────────┐");
    println!("  │ Strategy      │ Fidelity   │ SWAPs  │ Mapping             │");
    println!("  ├───────────────┼────────────┼────────┼─────────────────────┤");
    println!(
        "  │ Identity      │ {:>8.4}%  │   -    │ {:?}",
        identity_fidelity * 100.0,
        identity_mapping
    );
    println!(
        "  │ Placement     │ {:>8.4}%  │   -    │ {:?}",
        placement_fidelity * 100.0,
        placement_result.mapping
    );
    println!(
        "  │ Routing       │ {:>8.4}%  │  {:>2}    │ {:?}",
        routing_fidelity * 100.0,
        routing_swaps,
        identity_mapping
    );
    println!(
        "  │ Co-opt        │ {:>8.4}%  │  {:>2}    │ {:?}",
        co_opt_fidelity * 100.0,
        co_opt_swaps,
        co_opt_mapping
    );
    println!("  └───────────────┴────────────┴────────┴─────────────────────┘");

    // Show winner
    let improvement =
        ((best.1 - identity_fidelity) / identity_fidelity.max(0.001) * 100.0).max(0.0);
    if improvement > 0.01 {
        println!("  🏆 Winner: {} (+{:.2}% vs Identity)", best.0, improvement);
    } else {
        println!("  ⚖️  All strategies equivalent for this circuit");
    }
}

fn count_swaps(circuit: &CircuitGenome) -> usize {
    circuit
        .gates
        .iter()
        .filter(|g| matches!(g, Gate::SWAP(_, _)))
        .count()
}
