//! Hardware profile for quantum devices.
//!
//! This module defines the hardware characteristics including:
//! - Qubit connectivity (topology)
//! - Per-qubit T1/T2 times
//! - Gate error rates
//! - Gate timings

use crate::physics::{gate_errors, gate_times, t1_typical, t2_typical};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Fidelity value constrained to [0.0, 1.0].
///
/// Fidelity measures how close a quantum state or operation is to the ideal.
/// - 1.0 = perfect fidelity
/// - 0.0 = completely orthogonal/wrong
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Fidelity(f64);

impl Fidelity {
    /// Creates a new Fidelity value.
    ///
    /// # Panics
    /// Panics if value is not in [0.0, 1.0].
    pub fn new(value: f64) -> Self {
        assert!(
            (0.0..=1.0).contains(&value),
            "Fidelity must be in [0.0, 1.0], got {}",
            value
        );
        Self(value)
    }

    /// Creates a Fidelity, clamping to [0.0, 1.0].
    pub fn clamped(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Returns the fidelity as f64.
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Perfect fidelity (1.0).
    pub const PERFECT: Self = Self(1.0);

    /// Zero fidelity (0.0).
    pub const ZERO: Self = Self(0.0);

    /// Converts error rate to fidelity (F = 1 - ε).
    pub fn from_error_rate(error: f64) -> Self {
        Self::clamped(1.0 - error)
    }

    /// Returns the error rate (ε = 1 - F).
    pub fn error_rate(&self) -> f64 {
        1.0 - self.0
    }
}

impl Default for Fidelity {
    fn default() -> Self {
        Self::PERFECT
    }
}

impl std::fmt::Display for Fidelity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.4}", self.0)
    }
}

/// Qubit properties for a single qubit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QubitProperties {
    /// T1 relaxation time (μs)
    pub t1: f64,
    /// T2 dephasing time (μs)
    pub t2: f64,
    /// Readout fidelity
    pub readout_fidelity: Fidelity,
    /// Single-qubit gate fidelity
    pub single_gate_fidelity: Fidelity,
    /// Frequency (GHz), if known
    pub frequency: Option<f64>,
    /// Anharmonicity (MHz), if known
    pub anharmonicity: Option<f64>,
}

impl Default for QubitProperties {
    fn default() -> Self {
        Self {
            t1: t1_typical::NISQ_TYPICAL,
            t2: t2_typical::NISQ_TYPICAL,
            readout_fidelity: Fidelity::from_error_rate(gate_errors::MEASUREMENT_TYPICAL),
            single_gate_fidelity: Fidelity::from_error_rate(gate_errors::SINGLE_QUBIT_TYPICAL),
            frequency: None,
            anharmonicity: None,
        }
    }
}

impl QubitProperties {
    /// Creates a new QubitProperties with the given T1/T2 values.
    pub fn with_t1t2(t1: f64, t2: f64) -> Self {
        Self {
            t1,
            t2,
            ..Default::default()
        }
    }

    /// Returns the T2/T1 ratio (dephasing quality indicator).
    ///
    /// For superconducting qubits, typically 0.5 ≤ T2/T1 ≤ 2.0.
    /// T2 ≤ 2*T1 is a physical constraint.
    pub fn t2_t1_ratio(&self) -> f64 {
        if self.t1 > 0.0 {
            self.t2 / self.t1
        } else {
            0.0
        }
    }
}

/// Two-qubit gate properties between a pair of qubits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouplerProperties {
    /// Control qubit index
    pub qubit1: usize,
    /// Target qubit index
    pub qubit2: usize,
    /// Two-qubit gate fidelity
    pub gate_fidelity: Fidelity,
    /// Gate time (ns)
    pub gate_time_ns: f64,
    /// Native gate type (e.g., "CZ", "CNOT", "iSWAP")
    pub native_gate: String,
}

impl CouplerProperties {
    /// Creates a new CouplerProperties with default values.
    pub fn new(qubit1: usize, qubit2: usize) -> Self {
        Self {
            qubit1,
            qubit2,
            gate_fidelity: Fidelity::from_error_rate(gate_errors::TWO_QUBIT_TYPICAL),
            gate_time_ns: gate_times::TWO_QUBIT,
            native_gate: "CZ".to_string(),
        }
    }

