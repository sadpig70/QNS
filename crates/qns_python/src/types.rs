//! Core type bindings for Python.
//!
//! This module provides Python wrappers for QNS core types:
//! - PyGate: Quantum gate operations
//! - PyCircuit: Quantum circuit representation
//! - PyNoiseVector: Qubit noise parameters
//! - PyNoiseModel: Simulator noise configuration
//! - PyHardwareProfile: Hardware topology

use pyo3::prelude::*;
use pyo3::exceptions::{PyValueError, PyIndexError};

use qns_core::prelude::*;
use qns_simulator::NoiseModel;

// ============================================================================
// PyGate - Quantum Gate
// ============================================================================

/// Quantum gate representation.
///
/// Supported gates:
/// - Single-qubit: H, X, Y, Z, S, T, Rx, Ry, Rz
/// - Two-qubit: CNOT, CZ, SWAP
/// - Measurement: Measure
///
/// Example:
///     >>> gate = Gate.h(0)  # Hadamard on qubit 0
///     >>> gate = Gate.cnot(0, 1)  # CNOT with control=0, target=1
///     >>> gate = Gate.rx(0, 3.14159)  # Rx rotation
#[pyclass(name = "Gate")]
#[derive(Clone)]
pub struct PyGate {
    inner: Gate,
}

#[pymethods]
impl PyGate {
    /// Creates a Hadamard gate.
    #[staticmethod]
    fn h(qubit: usize) -> Self {
        Self { inner: Gate::H(qubit) }
    }
    
    /// Creates a Pauli-X (NOT) gate.
    #[staticmethod]
    fn x(qubit: usize) -> Self {
        Self { inner: Gate::X(qubit) }
    }
    
    /// Creates a Pauli-Y gate.
    #[staticmethod]
    fn y(qubit: usize) -> Self {
        Self { inner: Gate::Y(qubit) }
    }
    
    /// Creates a Pauli-Z gate.
    #[staticmethod]
    fn z(qubit: usize) -> Self {
        Self { inner: Gate::Z(qubit) }
    }
    
    /// Creates an S (√Z) gate.
    #[staticmethod]
    fn s(qubit: usize) -> Self {
        Self { inner: Gate::S(qubit) }
    }
    
    /// Creates a T (π/8) gate.
    #[staticmethod]
    fn t(qubit: usize) -> Self {
        Self { inner: Gate::T(qubit) }
    }
    
    /// Creates an Rx rotation gate.
    #[staticmethod]
    fn rx(qubit: usize, theta: f64) -> Self {
        Self { inner: Gate::Rx(qubit, theta) }
    }
    
    /// Creates an Ry rotation gate.
    #[staticmethod]
    fn ry(qubit: usize, theta: f64) -> Self {
        Self { inner: Gate::Ry(qubit, theta) }
    }
    
    /// Creates an Rz rotation gate.
    #[staticmethod]
    fn rz(qubit: usize, theta: f64) -> Self {
        Self { inner: Gate::Rz(qubit, theta) }
    }
    
    /// Creates a CNOT (CX) gate.
    #[staticmethod]
    fn cnot(control: usize, target: usize) -> Self {
        Self { inner: Gate::CNOT(control, target) }
    }
    
    /// Creates a CZ gate.
    #[staticmethod]
    fn cz(control: usize, target: usize) -> Self {
        Self { inner: Gate::CZ(control, target) }
    }
    
    /// Creates a SWAP gate.
    #[staticmethod]
    fn swap(qubit1: usize, qubit2: usize) -> Self {
        Self { inner: Gate::SWAP(qubit1, qubit2) }
    }
    
    /// Creates a Measure gate.
    #[staticmethod]
    fn measure(qubit: usize) -> Self {
        Self { inner: Gate::Measure(qubit) }
    }
    
    /// Returns the gate name.
    #[getter]
    fn name(&self) -> String {
        format!("{:?}", self.inner).split('(').next().unwrap_or("Unknown").to_string()
    }
    
    /// Returns the qubits this gate acts on.
    #[getter]
    fn qubits(&self) -> Vec<usize> {
        self.inner.qubits()
    }
    
    /// Returns True if this is a two-qubit gate.
    #[getter]
    fn is_two_qubit(&self) -> bool {
        self.inner.is_two_qubit()
    }
    
    /// Returns the rotation angle if applicable.
    #[getter]
    fn angle(&self) -> Option<f64> {
        self.inner.rotation_angle()
    }
    
    /// Returns the estimated error rate.
    #[getter]
    fn estimated_error(&self) -> f64 {
        self.inner.estimated_error()
    }
    
