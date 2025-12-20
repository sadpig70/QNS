//! Integration tests for QNS (Quantum Noise Symbiote).
//!
//! These tests verify the complete pipeline from noise profiling
//! through circuit optimization to simulation verification.

use qns_cli::pipeline::QnsSystem;
use qns_core::prelude::*;
use qns_profiler::DriftScanner;
use qns_rewire::{GateReorder, LiveRewirer};
use qns_simulator::{NoiseModel, NoisySimulator, StateVectorSimulator};
use std::time::{Duration, Instant};

// ============================================================================
// Full Pipeline Tests
// ============================================================================

#[test]
fn test_full_pipeline_bell_state() {
    // Create Bell state circuit
    let mut circuit = CircuitGenome::new(2);
    circuit.add_gate(Gate::H(0)).unwrap();
    circuit.add_gate(Gate::CNOT(0, 1)).unwrap();

    // Run full pipeline
    let mut system = QnsSystem::new();
    let result = system.optimize(circuit).unwrap();

    // Verify results
    assert!(
        result.original_fidelity >= 0.99,
        "Bell state should have near-perfect fidelity"
    );
    assert!(
        result.optimized_fidelity >= 0.99,
        "Optimized circuit should maintain fidelity"
    );
    assert!(
        result.total_time < Duration::from_secs(1),
        "Pipeline should complete quickly"
    );
}

#[test]
fn test_full_pipeline_ghz_state() {
    // Create GHZ state circuit (3 qubits)
    let mut circuit = CircuitGenome::new(3);
    circuit.add_gate(Gate::H(0)).unwrap();
    circuit.add_gate(Gate::CNOT(0, 1)).unwrap();
    circuit.add_gate(Gate::CNOT(1, 2)).unwrap();

    let mut system = QnsSystem::new();
    let result = system.optimize(circuit).unwrap();

    assert!(result.original_fidelity >= 0.99);
    assert_eq!(result.original_circuit.num_qubits, 3);
    assert_eq!(result.optimized_circuit.num_qubits, 3);
}

#[test]
fn test_full_pipeline_with_hardware() {
    let mut circuit = CircuitGenome::new(5);
    circuit.add_gate(Gate::H(0)).unwrap();
    circuit.add_gate(Gate::X(2)).unwrap();
    circuit.add_gate(Gate::CNOT(0, 1)).unwrap();
    circuit.add_gate(Gate::CNOT(2, 3)).unwrap();

    let mut system = QnsSystem::new();
    let hw = HardwareProfile::linear("test", 5);
    system.set_hardware(hw);

    let result = system.optimize(circuit).unwrap();
    assert!(!result.optimized_circuit.gates.is_empty());
}

// ============================================================================
// Cross-Module Integration Tests
// ============================================================================

#[test]
fn test_profiler_to_rewirer_integration() {
    // Step 1: Profile noise
    let mut scanner = DriftScanner::with_defaults();
    let noise_results = scanner.scan_batch(&[0, 1, 2]).unwrap();

    assert_eq!(noise_results.len(), 3);
    for nv in &noise_results {
        assert!(nv.t1_mean > 0.0);
        assert!(nv.t2_mean > 0.0);
        // Physical constraint: T2 <= 2*T1
        assert!(nv.t2_mean <= 2.0 * nv.t1_mean + 0.01);
    }

    // Step 2: Use noise profile for optimization
    let mut circuit = CircuitGenome::new(3);
    circuit.add_gate(Gate::H(0)).unwrap();
    circuit.add_gate(Gate::X(1)).unwrap();
    circuit.add_gate(Gate::Z(2)).unwrap();
    circuit.add_gate(Gate::CNOT(0, 1)).unwrap();

    let mut rewirer = LiveRewirer::new();
    rewirer.load(circuit).unwrap();

    // Use aggregated noise
    let avg_noise = NoiseVector::with_t1t2(
        0,
        noise_results.iter().map(|n| n.t1_mean).sum::<f64>() / 3.0,
        noise_results.iter().map(|n| n.t2_mean).sum::<f64>() / 3.0,
    );

    let optimized = rewirer.optimize(&avg_noise, 20).unwrap();
    assert!(!optimized.circuit.gates.is_empty());
}