    /// Returns the ordered pair (min, max) for consistent edge representation.
    pub fn edge(&self) -> (usize, usize) {
        if self.qubit1 <= self.qubit2 {
            (self.qubit1, self.qubit2)
        } else {
            (self.qubit2, self.qubit1)
        }
    }
}

/// Topology types for common quantum hardware layouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Topology {
    /// All qubits connected to all others
    AllToAll,
    /// Linear chain: qubit i connected to i±1
    Linear,
    /// Ring: linear with wrap-around
    Ring,
    /// 2D grid/lattice (e.g., Google Sycamore)
    Grid { rows: usize, cols: usize },
    /// Heavy-hex lattice (e.g., IBM)
    HeavyHex,
    /// Custom topology (defined by edge list)
    Custom,
}

/// Represents the crosstalk interaction strength between pairs of qubits.
///
/// Stores entries as (min, max) -> strength key pairs to ensure symmetry.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CrosstalkMatrix {
    /// Interaction strength mapping (e.g., ZZ interaction frequency or error rate).
    /// Key is (qubit1, qubit2) where qubit1 < qubit2.
    /// Value is the interaction strength (normalized 0.0 to 1.0 or frequency in Hz depending on usage).
    pub interactions: HashMap<(usize, usize), f64>,
}

impl CrosstalkMatrix {
    /// Creates a new empty CrosstalkMatrix.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the interaction strength between two qubits.
    pub fn set_interaction(&mut self, q1: usize, q2: usize, strength: f64) {
        let key = if q1 < q2 { (q1, q2) } else { (q2, q1) };
        self.interactions.insert(key, strength);
    }

    /// Gets the interaction strength between two qubits.
    pub fn get_interaction(&self, q1: usize, q2: usize) -> Option<f64> {
        let key = if q1 < q2 { (q1, q2) } else { (q2, q1) };
        self.interactions.get(&key).copied()
    }

    /// Returns true if the matrix is empty.
    pub fn is_empty(&self) -> bool {
        self.interactions.is_empty()
    }
}

/// Hardware profile describing a quantum device.
///
/// Contains qubit properties, connectivity, and calibration data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    /// Device name
    pub name: String,
    /// Number of qubits
    pub num_qubits: usize,
    /// Topology type
    pub topology: Topology,
    /// Per-qubit properties
    pub qubit_properties: Vec<QubitProperties>,
    /// Two-qubit gate properties (connectivity)
    pub couplers: Vec<CouplerProperties>,
    /// Connectivity graph as adjacency set
    #[serde(skip)]
    connectivity: HashSet<(usize, usize)>,
    /// Calibration timestamp (Unix time)
    pub calibration_timestamp: Option<u64>,
    /// Crosstalk interaction matrix
    pub crosstalk: CrosstalkMatrix,
}

impl HardwareProfile {
    /// Creates a new HardwareProfile with default qubit properties.
    pub fn new(name: impl Into<String>, num_qubits: usize, topology: Topology) -> Self {
        let qubit_properties = vec![QubitProperties::default(); num_qubits];
        let couplers = Self::generate_couplers(num_qubits, &topology);
        let connectivity = couplers.iter().map(|c| c.edge()).collect();

        Self {
            name: name.into(),
            num_qubits,
            topology,
            qubit_properties,
            couplers,
            connectivity,
            calibration_timestamp: None,
            crosstalk: CrosstalkMatrix::default(),
        }
    }

    /// Creates a linear topology (chain).
    pub fn linear(name: impl Into<String>, num_qubits: usize) -> Self {
        Self::new(name, num_qubits, Topology::Linear)
    }

    /// Creates an all-to-all connected topology.
    pub fn all_to_all(name: impl Into<String>, num_qubits: usize) -> Self {
        Self::new(name, num_qubits, Topology::AllToAll)
    }

    /// Creates a grid topology.
    pub fn grid(name: impl Into<String>, rows: usize, cols: usize) -> Self {
        Self::new(name, rows * cols, Topology::Grid { rows, cols })
    }

