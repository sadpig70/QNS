//! QNS System - Integrated pipeline for quantum circuit optimization.
//!
//! This module provides the main integration point for all QNS components:
//! - Noise profiling via DriftScanner
//! - Circuit optimization via LiveRewirer
//! - Simulation and verification via StateVectorSimulator
//!
//! ## Pipeline Overview
//!
//! ```text
//! Circuit Input
//!       │
//!       ▼
//! ┌─────────────┐
//! │ DriftScanner │ ─── Noise Profile
//! └─────────────┘
//!       │
//!       ▼
//! ┌─────────────┐
//! │ LiveRewirer  │ ─── Optimized Circuit
//! └─────────────┘
//!       │
//!       ▼
//! ┌─────────────┐
//! │  Simulator   │ ─── Verification
//! └─────────────┘
//!       │
//!       ▼
//! Optimization Result
//! ```

use qns_core::prelude::*;
use qns_profiler::{DriftScanner, ScanConfig};
use qns_rewire::{LiveRewirer, OptimizationResult, RewireConfig as LiveRewireConfig};
use qns_simulator::StateVectorSimulator;
use std::time::{Duration, Instant};

/// Configuration for the QNS pipeline.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Scanner configuration
    pub scanner: ScanConfig,
    /// Rewirer configuration
    pub rewirer: LiveRewireConfig,
    /// Number of shots for simulation verification
    pub simulation_shots: usize,
    /// Enable detailed logging
    pub verbose: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            scanner: ScanConfig::default(),
            rewirer: LiveRewireConfig::default(),
            simulation_shots: 1000,
            verbose: false,
        }
    }
}

/// Result of the full QNS pipeline.
#[derive(Debug, Clone)]
pub struct PipelineResult {
    /// Original circuit
    pub original_circuit: CircuitGenome,
    /// Optimized circuit
    pub optimized_circuit: CircuitGenome,
    /// Noise profile used for optimization
    pub noise_profile: NoiseVector,
    /// Optimization details
    pub optimization: OptimizationResult,
    /// Original circuit fidelity estimate
    pub original_fidelity: f64,
    /// Optimized circuit fidelity estimate
    pub optimized_fidelity: f64,
    /// Fidelity improvement
    pub fidelity_improvement: f64,
    /// Total pipeline execution time
    pub total_time: Duration,
    /// Time breakdown
    pub timing: PipelineTiming,
}

/// Timing breakdown for pipeline stages.
#[derive(Debug, Clone, Default)]
pub struct PipelineTiming {
    /// Time spent in noise profiling
    pub profiling_time: Duration,
    /// Time spent in circuit optimization
    pub optimization_time: Duration,
    /// Time spent in simulation
    pub simulation_time: Duration,
}

/// QNS System - Main integration point.
///
/// Provides a unified interface for the complete QNS pipeline.
///
/// # Example
///
/// ```rust
/// use qns_cli::pipeline::{QnsSystem, PipelineConfig};
/// use qns_core::prelude::*;
///
/// // Create a circuit
/// let mut circuit = CircuitGenome::new(2);
/// circuit.add_gate(Gate::H(0)).unwrap();
/// circuit.add_gate(Gate::X(1)).unwrap();
/// circuit.add_gate(Gate::CNOT(0, 1)).unwrap();
///
/// // Run optimization pipeline
/// let mut system = QnsSystem::new();
/// let result = system.optimize(circuit).unwrap();
///
/// println!("Fidelity improvement: {:.2}%", result.fidelity_improvement * 100.0);
/// ```
pub struct QnsSystem {
    /// Configuration
    config: PipelineConfig,
    /// Hardware profile (optional)
    hardware: Option<HardwareProfile>,
    /// Drift scanner
    scanner: DriftScanner,
    /// Live rewirer
    rewirer: LiveRewirer,
}

impl QnsSystem {
    /// Creates a new QNS system with default configuration.
    pub fn new() -> Self {
        let config = PipelineConfig::default();
        Self {
            scanner: DriftScanner::new(config.scanner.clone()),
            rewirer: LiveRewirer::new(),
            config,
            hardware: None,
        }
    }

