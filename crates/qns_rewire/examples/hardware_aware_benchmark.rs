//! Hardware-Aware Optimization Benchmark
//!
//! Demonstrates the difference between uniform scoring and
//! hardware-aware scoring with per-edge fidelities.
//!
//! Usage:
//!   cargo run --release --example hardware_aware_benchmark

use qns_core::prelude::*;
use qns_core::types::Fidelity;
use qns_rewire::{
    estimate_fidelity_with_hardware, estimate_fidelity_with_idle_tracking, LiveRewirer,
    NoiseAwareRouter, RewireConfig, Router, ScoreConfig,
};

/// Create a hardware profile with realistic edge fidelity variation
///
/// Simulates an IBM-like device where different edges have different
/// two-qubit gate fidelities (ranging from 95% to 99.5%)
fn create_realistic_hardware(num_qubits: usize) -> HardwareProfile {
    let mut hw = HardwareProfile::linear("ibm_heron_sim", num_qubits);

    // Simulate realistic fidelity variation
    // Good edges: 99.5%, Average edges: 98%, Poor edges: 95%
    let fidelities = [0.995, 0.98, 0.95, 0.99, 0.97, 0.985, 0.96, 0.975];

    for (i, coupler) in hw.couplers.iter_mut().enumerate() {
        let fidelity = fidelities[i % fidelities.len()];
        coupler.gate_fidelity = Fidelity::clamped(fidelity);
    }

    hw
}

