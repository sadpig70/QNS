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

# --- compatibility wrappers for legacy Python API ---
# imports from the compiled extension module
from .qns import (
    __version__,
    __author__,
    Gate as _Gate,
    Circuit as _Circuit,
    NoiseVector,
    NoiseModel,
    HardwareProfile,
    QnsOptimizer,
    OptimizationResult,
    SimulatorBackend as _SimulatorBackend,
    ExecutionResult as _ExecutionResult,
    CalibrationData,
    estimate_circuit_fidelity,
    score_circuit,
    convert,
)

# Circuit wrapper: ensures len(circuit) == number of gates and provides callable num_qubits()
class Circuit:
    def __init__(self, *args, **kwargs):
        # instantiate the underlying extension Circuit
        self._inner = _Circuit(*args, **kwargs)

    # make len() return number of gates (compat with tests)
    def __len__(self):
        # try common attributes used by the extension:
        # - if extension exposes explicit gates_count / num_gates attribute, use it
        if hasattr(self._inner, "gates_count"):
            return int(self._inner.gates_count)
        if hasattr(self._inner, "num_gates"):
            return int(self._inner.num_gates)
        # if extension exposes a sequence of gates
        if hasattr(self._inner, "gates"):
            try:
                return len(self._inner.gates)
            except Exception:
                pass
        # fallback: 0
        return 0

    # tests expect a callable num_qubits() in some places
    def num_qubits(self):
        # prefer explicit attribute names if present
        for attr in ("num_qubits", "qubits", "n_qubits"):
            if hasattr(self._inner, attr):
                val = getattr(self._inner, attr)
                # if attribute is callable (previous API), call it
                if callable(val):
                    return val()
                # else, return integer value
                try:
                    return int(val)
                except Exception:
                    pass
        # last resort: try to infer from gates or 0
        if hasattr(self._inner, "gates"):
            try:
                # assume gates keep max qubit index + 1 if stored; best-effort
                return getattr(self._inner, "num_qubits", 0)
            except Exception:
                pass
        return 0

    # delegate attribute access to the inner extension object
    def __getattr__(self, name):
        return getattr(self._inner, name)

    def __repr__(self):
        return f"Circuit({repr(self._inner)})"

# Gate wrapper: string formatting compatibility with tests
class Gate:
    def __init__(self, *args, **kwargs):
        self._inner = _Gate(*args, **kwargs)

    def __str__(self):
        # prefer the extension-provided name
        name = getattr(self._inner, "name", None) or getattr(self._inner, "label", None)
        # try to detect qubit operands (some Gate objects are defined with no bound qubits)
        qubits = getattr(self._inner, "qubits", None)
        if qubits:
            try:
                # format as "NAME(q0, q1)" when bound to qubits
                qtext = ", ".join(str(q) for q in qubits)
                return f"{name}({qtext})"
            except Exception:
                return name or str(self._inner)
        # if no qubits bound, return just the gate name (what tests expect)
        return name or str(self._inner)

    def __repr__(self):
        return f"Gate({str(self)})"

    def __getattr__(self, name):
        return getattr(self._inner, name)

# ExecutionResult wrapper: provide .values property expected by tests
class ExecutionResult:
    def __init__(self, *args, **kwargs):
        self._inner = _ExecutionResult(*args, **kwargs)

    @property
    def values(self):
        # try several likely attribute names on the inner object
        for attr in ("values", "results", "counts", "measurements", "data"):
            if hasattr(self._inner, attr):
                v = getattr(self._inner, attr)
                # avoid returning bound method objects
                if callable(v):
                    try:
                        return v()
                    except Exception:
                        continue
                return v
        # fall back to returning the inner object so callers at least get it
        return self._inner

    def __getattr__(self, name):
        return getattr(self._inner, name)

    def __repr__(self):
        return f"ExecutionResult({repr(self._inner)})"

# SimulatorBackend: keep as thin wrapper to ensure returned results are wrapped
class SimulatorBackend:
    def __init__(self, *args, **kwargs):
        self._inner = _SimulatorBackend(*args, **kwargs)

    def run(self, *args, **kwargs):
        # ensure ExecutionResult wrapper is used where appropriate
        res = self._inner.run(*args, **kwargs)
        # if the extension already returned an ExecutionResult instance, wrap it
        if isinstance(res, _ExecutionResult):
            wrapped = ExecutionResult.__new__(ExecutionResult)
            wrapped._inner = res
            return wrapped
        return res

    def __getattr__(self, name):
        return getattr(self._inner, name)

# Lazy import for ibm module to avoid hard dependency on qiskit-ibm-runtime
def __getattr__(name):
    if name == "ibm":
        from . import ibm as _ibm
        return _ibm
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")

# Re-export names for the rest of the package/tests
__all__ = [
    "__version__",
    "__author__",
    "Gate",
    "Circuit",
    "NoiseVector",
    "NoiseModel",
    "HardwareProfile",
    "QnsOptimizer",
    "OptimizationResult",
    "SimulatorBackend",
    "ExecutionResult",
    "CalibrationData",
    "estimate_circuit_fidelity",
    "score_circuit",
    "convert",
    "ibm",
]

# keep version/author variables from the extension
__version__ = __version__
__author__ = __author__