    /// Creates a QNS system with custom configuration.
    pub fn with_config(config: PipelineConfig) -> Self {
        Self {
            scanner: DriftScanner::new(config.scanner.clone()),
            rewirer: LiveRewirer::with_config(config.rewirer.clone()),
            config,
            hardware: None,
        }
    }

    /// Sets the hardware profile for hardware-aware optimization.
    pub fn set_hardware(&mut self, hardware: HardwareProfile) {
        self.hardware = Some(hardware.clone());
        self.rewirer.set_hardware(hardware);
    }

    /// Returns the current configuration.
    pub fn config(&self) -> &PipelineConfig {
        &self.config
    }

    /// Profiles noise for the given qubit IDs.
    pub fn profile_noise(&mut self, qubit_ids: &[usize]) -> Result<Vec<NoiseVector>> {
        self.scanner.scan_batch(qubit_ids)
    }

    /// Optimizes a circuit using the full pipeline.
    pub fn optimize(&mut self, circuit: CircuitGenome) -> Result<PipelineResult> {
        let start = Instant::now();
        let mut timing = PipelineTiming::default();

        let original_circuit = circuit.clone();

        // Step 1: Profile noise
        let profile_start = Instant::now();
        let qubit_ids: Vec<usize> = (0..circuit.num_qubits).collect();
        let scan_results = self.scanner.scan_batch(&qubit_ids)?;

        // Aggregate noise profile
        let noise_profile = self.aggregate_noise(&scan_results);
        timing.profiling_time = profile_start.elapsed();

        // Step 2: Optimize circuit
        let opt_start = Instant::now();
        self.rewirer.load(circuit)?;
        let optimization = self
            .rewirer
            .optimize(&noise_profile, self.config.rewirer.max_variants)?;
        timing.optimization_time = opt_start.elapsed();

        // Step 3: Verify with simulation
        let sim_start = Instant::now();
        let (original_fidelity, optimized_fidelity) =
            self.verify_optimization(&original_circuit, &optimization.circuit)?;
        timing.simulation_time = sim_start.elapsed();

        let fidelity_improvement = optimized_fidelity - original_fidelity;

        Ok(PipelineResult {
            original_circuit,
            optimized_circuit: optimization.circuit.clone(),
            noise_profile,
            optimization,
            original_fidelity,
            optimized_fidelity,
            fidelity_improvement,
            total_time: start.elapsed(),
            timing,
        })
    }

    /// Quick optimization without simulation verification.
    pub fn quick_optimize(&mut self, circuit: CircuitGenome) -> Result<CircuitGenome> {
        // Profile noise
        let qubit_ids: Vec<usize> = (0..circuit.num_qubits).collect();
        let scan_results = self.scanner.scan_batch(&qubit_ids)?;
        let noise_profile = self.aggregate_noise(&scan_results);

        // Optimize
        self.rewirer.load(circuit)?;
        let result = self
            .rewirer
            .optimize(&noise_profile, self.config.rewirer.max_variants)?;
        Ok(result.circuit)
    }

    /// Aggregates noise from multiple scan results.
    fn aggregate_noise(&self, results: &[NoiseVector]) -> NoiseVector {
        if results.is_empty() {
            return NoiseVector::new(0);
        }

        let mut noise = NoiseVector::new(0);
        let mut t1_sum = 0.0;
        let mut t2_sum = 0.0;
        let mut count = 0;

        for result in results {
            t1_sum += result.t1_mean;
            t2_sum += result.t2_mean;
            count += 1;
        }

        if count > 0 {
            noise.t1_mean = t1_sum / count as f64;
            noise.t2_mean = t2_sum / count as f64;
        }

        noise
    }

    /// Verifies optimization by comparing fidelity estimates.
    fn verify_optimization(
        &self,
        original: &CircuitGenome,
        optimized: &CircuitGenome,
    ) -> Result<(f64, f64)> {
        // Create ideal reference state (run on noise-free simulator)
        let mut ref_sim = StateVectorSimulator::new(original.num_qubits);
        ref_sim.run(original)?;
        let reference_state = ref_sim.statevector().to_vec();

        // Evaluate original circuit
        let mut orig_sim = StateVectorSimulator::new(original.num_qubits);
        orig_sim.run(original)?;
        let orig_fidelity = orig_sim.fidelity(&reference_state)?;

        // Evaluate optimized circuit
        let mut opt_sim = StateVectorSimulator::new(optimized.num_qubits);
        opt_sim.run(optimized)?;
        let opt_fidelity = opt_sim.fidelity(&reference_state)?;

        Ok((orig_fidelity, opt_fidelity))
    }

