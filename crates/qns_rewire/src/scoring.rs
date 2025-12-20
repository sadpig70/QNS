// QNS v2.0 - scoring.rs
// Phase 3 Journal Implementation: Scoring module for noise-adaptive optimization

use qns_core::prelude::{CircuitGenome, Gate, HardwareProfile, NoiseVector};

/// Scoring configuration with gate timing parameters (in nanoseconds)
#[derive(Debug, Clone)]
pub struct ScoreConfig {
    /// Single-qubit gate time (ns) - typical: 35ns for superconducting qubits
    pub gate_time_1q: f64,
    /// Two-qubit gate time (ns) - typical: 300ns for CX gates
    pub gate_time_2q: f64,
    /// Measurement time (ns) - typical: 1000ns
    pub measure_time: f64,
}

impl Default for ScoreConfig {
    fn default() -> Self {
        Self {
            gate_time_1q: 35.0,
            gate_time_2q: 300.0,
            measure_time: 1000.0,
        }
    }
}

/// Error type for scoring operations
#[derive(Debug, Clone, PartialEq)]
pub enum ScoringError {
    /// Invalid T1 value (must be positive)
    InvalidT1(f64),
    /// Invalid T2 value (must be positive)
    InvalidT2(f64),
    /// Invalid makespan (must be non-negative)
    InvalidMakespan(f64),
    /// Physical constraint violation: T2 > 2*T1
    T2ExceedsPhysicalLimit { t1: f64, t2: f64 },
}

impl std::fmt::Display for ScoringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScoringError::InvalidT1(v) => write!(f, "Invalid T1 value: {} (must be positive)", v),
            ScoringError::InvalidT2(v) => write!(f, "Invalid T2 value: {} (must be positive)", v),
            ScoringError::InvalidMakespan(v) => {
                write!(f, "Invalid makespan: {} (must be non-negative)", v)
            },
            ScoringError::T2ExceedsPhysicalLimit { t1, t2 } => {
                write!(
                    f,
                    "T2 ({}) > 2*T1 ({}): physically impossible, will be clamped",
                    t2,
                    2.0 * t1
                )
            },
        }
    }
}

impl std::error::Error for ScoringError {}

// ============================================================================
// Task 1.1: decay_estimation - T1/T2 based decay calculation
// ============================================================================

/// Calculates decay probability based on T1/T2 coherence times.
///
/// # Formula
/// `decay = 1 - exp(-makespan / T1) * exp(-makespan / T2)`
///
/// This models the combined effect of:
/// - Amplitude damping (T1 relaxation): |1⟩ → |0⟩ decay
/// - Phase damping (T2 dephasing): loss of superposition coherence
///
/// # Arguments
/// * `makespan_ns` - Total circuit execution time in nanoseconds
/// * `t1_us` - T1 relaxation time in microseconds
/// * `t2_us` - T2 dephasing time in microseconds
///
/// # Returns
/// * `Ok(decay_probability)` - Value in range [0.0, 1.0]
/// * `Err(ScoringError)` - If inputs are invalid
///
/// # Edge Cases
/// - `makespan = 0` → decay = 0 (no time for decoherence)
/// - `T1 = ∞` or `T2 = ∞` → reduced/no decay (ideal qubit)
/// - `T2 > 2*T1` → Warning logged, T2 clamped to 2*T1 (physical constraint)
///
/// # Example
/// ```ignore
/// let decay = decay_estimation(1000.0, 100.0, 80.0)?;
/// assert!(decay > 0.0 && decay < 1.0);
/// ```
pub fn decay_estimation(makespan_ns: f64, t1_us: f64, t2_us: f64) -> Result<f64, ScoringError> {
    // Validate inputs
    if makespan_ns < 0.0 {
        return Err(ScoringError::InvalidMakespan(makespan_ns));
    }

    if t1_us <= 0.0 && !t1_us.is_infinite() {
        return Err(ScoringError::InvalidT1(t1_us));
    }

    if t2_us <= 0.0 && !t2_us.is_infinite() {
        return Err(ScoringError::InvalidT2(t2_us));
    }

    // Handle ideal qubit case (infinite coherence times)
    if t1_us.is_infinite() && t2_us.is_infinite() {
        return Ok(0.0);
    }

    // Handle zero makespan case
    if makespan_ns == 0.0 {
        return Ok(0.0);
    }

    // Physical constraint: T2 <= 2*T1
    // If violated, clamp T2 and log warning (in production, use proper logging)
    let t2_clamped = if t2_us > 2.0 * t1_us && !t1_us.is_infinite() {
        #[cfg(debug_assertions)]
        eprintln!(
            "[WARN] T2 ({} μs) > 2*T1 ({} μs): physically impossible, clamping to {}",
            t2_us,
            t1_us,
            2.0 * t1_us
        );
        2.0 * t1_us
    } else {
        t2_us
    };

    // Convert makespan from ns to μs for consistent units
    let makespan_us = makespan_ns / 1000.0;

    // Calculate decay factors
    let t1_factor = if t1_us.is_infinite() {
        1.0 // No T1 decay for ideal qubit
    } else {
        (-makespan_us / t1_us).exp()
    };

    let t2_factor = if t2_clamped.is_infinite() {
        1.0 // No T2 decay for ideal qubit
    } else {
        (-makespan_us / t2_clamped).exp()
    };

    // Combined survival probability
    let survival = t1_factor * t2_factor;

    // Decay probability = 1 - survival probability
    let decay = 1.0 - survival;

    // Clamp to valid range (handles floating point edge cases)
    Ok(decay.clamp(0.0, 1.0))
}