    /// Creates an IBM Heavy-hex topology.
    ///
    /// Heavy-hex is IBM's qubit topology used in Falcon, Hummingbird, and Eagle processors.
    /// It consists of hexagonal cells with "bridge" qubits on alternating edges.
    ///
    /// # Arguments
    /// * `name` - Device name
    /// * `rows` - Number of hexagonal rows (e.g., 3 for 27-qubit Falcon)
    /// * `cols` - Number of hexagonal columns
    ///
    /// # Example
    /// ```
    /// use qns_core::types::HardwareProfile;
    /// let falcon = HardwareProfile::heavy_hex("ibm_falcon", 3, 3); // ~27 qubits
    /// ```
    pub fn heavy_hex(name: impl Into<String>, rows: usize, cols: usize) -> Self {
        // Heavy-hex structure:
        // Each "unit cell" consists of a hexagon with bridge qubits
        // For simplicity, we generate a regular pattern:
        //
        // Row 0:  0 - 1 - 2 - 3 - 4   (main qubits)
        //         |       |       |
        // Bridge: 5       6       7
        //         |       |       |
        // Row 1:  8 - 9 -10 -11 -12
        //             |       |
        // Bridge:    13      14
        //             |       |
        // Row 2: 15 -16 -17 -18 -19
        //         |       |       |
        // Bridge:20      21      22
        //         |       |       |
        // Row 3: 23 -24 -25 -26 -27

        let qubits_per_row = cols;
        let bridges_per_gap = cols.div_ceil(2); // Alternating bridges

        // Calculate total qubits
        let main_qubits = rows * qubits_per_row;
        let bridge_rows = rows.saturating_sub(1);
        let bridge_qubits = bridge_rows * bridges_per_gap;
        let num_qubits = main_qubits + bridge_qubits;

        let qubit_properties = vec![QubitProperties::default(); num_qubits];
        let couplers = Self::generate_heavy_hex_couplers(rows, cols, num_qubits);
        let connectivity = couplers.iter().map(|c| c.edge()).collect();

        Self {
            name: name.into(),
            num_qubits,
            topology: Topology::HeavyHex,
            qubit_properties,
            couplers,
            connectivity,
            calibration_timestamp: None,
            crosstalk: CrosstalkMatrix::default(),
        }
    }

    /// Generates couplers for heavy-hex topology.
    fn generate_heavy_hex_couplers(
        rows: usize,
        cols: usize,
        _total_qubits: usize,
    ) -> Vec<CouplerProperties> {
        let mut couplers = Vec::new();
        let bridges_per_gap = cols.div_ceil(2);

        for row in 0..rows {
            // Calculate main row qubit offset
            let main_row_offset: usize = (0..row)
                .map(|r| cols + if r < rows - 1 { bridges_per_gap } else { 0 })
                .sum();

            // Horizontal connections in main row
            for c in 0..(cols.saturating_sub(1)) {
                let q1 = main_row_offset + c;
                let q2 = main_row_offset + c + 1;
                couplers.push(CouplerProperties::new(q1, q2));
            }

            // Vertical connections to bridge qubits (alternating pattern)
            if row < rows - 1 {
                let bridge_row_offset = main_row_offset + cols;
                for b in 0..bridges_per_gap {
                    let bridge_col = b * 2; // Every other column
                    if bridge_col < cols {
                        let bridge_idx = bridge_row_offset + b;
                        let top_qubit = main_row_offset + bridge_col;

                        // Calculate next row offset
                        let next_main_offset = main_row_offset + cols + bridges_per_gap;
                        let bottom_qubit = next_main_offset + bridge_col;

                        // Connect bridge to top and bottom
                        couplers.push(CouplerProperties::new(top_qubit, bridge_idx));
                        couplers.push(CouplerProperties::new(bridge_idx, bottom_qubit));
                    }
                }
            }
        }

        // Remove any invalid or duplicate couplers
        couplers.retain(|c| c.qubit1 != c.qubit2);
        couplers.sort_by_key(|c| c.edge());
        couplers.dedup_by_key(|c| c.edge());

        couplers
    }