    /// Runs a benchmark of the pipeline.
    pub fn benchmark(
        &mut self,
        num_qubits: usize,
        num_gates: usize,
        iterations: usize,
    ) -> BenchmarkResult {
        let mut profile_time = Duration::ZERO;
        let mut optimize_time = Duration::ZERO;
        let mut simulate_time = Duration::ZERO;

        for _ in 0..iterations {
            // Create test circuit
            let circuit = Self::create_test_circuit(num_qubits, num_gates);

            // Profile
            let start = Instant::now();
            let qubit_ids: Vec<usize> = (0..num_qubits).collect();
            let scan_results = self.scanner.scan_batch(&qubit_ids).unwrap_or_default();
            let noise = self.aggregate_noise(&scan_results);
            profile_time += start.elapsed();

            // Optimize
            let start = Instant::now();
            self.rewirer.load(circuit.clone()).unwrap();
            let _ = self.rewirer.optimize(&noise, 10);
            optimize_time += start.elapsed();

            // Simulate
            let start = Instant::now();
            let mut sim = StateVectorSimulator::new(num_qubits);
            let _ = sim.run(&circuit);
            let _ = sim.measure(100);
            simulate_time += start.elapsed();
        }

        let total_time = profile_time + optimize_time + simulate_time;

        BenchmarkResult {
            iterations,
            num_qubits,
            num_gates,
            total_time,
            avg_total: total_time / iterations as u32,
            avg_profile: profile_time / iterations as u32,
            avg_optimize: optimize_time / iterations as u32,
            avg_simulate: simulate_time / iterations as u32,
        }
    }

    /// Creates a test circuit for benchmarking.
    fn create_test_circuit(num_qubits: usize, num_gates: usize) -> CircuitGenome {
        let mut circuit = CircuitGenome::new(num_qubits);

        for i in 0..num_gates {
            match i % 6 {
                0 => circuit.add_gate(Gate::H(i % num_qubits)).unwrap(),
                1 => circuit.add_gate(Gate::X(i % num_qubits)).unwrap(),
                2 => circuit.add_gate(Gate::T(i % num_qubits)).unwrap(),
                3 => circuit.add_gate(Gate::S(i % num_qubits)).unwrap(),
                4 if num_qubits > 1 => circuit
                    .add_gate(Gate::CNOT(i % num_qubits, (i + 1) % num_qubits))
                    .unwrap(),
                _ => circuit.add_gate(Gate::Z(i % num_qubits)).unwrap(),
            }
        }

        circuit
    }
}

impl Default for QnsSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a benchmark run.
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Number of iterations
    pub iterations: usize,
    /// Number of qubits
    pub num_qubits: usize,
    /// Number of gates
    pub num_gates: usize,
    /// Total time for all iterations
    pub total_time: Duration,
    /// Average total time per iteration
    pub avg_total: Duration,
    /// Average profiling time
    pub avg_profile: Duration,
    /// Average optimization time
    pub avg_optimize: Duration,
    /// Average simulation time
    pub avg_simulate: Duration,
}