    /// Checks if this gate commutes with another.
    fn commutes_with(&self, other: &PyGate) -> bool {
        self.inner.commutes_with(&other.inner)
    }
    
    fn __repr__(&self) -> String {
        format!("{:?}", self.inner)
    }
    
    fn __str__(&self) -> String {
        self.__repr__()
    }
}

impl PyGate {
    pub fn inner(&self) -> &Gate {
        &self.inner
    }
    
    pub fn from_gate(gate: Gate) -> Self {
        Self { inner: gate }
    }
}

// ============================================================================
// PyCircuit - Quantum Circuit
// ============================================================================

/// Quantum circuit representation.
///
/// A circuit is a sequence of quantum gates applied to a set of qubits.
///
/// Example:
///     >>> circuit = Circuit(num_qubits=3)
///     >>> circuit.h(0)
///     >>> circuit.cnot(0, 1)
///     >>> circuit.cnot(1, 2)
///     >>> print(f"Depth: {circuit.depth}, Gates: {len(circuit)}")
#[pyclass(name = "Circuit")]
#[derive(Clone)]
pub struct PyCircuit {
    inner: CircuitGenome,
}

#[pymethods]
impl PyCircuit {
    /// Creates a new circuit with the specified number of qubits.
    #[new]
    fn new(num_qubits: usize) -> Self {
        Self {
            inner: CircuitGenome::new(num_qubits),
        }
    }
    
    /// Number of qubits in the circuit.
    #[getter]
    fn num_qubits(&self) -> usize {
        self.inner.num_qubits
    }
    
    /// Number of gates in the circuit.
    fn __len__(&self) -> usize {
        self.inner.gates.len()
    }
    
    /// Circuit depth (longest path through the circuit).
    #[getter]
    fn depth(&self) -> usize {
        self.inner.depth()
    }
    
    /// Number of two-qubit gates.
    #[getter]
    fn two_qubit_count(&self) -> usize {
        self.inner.gates.iter().filter(|g| g.is_two_qubit()).count()
    }
    
    /// Adds a gate to the circuit.
    fn add_gate(&mut self, gate: &PyGate) -> PyResult<()> {
        self.inner.add_gate(gate.inner.clone())
            .map_err(|e| PyValueError::new_err(format!("{}", e)))
    }
    
    /// Adds a Hadamard gate.
    fn h(&mut self, qubit: usize) -> PyResult<()> {
        self.inner.add_gate(Gate::H(qubit))
            .map_err(|e| PyValueError::new_err(format!("{}", e)))
    }
    
    /// Adds a Pauli-X gate.
    fn x(&mut self, qubit: usize) -> PyResult<()> {
        self.inner.add_gate(Gate::X(qubit))
            .map_err(|e| PyValueError::new_err(format!("{}", e)))
    }
    
    /// Adds a Pauli-Y gate.
    fn y(&mut self, qubit: usize) -> PyResult<()> {
        self.inner.add_gate(Gate::Y(qubit))
            .map_err(|e| PyValueError::new_err(format!("{}", e)))
    }
    
    /// Adds a Pauli-Z gate.
    fn z(&mut self, qubit: usize) -> PyResult<()> {
        self.inner.add_gate(Gate::Z(qubit))
            .map_err(|e| PyValueError::new_err(format!("{}", e)))
    }
    
    /// Adds an S gate.
    fn s(&mut self, qubit: usize) -> PyResult<()> {
        self.inner.add_gate(Gate::S(qubit))
            .map_err(|e| PyValueError::new_err(format!("{}", e)))
    }
    
    /// Adds a T gate.
    fn t(&mut self, qubit: usize) -> PyResult<()> {
        self.inner.add_gate(Gate::T(qubit))
            .map_err(|e| PyValueError::new_err(format!("{}", e)))
    }
    
    /// Adds an Rx rotation gate.
    fn rx(&mut self, qubit: usize, theta: f64) -> PyResult<()> {
        self.inner.add_gate(Gate::Rx(qubit, theta))
            .map_err(|e| PyValueError::new_err(format!("{}", e)))
    }
    
    /// Adds an Ry rotation gate.
    fn ry(&mut self, qubit: usize, theta: f64) -> PyResult<()> {
        self.inner.add_gate(Gate::Ry(qubit, theta))
            .map_err(|e| PyValueError::new_err(format!("{}", e)))
    }
    
    /// Adds an Rz rotation gate.
    fn rz(&mut self, qubit: usize, theta: f64) -> PyResult<()> {
        self.inner.add_gate(Gate::Rz(qubit, theta))
            .map_err(|e| PyValueError::new_err(format!("{}", e)))
    }
    
