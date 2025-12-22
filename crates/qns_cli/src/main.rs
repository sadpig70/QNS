//! QNS CLI - Quantum Noise Symbiote Command Line Interface
//!
//! Provides commands for:
//! - Running quantum circuits through the optimization pipeline
//! - Benchmarking performance
//! - Profiling noise characteristics

use std::path::PathBuf;
use std::time::Instant;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

use qns_cli::pipeline::QnsSystem;
use qns_core::prelude::*;
use qns_qasm::{parse_qasm, resolve_includes};

/// QNS - Quantum Noise Symbiote
///
/// A noise-aware quantum circuit optimizer for NISQ devices.
#[derive(Parser)]
#[command(name = "qns")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Output format (text, json)
    #[arg(short, long, global = true, default_value = "text")]
    format: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a quantum circuit through the optimization pipeline
    Run {
        /// Path to QASM file
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Hardware topology (linear, grid, all-to-all)
        #[arg(short, long, default_value = "linear")]
        topology: String,

        /// Number of simulation shots
        #[arg(short, long, default_value = "1000")]
        shots: usize,

        /// Backend type (simulator, aer-ideal, aer-noisy, aer-ibm)
        #[arg(short, long, default_value = "simulator")]
        backend: String,

        /// IBM backend name (for aer-ibm backend only)
        #[arg(long)]
        ibm_backend: Option<String>,

        /// Skip optimization (just parse and simulate)
        #[arg(long)]
        no_optimize: bool,

        /// Weight for crosstalk-aware routing (0.0 = disabled, >0.0 = enabled)
        #[arg(long, default_value = "0.0")]
        crosstalk_weight: f64,
    },

    /// Benchmark the QNS pipeline
    Benchmark {
        /// Number of qubits
        #[arg(short = 'q', long, default_value = "4")]
        qubits: usize,

        /// Number of gates
        #[arg(short = 'g', long, default_value = "20")]
        gates: usize,

        /// Number of iterations
        #[arg(short, long, default_value = "10")]
        iterations: usize,
    },

    /// Profile noise characteristics
    Profile {
        /// Number of qubits to profile
        #[arg(short = 'q', long, default_value = "4")]
        qubits: usize,
    },

    /// Show system information
    Info,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let level = if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };
    let _ = FmtSubscriber::builder()
        .with_max_level(level)
        .with_target(false)
        .without_time()
        .try_init();

    match cli.command {
        Commands::Run {
            input,
            topology,
            shots,
            backend,
            ibm_backend,
            no_optimize,
            crosstalk_weight,
        } => cmd_run(
            &input,
            &topology,
            shots,
            &backend,
            ibm_backend.as_deref(),
            no_optimize,
            cli.format,
            crosstalk_weight,
        ),
        Commands::Benchmark {
            qubits,
            gates,
            iterations,
        } => cmd_benchmark(qubits, gates, iterations, cli.format),
        Commands::Profile { qubits } => cmd_profile(qubits, cli.format),
        Commands::Info => cmd_info(cli.format),
    }
}