impl std::fmt::Display for BenchmarkResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "QNS Benchmark Results")?;
        writeln!(f, "=====================")?;
        writeln!(f, "Configuration:")?;
        writeln!(f, "  Qubits: {}", self.num_qubits)?;
        writeln!(f, "  Gates: {}", self.num_gates)?;
        writeln!(f, "  Iterations: {}", self.iterations)?;
        writeln!(f)?;
        writeln!(f, "Timing:")?;
        writeln!(f, "  Avg Total:    {:>10.2?}", self.avg_total)?;
        writeln!(f, "  Avg Profile:  {:>10.2?}", self.avg_profile)?;
        writeln!(f, "  Avg Optimize: {:>10.2?}", self.avg_optimize)?;
        writeln!(f, "  Avg Simulate: {:>10.2?}", self.avg_simulate)?;
        writeln!(f)?;
        writeln!(f, "Total Time: {:?}", self.total_time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qns_system_new() {
        let system = QnsSystem::new();
        assert!(system.hardware.is_none());
    }

    #[test]
    fn test_quick_optimize() {
        let mut system = QnsSystem::new();

        let mut circuit = CircuitGenome::new(2);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::X(1)).unwrap();
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();

        let result = system.quick_optimize(circuit);
        assert!(result.is_ok());

        let optimized = result.unwrap();
        assert_eq!(optimized.num_qubits, 2);
    }

    #[test]
    fn test_full_pipeline() {
        let mut system = QnsSystem::new();

        let mut circuit = CircuitGenome::new(2);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::X(1)).unwrap();
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();

        let result = system.optimize(circuit);
        assert!(result.is_ok());

        let pipeline_result = result.unwrap();
        assert!(pipeline_result.original_fidelity >= 0.0);
        assert!(pipeline_result.optimized_fidelity >= 0.0);
    }

    #[test]
    fn test_profile_noise() {
        let mut system = QnsSystem::new();
        let results = system.profile_noise(&[0, 1, 2]).unwrap();

        assert_eq!(results.len(), 3);
        for result in &results {
            assert!(result.t1_mean > 0.0);
            assert!(result.t2_mean > 0.0);
        }
    }

    #[test]
    fn test_with_hardware() {
        let mut system = QnsSystem::new();
        let hw = HardwareProfile::linear("test", 5);
        system.set_hardware(hw);

        assert!(system.hardware.is_some());
    }

    #[test]
    fn test_benchmark() {
        let mut system = QnsSystem::new();
        let result = system.benchmark(3, 10, 5);

        assert_eq!(result.iterations, 5);
        assert_eq!(result.num_qubits, 3);
        assert!(result.avg_total > Duration::ZERO);
    }

    #[test]
    fn test_create_test_circuit() {
        let circuit = QnsSystem::create_test_circuit(3, 15);

        assert_eq!(circuit.num_qubits, 3);
        assert_eq!(circuit.gates.len(), 15);
    }
}

// ==================== End-to-End Integration Tests ====================
// NOTE: These tests are temporarily disabled until SimulatorBackend API is fully implemented.
// The tests require methods like `ideal()`, `with_noise()`, `get_qubit()` etc.

#[cfg(test)]
#[cfg(any())] // Disabled until SimulatorBackend API is fully implemented
mod e2e_tests {
    use super::*;
    use qns_rewire::{score_circuit_variant, BeamSearchConfig, GateReorder};
    use qns_simulator::{NoiseModel, SimulatorBackend};

    /// E2E Test: Full pipeline with SimulatorBackend
    ///
    /// Tests the complete flow:
    /// Circuit → DriftScanner → LiveRewirer → SimulatorBackend → Result
    #[test]
    fn test_e2e_full_pipeline_with_backend() {
        // 1. Create a test circuit (GHZ state)
        let mut circuit = CircuitGenome::new(3);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();
        circuit.add_gate(Gate::CNOT(1, 2)).unwrap();

        // 2. Create SimulatorBackend
        let noise = NoiseModel::with_t1t2(100.0, 80.0);
        let backend = SimulatorBackend::with_noise(3, noise);

        // 3. Get calibration data from backend
        let calibration = backend.get_calibration().unwrap();
        assert_eq!(calibration.num_qubits, 3);

        // 4. Create NoiseVector from calibration
        let noise_vec = calibration.get_qubit(0).unwrap().clone();
        assert!(noise_vec.t1_mean > 0.0);

        // 5. Optimize circuit using LiveRewirer
        let mut rewirer = LiveRewirer::new();
        rewirer.load(circuit.clone()).unwrap();
        let optimized = rewirer.optimize(&noise_vec, 50).unwrap();

        // 6. Execute both circuits on backend
        let original_result = backend
            .execute(&circuit, 1000, qns_core::backend::ExecutionMode::Sync)
            .unwrap();

        let optimized_result = backend
            .execute(&optimized, 1000, qns_core::backend::ExecutionMode::Sync)
            .unwrap();

        // 7. Verify results
        assert_eq!(original_result.shots, 1000);
        assert_eq!(optimized_result.shots, 1000);
        assert!(!original_result.counts.is_empty());
        assert!(!optimized_result.counts.is_empty());

        // Both should produce GHZ-like distribution (|000⟩ and |111⟩ dominant)
        let p000_orig = original_result.probability("000");
        let p111_orig = original_result.probability("111");
        assert!(
            p000_orig + p111_orig > 0.7,
            "GHZ state should have |000⟩+|111⟩ > 70%, got {:.1}%",
            (p000_orig + p111_orig) * 100.0
        );
    }