/// Convenience function for decay estimation with NoiseVector
pub fn decay_estimation_from_noise(
    makespan_ns: f64,
    noise: &NoiseVector,
) -> Result<f64, ScoringError> {
    decay_estimation(makespan_ns, noise.t1_mean, noise.t2_mean)
}

// ============================================================================
// Task 1.2: gate_error_sum - Accumulated gate error calculation
// ============================================================================

/// Calculates total accumulated gate error for a circuit.
///
/// # Formula
/// `total_error = count_1q * gate_error_1q + count_2q * gate_error_2q + count_measure * readout_error`
///
/// # Arguments
/// * `circuit` - The circuit to analyze
/// * `noise` - Noise parameters including gate error rates
///
/// # Returns
/// Total accumulated error (may exceed 1.0 for deep circuits)
pub fn gate_error_sum(circuit: &CircuitGenome, noise: &NoiseVector) -> f64 {
    let mut count_1q = 0usize;
    let mut count_2q = 0usize;
    let mut count_measure = 0usize;

    for gate in &circuit.gates {
        match gate {
            // Single-qubit gates
            Gate::H(_)
            | Gate::X(_)
            | Gate::Y(_)
            | Gate::Z(_)
            | Gate::S(_)
            | Gate::T(_)
            | Gate::Rx(_, _)
            | Gate::Ry(_, _)
            | Gate::Rz(_, _) => {
                count_1q += 1;
            },
            // Two-qubit gates
            Gate::CNOT(_, _) | Gate::CZ(_, _) | Gate::SWAP(_, _) => {
                count_2q += 1;
            },
            // Measurement
            Gate::Measure(_) => {
                count_measure += 1;
            },
        }
    }

    (count_1q as f64) * noise.gate_error_1q
        + (count_2q as f64) * noise.gate_error_2q
        + (count_measure as f64) * noise.readout_error
}

// ============================================================================
// NEW: Hardware-aware gate error calculation with per-edge fidelity
// ============================================================================

/// Calculates total accumulated gate error using hardware-specific edge fidelities.
///
/// Unlike `gate_error_sum`, this function uses the actual per-edge error rates
/// from the HardwareProfile for two-qubit gates, enabling optimization based
/// on routing through higher-fidelity edges.
///
/// # Arguments
/// * `circuit` - The circuit to analyze
/// * `noise` - Noise parameters for single-qubit gates and fallback
/// * `hardware` - Hardware profile with per-edge fidelities
///
/// # Returns
/// Total accumulated error using edge-specific error rates
pub fn gate_error_sum_with_hardware(
    circuit: &CircuitGenome,
    noise: &NoiseVector,
    hardware: &HardwareProfile,
) -> f64 {
    let mut total_error = 0.0;

    for gate in &circuit.gates {
        match gate {
            // Single-qubit gates: use noise model
            Gate::H(_)
            | Gate::X(_)
            | Gate::Y(_)
            | Gate::Z(_)
            | Gate::S(_)
            | Gate::T(_)
            | Gate::Rx(_, _)
            | Gate::Ry(_, _)
            | Gate::Rz(_, _) => {
                total_error += noise.gate_error_1q;
            },
            // Two-qubit gates: use per-edge fidelity from hardware
            Gate::CNOT(q1, q2) | Gate::CZ(q1, q2) | Gate::SWAP(q1, q2) => {
                if let Some(coupler) = hardware.get_coupler(*q1, *q2) {
                    // Error = 1 - fidelity
                    total_error += coupler.gate_fidelity.error_rate();
                } else {
                    // Non-existent edge: high penalty (would require SWAP routing)
                    // Use 3x the noise model's 2Q error as penalty, minimum 0.15
                    total_error += (noise.gate_error_2q * 3.0).max(0.15);
                }
            },
            // Measurement
            Gate::Measure(_) => {
                total_error += noise.readout_error;
            },
        }
    }

    total_error
}