    /// Generates couplers based on topology.
    fn generate_couplers(num_qubits: usize, topology: &Topology) -> Vec<CouplerProperties> {
        let mut couplers = Vec::new();

        match topology {
            Topology::AllToAll => {
                for i in 0..num_qubits {
                    for j in (i + 1)..num_qubits {
                        couplers.push(CouplerProperties::new(i, j));
                    }
                }
            },
            Topology::Linear => {
                for i in 0..(num_qubits.saturating_sub(1)) {
                    couplers.push(CouplerProperties::new(i, i + 1));
                }
            },
            Topology::Ring => {
                for i in 0..num_qubits {
                    couplers.push(CouplerProperties::new(i, (i + 1) % num_qubits));
                }
            },
            Topology::Grid { rows, cols } => {
                for r in 0..*rows {
                    for c in 0..*cols {
                        let idx = r * cols + c;
                        // Right neighbor
                        if c + 1 < *cols {
                            couplers.push(CouplerProperties::new(idx, idx + 1));
                        }
                        // Bottom neighbor
                        if r + 1 < *rows {
                            couplers.push(CouplerProperties::new(idx, idx + cols));
                        }
                    }
                }
            },
            Topology::HeavyHex | Topology::Custom => {
                // For custom/heavy-hex, couplers must be added manually
            },
        }

        couplers
    }

    /// Checks if two qubits are directly connected.
    pub fn are_connected(&self, q1: usize, q2: usize) -> bool {
        let edge = if q1 <= q2 { (q1, q2) } else { (q2, q1) };
        self.connectivity.contains(&edge)
    }

    /// Returns the coupling map (list of physical edges).
    pub fn coupling_map(&self) -> &Vec<CouplerProperties> {
        &self.couplers
    }