    /// E2E Test: Beam Search optimization with scoring
    #[test]
    fn test_e2e_beam_search_optimization() {
        // Create a circuit with commutable gates
        let mut circuit = CircuitGenome::new(5);
        for i in 0..5 {
            circuit.add_gate(Gate::H(i)).unwrap();
        }
        for i in 0..4 {
            circuit.add_gate(Gate::CNOT(i, i + 1)).unwrap();
        }

        // Get noise profile
        let backend = SimulatorBackend::new(5);
        let calibration = backend.get_calibration().unwrap();
        let noise_vec = calibration.get_qubit(0).unwrap().clone();

        // Use Beam Search
        let reorder = GateReorder::default();
        let (best_circuit, best_score) =
            reorder.beam_search_reorderings(&circuit, BeamSearchConfig::default(), |c| {
                score_circuit_variant(c, &noise_vec)
            });

        // Verify
        assert_eq!(best_circuit.gates.len(), circuit.gates.len());
        assert!(best_score >= 0.0 && best_score <= 1.0);

        // Execute optimized circuit
        let result = backend
            .execute(&best_circuit, 100, qns_core::backend::ExecutionMode::Sync)
            .unwrap();
        assert!(!result.counts.is_empty());
    }

    /// E2E Test: Topology-aware optimization
    #[test]
    fn test_e2e_topology_aware() {
        // Create backend and get topology
        let backend = SimulatorBackend::new(5);
        let topology = backend.get_topology().unwrap();

        // Verify linear topology
        assert!(topology.are_connected(0, 1));
        assert!(topology.are_connected(1, 2));
        assert!(!topology.are_connected(0, 2)); // Not directly connected

        // Create hardware-aware system
        let mut system = QnsSystem::new();
        system.set_hardware(topology);

        // Create and optimize circuit
        let mut circuit = CircuitGenome::new(3);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();
        circuit.add_gate(Gate::CNOT(1, 2)).unwrap();

        let result = system.optimize(circuit);
        assert!(result.is_ok());
    }

    /// E2E Test: Noisy vs Ideal simulation comparison
    #[test]
    fn test_e2e_noisy_vs_ideal() {
        let mut circuit = CircuitGenome::new(2);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();

        // Ideal simulation
        let ideal_backend = SimulatorBackend::ideal(2);
        let ideal_result = ideal_backend
            .execute(&circuit, 1000, qns_core::backend::ExecutionMode::Sync)
            .unwrap();

        // Noisy simulation
        let noise = NoiseModel::with_t1t2(50.0, 40.0); // High noise
        let noisy_backend = SimulatorBackend::with_noise(2, noise);
        let noisy_result = noisy_backend
            .execute(&circuit, 1000, qns_core::backend::ExecutionMode::Sync)
            .unwrap();

        // Ideal should have perfect Bell state
        let p00_ideal = ideal_result.probability("00");
        let p11_ideal = ideal_result.probability("11");
        assert!((p00_ideal - 0.5).abs() < 0.1, "Ideal |00⟩ should be ~0.5");
        assert!((p11_ideal - 0.5).abs() < 0.1, "Ideal |11⟩ should be ~0.5");

        // Noisy should have some degradation (|01⟩ and |10⟩ appear)
        let p01_noisy = noisy_result.probability("01");
        let p10_noisy = noisy_result.probability("10");
        // With high noise, error states should appear
        assert!(
            p01_noisy > 0.0 || p10_noisy > 0.0 || noisy_result.counts.len() > 2,
            "Noisy simulation should show some errors"
        );
    }

