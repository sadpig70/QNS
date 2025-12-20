//! Heavy-Hex Topology Benchmark
//!
//! Tests co-optimization on IBM Heron-like Heavy-hex topology.
//! Heavy-hex is used in IBM's latest quantum processors (127+ qubits).
//!
//! Topology structure (simplified 12-qubit unit):
//! ```
//!     0 --- 1 --- 2 --- 3
//!     |           |
//!     4           5
//!     |           |
//!     6 --- 7 --- 8 --- 9
//!     |           |
//!    10          11
//! ```

use qns_core::types::{CouplerProperties, Fidelity, Gate, Topology};
use qns_core::{CircuitGenome, HardwareProfile, NoiseVector};
use qns_rewire::scoring::{estimate_fidelity_with_hardware, ScoreConfig};
use qns_rewire::{LiveRewirer, PlacementOptimizer};

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║         Heavy-Hex Topology Benchmark                         ║");
    println!("║         IBM Heron-like Quantum Processor                     ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Create Heavy-hex-like hardware (12 qubits)
    let hw = create_heavy_hex_hardware();

    println!("📊 Heavy-Hex Topology (12-qubit unit):");
    println!("     0 ─99%─ 1 ─98%─ 2 ─97%─ 3");
    println!("     │             │");
    println!("    95%           96%");
    println!("     │             │");
    println!("     4             5");
    println!("     │             │");
    println!("    94%           95%");
    println!("     │             │");
    println!("     6 ─98%─ 7 ─97%─ 8 ─96%─ 9");
    println!("     │             │");
    println!("    93%           94%");
    println!("     │             │");
    println!("    10            11");
    println!();

    let noise = NoiseVector::comprehensive(0, 100.0, 80.0, 0.001, 0.01, 0.02);
    let config = ScoreConfig::default();

    // Test Case 1: Adjacent qubits (already optimal)
    println!("═══════════════════════════════════════════════════════════════");
    println!("🔬 Test 1: CNOT(0, 1) - Adjacent qubits (best edge)");
    println!("═══════════════════════════════════════════════════════════════");

    let mut circuit1 = CircuitGenome::new(12);
    circuit1.add_gate(Gate::CNOT(0, 1)).unwrap();
    run_comparison(&circuit1, &hw, &noise, &config);

    // Test Case 2: Vertical connection through bridge qubit
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("🔬 Test 2: CNOTs through vertical bridge (0→4→6)");
    println!("═══════════════════════════════════════════════════════════════");

    let mut circuit2 = CircuitGenome::new(12);
    for _ in 0..3 {
        circuit2.add_gate(Gate::CNOT(0, 6)).unwrap(); // Long-range vertical
    }
    run_comparison(&circuit2, &hw, &noise, &config);

    // Test Case 3: Cross-row connection (worst case)
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("🔬 Test 3: CNOT(3, 6) - Cross-row (requires routing)");
    println!("═══════════════════════════════════════════════════════════════");

    let mut circuit3 = CircuitGenome::new(12);
    circuit3.add_gate(Gate::CNOT(3, 6)).unwrap();
    run_comparison(&circuit3, &hw, &noise, &config);

    // Test Case 4: Real-world pattern - GHZ state preparation
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("🔬 Test 4: GHZ-like pattern (chained CNOTs)");
    println!("═══════════════════════════════════════════════════════════════");

    let mut circuit4 = CircuitGenome::new(12);
    circuit4.add_gate(Gate::H(0)).unwrap();
    // Chain: 0 -> 1 -> 2 -> 5 -> 8 -> 9
    circuit4.add_gate(Gate::CNOT(0, 1)).unwrap();
    circuit4.add_gate(Gate::CNOT(1, 2)).unwrap();
    circuit4.add_gate(Gate::CNOT(2, 5)).unwrap();
    circuit4.add_gate(Gate::CNOT(5, 8)).unwrap();
    circuit4.add_gate(Gate::CNOT(8, 9)).unwrap();
    run_comparison(&circuit4, &hw, &noise, &config);

    // Test Case 5: High-frequency pair on low-fidelity edge
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("🔬 Test 5: 5 CNOTs on L6-L10 (93% edge - worst)");
    println!("═══════════════════════════════════════════════════════════════");

    let mut circuit5 = CircuitGenome::new(12);
    for _ in 0..5 {
        circuit5.add_gate(Gate::CNOT(6, 10)).unwrap();
    }
    run_comparison(&circuit5, &hw, &noise, &config);

    // Summary
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                         Summary                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("✅ Heavy-hex topology characteristics:");
    println!("   • Sparse connectivity (degree 2-3 per qubit)");
    println!("   • Bridge qubits connect rows (vertical links)");
    println!("   • Long-range gates require multiple SWAPs");
    println!();
    println!("📈 Co-optimization benefits:");
    println!("   • Routes high-frequency pairs to best edges");
    println!("   • Reduces SWAP overhead through smart placement");
    println!("   • Critical for real hardware with varying fidelities");
}