    /// Adds a CNOT gate.
    fn cnot(&mut self, control: usize, target: usize) -> PyResult<()> {
        self.inner.add_gate(Gate::CNOT(control, target))
            .map_err(|e| PyValueError::new_err(format!("{}", e)))
    }
    
    /// Adds a CZ gate.
    fn cz(&mut self, control: usize, target: usize) -> PyResult<()> {
        self.inner.add_gate(Gate::CZ(control, target))
            .map_err(|e| PyValueError::new_err(format!("{}", e)))
    }
    
    /// Adds a SWAP gate.
    fn swap(&mut self, qubit1: usize, qubit2: usize) -> PyResult<()> {
        self.inner.add_gate(Gate::SWAP(qubit1, qubit2))
            .map_err(|e| PyValueError::new_err(format!("{}", e)))
    }
    
    /// Adds a measurement gate.
    fn measure(&mut self, qubit: usize) -> PyResult<()> {
        self.inner.add_gate(Gate::Measure(qubit))
            .map_err(|e| PyValueError::new_err(format!("{}", e)))
    }
    
    /// Measures all qubits.
    fn measure_all(&mut self) -> PyResult<()> {
        for i in 0..self.inner.num_qubits {
            self.inner.add_gate(Gate::Measure(i))
                .map_err(|e| PyValueError::new_err(format!("{}", e)))?;
        }
        Ok(())
    }
    
    /// Returns the list of gates.
    #[getter]
    fn gates(&self) -> Vec<PyGate> {
        self.inner.gates.iter().map(|g| PyGate::from_gate(g.clone())).collect()
    }
    
    /// Gets a gate by index.
    fn __getitem__(&self, index: usize) -> PyResult<PyGate> {
        self.inner.gates.get(index)
            .map(|g| PyGate::from_gate(g.clone()))
            .ok_or_else(|| PyIndexError::new_err("Gate index out of range"))
    }
    
    /// Creates a copy of the circuit.
    fn copy(&self) -> Self {
        self.clone()
    }
    
    /// Clears all gates from the circuit.
    fn clear(&mut self) {
        self.inner.gates.clear();
    }
    
    /// Converts to JSON string.
    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner)
            .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))
    }
    
    /// Creates a circuit from JSON string.
    #[staticmethod]
    fn from_json(json_str: &str) -> PyResult<Self> {
        let inner: CircuitGenome = serde_json::from_str(json_str)
            .map_err(|e| PyValueError::new_err(format!("Deserialization error: {}", e)))?;
        Ok(Self { inner })
    }
    
    fn __repr__(&self) -> String {
        format!("Circuit(num_qubits={}, gates={})", self.inner.num_qubits, self.inner.gates.len())
    }
}

impl PyCircuit {
    pub fn inner(&self) -> &CircuitGenome {
        &self.inner
    }
    
    pub fn inner_mut(&mut self) -> &mut CircuitGenome {
        &mut self.inner
    }
    
    pub fn from_genome(genome: CircuitGenome) -> Self {
        Self { inner: genome }
    }
    
    pub fn into_inner(self) -> CircuitGenome {
        self.inner
    }
}

// ============================================================================
// PyNoiseVector - Qubit Noise Parameters
// ============================================================================

/// Noise parameters for a single qubit.
///
/// Contains T1/T2 coherence times and error rates.
///
/// Example:
///     >>> noise = NoiseVector(qubit_id=0, t1=100.0, t2=80.0)
///     >>> print(f"T1: {noise.t1} μs, T2: {noise.t2} μs")
///     >>> print(f"T_phi: {noise.t_phi} μs")
#[pyclass(name = "NoiseVector")]
#[derive(Clone)]
pub struct PyNoiseVector {
    inner: NoiseVector,
}

#[pymethods]
impl PyNoiseVector {
    /// Creates a new NoiseVector.
    ///
    /// Args:
    ///     qubit_id: Qubit identifier
    ///     t1: T1 relaxation time in μs (default: 100.0)
    ///     t2: T2 dephasing time in μs (default: 80.0)
    ///     gate_error_1q: Single-qubit gate error (default: 0.001)
    ///     gate_error_2q: Two-qubit gate error (default: 0.01)
    ///     readout_error: Measurement error (default: 0.01)
    #[new]
    #[pyo3(signature = (qubit_id=0, t1=100.0, t2=80.0, gate_error_1q=0.001, gate_error_2q=0.01, readout_error=0.01))]
    fn new(
        qubit_id: usize,
        t1: f64,
        t2: f64,
        gate_error_1q: f64,
        gate_error_2q: f64,
        readout_error: f64,
    ) -> Self {
        Self {
            inner: NoiseVector::comprehensive(qubit_id, t1, t2, gate_error_1q, gate_error_2q, readout_error),
        }
    }
    