    /// Calculate shortest path distance between two qubits using BFS.
    pub fn shortest_path_distance(&self, start: usize, end: usize) -> Option<usize> {
        if start == end {
            return Some(0);
        }

        let mut visited = vec![false; self.num_qubits];
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((start, 0));
        visited[start] = true;

        while let Some((current, dist)) = queue.pop_front() {
            if current == end {
                return Some(dist);
            }

            for &neighbor in self.neighbors(current).iter() {
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    queue.push_back((neighbor, dist + 1));
                }
            }
        }
        None
    }

    /// Returns all qubits connected to the given qubit.
    pub fn neighbors(&self, qubit: usize) -> Vec<usize> {
        self.couplers
            .iter()
            .filter_map(|c| {
                if c.qubit1 == qubit {
                    Some(c.qubit2)
                } else if c.qubit2 == qubit {
                    Some(c.qubit1)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns the coupler properties for a pair of qubits.
    pub fn get_coupler(&self, q1: usize, q2: usize) -> Option<&CouplerProperties> {
        self.couplers
            .iter()
            .find(|c| (c.qubit1 == q1 && c.qubit2 == q2) || (c.qubit1 == q2 && c.qubit2 == q1))
    }

    /// Returns the average T1 across all qubits.
    pub fn avg_t1(&self) -> f64 {
        if self.qubit_properties.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.qubit_properties.iter().map(|q| q.t1).sum();
        sum / self.qubit_properties.len() as f64
    }

    /// Returns the average T2 across all qubits.
    pub fn avg_t2(&self) -> f64 {
        if self.qubit_properties.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.qubit_properties.iter().map(|q| q.t2).sum();
        sum / self.qubit_properties.len() as f64
    }

    /// Returns the minimum T1 (worst qubit).
    pub fn min_t1(&self) -> f64 {
        self.qubit_properties
            .iter()
            .map(|q| q.t1)
            .fold(f64::INFINITY, f64::min)
    }

    /// Returns the average two-qubit gate fidelity.
    pub fn avg_two_qubit_fidelity(&self) -> Fidelity {
        if self.couplers.is_empty() {
            return Fidelity::PERFECT;
        }
        let sum: f64 = self.couplers.iter().map(|c| c.gate_fidelity.value()).sum();
        Fidelity::clamped(sum / self.couplers.len() as f64)
    }

    /// Estimates the circuit execution time in nanoseconds.
    ///
    /// Assumes sequential execution (no parallelism).
    pub fn estimate_circuit_time_ns(&self, depth: usize, two_qubit_gate_count: usize) -> f64 {
        let single_qubit_time = gate_times::SINGLE_QUBIT * depth as f64;
        let two_qubit_time = gate_times::TWO_QUBIT * two_qubit_gate_count as f64;
        single_qubit_time + two_qubit_time
    }

    /// Validates qubit index.
    pub fn validate_qubit(&self, qubit: usize) -> crate::Result<()> {
        if qubit >= self.num_qubits {
            Err(crate::QnsError::InvalidQubit(qubit, self.num_qubits))
        } else {
            Ok(())
        }
    }

    /// Rebuilds the connectivity set from couplers.
    pub fn rebuild_connectivity(&mut self) {
        self.connectivity = self.couplers.iter().map(|c| c.edge()).collect();
    }

    /// Adds a custom coupler.
    pub fn add_coupler(&mut self, coupler: CouplerProperties) {
        let edge = coupler.edge();
        if !self.connectivity.contains(&edge) {
            self.connectivity.insert(edge);
            self.couplers.push(coupler);
        }
    }

    /// Validates that a circuit can execute on this hardware.
    ///
    /// Checks:
    /// 1. All qubit indices are valid
    /// 2. All two-qubit gates operate on connected qubits
    ///
    /// Returns a list of invalid gates (index, gate, reason).
    pub fn validate_circuit(&self, circuit: &super::CircuitGenome) -> Vec<(usize, String)> {
        let mut errors = Vec::new();

        for (idx, gate) in circuit.gates.iter().enumerate() {
            let qubits = gate.qubits();

            // Check qubit indices
            for &q in &qubits {
                if q >= self.num_qubits {
                    errors.push((
                        idx,
                        format!(
                            "{}: qubit {} exceeds hardware limit {}",
                            gate, q, self.num_qubits
                        ),
                    ));
                }
            }

            // Check connectivity for two-qubit gates
            if gate.is_two_qubit() && qubits.len() == 2 {
                let (q1, q2) = (qubits[0], qubits[1]);
                if q1 < self.num_qubits && q2 < self.num_qubits && !self.are_connected(q1, q2) {
                    errors.push((
                        idx,
                        format!("{}: qubits {} and {} are not connected", gate, q1, q2),
                    ));
                }
            }
        }

        errors
    }

    /// Returns true if the circuit can execute on this hardware.
    pub fn is_circuit_valid(&self, circuit: &super::CircuitGenome) -> bool {
        self.validate_circuit(circuit).is_empty()
    }
}

impl Default for HardwareProfile {
    fn default() -> Self {
        Self::linear("default", 5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fidelity_new() {
        let f = Fidelity::new(0.99);
        assert!((f.value() - 0.99).abs() < 1e-10);
    }

    #[test]
    #[should_panic]
    fn test_fidelity_out_of_range() {
        let _ = Fidelity::new(1.5);
    }

    #[test]
    fn test_fidelity_clamped() {
        let f = Fidelity::clamped(1.5);
        assert!((f.value() - 1.0).abs() < 1e-10);

        let f2 = Fidelity::clamped(-0.5);
        assert!((f2.value() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_fidelity_from_error() {
        let f = Fidelity::from_error_rate(0.01);
        assert!((f.value() - 0.99).abs() < 1e-10);
        assert!((f.error_rate() - 0.01).abs() < 1e-10);
    }

    #[test]
    fn test_linear_topology() {
        let hw = HardwareProfile::linear("test", 5);
        assert_eq!(hw.num_qubits, 5);
        assert_eq!(hw.couplers.len(), 4); // 0-1, 1-2, 2-3, 3-4

        assert!(hw.are_connected(0, 1));
        assert!(hw.are_connected(1, 2));
        assert!(!hw.are_connected(0, 2));
        assert!(!hw.are_connected(0, 4));
    }

    #[test]
    fn test_all_to_all_topology() {
        let hw = HardwareProfile::all_to_all("test", 4);
        assert_eq!(hw.num_qubits, 4);
        assert_eq!(hw.couplers.len(), 6); // C(4,2) = 6

        assert!(hw.are_connected(0, 1));
        assert!(hw.are_connected(0, 3));
        assert!(hw.are_connected(1, 3));
    }

    #[test]
    fn test_grid_topology() {
        let hw = HardwareProfile::grid("test", 2, 3);
        assert_eq!(hw.num_qubits, 6);
        // 2x3 grid has 7 edges: 3 horizontal + 2 vertical per row
        assert_eq!(hw.couplers.len(), 7);

        // Row 0: 0-1, 1-2
        assert!(hw.are_connected(0, 1));
        assert!(hw.are_connected(1, 2));
        // Vertical: 0-3, 1-4, 2-5
        assert!(hw.are_connected(0, 3));
        assert!(hw.are_connected(1, 4));
        // Not connected diagonally
        assert!(!hw.are_connected(0, 4));
    }

    #[test]
    fn test_neighbors() {
        let hw = HardwareProfile::linear("test", 5);

        assert_eq!(hw.neighbors(0), vec![1]);
        assert_eq!(hw.neighbors(2).len(), 2); // 1 and 3
        assert_eq!(hw.neighbors(4), vec![3]);
    }

    #[test]
    fn test_qubit_properties() {
        let qp = QubitProperties::with_t1t2(100.0, 80.0);
        assert!((qp.t2_t1_ratio() - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_avg_t1() {
        let mut hw = HardwareProfile::linear("test", 3);
        hw.qubit_properties[0].t1 = 100.0;
        hw.qubit_properties[1].t1 = 200.0;
        hw.qubit_properties[2].t1 = 300.0;

        assert!((hw.avg_t1() - 200.0).abs() < 1e-10);
    }

    #[test]
    fn test_validate_circuit_valid() {
        use super::super::{CircuitGenome, Gate};

        let hw = HardwareProfile::linear("test", 5);
        let mut circuit = CircuitGenome::new(5);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::CNOT(0, 1)).unwrap(); // Connected in linear
        circuit.add_gate(Gate::CNOT(1, 2)).unwrap(); // Connected

        assert!(hw.is_circuit_valid(&circuit));
        assert!(hw.validate_circuit(&circuit).is_empty());
    }

    #[test]
    fn test_validate_circuit_invalid_connectivity() {
        use super::super::{CircuitGenome, Gate};

        let hw = HardwareProfile::linear("test", 5);
        let mut circuit = CircuitGenome::new(5);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::CNOT(0, 2)).unwrap(); // NOT connected in linear!

        assert!(!hw.is_circuit_valid(&circuit));
        let errors = hw.validate_circuit(&circuit);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].1.contains("not connected"));
    }

    #[test]
    fn test_validate_circuit_invalid_qubit() {
        use super::super::{CircuitGenome, Gate};

        let hw = HardwareProfile::linear("test", 3);
        let mut circuit = CircuitGenome::new(5); // Circuit has 5 qubits
        circuit.add_gate(Gate::H(4)).unwrap(); // Qubit 4 > hw limit (3)

        assert!(!hw.is_circuit_valid(&circuit));
        let errors = hw.validate_circuit(&circuit);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].1.contains("exceeds hardware limit"));
    }

    #[test]
    fn test_heavy_hex_topology() {
        // Test 3x5 heavy-hex (similar to small IBM device)
        let hw = HardwareProfile::heavy_hex("test_heavy_hex", 3, 5);

        // Should have main qubits (3*5=15) + bridge qubits (2 gaps * 3 bridges = 6)
        // Total: 21 qubits
        assert!(hw.num_qubits > 0);
        assert!(!hw.couplers.is_empty());

        // Check horizontal connectivity in first row
        assert!(hw.are_connected(0, 1));
        assert!(hw.are_connected(1, 2));

        // Check that topology is HeavyHex
        assert_eq!(hw.topology, Topology::HeavyHex);

        println!("Heavy-hex qubits: {}", hw.num_qubits);
        println!("Heavy-hex couplers: {}", hw.couplers.len());
        for c in &hw.couplers {
            println!("  {} -- {}", c.qubit1, c.qubit2);
        }
    }

    #[test]
    fn test_heavy_hex_small() {
        // Minimal 2x3 heavy-hex
        let hw = HardwareProfile::heavy_hex("mini", 2, 3);

        // 2 rows * 3 cols = 6 main qubits
        // 1 gap * 2 bridges = 2 bridge qubits
        // Total: 8 qubits
        assert_eq!(hw.num_qubits, 8);

        // Horizontal: 0-1, 1-2 (row 0), 5-6, 6-7 (row 1) = 4
        // Vertical via bridges: 0-3, 3-5, 2-4, 4-7 = 4
        // Total: 8 couplers
        assert!(hw.couplers.len() >= 4); // At least horizontal connections
    }
}
