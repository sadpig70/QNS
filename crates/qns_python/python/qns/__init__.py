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

# === compatibility: augment compiled extension classes (do NOT wrap) ===
from .qns import (
    __version__, __author__,
    Gate as _Gate,
    Circuit as _Circuit,
    ExecutionResult as _ExecutionResult,
    SimulatorBackend as _SimulatorBackend,
    # other names left as-is
    NoiseVector, NoiseModel, HardwareProfile, QnsOptimizer, OptimizationResult,
    CalibrationData, estimate_circuit_fidelity, score_circuit, convert,
)

# make len(circuit) return number of gates
def _circuit___len__(self):
    for attr in ("gates_count", "num_gates", "n_gates"):
        if hasattr(self, attr):
            try:
                return int(getattr(self, attr))
            except Exception:
                pass
    if hasattr(self, "gates"):
        try:
            return len(self.gates)
        except Exception:
            pass
    return 0

_Circuit.__len__ = _circuit___len__

# NOTE: do NOT override an existing 'num_qubits' attribute on the compiled Circuit type.
# Creating a property named 'num_qubits' causes getattr(self, "num_qubits") to re-enter the property.
# Provide a module-level helper that reads the value without shadowing.
def circuit_num_qubits_or_default(circuit):
    for attr in ("num_qubits", "qubits", "n_qubits"):
        try:
            val = getattr(circuit, attr)
        except Exception:
            continue
        # If the attribute is callable (older API), call it
        if callable(val):
            try:
                return int(val())
            except Exception:
                continue
        try:
            return int(val)
        except Exception:
            continue
    # fallback
    return 0

# Gate __str__ formatting: return name only for unbound gates, include qubits when bound
def _gate___str__(self):
    name = getattr(self, "name", None) or getattr(self, "label", None)
    qubits = getattr(self, "qubits", None)
    if qubits:
        try:
            qtext = ", ".join(str(q) for q in qubits)
            return f"{name}({qtext})"
        except Exception:
            pass
    return name or object.__str__(self)

_Gate.__str__ = _gate___str__

# ExecutionResult.values property to match tests
def _executionresult_values(self):
    for attr in ("values", "results", "counts", "measurements", "data"):
        if hasattr(self, attr):
            v = getattr(self, attr)
            if callable(v):
                try:
                    return v()
                except Exception:
                    continue
            return v
    return None

_ExecutionResult.values = property(_executionresult_values)

# Lazy import for ibm module to avoid hard dependency on qiskit-ibm-runtime
def __getattr__(name):
    if name == "ibm":
        from . import ibm as _ibm
        return _ibm
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")

# Public API aliases: using native types directly
Gate = _Gate
Circuit = _Circuit
ExecutionResult = _ExecutionResult
SimulatorBackend = _SimulatorBackend

__all__ = [
    "__version__", "__author__",
    "Gate", "Circuit", "NoiseVector", "NoiseModel", "HardwareProfile",
    "QnsOptimizer", "OptimizationResult", "SimulatorBackend", "ExecutionResult",
    "CalibrationData", "estimate_circuit_fidelity", "score_circuit", "convert",
    "ibm",
]