#[test]
fn test_rewirer_to_simulator_integration() {
    // Step 1: Create and optimize circuit
    let mut circuit = CircuitGenome::new(2);
    circuit.add_gate(Gate::H(0)).unwrap();
    circuit.add_gate(Gate::X(1)).unwrap(); // Commutes with H(0)
    circuit.add_gate(Gate::CNOT(0, 1)).unwrap();

    let mut rewirer = LiveRewirer::new();
    rewirer.load(circuit.clone()).unwrap();

    let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
    let optimized = rewirer.optimize(&noise, 10).unwrap();

    // Step 2: Simulate both circuits
    let mut sim_orig = StateVectorSimulator::new(2);
    sim_orig.run(&circuit).unwrap();

    let mut sim_opt = StateVectorSimulator::new(2);
    sim_opt.run(&optimized.circuit).unwrap();

    // Both should produce valid quantum states
    assert!(sim_orig.is_normalized());
    assert!(sim_opt.is_normalized());

    // Compare fidelity (should be equivalent for commuting gate reorderings)
    let fidelity = sim_orig.fidelity_with(&sim_opt).unwrap();
    assert!(
        fidelity > 0.99,
        "Commuting reorderings should preserve fidelity: {}",
        fidelity
    );
}

#[test]
fn test_profiler_to_noisy_simulator_integration() {
    // Profile noise
    let mut scanner = DriftScanner::fast();
    let noise_vec = scanner.scan(0).unwrap();

    // Create noise model from profile
    let noise_model = NoiseModel::with_t1t2(noise_vec.t1_mean, noise_vec.t2_mean);

    // Create noisy simulator
    let mut noisy_sim = NoisySimulator::new(2, noise_model);

    // Run circuit
    noisy_sim.apply_gate(&Gate::H(0)).unwrap();
    noisy_sim.apply_gate(&Gate::CNOT(0, 1)).unwrap();

    // Should produce valid state
    assert!(noisy_sim.is_normalized());
    assert!(noisy_sim.gate_count() == 2);
}

// ============================================================================
// Performance Tests
// ============================================================================

#[test]
fn test_pipeline_performance_targets() {
    let start = Instant::now();

    // Profile: Target <10ms per qubit
    let profile_start = Instant::now();
    let mut scanner = DriftScanner::fast();
    for i in 0..5 {
        let _ = scanner.scan(i).unwrap();
    }
    let profile_time = profile_start.elapsed();

    // Optimize: Target <100ms for 50 variants
    let opt_start = Instant::now();
    let mut circuit = CircuitGenome::new(5);
    for i in 0..20 {
        match i % 4 {
            0 => circuit.add_gate(Gate::H(i % 5)).unwrap(),
            1 => circuit.add_gate(Gate::X(i % 5)).unwrap(),
            2 => circuit.add_gate(Gate::CNOT(i % 5, (i + 1) % 5)).unwrap(),
            _ => circuit.add_gate(Gate::T(i % 5)).unwrap(),
        }
    }

    let mut rewirer = LiveRewirer::new();
    rewirer.load(circuit).unwrap();
    let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
    let _ = rewirer.optimize(&noise, 50).unwrap();
    let opt_time = opt_start.elapsed();

    // Simulate: Target <50ms for 5 qubits
    let sim_start = Instant::now();
    let mut sim = StateVectorSimulator::new(5);
    sim.prepare_ghz_state().unwrap();
    let _ = sim.measure(1000).unwrap();
    let sim_time = sim_start.elapsed();

    let total_time = start.elapsed();

    // Verify targets (with generous margins for CI variability)
    let margin = 10.0; // 10x margin for CI
    assert!(
        profile_time < Duration::from_millis(50 * 5 * margin as u64),
        "Profile time exceeded: {:?}",
        profile_time
    );
    assert!(
        opt_time < Duration::from_millis(100 * margin as u64),
        "Optimize time exceeded: {:?}",
        opt_time
    );
    assert!(
        sim_time < Duration::from_millis(50 * margin as u64),
        "Simulate time exceeded: {:?}",
        sim_time
    );
    assert!(
        total_time < Duration::from_millis(200 * margin as u64),
        "Total time exceeded: {:?}",
        total_time
    );

    println!("Performance Results:");
    println!("  Profile (5 qubits): {:?}", profile_time);
    println!("  Optimize (50 variants): {:?}", opt_time);
    println!("  Simulate (5 qubits, 1000 shots): {:?}", sim_time);
    println!("  Total: {:?}", total_time);
}