/// Estimates fidelity with hardware-specific per-edge error rates.
///
/// This combines:
/// 1. Per-qubit idle time decoherence (from T1/T2)
/// 2. Per-edge two-qubit gate error rates (from HardwareProfile)
///
/// This enables true hardware-aware optimization where:
/// - Different CNOT paths have different fidelity costs
/// - Routing through higher-fidelity edges improves circuit fidelity
///
/// # Example Use Case
/// On IBM Heron, edge (0,1) might have 99.5% fidelity while (1,2) has 98%.
/// This function scores circuits that use (0,1) higher than those using (1,2).
pub fn estimate_fidelity_with_hardware(
    circuit: &CircuitGenome,
    noise: &NoiseVector,
    hardware: &HardwareProfile,
    config: &ScoreConfig,
) -> f64 {
    if circuit.gates.is_empty() {
        return 1.0;
    }

    let (schedules, _makespan) = calculate_qubit_schedules(circuit, config);

    // Calculate per-qubit survival probability based on idle time
    let mut total_survival = 1.0;

    for schedule in &schedules {
        if !schedule.activities.is_empty() && schedule.idle_time > 0.0 {
            let decay = match decay_estimation(schedule.idle_time, noise.t1_mean, noise.t2_mean) {
                Ok(d) => d,
                Err(_) => return 0.0,
            };
            total_survival *= 1.0 - decay;
        }
    }

    // Hardware-aware gate error (uses per-edge fidelities)
    let gate_error = gate_error_sum_with_hardware(circuit, noise, hardware);

    // Combined fidelity
    let fidelity = total_survival * (1.0 - gate_error.min(1.0));
    fidelity.clamp(0.0, 1.0)
}

// ============================================================================
// Task 1.3: critical_path - Makespan calculation
// ============================================================================

/// Calculates the critical path (makespan) of a circuit.
///
/// The makespan is the total execution time considering parallel gate execution.
/// Gates on different qubits can execute in parallel if they don't share qubits.
///
/// # Arguments
/// * `circuit` - The circuit to analyze
/// * `config` - Gate timing configuration
///
/// # Returns
/// Makespan in nanoseconds
pub fn critical_path(circuit: &CircuitGenome, config: &ScoreConfig) -> f64 {
    if circuit.gates.is_empty() {
        return 0.0;
    }

    // Track end time for each qubit
    let mut qubit_end_times: Vec<f64> = vec![0.0; circuit.num_qubits];

    for gate in &circuit.gates {
        let (qubits, gate_time) = match gate {
            // Single-qubit gates
            Gate::H(q) | Gate::X(q) | Gate::Y(q) | Gate::Z(q) | Gate::S(q) | Gate::T(q) => {
                (vec![*q], config.gate_time_1q)
            },
            Gate::Rx(q, _) | Gate::Ry(q, _) | Gate::Rz(q, _) => (vec![*q], config.gate_time_1q),
            // Two-qubit gates
            Gate::CNOT(q1, q2) | Gate::CZ(q1, q2) | Gate::SWAP(q1, q2) => {
                (vec![*q1, *q2], config.gate_time_2q)
            },
            // Measurement
            Gate::Measure(q) => (vec![*q], config.measure_time),
        };

        // Find the latest end time among involved qubits
        let start_time = qubits
            .iter()
            .filter_map(|&q| qubit_end_times.get(q).copied())
            .fold(0.0_f64, f64::max);

        let end_time = start_time + gate_time;

        // Update end times for all involved qubits
        for &q in &qubits {
            if q < qubit_end_times.len() {
                qubit_end_times[q] = end_time;
            }
        }
    }

    // Return the maximum end time (critical path)
    qubit_end_times.into_iter().fold(0.0_f64, f64::max)
}

// ============================================================================
// NEW: Per-qubit idle time tracking for order-sensitive scoring
// ============================================================================

/// Qubit schedule entry: (start_time, end_time) for each gate on this qubit
#[derive(Debug, Clone)]
pub struct QubitSchedule {
    /// List of (start, end) times for gates on this qubit
    pub activities: Vec<(f64, f64)>,
    /// Total time this qubit is active (executing gates)
    pub active_time: f64,
    /// Total time this qubit is idle (waiting)
    pub idle_time: f64,
    /// When this qubit finishes its last gate
    pub end_time: f64,
}

