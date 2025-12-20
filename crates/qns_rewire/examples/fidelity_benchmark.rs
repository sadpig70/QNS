//! Fidelity Comparison Benchmark for QNS Phase 3 Journal
//!
//! Compares circuit fidelity before and after noise-adaptive optimization.
//! Outputs JSON results suitable for publication.
//!
//! Usage:
//!   cargo run --release --example fidelity_benchmark
//!   cargo run --release --example fidelity_benchmark -- --json
//!   cargo run --release --example fidelity_benchmark -- --runs 100

use qns_core::prelude::*;
use qns_rewire::{estimate_fidelity_with_idle_tracking, LiveRewirer, RewireConfig, ScoreConfig};
use std::time::Instant;

/// Benchmark result for a single circuit
#[derive(Debug, Clone)]
struct BenchmarkResult {
    circuit_name: String,
    num_qubits: usize,
    num_gates: usize,
    original_fidelity: f64,
    optimized_fidelity: f64,
    improvement: f64,
    improvement_percent: f64,
    variants_evaluated: usize,
    optimization_time_ms: f64,
    strategy: String,
}

/// Generate standard benchmark circuits
fn generate_benchmark_circuits() -> Vec<(String, CircuitGenome)> {
    vec![
        // 1. Simple diagnostic circuit - verify reordering works
        ("diagnostic_n2".to_string(), create_diagnostic_circuit()),
        // 2. QFT-like circuit (4 qubits)
        ("qft_n4".to_string(), create_qft_circuit(4)),
        // 3. GHZ state preparation (4 qubits)
        ("ghz_n4".to_string(), create_ghz_circuit(4)),
        // 4. Random circuit with commuting gates (5 qubits)
        (
            "random_commuting_n5".to_string(),
            create_random_commuting_circuit(5, 20),
        ),
        // 5. Variational circuit (VQE-like, 4 qubits)
        ("vqe_n4".to_string(), create_vqe_circuit(4, 2)),
        // 6. Deep circuit with many layers (3 qubits)
        ("deep_n3".to_string(), create_deep_circuit(3, 10)),
    ]
}

/// Diagnostic circuit to verify reordering generates variants
/// H(0), H(1), Z(0), Z(1) - all gates on different qubits commute
fn create_diagnostic_circuit() -> CircuitGenome {
    let mut circuit = CircuitGenome::new(2);
    circuit.add_gate(Gate::H(0)).unwrap();
    circuit.add_gate(Gate::H(1)).unwrap();
    circuit.add_gate(Gate::Z(0)).unwrap();
    circuit.add_gate(Gate::Z(1)).unwrap();
    circuit
}

/// Create QFT-like circuit
fn create_qft_circuit(n: usize) -> CircuitGenome {
    let mut circuit = CircuitGenome::new(n);

    for i in 0..n {
        circuit.add_gate(Gate::H(i)).unwrap();
        for j in (i + 1)..n {
            // Controlled rotation (simplified as CZ)
            circuit.add_gate(Gate::CZ(i, j)).unwrap();
        }
    }

    // SWAP network for bit reversal
    for i in 0..(n / 2) {
        circuit.add_gate(Gate::SWAP(i, n - 1 - i)).unwrap();
    }

    circuit
}

/// Create GHZ state preparation circuit
fn create_ghz_circuit(n: usize) -> CircuitGenome {
    let mut circuit = CircuitGenome::new(n);

    circuit.add_gate(Gate::H(0)).unwrap();
    for i in 1..n {
        circuit.add_gate(Gate::CNOT(0, i)).unwrap();
    }

    circuit
}

/// Create random circuit with commuting gates
fn create_random_commuting_circuit(n: usize, gates: usize) -> CircuitGenome {
    let mut circuit = CircuitGenome::new(n);

    for i in 0..gates {
        let q = i % n;
        match i % 4 {
            0 => circuit.add_gate(Gate::H(q)).unwrap(),
            1 => circuit.add_gate(Gate::Z(q)).unwrap(),
            2 => circuit.add_gate(Gate::T(q)).unwrap(),
            _ => circuit.add_gate(Gate::S(q)).unwrap(),
        }
    }

    // Add some 2-qubit gates
    for i in 0..(n - 1) {
        circuit.add_gate(Gate::CNOT(i, i + 1)).unwrap();
    }

    circuit
}