/// Run a QASM circuit through the pipeline
#[allow(clippy::too_many_arguments)]
fn cmd_run(
    input: &PathBuf,
    topology: &str,
    shots: usize,
    backend: &str,
    ibm_backend: Option<&str>,
    no_optimize: bool,
    format: OutputFormat,
    crosstalk_weight: f64,
) -> Result<()> {
    let start = Instant::now();

    // Handle Qiskit backends
    if backend != "simulator" {
        return cmd_run_qiskit(input, backend, ibm_backend, shots, format);
    }

    // Read and parse QASM file
    let qasm_content = std::fs::read_to_string(input)
        .with_context(|| format!("Failed to read QASM file: {}", input.display()))?;

    // Resolve includes relative to the input file's directory
    let base_path = input.parent().unwrap_or(std::path::Path::new("."));
    let resolved_content =
        resolve_includes(&qasm_content, base_path).with_context(|| "Failed to resolve includes")?;

    let circuit = parse_qasm(&resolved_content).with_context(|| "Failed to parse QASM")?;

    let original_gates = circuit.gates.len();
    let num_qubits = circuit.num_qubits;

    info!(
        "Parsed circuit: {} qubits, {} gates",
        num_qubits, original_gates
    );

    // Create hardware profile based on topology
    let hardware = match topology {
        "linear" => HardwareProfile::linear("qns-linear", num_qubits),
        "grid" => {
            let side = (num_qubits as f64).sqrt().ceil() as usize;
            HardwareProfile::grid("qns-grid", side, side)
        },
        "all-to-all" | "full" => HardwareProfile::all_to_all("qns-full", num_qubits),
        _ => {
            warn!("Unknown topology '{}', using linear", topology);
            HardwareProfile::linear("qns-linear", num_qubits)
        },
    };

    // Create QNS system
    let mut config = qns_cli::pipeline::PipelineConfig::default();

    // Configure crosstalk awareness
    if crosstalk_weight > 0.0 {
        config.rewirer.crosstalk_weight = crosstalk_weight;
        config.rewirer.use_sabre = true;
        config.rewirer.hardware_aware = true;
        info!(
            "Crosstalk-aware routing enabled (weight: {})",
            crosstalk_weight
        );
    }

    let mut system = QnsSystem::with_config(config);
    system.set_hardware(hardware);

    let result = if no_optimize {
        // Just simulate without optimization
        RunResult {
            input_file: input.display().to_string(),
            num_qubits,
            original_gates,
            routed_gates: original_gates,
            swap_count: 0,
            circuit_depth: circuit.depth(),
            fidelity_before: 1.0,
            fidelity_after: 1.0,
            improvement_percent: 0.0,
            total_time_ms: start.elapsed().as_secs_f64() * 1000.0,
            topology: topology.to_string(),
        }
    } else {
        // Full optimization pipeline
        let pipeline_result = system
            .optimize(circuit.clone())
            .with_context(|| "Optimization failed")?;

        let routed_gates = pipeline_result.optimized_circuit.gates.len();
        let swap_count = if routed_gates > original_gates {
            (routed_gates - original_gates) / 3 // Approximate SWAP count
        } else {
            0
        };

        RunResult {
            input_file: input.display().to_string(),
            num_qubits,
            original_gates,
            routed_gates,
            swap_count,
            circuit_depth: pipeline_result.optimized_circuit.depth(),
            fidelity_before: pipeline_result.original_fidelity,
            fidelity_after: pipeline_result.optimized_fidelity,
            improvement_percent: pipeline_result.fidelity_improvement * 100.0,
            total_time_ms: start.elapsed().as_secs_f64() * 1000.0,
            topology: topology.to_string(),
        }
    };

    // Output result
    match format {
        OutputFormat::Text => {
            println!("\n=== QNS Run Result ===");
            println!("Input:      {}", result.input_file);
            println!("Topology:   {}", result.topology);
            println!("Qubits:     {}", result.num_qubits);
            println!();
            println!("Parsed:     {} gates", result.original_gates);
            println!("Routed:     {} gates", result.routed_gates);
            println!("SWAPs:      {}", result.swap_count);
            println!("Depth:      {}", result.circuit_depth);
            println!();
            println!("Fidelity Before: {:.4}", result.fidelity_before);
            println!("Fidelity After:  {:.4}", result.fidelity_after);
            println!("Improvement:     {:.2}%", result.improvement_percent);
            println!();
            println!("Time:       {:.2} ms", result.total_time_ms);
        },
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        },
    }

    Ok(())
}

/// Run benchmark
fn cmd_benchmark(
    qubits: usize,
    gates: usize,
    iterations: usize,
    format: OutputFormat,
) -> Result<()> {
    info!(
        "Running benchmark: {} qubits, {} gates, {} iterations",
        qubits, gates, iterations
    );

    let mut system = QnsSystem::new();
    let result = system.benchmark(qubits, gates, iterations);

    match format {
        OutputFormat::Text => {
            println!("{}", result);
        },
        OutputFormat::Json => {
            let json_result = BenchmarkJsonResult {
                iterations,
                num_qubits: qubits,
                num_gates: gates,
                total_time_ms: result.total_time.as_secs_f64() * 1000.0,
                avg_total_ms: result.avg_total.as_secs_f64() * 1000.0,
                avg_profile_ms: result.avg_profile.as_secs_f64() * 1000.0,
                avg_optimize_ms: result.avg_optimize.as_secs_f64() * 1000.0,
                avg_simulate_ms: result.avg_simulate.as_secs_f64() * 1000.0,
            };
            println!("{}", serde_json::to_string_pretty(&json_result)?);
        },
    }

    Ok(())
}

