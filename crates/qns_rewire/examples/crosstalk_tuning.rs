//! Hyper-parameter tuning for Crosstalk-Aware Routing
//!
//! This example sweeps the `crosstalk_weight` parameter to find the optimal
//! configuration for a given noise model.

use qns_core::prelude::*;
use qns_rewire::{estimate_fidelity_with_hardware, LiveRewirer, RewireConfig, ScoreConfig};

fn main() {
    println!("=== QNS Crosstalk Tuning ===");
    println!("Sweeping crosstalk_weight [0.0 - 1.0]");
    println!("--------------------------------------------------");

    // 1. Setup Mock Hardware with high crosstalk
    // Linear topology: 0-1-2-3-4
    // But we'll inject strong crosstalk between (0,1) and (3,4) if they run parallel
    let num_qubits = 5;
    let mut hardware = HardwareProfile::linear("tuning-backend", num_qubits);

    // Inject heavy crosstalk between edges (0,1) and (1,2) to force spacing?
    // Actually, let's try to simulate a scenario where spacing out operations helps.
    // Let's say (0,1) and (2,3) have high crosstalk.
    hardware.crosstalk.set_interaction(0, 2, 0.1); // Strong ZZ between 0 and 2
    hardware.crosstalk.set_interaction(1, 3, 0.1); // Strong ZZ between 1 and 3

    // Also basic gate errors
    for i in 0..num_qubits {
        if i + 1 < num_qubits {
            let fid = if i == 1 { 0.95 } else { 0.99 }; // Edge 1-2 is bad (95%)

            // hardware.set_gate_fidelity(i, i+1, fid); // No such method, manual update:
            for coupler in &mut hardware.couplers {
                if (coupler.qubit1 == i && coupler.qubit2 == i + 1)
                    || (coupler.qubit1 == i + 1 && coupler.qubit2 == i)
                {
                    coupler.gate_fidelity = qns_core::types::Fidelity::new(fid);
                }
            }
        }
    }

    // Noise Vector for Simulation (simulating uniform noise for tuning)
    // T1=100us, T2=80us, 1Q err=0.1%, 2Q err=1.0%, Readout=2.0%
    let noise = NoiseVector::comprehensive(0, 100.0, 80.0, 0.001, 0.01, 0.02);

    // 2. Define Weights to Sweep
    let weights = vec![0.0, 0.25, 0.5, 0.75, 1.0, 1.5, 2.0];

    // 3. Define Benchmark Circuits
    let circuits = vec![
        ("GHZ_5", create_ghz(5)),
        ("QFT_4", create_qft(4)),
        ("Bernstein_5", create_bv(5)),
    ];

    println!(
        "{:<20} | {:<8} | {:<10} | {:<10} | {:<15}",
        "Circuit", "Weight", "Fidelity", "Imprv %", "SWAPs"
    );
    println!("-------------------------------------------------------------------------");

    for (name, circuit) in circuits {
        let base_fidelity =
            estimate_fidelity_with_hardware(&circuit, &noise, &hardware, &ScoreConfig::default());

        for &w in &weights {
            let config = RewireConfig {
                crosstalk_weight: w,
                use_sabre: true,
                hardware_aware: true,
                ..RewireConfig::default()
            };

            let mut rewirer = LiveRewirer::with_config(config);
            rewirer.set_hardware(hardware.clone());
            rewirer.load(circuit.clone()).unwrap();

            let result = rewirer.optimize(&noise, 20).unwrap();

            let fid_imprv = (result.fidelity - base_fidelity) / base_fidelity * 100.0;
            let swaps = count_swaps(&result.circuit);

            println!(
                "{:<20} | {:<8.2} | {:<10.4} | {:<+10.2}% | {:<15}",
                name, w, result.fidelity, fid_imprv, swaps
            );
        }
        println!("-");
    }
}

fn count_swaps(circuit: &CircuitGenome) -> usize {
    circuit
        .gates
        .iter()
        .filter(|g| matches!(g, Gate::SWAP(_, _)))
        .count()
}

fn create_ghz(n: usize) -> CircuitGenome {
    let mut c = CircuitGenome::new(n);
    c.add_gate(Gate::H(0)).unwrap();
    for i in 0..n - 1 {
        c.add_gate(Gate::CNOT(i, i + 1)).unwrap();
    }
    c
}

fn create_qft(n: usize) -> CircuitGenome {
    let mut c = CircuitGenome::new(n);
    // Simplified QFT-like structure (just H and CNOTs for entanglement/swapping)
    for i in 0..n {
        c.add_gate(Gate::H(i)).unwrap();
        for j in i + 1..n {
            c.add_gate(Gate::CNOT(j, i)).unwrap();
        }
    }
    c
}

fn create_bv(n: usize) -> CircuitGenome {
    let mut c = CircuitGenome::new(n);
    // Bernstein-Vazirani
    for i in 0..n {
        c.add_gate(Gate::H(i)).unwrap();
    }
    // Oracle (all 1s)
    for i in 0..n - 1 {
        c.add_gate(Gate::CNOT(i, n - 1)).unwrap();
    }
    for i in 0..n {
        c.add_gate(Gate::H(i)).unwrap();
    }
    c
}
