"""
QNS - Quantum Noise Symbiote

Noise-aware quantum circuit optimization library.

Quick Start:
    >>> from qns import Circuit, QnsOptimizer, NoiseModel
    >>>
    >>> # Create a circuit
    >>> circuit = Circuit(num_qubits=3)
    >>> circuit.h(0)
    >>> circuit.cnot(0, 1)
    >>> circuit.cnot(1, 2)
    >>>
    >>> # Optimize with noise awareness
    >>> noise = NoiseModel(t1=100.0, t2=80.0)
    >>> optimizer = QnsOptimizer(num_qubits=3, noise_model=noise)
    >>> result = optimizer.optimize(circuit)
    >>> print(f"Improvement: {result.score_improvement:.2%}")

For Qiskit users:
    >>> from qiskit import QuantumCircuit
    >>> from qns import Circuit
    >>> from qns.convert import circuit_to_qasm, circuit_from_qasm
    >>>
    >>> # Convert from Qiskit
    >>> qc = QuantumCircuit(3)
    >>> qc.h(0)
    >>> qc.cx(0, 1)
    >>> qasm = qc.qasm()
    >>> circuit = circuit_from_qasm(qasm)

For IBM Quantum:
    >>> from qns.ibm import IBMBackend, run_benchmark_suite
    >>>
    >>> # Use fake backend for testing
    >>> backend = IBMBackend.from_fake("manila")
    >>> result = backend.run_optimized(circuit)
    >>>
    >>> # Use real hardware
    >>> backend = IBMBackend.from_service("ibm_brisbane", token="...")
"""

# Re-export from Rust extension
from .qns import (
    # Version
    __version__,
    __author__,
    
    # Core types
    Gate,
    Circuit,
    NoiseVector,
    NoiseModel,
    HardwareProfile,
    
    # Optimizer
    QnsOptimizer,
    OptimizationResult,
    
    # Backend
    SimulatorBackend,
    ExecutionResult,
    CalibrationData,
    
    # Utility functions
    estimate_circuit_fidelity,
    score_circuit,
    
    # Submodules
    convert,
)

__all__ = [
    # Version
    "__version__",
    "__author__",
    
    # Core types
    "Gate",
    "Circuit",
    "NoiseVector",
    "NoiseModel",
    "HardwareProfile",
    
    # Optimizer
    "QnsOptimizer",
    "OptimizationResult",
    
    # Backend
    "SimulatorBackend",
    "ExecutionResult",
    "CalibrationData",
    
    # Utility functions
    "estimate_circuit_fidelity",
    "score_circuit",
    
    # Submodules
    "convert",
    "ibm",
]

# Lazy import for ibm module to avoid hard dependency on qiskit-ibm-runtime
def __getattr__(name):
    if name == "ibm":
        from . import ibm as _ibm
        return _ibm
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")