/// Profile noise characteristics
fn cmd_profile(qubits: usize, format: OutputFormat) -> Result<()> {
    info!("Profiling noise for {} qubits", qubits);

    let mut system = QnsSystem::new();
    let qubit_ids: Vec<usize> = (0..qubits).collect();
    let profiles = system.profile_noise(&qubit_ids)?;

    match format {
        OutputFormat::Text => {
            println!("\n=== Noise Profile ===");
            for (i, profile) in profiles.iter().enumerate() {
                println!("Qubit {}:", i);
                println!("  T1: {:.2} us", profile.t1_mean);
                println!("  T2: {:.2} us", profile.t2_mean);
                println!("  Gate Error 1Q: {:.4}", profile.gate_error_1q);
                println!("  Gate Error 2Q: {:.4}", profile.gate_error_2q);
                println!("  Readout Error: {:.4}", profile.readout_error);
                println!();
            }
        },
        OutputFormat::Json => {
            let json_profiles: Vec<NoiseProfileJson> = profiles
                .iter()
                .enumerate()
                .map(|(i, p)| NoiseProfileJson {
                    qubit_id: i,
                    t1_us: p.t1_mean,
                    t2_us: p.t2_mean,
                    gate_error_1q: p.gate_error_1q,
                    gate_error_2q: p.gate_error_2q,
                    readout_error: p.readout_error,
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json_profiles)?);
        },
    }

    Ok(())
}

/// Show system information
fn cmd_info(format: OutputFormat) -> Result<()> {
    let info = SystemInfo {
        name: "QNS (Quantum Noise Symbiote)".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        rust_version: env!("CARGO_PKG_RUST_VERSION").to_string(),
        features: vec![
            "Noise-aware optimization".to_string(),
            "Hardware-aware routing".to_string(),
            "Gate reordering".to_string(),
            "StateVector simulation".to_string(),
            "Noisy simulation".to_string(),
        ],
        supported_gates: vec![
            "H", "X", "Y", "Z", "S", "T", "Rx", "Ry", "Rz", "CNOT", "CZ", "SWAP", "Measure",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect(),
    };

    match format {
        OutputFormat::Text => {
            println!("\n=== {} ===", info.name);
            println!("Version: {}", info.version);
            println!("Rust:    {}", info.rust_version);
            println!();
            println!("Features:");
            for feature in &info.features {
                println!("  - {}", feature);
            }
            println!();
            println!("Supported Gates:");
            println!("  {}", info.supported_gates.join(", "));
        },
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&info)?);
        },
    }

    Ok(())
}

// JSON output structures

#[derive(serde::Serialize)]
struct RunResult {
    input_file: String,
    num_qubits: usize,
    original_gates: usize,
    routed_gates: usize,
    swap_count: usize,
    circuit_depth: usize,
    fidelity_before: f64,
    fidelity_after: f64,
    improvement_percent: f64,
    total_time_ms: f64,
    topology: String,
}

#[derive(serde::Serialize)]
struct BenchmarkJsonResult {
    iterations: usize,
    num_qubits: usize,
    num_gates: usize,
    total_time_ms: f64,
    avg_total_ms: f64,
    avg_profile_ms: f64,
    avg_optimize_ms: f64,
    avg_simulate_ms: f64,
}

#[derive(serde::Serialize)]
struct NoiseProfileJson {
    qubit_id: usize,
    t1_us: f64,
    t2_us: f64,
    gate_error_1q: f64,
    gate_error_2q: f64,
    readout_error: f64,
}

#[derive(serde::Serialize)]
struct SystemInfo {
    name: String,
    version: String,
    rust_version: String,
    features: Vec<String>,
    supported_gates: Vec<String>,
}

/// Run circuit using Qiskit backend
fn cmd_run_qiskit(
    input: &std::path::Path,
    backend: &str,
    ibm_backend: Option<&str>,
    shots: usize,
    format: OutputFormat,
) -> Result<()> {
    use std::process::Command;

    info!("Running with Qiskit backend: {}", backend);

    // Build Python script path
    let qns_python_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("qns_python")
        .join("python");

    // Prepare arguments for Python runner
    let mut args = vec![
        "--input".to_string(),
        input.display().to_string(),
        "--backend".to_string(),
        backend.to_string(),
        "--shots".to_string(),
        shots.to_string(),
        "--format".to_string(),
        match format {
            OutputFormat::Text => "text",
            OutputFormat::Json => "json",
        }
        .to_string(),
    ];

    if let Some(ibm) = ibm_backend {
        args.push("--ibm-backend".to_string());
        args.push(ibm.to_string());
    }

    // Call Python runner script
    let output = Command::new("python")
        .arg(qns_python_path.join("cli_runner.py"))
        .args(&args)
        .output()
        .with_context(|| "Failed to execute Python runner")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Qiskit runner failed:\n{}", stderr);
    }

    // Display output
    let stdout = String::from_utf8_lossy(&output.stdout);
    print!("{}", stdout);

    Ok(())
}