    /// E2E Test: Performance benchmark
    #[test]
    fn test_e2e_performance() {
        let backend = SimulatorBackend::new(5);

        // Create test circuit
        let mut circuit = CircuitGenome::new(5);
        for _ in 0..20 {
            for i in 0..5 {
                circuit.add_gate(Gate::H(i)).unwrap();
            }
        }

        // Measure execution time
        let start = std::time::Instant::now();
        let _result = backend.execute(&circuit, 1000, qns_core::backend::ExecutionMode::Sync);
        let elapsed = start.elapsed();

        // Should complete in reasonable time (< 1 second for 5 qubits)
        assert!(
            elapsed.as_millis() < 1000,
            "5-qubit, 100-gate circuit took {}ms",
            elapsed.as_millis()
        );
    }

    /// E2E Test: Full optimization pipeline with fidelity measurement
    #[test]
    fn test_e2e_fidelity_optimization() {
        // Create complex circuit
        let mut circuit = CircuitGenome::new(3);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::H(1)).unwrap();
        circuit.add_gate(Gate::H(2)).unwrap();
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();
        circuit.add_gate(Gate::CNOT(1, 2)).unwrap();
        circuit.add_gate(Gate::T(0)).unwrap();
        circuit.add_gate(Gate::S(1)).unwrap();
        circuit.add_gate(Gate::Z(2)).unwrap();

        // Run full QNS pipeline
        let mut system = QnsSystem::new();
        let result = system.optimize(circuit).unwrap();

        // Verify timing breakdown
        assert!(result.timing.profiling_time > Duration::ZERO);
        assert!(result.timing.optimization_time > Duration::ZERO);
        assert!(result.timing.simulation_time > Duration::ZERO);

        // Total time should be sum of parts (approximately)
        let sum = result.timing.profiling_time
            + result.timing.optimization_time
            + result.timing.simulation_time;
        // Allow some overhead
        assert!(
            result.total_time >= sum * 8 / 10, // 80% of sum
            "Total time should be at least 80% of sum of parts"
        );
    }

    /// E2E Test: Auto-reorder algorithm selection
    #[test]
    fn test_e2e_auto_reorder() {
        // Small circuit - should use BFS
        let mut small_circuit = CircuitGenome::new(3);
        for i in 0..10 {
            small_circuit.add_gate(Gate::H(i % 3)).unwrap();
        }

        let backend = SimulatorBackend::new(3);
        let calibration = backend.get_calibration().unwrap();
        let noise_vec = calibration.get_qubit(0).unwrap().clone();

        let reorder = GateReorder::default();
        let (best, score) =
            reorder.auto_reorder(&small_circuit, |c| score_circuit_variant(c, &noise_vec));

        assert_eq!(best.gates.len(), small_circuit.gates.len());
        assert!(score >= 0.0);
    }

    /// E2E Test: Calibration data consistency
    #[test]
    fn test_e2e_calibration_consistency() {
        let noise = NoiseModel::with_t1t2(150.0, 120.0)
            .with_gate_errors(0.001, 0.01)
            .with_readout_error(0.02);

        let backend = SimulatorBackend::with_noise(5, noise);
        let calibration = backend.get_calibration().unwrap();

        // Check all qubits have consistent data
        for i in 0..5 {
            let qubit = calibration.get_qubit(i).unwrap();
            assert_eq!(qubit.qubit_id, i);
            assert!((qubit.t1_mean - 150.0).abs() < 1e-10);
            assert!((qubit.t2_mean - 120.0).abs() < 1e-10);
            assert!((qubit.gate_error_1q - 0.001).abs() < 1e-10);
            assert!((qubit.gate_error_2q - 0.01).abs() < 1e-10);
            assert!((qubit.readout_error - 0.02).abs() < 1e-10);
        }

        // Check averages
        assert!((calibration.avg_t1() - 150.0).abs() < 1e-10);
        assert!((calibration.avg_t2() - 120.0).abs() < 1e-10);
    }
}