/// Creates a Heavy-hex-like hardware profile with realistic fidelities.
///
/// Topology (12 qubits):
/// ```
///     0 --- 1 --- 2 --- 3
///     |           |
///     4           5
///     |           |
///     6 --- 7 --- 8 --- 9
///     |           |
///    10          11
/// ```
fn create_heavy_hex_hardware() -> HardwareProfile {
    let mut hw = HardwareProfile::new("ibm_heron_like", 12, Topology::HeavyHex);

    // HeavyHex topology starts with empty couplers, so we add them manually
    // Top row: 0-1-2-3
    hw.add_coupler(coupler(0, 1, 0.99)); // Best edge
    hw.add_coupler(coupler(1, 2, 0.98));
    hw.add_coupler(coupler(2, 3, 0.97));

    // Vertical bridges (left side): 0-4-6-10
    hw.add_coupler(coupler(0, 4, 0.95));
    hw.add_coupler(coupler(4, 6, 0.94));
    hw.add_coupler(coupler(6, 10, 0.93)); // Worst edge

    // Vertical bridges (right side): 2-5-8-11
    hw.add_coupler(coupler(2, 5, 0.96));
    hw.add_coupler(coupler(5, 8, 0.95));
    hw.add_coupler(coupler(8, 11, 0.94));

    // Middle row: 6-7-8-9
    hw.add_coupler(coupler(6, 7, 0.98));
    hw.add_coupler(coupler(7, 8, 0.97));
    hw.add_coupler(coupler(8, 9, 0.96));

    hw
}

fn coupler(q1: usize, q2: usize, fidelity: f64) -> CouplerProperties {
    let mut c = CouplerProperties::new(q1, q2);
    c.gate_fidelity = Fidelity::new(fidelity);
    c
}

fn run_comparison(
    circuit: &CircuitGenome,
    hw: &HardwareProfile,
    noise: &NoiseVector,
    config: &ScoreConfig,
) {
    // Identity baseline
    let identity_fidelity = estimate_fidelity_with_hardware(circuit, noise, hw, config);

    // Placement optimization
    let placement_opt = PlacementOptimizer::new(100, false);
    let placement_result = placement_opt.optimize(circuit, hw);
    let placement_fidelity =
        estimate_fidelity_with_hardware(&placement_result.circuit, noise, hw, config);

    // Full co-optimization
    let mut rewirer = LiveRewirer::new();
    rewirer.load(circuit.clone()).unwrap();
    let co_opt_result = rewirer.optimize_with_routing(noise, hw, 50);

    let (co_opt_fidelity, co_opt_swaps, co_opt_mapping) = match &co_opt_result {
        Ok(r) => (r.fidelity, r.swaps_inserted, r.mapping.clone()),
        Err(e) => {
            println!("  ⚠️  Co-optimization failed: {}", e);
            (0.0, 0, vec![])
        },
    };

    // Results
    println!();
    println!("  📋 Results:");
    println!("  ┌────────────────┬────────────┬────────┐");
    println!("  │ Strategy       │ Fidelity   │ SWAPs  │");
    println!("  ├────────────────┼────────────┼────────┤");
    println!(
        "  │ Identity       │ {:>8.4}%  │   -    │",
        identity_fidelity * 100.0
    );
    println!(
        "  │ Placement      │ {:>8.4}%  │   -    │",
        placement_fidelity * 100.0
    );
    println!(
        "  │ Co-optimization│ {:>8.4}%  │  {:>2}    │",
        co_opt_fidelity * 100.0,
        co_opt_swaps
    );
    println!("  └────────────────┴────────────┴────────┘");

    if !co_opt_mapping.is_empty() {
        // Show simplified mapping
        let changed: Vec<_> = co_opt_mapping
            .iter()
            .enumerate()
            .filter(|(i, &p)| *i != p)
            .map(|(l, p)| format!("L{}→P{}", l, p))
            .collect();
        if changed.is_empty() {
            println!("  📍 Mapping: identity (no remapping needed)");
        } else {
            println!("  📍 Mapping changes: {}", changed.join(", "));
        }
    }

    // Improvement
    let improvement = if identity_fidelity > 0.001 {
        (co_opt_fidelity - identity_fidelity) / identity_fidelity * 100.0
    } else {
        0.0
    };

    if improvement > 0.1 {
        println!("  🏆 Improvement: +{:.2}%", improvement);
    } else if improvement < -0.1 {
        println!("  ⚠️  Regression: {:.2}%", improvement);
    } else {
        println!("  ⚖️  No significant change");
    }
}