#[test]
fn test_scalability_qubits() {
    // Test scaling with qubit count
    for num_qubits in [2, 3, 5, 8] {
        let start = Instant::now();

        let mut sim = StateVectorSimulator::new(num_qubits);
        sim.prepare_ghz_state().unwrap();
        let results = sim.measure(100).unwrap();

        let elapsed = start.elapsed();

        // Should complete in reasonable time
        assert!(
            elapsed < Duration::from_secs(1),
            "{} qubits took {:?}",
            num_qubits,
            elapsed
        );

        // GHZ should produce only |0...0> and |1...1>
        for (bitstring, count) in &results {
            let is_all_zeros = bitstring.chars().all(|c| c == '0');
            let is_all_ones = bitstring.chars().all(|c| c == '1');
            assert!(
                is_all_zeros || is_all_ones || *count == 0,
                "Unexpected outcome: {} = {}",
                bitstring,
                count
            );
        }
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_empty_circuit() {
    let circuit = CircuitGenome::new(2);

    let mut system = QnsSystem::new();
    let result = system.optimize(circuit).unwrap();

    assert_eq!(result.optimized_circuit.gates.len(), 0);
    assert!((result.original_fidelity - 1.0).abs() < 1e-10);
}

#[test]
fn test_single_gate_circuit() {
    let mut circuit = CircuitGenome::new(1);
    circuit.add_gate(Gate::H(0)).unwrap();

    let mut system = QnsSystem::new();
    let result = system.optimize(circuit).unwrap();

    assert_eq!(result.optimized_circuit.gates.len(), 1);
}

#[test]
fn test_max_qubits() {
    // Test near maximum qubit count (10 for reasonable memory)
    let mut sim = StateVectorSimulator::new(10);

    // Apply some gates
    for i in 0..10 {
        sim.apply_gate(&Gate::H(i)).unwrap();
    }

    assert!(sim.is_normalized());
    assert_eq!(sim.dimension(), 1024); // 2^10
}

#[test]
fn test_deep_circuit() {
    // Test circuit with many gates
    let mut circuit = CircuitGenome::new(3);
    for i in 0..100 {
        circuit.add_gate(Gate::Rz(i % 3, 0.1 * i as f64)).unwrap();
    }

    let mut sim = StateVectorSimulator::new(3);
    sim.run(&circuit).unwrap();

    assert!(sim.is_normalized());
}

// ============================================================================
// Correctness Tests
// ============================================================================

#[test]
fn test_bell_state_correctness() {
    let mut sim = StateVectorSimulator::new(2);
    sim.prepare_bell_state().unwrap();

    let results = sim.measure(10000).unwrap();

    let count_00 = results.get("00").copied().unwrap_or(0);
    let count_11 = results.get("11").copied().unwrap_or(0);
    let count_01 = results.get("01").copied().unwrap_or(0);
    let count_10 = results.get("10").copied().unwrap_or(0);

    // Bell state should give 50% |00> and 50% |11>
    assert!(
        count_00 > 4000 && count_00 < 6000,
        "Expected ~5000 |00>, got {}",
        count_00
    );
    assert!(
        count_11 > 4000 && count_11 < 6000,
        "Expected ~5000 |11>, got {}",
        count_11
    );
    assert!(count_01 < 100, "Expected ~0 |01>, got {}", count_01);
    assert!(count_10 < 100, "Expected ~0 |10>, got {}", count_10);
}

#[test]
fn test_ghz_state_correctness() {
    let mut sim = StateVectorSimulator::new(3);
    sim.prepare_ghz_state().unwrap();

    let probs = sim.probabilities();

    // GHZ state: (|000> + |111>) / sqrt(2)
    assert!((probs[0] - 0.5).abs() < 0.001, "P(|000>) = {}", probs[0]);
    assert!((probs[7] - 0.5).abs() < 0.001, "P(|111>) = {}", probs[7]);

    // All others should be 0
    for (i, &prob) in probs.iter().enumerate().take(7).skip(1) {
        assert!(prob < 0.001, "P(|{}>) = {}", i, prob);
    }
}

#[test]
fn test_gate_commutation_correctness() {
    let reorder = GateReorder::default();

    // Create circuit with commuting gates
    let mut circuit = CircuitGenome::new(3);
    circuit.add_gate(Gate::H(0)).unwrap();
    circuit.add_gate(Gate::X(1)).unwrap();
    circuit.add_gate(Gate::Z(2)).unwrap();

    let pairs = reorder.find_adjacent_commuting_pairs(&circuit);

    // All pairs should commute (different qubits)
    assert!(pairs.len() >= 2);

    // Verify variants produce same final state
    let variants = reorder.generate_reorderings(&circuit);

    let mut ref_sim = StateVectorSimulator::new(3);
    ref_sim.run(&circuit).unwrap();

    for variant in &variants {
        let mut var_sim = StateVectorSimulator::new(3);
        var_sim.run(variant).unwrap();

        let fidelity = ref_sim.fidelity_with(&var_sim).unwrap();
        assert!(
            fidelity > 0.99,
            "Variant should produce same state, fidelity = {}",
            fidelity
        );
    }
}

#[test]
fn test_noisy_simulation_reduces_fidelity() {
    // Create reference state
    let mut ideal = StateVectorSimulator::new(2);
    ideal.prepare_bell_state().unwrap();
    let reference = ideal.statevector().to_vec();

    // Create noisy simulation with significant noise
    let noise = NoiseModel::with_t1t2(20.0, 15.0) // Short T1/T2
        .with_gate_errors(0.02, 0.10); // High gate errors

    // Run multiple trials and average fidelity
    let mut total_fidelity = 0.0;
    let trials = 50;

    for _ in 0..trials {
        let mut noisy = NoisySimulator::new(2, noise.clone());
        noisy.apply_gate(&Gate::H(0)).unwrap();
        noisy.apply_gate(&Gate::CNOT(0, 1)).unwrap();

        if let Ok(f) = noisy.fidelity(&reference) {
            total_fidelity += f;
        }
    }

    let avg_fidelity = total_fidelity / trials as f64;

    // With noise, fidelity should be noticeably less than 1
    assert!(
        avg_fidelity < 0.99,
        "Noisy simulation should reduce fidelity: {}",
        avg_fidelity
    );
    assert!(
        avg_fidelity > 0.3,
        "Fidelity shouldn't be too low: {}",
        avg_fidelity
    );
}

// ============================================================================
// API Consistency Tests
// ============================================================================

#[test]
fn test_api_consistency() {
    // All modules should follow consistent patterns

    // Scanner: scan() and scan_batch()
    let mut scanner = DriftScanner::with_defaults();
    let single = scanner.scan(0).unwrap();
    let batch = scanner.scan_batch(&[0]).unwrap();
    assert_eq!(single.qubit_id, batch[0].qubit_id);

    // Rewirer: load() then optimize()
    let mut rewirer = LiveRewirer::new();
    let mut circuit = CircuitGenome::new(2);
    circuit.add_gate(Gate::H(0)).unwrap();
    rewirer.load(circuit).unwrap();
    let noise = NoiseVector::new(0);
    let _ = rewirer.optimize(&noise, 5).unwrap();

    // Simulator: new(), apply_gate(), measure()
    let mut sim = StateVectorSimulator::new(2);
    sim.apply_gate(&Gate::H(0)).unwrap();
    let _ = sim.measure(10).unwrap();

    // System: new(), optimize()
    let mut system = QnsSystem::new();
    let mut circuit = CircuitGenome::new(2);
    circuit.add_gate(Gate::H(0)).unwrap();
    let _ = system.optimize(circuit).unwrap();
}