/// Create test circuits
fn create_test_circuits(num_qubits: usize) -> Vec<(String, CircuitGenome)> {
    let mut circuits = Vec::new();

    // 1. Single CNOT on each edge
    for i in 0..(num_qubits - 1) {
        let mut circuit = CircuitGenome::new(num_qubits);
        circuit.add_gate(Gate::H(i)).unwrap();
        circuit.add_gate(Gate::CNOT(i, i + 1)).unwrap();
        circuits.push((format!("cnot_edge_{}", i), circuit));
    }

    // 2. Chain of CNOTs
    let mut chain = CircuitGenome::new(num_qubits);
    for i in 0..(num_qubits - 1) {
        chain.add_gate(Gate::CNOT(i, i + 1)).unwrap();
    }
    circuits.push(("cnot_chain".to_string(), chain));

    // 3. Multi-layer circuit
    let mut multi = CircuitGenome::new(num_qubits);
    for q in 0..num_qubits {
        multi.add_gate(Gate::H(q)).unwrap();
    }
    for i in 0..(num_qubits - 1) {
        multi.add_gate(Gate::CNOT(i, i + 1)).unwrap();
    }
    for q in 0..num_qubits {
        multi
            .add_gate(Gate::Rz(q, std::f64::consts::PI / 4.0))
            .unwrap();
    }
    circuits.push(("multi_layer".to_string(), multi));

    circuits
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║         Hardware-Aware Optimization Benchmark                    ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    let num_qubits = 6;
    let hw = create_realistic_hardware(num_qubits);
    let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
    let config = ScoreConfig::default();

    // Display hardware configuration
    println!("Hardware: {} qubits, linear topology", num_qubits);
    println!("Edge fidelities:");
    for coupler in &hw.couplers {
        println!(
            "  Edge ({}-{}): {:.2}% fidelity ({:.2}% error)",
            coupler.qubit1,
            coupler.qubit2,
            coupler.gate_fidelity.value() * 100.0,
            coupler.gate_fidelity.error_rate() * 100.0
        );
    }
    println!();

    // Test circuits
    let circuits = create_test_circuits(num_qubits);

    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║  Scoring Comparison: Uniform vs Hardware-Aware                   ║");
    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!(
        "║ {:20} │ {:>12} │ {:>12} │ {:>10} ║",
        "Circuit", "Uniform", "HW-Aware", "Difference"
    );
    println!("╠══════════════════════════════════════════════════════════════════╣");

    for (name, circuit) in &circuits {
        let uniform = estimate_fidelity_with_idle_tracking(circuit, &noise, &config);
        let hw_aware = estimate_fidelity_with_hardware(circuit, &noise, &hw, &config);
        let diff = (hw_aware - uniform) * 100.0;

        println!(
            "║ {:20} │ {:>11.4}% │ {:>11.4}% │ {:>+9.2}% ║",
            name,
            uniform * 100.0,
            hw_aware * 100.0,
            diff
        );
    }
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    // Demonstrate optimization with hardware awareness
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║  Optimization Comparison                                         ║");
    println!("╠══════════════════════════════════════════════════════════════════╣");

    let rewire_config = RewireConfig {
        max_variants: 100,
        max_depth: 4,
        ..Default::default()
    };

    let mut rewirer = LiveRewirer::with_config(rewire_config.clone());

    // Test optimization on multi-layer circuit
    let (name, circuit) = &circuits.iter().find(|(n, _)| n == "multi_layer").unwrap();
    rewirer.load(circuit.clone()).unwrap();

    // Standard optimization (uniform scoring)
    let uniform_result = rewirer.optimize(&noise, 100).unwrap();

    // Hardware-aware optimization
    let hw_result = rewirer.optimize_with_hardware(&noise, &hw, 100).unwrap();

    println!(
        "║ Circuit: {}                                            ║",
        name
    );
    println!("║                                                                    ║");
    println!("║ Uniform optimization:                                              ║");
    println!(
        "║   Original fidelity:  {:>10.4}%                                ║",
        estimate_fidelity_with_idle_tracking(circuit, &noise, &config) * 100.0
    );
    println!(
        "║   Optimized fidelity: {:>10.4}%                                ║",
        uniform_result.fidelity * 100.0
    );
    println!(
        "║   Improvement:        {:>+10.4}%                                ║",
        uniform_result.improvement * 100.0
    );
    println!(
        "║   Strategy:           {:>10}                                 ║",
        uniform_result.strategy
    );
    println!("║                                                                    ║");
    println!("║ Hardware-aware optimization:                                       ║");
    println!(
        "║   Original fidelity:  {:>10.4}%                                ║",
        estimate_fidelity_with_hardware(circuit, &noise, &hw, &config) * 100.0
    );
    println!(
        "║   Optimized fidelity: {:>10.4}%                                ║",
        hw_result.fidelity * 100.0
    );
    println!(
        "║   Improvement:        {:>+10.4}%                                ║",
        hw_result.improvement * 100.0
    );
    println!(
        "║   Strategy:           {:>10}                                 ║",
        hw_result.strategy
    );
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    // Demonstrate NoiseAwareRouter
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║  NoiseAwareRouter Demo                                           ║");
    println!("╠══════════════════════════════════════════════════════════════════╣");

    // Create a circuit that requires routing (CNOT between non-adjacent qubits)
    let mut routing_circuit = CircuitGenome::new(num_qubits);
    routing_circuit.add_gate(Gate::H(0)).unwrap();
    routing_circuit.add_gate(Gate::CNOT(0, 3)).unwrap(); // Non-adjacent in linear topology

    let router = NoiseAwareRouter::default();

    match router.route(&routing_circuit, &hw) {
        Ok(routed) => {
            println!("║ Original circuit: H(0), CNOT(0,3)                                ║");
            println!("║ Requires routing (qubits 0,3 not adjacent)                       ║");
            println!("║                                                                    ║");
            println!(
                "║ Routed circuit ({} gates):                                        ║",
                routed.gates.len()
            );
            for (i, gate) in routed.gates.iter().enumerate() {
                println!(
                    "║   {}: {}                                                     ║",
                    i + 1,
                    gate
                );
            }
            println!("║                                                                    ║");
            println!(
                "║ Original fidelity: {:>10.4}%                                   ║",
                estimate_fidelity_with_hardware(&routing_circuit, &noise, &hw, &config) * 100.0
            );
            println!(
                "║ Routed fidelity:   {:>10.4}%                                   ║",
                estimate_fidelity_with_hardware(&routed, &noise, &hw, &config) * 100.0
            );
        },
        Err(e) => {
            println!(
                "║ Routing failed: {}                                               ║",
                e
            );
        },
    }
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    println!("Summary:");
    println!("  - Hardware-aware scoring uses per-edge fidelity from HardwareProfile");
    println!("  - Circuits using higher-fidelity edges score better");
    println!("  - NoiseAwareRouter prefers routing through better edges");
    println!("  - This enables true noise-adaptive optimization on real hardware");
}