/// Create VQE-like variational circuit
fn create_vqe_circuit(n: usize, layers: usize) -> CircuitGenome {
    let mut circuit = CircuitGenome::new(n);

    for _ in 0..layers {
        // Rotation layer
        for q in 0..n {
            circuit
                .add_gate(Gate::Ry(q, std::f64::consts::PI / 4.0))
                .unwrap();
            circuit
                .add_gate(Gate::Rz(q, std::f64::consts::PI / 3.0))
                .unwrap();
        }

        // Entanglement layer
        for q in 0..(n - 1) {
            circuit.add_gate(Gate::CNOT(q, q + 1)).unwrap();
        }
    }

    circuit
}

/// Create deep circuit with many layers
fn create_deep_circuit(n: usize, depth: usize) -> CircuitGenome {
    let mut circuit = CircuitGenome::new(n);

    for layer in 0..depth {
        for q in 0..n {
            match layer % 3 {
                0 => circuit.add_gate(Gate::H(q)).unwrap(),
                1 => circuit.add_gate(Gate::T(q)).unwrap(),
                _ => circuit.add_gate(Gate::S(q)).unwrap(),
            }
        }

        // Alternating CNOT pattern
        if layer % 2 == 0 {
            for q in (0..n - 1).step_by(2) {
                circuit.add_gate(Gate::CNOT(q, q + 1)).unwrap();
            }
        } else {
            for q in (1..n - 1).step_by(2) {
                circuit.add_gate(Gate::CNOT(q, q + 1)).unwrap();
            }
        }
    }

    circuit
}

/// Run benchmark on a single circuit
fn benchmark_circuit(
    name: &str,
    circuit: &CircuitGenome,
    noise: &NoiseVector,
    config: &RewireConfig,
) -> BenchmarkResult {
    let score_config = ScoreConfig::default();

    // Calculate original fidelity using idle-time aware scoring
    let original_fidelity = estimate_fidelity_with_idle_tracking(circuit, noise, &score_config);

    // Run optimization
    let start = Instant::now();
    let mut rewirer = LiveRewirer::with_config(config.clone());
    rewirer.load(circuit.clone()).unwrap();
    let result = rewirer.optimize(noise, config.max_variants).unwrap();
    let optimization_time = start.elapsed().as_secs_f64() * 1000.0;

    let improvement = result.improvement;
    let improvement_percent = if original_fidelity > 0.0 {
        (improvement / original_fidelity) * 100.0
    } else {
        0.0
    };

    BenchmarkResult {
        circuit_name: name.to_string(),
        num_qubits: circuit.num_qubits,
        num_gates: circuit.gates.len(),
        original_fidelity,
        optimized_fidelity: result.fidelity,
        improvement,
        improvement_percent,
        variants_evaluated: result.variants_evaluated,
        optimization_time_ms: optimization_time,
        strategy: result.strategy,
    }
}

/// Run all benchmarks with multiple noise configurations
fn run_benchmarks(num_runs: usize) -> Vec<BenchmarkResult> {
    let circuits = generate_benchmark_circuits();

    // Different noise configurations (IBM Heron-like)
    let noise_configs = vec![
        ("low_noise", NoiseVector::with_t1t2(0, 300.0, 200.0)),
        ("medium_noise", NoiseVector::with_t1t2(0, 100.0, 80.0)),
        ("high_noise", NoiseVector::with_t1t2(0, 50.0, 40.0)),
    ];

    let config = RewireConfig {
        max_variants: 100,
        max_depth: 4,
        min_fidelity_threshold: 0.1,
        hardware_aware: true,
        score_config: ScoreConfig::default(),
        beam_width: 20,
        beam_search_threshold: 50,
        parallel: true,
    };

    let mut all_results = Vec::new();

    for (noise_name, noise) in &noise_configs {
        eprintln!("\n=== Noise Configuration: {} ===", noise_name);
        eprintln!("T1={:.0}μs, T2={:.0}μs\n", noise.t1_mean, noise.t2_mean);

        for (circuit_name, circuit) in &circuits {
            let full_name = format!("{}_{}", circuit_name, noise_name);

            // Run multiple times for statistical stability
            let mut run_results = Vec::new();
            for _ in 0..num_runs {
                let result = benchmark_circuit(&full_name, circuit, noise, &config);
                run_results.push(result);
            }

            // Use median result
            run_results.sort_by(|a, b| a.improvement.partial_cmp(&b.improvement).unwrap());
            let median_result = run_results[run_results.len() / 2].clone();

            eprintln!(
                "{:30} | Q:{} G:{:2} | F: {:.4}→{:.4} | Δ:{:+.4} ({:+.2}%) | V:{:3} | {:.1}ms",
                median_result.circuit_name,
                median_result.num_qubits,
                median_result.num_gates,
                median_result.original_fidelity,
                median_result.optimized_fidelity,
                median_result.improvement,
                median_result.improvement_percent,
                median_result.variants_evaluated,
                median_result.optimization_time_ms,
            );

            all_results.push(median_result);
        }
    }

    all_results
}

