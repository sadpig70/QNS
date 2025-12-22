use pyo3::prelude::*;
use qns_core::{types::Gate, CircuitGenome};

/// Python wrapper for quantum circuit
#[pyclass]
pub struct QuantumCircuit {
    pub(crate) inner: CircuitGenome,
}

#[pymethods]
impl QuantumCircuit {
    /// Create a new quantum circuit
    ///
    /// Args:
    ///     num_qubits: Number of qubits in the circuit
    #[new]
    fn new(num_qubits: usize) -> Self {
        Self {
            inner: CircuitGenome::new(num_qubits),
        }
    }

    /// Apply Hadamard gate
    ///
    /// Args:
    ///     qubit: Target qubit index
    fn h(&mut self, qubit: usize) -> PyResult<()> {
        self.inner
            .add_gate(Gate::H(qubit))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    /// Apply Pauli-X gate
    ///
    /// Args:
    ///     qubit: Target qubit index
    fn x(&mut self, qubit: usize) -> PyResult<()> {
        self.inner
            .add_gate(Gate::X(qubit))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    /// Apply Pauli-Y gate
    ///
    /// Args:
    ///     qubit: Target qubit index
    fn y(&mut self, qubit: usize) -> PyResult<()> {
        self.inner
            .add_gate(Gate::Y(qubit))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    /// Apply Pauli-Z gate
    ///
    /// Args:
    ///     qubit: Target qubit index
    fn z(&mut self, qubit: usize) -> PyResult<()> {
        self.inner
            .add_gate(Gate::Z(qubit))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    /// Apply CNOT gate
    ///
    /// Args:
    ///     control: Control qubit index
    ///     target: Target qubit index
    fn cx(&mut self, control: usize, target: usize) -> PyResult<()> {
        self.inner
            .add_gate(Gate::CNOT(control, target))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    /// Apply CZ gate
    ///
    /// Args:
    ///     control: Control qubit index
    ///     target: Target qubit index
    fn cz(&mut self, control: usize, target: usize) -> PyResult<()> {
        self.inner
            .add_gate(Gate::CZ(control, target))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    /// Apply RX rotation gate
    ///
    /// Args:
    ///     qubit: Target qubit index
    ///     theta: Rotation angle in radians
    fn rx(&mut self, qubit: usize, theta: f64) -> PyResult<()> {
        self.inner
            .add_gate(Gate::Rx(qubit, theta))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    /// Apply RY rotation gate
    ///
    /// Args:
    ///     qubit: Target qubit index
    ///     theta: Rotation angle in radians
    fn ry(&mut self, qubit: usize, theta: f64) -> PyResult<()> {
        self.inner
            .add_gate(Gate::Ry(qubit, theta))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    /// Apply RZ rotation gate
    ///
    /// Args:
    ///     qubit: Target qubit index
    ///     theta: Rotation angle in radians
    fn rz(&mut self, qubit: usize, theta: f64) -> PyResult<()> {
        self.inner
            .add_gate(Gate::Rz(qubit, theta))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    /// Add measurement to all qubits
    fn measure_all(&mut self) -> PyResult<()> {
        for i in 0..self.inner.num_qubits {
            self.inner
                .add_gate(Gate::Measure(i))
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        }
        Ok(())
    }

    /// Get number of qubits
    #[getter]
    fn num_qubits(&self) -> usize {
        self.inner.num_qubits
    }

    /// Get number of gates
    #[getter]
    fn num_gates(&self) -> usize {
        self.inner.gates.len()
    }

    /// String representation
    fn __repr__(&self) -> String {
        format!(
            "QuantumCircuit({} qubits, {} gates)",
            self.inner.num_qubits,
            self.inner.gates.len()
        )
    }

    /// String representation
    fn __str__(&self) -> String {
        self.__repr__()
    }
}