    /// Qubit identifier.
    #[getter]
    fn qubit_id(&self) -> usize {
        self.inner.qubit_id
    }
    
    /// T1 relaxation time in μs.
    #[getter]
    fn t1(&self) -> f64 {
        self.inner.t1_mean
    }
    
    #[setter]
    fn set_t1(&mut self, value: f64) {
        self.inner.t1_mean = value;
    }
    
    /// T2 dephasing time in μs.
    #[getter]
    fn t2(&self) -> f64 {
        self.inner.t2_mean
    }
    
    #[setter]
    fn set_t2(&mut self, value: f64) {
        self.inner.t2_mean = value;
    }
    
    /// Pure dephasing time T_phi in μs.
    /// Returns None if T2 >= 2*T1.
    #[getter]
    fn t_phi(&self) -> Option<f64> {
        self.inner.t_phi()
    }
    
    /// Single-qubit gate error rate.
    #[getter]
    fn gate_error_1q(&self) -> f64 {
        self.inner.gate_error_1q
    }
    
    #[setter]
    fn set_gate_error_1q(&mut self, value: f64) {
        self.inner.gate_error_1q = value;
    }
    
    /// Two-qubit gate error rate.
    #[getter]
    fn gate_error_2q(&self) -> f64 {
        self.inner.gate_error_2q
    }
    
    #[setter]
    fn set_gate_error_2q(&mut self, value: f64) {
        self.inner.gate_error_2q = value;
    }
    
    /// Readout (measurement) error rate.
    #[getter]
    fn readout_error(&self) -> f64 {
        self.inner.readout_error
    }
    
    #[setter]
    fn set_readout_error(&mut self, value: f64) {
        self.inner.readout_error = value;
    }
    
    /// T2/T1 ratio (should be <= 2.0 for physical systems).
    #[getter]
    fn t2_t1_ratio(&self) -> f64 {
        self.inner.t2_t1_ratio()
    }
    
    /// Validates physical constraints.
    fn validate(&self) -> PyResult<()> {
        self.inner.validate()
            .map_err(|e| PyValueError::new_err(e))
    }
    
    /// Estimates gate fidelity.
    fn estimate_gate_fidelity(&self, gate_time_ns: f64, is_two_qubit: bool) -> f64 {
        self.inner.estimate_gate_fidelity(gate_time_ns, is_two_qubit)
    }
    
    fn __repr__(&self) -> String {
        format!(
            "NoiseVector(qubit_id={}, t1={:.1}, t2={:.1}, gate_error_1q={:.4}, gate_error_2q={:.4})",
            self.inner.qubit_id, self.inner.t1_mean, self.inner.t2_mean,
            self.inner.gate_error_1q, self.inner.gate_error_2q
        )
    }
}

impl PyNoiseVector {
    pub fn inner(&self) -> &NoiseVector {
        &self.inner
    }
    
    pub fn from_noise_vector(nv: NoiseVector) -> Self {
        Self { inner: nv }
    }
}

// ============================================================================
// PyNoiseModel - Simulator Noise Configuration
// ============================================================================

/// Noise model for quantum simulation.
///
/// Configures T1/T2 decoherence, gate errors, and measurement errors.
///
/// Example:
///     >>> noise = NoiseModel(t1=100.0, t2=80.0)
///     >>> noise.gate_error_1q = 0.001
///     >>> noise.readout_error = 0.02
#[pyclass(name = "NoiseModel")]
#[derive(Clone)]
pub struct PyNoiseModel {
    inner: NoiseModel,
}

#[pymethods]
impl PyNoiseModel {
    /// Creates a new NoiseModel.
    ///
    /// Args:
    ///     t1: T1 relaxation time in μs (default: 100.0)
    ///     t2: T2 dephasing time in μs (default: 80.0)
    ///     gate_error_1q: Single-qubit gate error (default: 0.001)
    ///     gate_error_2q: Two-qubit gate error (default: 0.01)
    ///     readout_error: Measurement error (default: 0.01)
    #[new]
    #[pyo3(signature = (t1=100.0, t2=80.0, gate_error_1q=0.001, gate_error_2q=0.01, readout_error=0.01))]
    fn new(t1: f64, t2: f64, gate_error_1q: f64, gate_error_2q: f64, readout_error: f64) -> Self {
        let inner = NoiseModel::with_t1t2(t1, t2)
            .with_gate_errors(gate_error_1q, gate_error_2q)
            .with_readout_error(readout_error);
        Self { inner }
    }
    