/// Output results as JSON
fn output_json(results: &[BenchmarkResult]) {
    println!("{{");
    println!("  \"benchmark\": \"qns_fidelity_comparison\",");
    println!("  \"version\": \"0.1.0\",");
    println!("  \"results\": [");

    for (i, result) in results.iter().enumerate() {
        let comma = if i < results.len() - 1 { "," } else { "" };
        println!("    {{");
        println!("      \"circuit\": \"{}\",", result.circuit_name);
        println!("      \"num_qubits\": {},", result.num_qubits);
        println!("      \"num_gates\": {},", result.num_gates);
        println!(
            "      \"original_fidelity\": {:.6},",
            result.original_fidelity
        );
        println!(
            "      \"optimized_fidelity\": {:.6},",
            result.optimized_fidelity
        );
        println!("      \"improvement\": {:.6},", result.improvement);
        println!(
            "      \"improvement_percent\": {:.2},",
            result.improvement_percent
        );
        println!(
            "      \"variants_evaluated\": {},",
            result.variants_evaluated
        );
        println!(
            "      \"optimization_time_ms\": {:.2},",
            result.optimization_time_ms
        );
        println!("      \"strategy\": \"{}\"", result.strategy);
        println!("    }}{}", comma);
    }

    println!("  ]");
    println!("}}");
}

/// Output results as table
fn output_table(results: &[BenchmarkResult]) {
    println!("\n{}", "=".repeat(120));
    println!("{:^120}", "QNS Fidelity Comparison Benchmark Results");
    println!("{}", "=".repeat(120));

    println!(
        "{:30} | {:>6} | {:>5} | {:>10} | {:>10} | {:>10} | {:>8} | {:>8}",
        "Circuit", "Qubits", "Gates", "Original", "Optimized", "Improve", "Δ%", "Time(ms)"
    );
    println!("{}", "-".repeat(120));

    for result in results {
        println!(
            "{:30} | {:>6} | {:>5} | {:>10.4} | {:>10.4} | {:>+10.4} | {:>+7.2}% | {:>8.1}",
            result.circuit_name,
            result.num_qubits,
            result.num_gates,
            result.original_fidelity,
            result.optimized_fidelity,
            result.improvement,
            result.improvement_percent,
            result.optimization_time_ms,
        );
    }

    println!("{}", "=".repeat(120));

    // Summary statistics
    let improvements: Vec<f64> = results.iter().map(|r| r.improvement).collect();
    let percent_improvements: Vec<f64> = results.iter().map(|r| r.improvement_percent).collect();

    let avg_improvement = improvements.iter().sum::<f64>() / improvements.len() as f64;
    let avg_percent = percent_improvements.iter().sum::<f64>() / percent_improvements.len() as f64;
    let max_improvement = improvements
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let improved_count = results.iter().filter(|r| r.improvement > 1e-6).count();

    println!("\nSummary:");
    println!("  - Total circuits tested: {}", results.len());
    println!(
        "  - Circuits improved: {} ({:.1}%)",
        improved_count,
        100.0 * improved_count as f64 / results.len() as f64
    );
    println!("  - Average fidelity improvement: {:.4}", avg_improvement);
    println!("  - Average improvement percentage: {:.2}%", avg_percent);
    println!("  - Maximum improvement: {:.4}", max_improvement);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let json_output = args.iter().any(|a| a == "--json");
    let num_runs = args
        .iter()
        .position(|a| a == "--runs")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);

    if !json_output {
        eprintln!(
            "Running fidelity benchmark ({} runs per circuit)...",
            num_runs
        );
    }

    let results = run_benchmarks(num_runs);

    if json_output {
        output_json(&results);
    } else {
        output_table(&results);
    }
}