/// Calculates detailed per-qubit schedules for a circuit.
///
/// Unlike `critical_path`, this tracks exactly when each qubit is active vs idle,
/// which is crucial for accurate decoherence modeling.
///
/// # Returns
/// Vector of QubitSchedule for each qubit, plus total circuit makespan
pub fn calculate_qubit_schedules(
    circuit: &CircuitGenome,
    config: &ScoreConfig,
) -> (Vec<QubitSchedule>, f64) {
    let n = circuit.num_qubits;
    let mut schedules: Vec<QubitSchedule> = (0..n)
        .map(|_| QubitSchedule {
            activities: Vec::new(),
            active_time: 0.0,
            idle_time: 0.0,
            end_time: 0.0,
        })
        .collect();

    let mut qubit_end_times: Vec<f64> = vec![0.0; n];

    for gate in &circuit.gates {
        let (qubits, gate_time) = match gate {
            Gate::H(q) | Gate::X(q) | Gate::Y(q) | Gate::Z(q) | Gate::S(q) | Gate::T(q) => {
                (vec![*q], config.gate_time_1q)
            },
            Gate::Rx(q, _) | Gate::Ry(q, _) | Gate::Rz(q, _) => (vec![*q], config.gate_time_1q),
            Gate::CNOT(q1, q2) | Gate::CZ(q1, q2) | Gate::SWAP(q1, q2) => {
                (vec![*q1, *q2], config.gate_time_2q)
            },
            Gate::Measure(q) => (vec![*q], config.measure_time),
        };

        // Start time is max of all involved qubits' end times
        let start_time = qubits
            .iter()
            .filter_map(|&q| qubit_end_times.get(q).copied())
            .fold(0.0_f64, f64::max);

        let end_time = start_time + gate_time;

        // Update each qubit's schedule
        for &q in &qubits {
            if q < n {
                schedules[q].activities.push((start_time, end_time));
                schedules[q].active_time += gate_time;
                qubit_end_times[q] = end_time;
            }
        }
    }

    // Calculate idle time for each qubit
    let makespan = qubit_end_times.iter().cloned().fold(0.0_f64, f64::max);

    for (q, schedule) in schedules.iter_mut().enumerate() {
        schedule.end_time = qubit_end_times[q];

        // Idle time = makespan - active_time (for qubits that participate)
        // OR time between activities
        if !schedule.activities.is_empty() {
            let mut prev_end = 0.0;
            for &(start, end) in &schedule.activities {
                // Idle time before this gate
                if start > prev_end {
                    schedule.idle_time += start - prev_end;
                }
                prev_end = end;
            }
            // Idle time after last gate until circuit ends
            if makespan > prev_end {
                schedule.idle_time += makespan - prev_end;
            }
        }
    }

    (schedules, makespan)
}

/// Calculates total idle time across all qubits.
///
/// This is the key metric for order-sensitive optimization:
/// different gate orderings can result in different total idle times.
pub fn calculate_total_idle_time(circuit: &CircuitGenome, config: &ScoreConfig) -> f64 {
    let (schedules, _) = calculate_qubit_schedules(circuit, config);
    schedules.iter().map(|s| s.idle_time).sum()
}

/// Estimates fidelity with per-qubit idle time tracking.
///
/// This function differs from `estimate_fidelity_with_scheduling` by considering
/// that each qubit experiences decoherence during its idle periods.
///
/// # Formula
/// For each qubit q:
///   decay_q = 1 - exp(-idle_time_q / T1) * exp(-idle_time_q / T2)
///
/// Total survival = product of (1 - decay_q) for all active qubits
/// Fidelity = total_survival * (1 - gate_error)
///
/// # Why this matters for reordering
/// Consider circuit [H(0), CNOT(0,1), H(1)]:
/// - Qubit 1 is idle for 35ns before CNOT
///
/// Reordered to [H(0), H(1), CNOT(0,1)]:
/// - Qubit 1 has 0ns idle time (H(1) runs parallel with H(0))
///
/// The second ordering has less total idle time → higher fidelity
pub fn estimate_fidelity_with_idle_tracking(
    circuit: &CircuitGenome,
    noise: &NoiseVector,
    config: &ScoreConfig,
) -> f64 {
    if circuit.gates.is_empty() {
        return 1.0;
    }

    let (schedules, _makespan) = calculate_qubit_schedules(circuit, config);

    // Calculate per-qubit survival probability based on idle time
    let mut total_survival = 1.0;

    for schedule in &schedules {
        // Only consider qubits that have gates (are active in the circuit)
        if !schedule.activities.is_empty() && schedule.idle_time > 0.0 {
            let decay = match decay_estimation(schedule.idle_time, noise.t1_mean, noise.t2_mean) {
                Ok(d) => d,
                Err(_) => return 0.0,
            };
            total_survival *= 1.0 - decay;
        }
    }

    // Gate error contribution (unchanged from original)
    let gate_error = gate_error_sum(circuit, noise);

    // Combined fidelity
    let fidelity = total_survival * (1.0 - gate_error.min(1.0));
    fidelity.clamp(0.0, 1.0)
}

// ============================================================================
// Task 1.4: estimate_fidelity_with_scheduling - Integration
// ============================================================================