    /// Creates an ideal (noise-free) model.
    #[staticmethod]
    fn ideal() -> Self {
        Self { inner: NoiseModel::ideal() }
    }
    
    /// T1 relaxation time in μs.
    #[getter]
    fn t1(&self) -> f64 {
        self.inner.t1()
    }
    
    /// T2 dephasing time in μs.
    #[getter]
    fn t2(&self) -> f64 {
        self.inner.t2()
    }
    
    /// Single-qubit gate error rate.
    #[getter]
    fn gate_error_1q(&self) -> f64 {
        self.inner.gate_error_1q()
    }
    
    /// Two-qubit gate error rate.
    #[getter]
    fn gate_error_2q(&self) -> f64 {
        self.inner.gate_error_2q()
    }
    
    /// Readout error rate.
    #[getter]
    fn readout_error(&self) -> f64 {
        self.inner.readout_error()
    }
    
    /// Checks if the model is physically valid (T2 <= 2*T1).
    fn is_valid(&self) -> bool {
        self.inner.is_valid()
    }
    
    /// Creates a NoiseVector from this model.
    fn to_noise_vector(&self, qubit_id: usize) -> PyNoiseVector {
        PyNoiseVector::new(
            qubit_id,
            self.inner.t1(),
            self.inner.t2(),
            self.inner.gate_error_1q(),
            self.inner.gate_error_2q(),
            self.inner.readout_error(),
        )
    }
    
    fn __repr__(&self) -> String {
        format!(
            "NoiseModel(t1={:.1}, t2={:.1}, gate_error_1q={:.4}, gate_error_2q={:.4})",
            self.inner.t1(), self.inner.t2(),
            self.inner.gate_error_1q(), self.inner.gate_error_2q()
        )
    }
}

impl PyNoiseModel {
    pub fn inner(&self) -> &NoiseModel {
        &self.inner
    }
    
    pub fn into_inner(self) -> NoiseModel {
        self.inner
    }
}

// ============================================================================
// PyHardwareProfile - Hardware Topology
// ============================================================================

/// Hardware topology and qubit properties.
///
/// Describes the physical connectivity and characteristics of a quantum device.
///
/// Example:
///     >>> hw = HardwareProfile.linear(name="test", num_qubits=5)
///     >>> print(hw.are_connected(0, 1))  # True
///     >>> print(hw.are_connected(0, 2))  # False
#[pyclass(name = "HardwareProfile")]
#[derive(Clone)]
pub struct PyHardwareProfile {
    inner: HardwareProfile,
}

#[pymethods]
impl PyHardwareProfile {
    /// Creates a linear topology (0-1-2-...-n).
    #[staticmethod]
    fn linear(name: &str, num_qubits: usize) -> Self {
        Self {
            inner: HardwareProfile::linear(name, num_qubits),
        }
    }
    
    /// Creates a ring topology.
    #[staticmethod]
    fn ring(name: &str, num_qubits: usize) -> Self {
        Self {
            inner: HardwareProfile::ring(name, num_qubits),
        }
    }
    
    /// Creates a fully connected topology.
    #[staticmethod]
    fn fully_connected(name: &str, num_qubits: usize) -> Self {
        Self {
            inner: HardwareProfile::fully_connected(name, num_qubits),
        }
    }
    
    /// Backend name.
    #[getter]
    fn name(&self) -> &str {
        self.inner.name()
    }
    
    /// Number of qubits.
    #[getter]
    fn num_qubits(&self) -> usize {
        self.inner.num_qubits()
    }
    
    /// Checks if two qubits are directly connected.
    fn are_connected(&self, q1: usize, q2: usize) -> bool {
        self.inner.are_connected(q1, q2)
    }
    
    /// Returns neighbors of a qubit.
    fn neighbors(&self, qubit: usize) -> Vec<usize> {
        self.inner.neighbors(qubit)
    }
    
    /// Returns the coupling map as a list of (q1, q2) tuples.
    fn coupling_map(&self) -> Vec<(usize, usize)> {
        self.inner.edges().collect()
    }
    
    fn __repr__(&self) -> String {
        format!(
            "HardwareProfile(name='{}', num_qubits={}, topology={:?})",
            self.inner.name(), self.inner.num_qubits(), self.inner.topology()
        )
    }
}

impl PyHardwareProfile {
    pub fn inner(&self) -> &HardwareProfile {
        &self.inner
    }
    
    pub fn from_profile(profile: HardwareProfile) -> Self {
        Self { inner: profile }
    }
}