/// Estimates circuit fidelity considering both decoherence and gate errors.
///
/// # Formula
/// `fidelity = (1 - decay) * (1 - gate_error)`
///
/// Where:
/// - `decay` is computed from T1/T2 coherence over the makespan
/// - `gate_error` is the accumulated error from all gates
///
/// # Arguments
/// * `circuit` - The circuit to evaluate
/// * `noise` - Noise parameters (T1, T2, gate errors)
/// * `config` - Gate timing configuration
///
/// # Returns
/// Estimated fidelity in range [0.0, 1.0]
pub fn estimate_fidelity_with_scheduling(
    circuit: &CircuitGenome,
    noise: &NoiseVector,
    config: &ScoreConfig,
) -> f64 {
    // Empty circuit has perfect fidelity
    if circuit.gates.is_empty() {
        return 1.0;
    }

    // Calculate makespan
    let makespan = critical_path(circuit, config);

    // Calculate decay probability
    let decay = match decay_estimation(makespan, noise.t1_mean, noise.t2_mean) {
        Ok(d) => d,
        Err(_) => {
            // On error, assume worst case for safety
            return 0.0;
        },
    };

    // Calculate gate error
    let gate_error = gate_error_sum(circuit, noise);

    // Combined fidelity: survival from decay * survival from gate errors
    // Using multiplicative model for independent error sources
    let fidelity = (1.0 - decay) * (1.0 - gate_error.min(1.0));

    // Clamp to valid range
    fidelity.clamp(0.0, 1.0)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- decay_estimation tests ----------

    #[test]
    fn test_decay_estimation_zero_makespan() {
        let decay = decay_estimation(0.0, 100.0, 80.0).unwrap();
        assert_eq!(decay, 0.0, "Zero makespan should give zero decay");
    }

    #[test]
    fn test_decay_estimation_at_t1() {
        // At makespan = T1 (converted to ns), decay should be approximately 1 - e^(-1) * e^(-T1/T2)
        let t1_us = 100.0;
        let t2_us = 80.0;
        let makespan_ns = t1_us * 1000.0; // T1 in nanoseconds

        let decay = decay_estimation(makespan_ns, t1_us, t2_us).unwrap();

        // Expected: 1 - exp(-1) * exp(-100/80) = 1 - 0.368 * 0.287 ≈ 0.894
        let expected = 1.0 - (-1.0_f64).exp() * (-t1_us / t2_us).exp();
        assert!(
            (decay - expected).abs() < 0.001,
            "decay={}, expected={}",
            decay,
            expected
        );
    }

    #[test]
    fn test_decay_estimation_ideal_qubit() {
        let decay = decay_estimation(10000.0, f64::INFINITY, f64::INFINITY).unwrap();
        assert_eq!(decay, 0.0, "Ideal qubit should have zero decay");
    }

    #[test]
    fn test_decay_estimation_t2_clamping() {
        // T2 > 2*T1 should be clamped (physically impossible)
        let t1 = 100.0;
        let t2 = 300.0; // > 2*T1
        let makespan = 1000.0;

        let decay_clamped = decay_estimation(makespan, t1, t2).unwrap();
        let decay_at_2t1 = decay_estimation(makespan, t1, 2.0 * t1).unwrap();

        assert_eq!(decay_clamped, decay_at_2t1, "T2 should be clamped to 2*T1");
    }

    #[test]
    fn test_decay_estimation_invalid_t1() {
        let result = decay_estimation(1000.0, -50.0, 80.0);
        assert!(matches!(result, Err(ScoringError::InvalidT1(_))));
    }

    #[test]
    fn test_decay_estimation_invalid_t2() {
        let result = decay_estimation(1000.0, 100.0, -50.0);
        assert!(matches!(result, Err(ScoringError::InvalidT2(_))));
    }

    #[test]
    fn test_decay_estimation_invalid_makespan() {
        let result = decay_estimation(-1000.0, 100.0, 80.0);
        assert!(matches!(result, Err(ScoringError::InvalidMakespan(_))));
    }

    // ---------- gate_error_sum tests ----------

    #[test]
    fn test_gate_error_sum_empty_circuit() {
        let circuit = CircuitGenome::new(2);
        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);

        let error = gate_error_sum(&circuit, &noise);
        assert_eq!(error, 0.0, "Empty circuit should have zero gate error");
    }

    #[test]
    fn test_gate_error_sum_single_1q_gate() {
        let mut circuit = CircuitGenome::new(2);
        circuit.add_gate(Gate::H(0)).unwrap();

        let mut noise = NoiseVector::new(0);
        noise.gate_error_1q = 0.001;

        let error = gate_error_sum(&circuit, &noise);
        assert!(
            (error - 0.001).abs() < 1e-10,
            "Single H gate error should be gate_error_1q"
        );
    }

    #[test]
    fn test_gate_error_sum_single_2q_gate() {
        let mut circuit = CircuitGenome::new(2);
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();

        let mut noise = NoiseVector::new(0);
        noise.gate_error_2q = 0.01;

        let error = gate_error_sum(&circuit, &noise);
        assert!(
            (error - 0.01).abs() < 1e-10,
            "Single CNOT gate error should be gate_error_2q"
        );
    }

    #[test]
    fn test_gate_error_sum_mixed_circuit() {
        let mut circuit = CircuitGenome::new(3);
        circuit.add_gate(Gate::H(0)).unwrap(); // 1Q
        circuit.add_gate(Gate::H(1)).unwrap(); // 1Q
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap(); // 2Q
        circuit.add_gate(Gate::X(2)).unwrap(); // 1Q

        let mut noise = NoiseVector::new(0);
        noise.gate_error_1q = 0.001;
        noise.gate_error_2q = 0.01;

        let error = gate_error_sum(&circuit, &noise);
        let expected = 3.0 * 0.001 + 1.0 * 0.01;
        assert!(
            (error - expected).abs() < 1e-10,
            "Mixed circuit error calculation"
        );
    }

    // ---------- critical_path tests ----------

    #[test]
    fn test_critical_path_empty_circuit() {
        let circuit = CircuitGenome::new(2);
        let config = ScoreConfig::default();

        let makespan = critical_path(&circuit, &config);
        assert_eq!(makespan, 0.0, "Empty circuit should have zero makespan");
    }

    #[test]
    fn test_critical_path_serial_circuit() {
        // All gates on same qubit = serial execution
        let mut circuit = CircuitGenome::new(1);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::X(0)).unwrap();
        circuit.add_gate(Gate::Y(0)).unwrap();

        let config = ScoreConfig::default();
        let makespan = critical_path(&circuit, &config);

        let expected = 3.0 * config.gate_time_1q;
        assert!(
            (makespan - expected).abs() < 1e-10,
            "Serial circuit makespan"
        );
    }

    #[test]
    fn test_critical_path_parallel_circuit() {
        // Gates on different qubits = parallel execution
        let mut circuit = CircuitGenome::new(3);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::H(1)).unwrap();
        circuit.add_gate(Gate::H(2)).unwrap();

        let config = ScoreConfig::default();
        let makespan = critical_path(&circuit, &config);

        // All H gates execute in parallel
        let expected = config.gate_time_1q;
        assert!(
            (makespan - expected).abs() < 1e-10,
            "Parallel circuit makespan"
        );
    }

    #[test]
    fn test_critical_path_mixed_circuit() {
        // H(0), H(1) in parallel, then CNOT(0,1)
        let mut circuit = CircuitGenome::new(2);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::H(1)).unwrap();
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();

        let config = ScoreConfig::default();
        let makespan = critical_path(&circuit, &config);

        // H gates parallel (35ns), then CNOT (300ns)
        let expected = config.gate_time_1q + config.gate_time_2q;
        assert!(
            (makespan - expected).abs() < 1e-10,
            "Mixed circuit makespan"
        );
    }

    // ---------- estimate_fidelity_with_scheduling tests ----------

    #[test]
    fn test_fidelity_ideal_noise() {
        let mut circuit = CircuitGenome::new(2);
        circuit.add_gate(Gate::H(0)).unwrap();

        // Ideal noise: infinite T1/T2, zero gate errors
        let mut noise = NoiseVector::new(0);
        noise.t1_mean = f64::INFINITY;
        noise.t2_mean = f64::INFINITY;
        noise.gate_error_1q = 0.0;
        noise.gate_error_2q = 0.0;

        let config = ScoreConfig::default();
        let fidelity = estimate_fidelity_with_scheduling(&circuit, &noise, &config);

        assert_eq!(fidelity, 1.0, "Ideal noise should give perfect fidelity");
    }

    #[test]
    fn test_fidelity_realistic_noise() {
        let mut circuit = CircuitGenome::new(2);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();

        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
        let config = ScoreConfig::default();

        let fidelity = estimate_fidelity_with_scheduling(&circuit, &noise, &config);

        assert!(
            fidelity > 0.0 && fidelity < 1.0,
            "Realistic fidelity: {}",
            fidelity
        );
    }

    #[test]
    fn test_fidelity_extreme_noise() {
        let mut circuit = CircuitGenome::new(2);
        circuit.add_gate(Gate::H(0)).unwrap();

        // Extreme noise: very short T1/T2, high gate errors
        let mut noise = NoiseVector::new(0);
        noise.t1_mean = 0.001; // 1ns T1
        noise.t2_mean = 0.001; // 1ns T2
        noise.gate_error_1q = 0.5;

        let config = ScoreConfig::default();
        let fidelity = estimate_fidelity_with_scheduling(&circuit, &noise, &config);

        assert!(
            fidelity < 0.1,
            "Extreme noise should give near-zero fidelity: {}",
            fidelity
        );
    }

    #[test]
    fn test_fidelity_empty_circuit() {
        let circuit = CircuitGenome::new(2);
        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
        let config = ScoreConfig::default();

        let fidelity = estimate_fidelity_with_scheduling(&circuit, &noise, &config);
        assert_eq!(fidelity, 1.0, "Empty circuit should have perfect fidelity");
    }

    // ---------- idle time tracking tests ----------

    #[test]
    fn test_qubit_schedules_parallel_gates() {
        // H(0), H(1) should execute in parallel
        let mut circuit = CircuitGenome::new(2);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::H(1)).unwrap();

        let config = ScoreConfig::default();
        let (schedules, makespan) = calculate_qubit_schedules(&circuit, &config);

        // Both start at 0, end at gate_time_1q
        assert_eq!(schedules[0].activities.len(), 1);
        assert_eq!(schedules[1].activities.len(), 1);
        assert!((schedules[0].activities[0].0 - 0.0).abs() < 1e-10);
        assert!((schedules[1].activities[0].0 - 0.0).abs() < 1e-10);
        assert!((makespan - config.gate_time_1q).abs() < 1e-10);
    }

    #[test]
    fn test_qubit_schedules_serial_gates() {
        // H(0), CNOT(0,1), H(1) - qubit 1 has idle time before CNOT
        let mut circuit = CircuitGenome::new(2);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap();
        circuit.add_gate(Gate::H(1)).unwrap();

        let config = ScoreConfig::default();
        let (schedules, makespan) = calculate_qubit_schedules(&circuit, &config);

        // Makespan = 35 (H) + 300 (CNOT) + 35 (H) = 370ns
        assert!(
            (makespan - 370.0).abs() < 1e-10,
            "Makespan should be 370ns: {}",
            makespan
        );

        // Qubit 0: H(0-35), CNOT(35-335), idle(335-370) = 35ns idle
        // Qubit 1: idle(0-35), CNOT(35-335), H(335-370) = 35ns idle
        // Both have 35ns of idle time (Q0 at end, Q1 at start)
        assert!(
            (schedules[0].idle_time - config.gate_time_1q).abs() < 1e-10,
            "Q0 should have 35ns idle time (after last gate): {}",
            schedules[0].idle_time
        );
        assert!(
            (schedules[1].idle_time - config.gate_time_1q).abs() < 1e-10,
            "Q1 should have 35ns idle time (before CNOT): {}",
            schedules[1].idle_time
        );
    }

    #[test]
    fn test_idle_time_reordering_effect() {
        let config = ScoreConfig::default();

        // Suboptimal: H(0), CNOT(0,1), H(1)
        let mut suboptimal = CircuitGenome::new(2);
        suboptimal.add_gate(Gate::H(0)).unwrap();
        suboptimal.add_gate(Gate::CNOT(0, 1)).unwrap();
        suboptimal.add_gate(Gate::H(1)).unwrap();

        // Optimal: H(0), H(1), CNOT(0,1)
        let mut optimal = CircuitGenome::new(2);
        optimal.add_gate(Gate::H(0)).unwrap();
        optimal.add_gate(Gate::H(1)).unwrap();
        optimal.add_gate(Gate::CNOT(0, 1)).unwrap();

        let idle_suboptimal = calculate_total_idle_time(&suboptimal, &config);
        let idle_optimal = calculate_total_idle_time(&optimal, &config);

        // Optimal ordering should have less idle time
        assert!(
            idle_optimal < idle_suboptimal,
            "Optimal ordering should have less idle time: {} < {}",
            idle_optimal,
            idle_suboptimal
        );
    }

    #[test]
    fn test_idle_fidelity_reordering_improves() {
        let config = ScoreConfig::default();
        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);

        // Suboptimal ordering
        let mut suboptimal = CircuitGenome::new(2);
        suboptimal.add_gate(Gate::H(0)).unwrap();
        suboptimal.add_gate(Gate::CNOT(0, 1)).unwrap();
        suboptimal.add_gate(Gate::H(1)).unwrap();

        // Optimal ordering
        let mut optimal = CircuitGenome::new(2);
        optimal.add_gate(Gate::H(0)).unwrap();
        optimal.add_gate(Gate::H(1)).unwrap();
        optimal.add_gate(Gate::CNOT(0, 1)).unwrap();

        let fidelity_suboptimal =
            estimate_fidelity_with_idle_tracking(&suboptimal, &noise, &config);
        let fidelity_optimal = estimate_fidelity_with_idle_tracking(&optimal, &noise, &config);

        // Optimal ordering should have higher fidelity
        assert!(
            fidelity_optimal > fidelity_suboptimal,
            "Optimal ordering should have higher fidelity: {} > {}",
            fidelity_optimal,
            fidelity_suboptimal
        );
    }

    #[test]
    fn test_idle_fidelity_empty_circuit() {
        let circuit = CircuitGenome::new(2);
        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
        let config = ScoreConfig::default();

        let fidelity = estimate_fidelity_with_idle_tracking(&circuit, &noise, &config);
        assert_eq!(fidelity, 1.0, "Empty circuit should have perfect fidelity");
    }

    // ---------- hardware-aware scoring tests ----------

    fn create_hardware_with_varying_fidelity() -> HardwareProfile {
        use qns_core::types::Fidelity;

        // Linear chain: 0 -- 1 -- 2
        //               99%   95%
        let mut hw = HardwareProfile::linear("test", 3);
        hw.couplers[0].gate_fidelity = Fidelity::new(0.99); // Edge 0-1: 1% error
        hw.couplers[1].gate_fidelity = Fidelity::new(0.95); // Edge 1-2: 5% error
        hw
    }

    #[test]
    fn test_gate_error_sum_with_hardware_uses_edge_fidelity() {
        let hw = create_hardware_with_varying_fidelity();

        // Circuit using high-fidelity edge (0,1)
        let mut circuit_high = CircuitGenome::new(3);
        circuit_high.add_gate(Gate::CNOT(0, 1)).unwrap();

        // Circuit using low-fidelity edge (1,2)
        let mut circuit_low = CircuitGenome::new(3);
        circuit_low.add_gate(Gate::CNOT(1, 2)).unwrap();

        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);

        let error_high = gate_error_sum_with_hardware(&circuit_high, &noise, &hw);
        let error_low = gate_error_sum_with_hardware(&circuit_low, &noise, &hw);

        // High-fidelity edge should have lower error
        assert!(
            error_high < error_low,
            "High-fidelity edge should have lower error: {} < {}",
            error_high,
            error_low
        );

        // Check specific values
        assert!(
            (error_high - 0.01).abs() < 1e-10,
            "Edge 0-1 should have 1% error: {}",
            error_high
        );
        assert!(
            (error_low - 0.05).abs() < 1e-10,
            "Edge 1-2 should have 5% error: {}",
            error_low
        );
    }

    #[test]
    fn test_gate_error_sum_with_hardware_fallback() {
        let hw = HardwareProfile::linear("test", 3);

        // Circuit using edge that doesn't exist in hardware
        let mut circuit = CircuitGenome::new(5);
        circuit.add_gate(Gate::CNOT(3, 4)).unwrap(); // Qubits 3,4 not in 3-qubit hardware

        let mut noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
        noise.gate_error_2q = 0.02; // 2% base error

        let error = gate_error_sum_with_hardware(&circuit, &noise, &hw);

        // Non-existent edge penalty: max(0.02 * 3, 0.15) = 0.15
        assert!(
            (error - 0.15).abs() < 1e-10,
            "Should use penalty for non-existent edge: {}",
            error
        );
    }

    #[test]
    fn test_estimate_fidelity_with_hardware_empty_circuit() {
        let circuit = CircuitGenome::new(2);
        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
        let hw = HardwareProfile::linear("test", 2);
        let config = ScoreConfig::default();

        let fidelity = estimate_fidelity_with_hardware(&circuit, &noise, &hw, &config);
        assert_eq!(fidelity, 1.0, "Empty circuit should have perfect fidelity");
    }

    #[test]
    fn test_estimate_fidelity_with_hardware_differs_by_edge() {
        let hw = create_hardware_with_varying_fidelity();
        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
        let config = ScoreConfig::default();

        // Circuit using high-fidelity edge
        let mut circuit_high = CircuitGenome::new(3);
        circuit_high.add_gate(Gate::CNOT(0, 1)).unwrap();

        // Circuit using low-fidelity edge
        let mut circuit_low = CircuitGenome::new(3);
        circuit_low.add_gate(Gate::CNOT(1, 2)).unwrap();

        let fidelity_high = estimate_fidelity_with_hardware(&circuit_high, &noise, &hw, &config);
        let fidelity_low = estimate_fidelity_with_hardware(&circuit_low, &noise, &hw, &config);

        // High-fidelity edge should give higher fidelity
        assert!(
            fidelity_high > fidelity_low,
            "High-fidelity edge should give higher fidelity: {} > {}",
            fidelity_high,
            fidelity_low
        );
    }

    #[test]
    fn test_hardware_aware_vs_uniform_scoring() {
        let hw = create_hardware_with_varying_fidelity();
        let noise = NoiseVector::with_t1t2(0, 100.0, 80.0);
        let config = ScoreConfig::default();

        // Circuit using low-fidelity edge (5% error)
        let mut circuit = CircuitGenome::new(3);
        circuit.add_gate(Gate::CNOT(1, 2)).unwrap();

        let uniform_fidelity = estimate_fidelity_with_idle_tracking(&circuit, &noise, &config);
        let hw_fidelity = estimate_fidelity_with_hardware(&circuit, &noise, &hw, &config);

        // Uniform uses ~1% error (noise.gate_error_2q default)
        // Hardware uses 5% error (from edge 1-2)
        // So hardware-aware fidelity should be lower
        assert!(
            hw_fidelity < uniform_fidelity,
            "Hardware-aware should give lower fidelity for noisy edge: {} < {}",
            hw_fidelity,
            uniform_fidelity
        );
    }
}
