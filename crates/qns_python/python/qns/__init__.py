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

# helper to wrap extension objects back into Python shims
def _wrap(obj):
    try:
        if isinstance(obj, _Circuit):
            wrapper = Circuit.__new__(Circuit)
            wrapper._inner = obj
            return wrapper
        if isinstance(obj, _Gate):
            wrapper = Gate.__new__(Gate)
            wrapper._inner = obj
            return wrapper
        if isinstance(obj, _ExecutionResult):
            wrapper = ExecutionResult.__new__(ExecutionResult)
            wrapper._inner = obj
            return wrapper
    except Exception:
        # if extension types are not available or isinstance fails, return raw object
        pass
    return obj

# metaclass to forward class-level attributes to the extension class
class ForwardMeta(type):
    def __getattr__(cls, name):
        # the wrapper class must set _inner_cls to the extension class
        inner_cls = getattr(cls, "_inner_cls", None)
        if inner_cls is None:
            raise AttributeError(f"{cls!r} has no attribute {name!r}")
        attr = getattr(inner_cls, name)
        # If attribute is callable, return a wrapper that calls the inner and wraps the result
        if callable(attr):
            def _callable(*args, **kwargs):
                res = attr(*args, **kwargs)
                return _wrap(res)
            return _callable
        # Otherwise return wrapped attribute/object
        return _wrap(attr)

# Circuit wrapper
class Circuit(metaclass=ForwardMeta):
    _inner_cls = _Circuit

    def __init__(self, *args, **kwargs):
        self._inner = _Circuit(*args, **kwargs)

    def __len__(self):
        # prefer explicit gate-count attributes exposed by the extension
        for attr in ("gates_count", "num_gates"):
            if hasattr(self._inner, attr):
                try:
                    return int(getattr(self._inner, attr))
                except Exception:
                    pass
        # sequence of gates
        if hasattr(self._inner, "gates"):
            try:
                return len(self._inner.gates)
            except Exception:
                pass
        return 0

    @property
    def num_qubits(self):
        # expose num_qubits as a property (tests expect this to be an int)
        for attr in ("num_qubits", "qubits", "n_qubits"):
            if hasattr(self._inner, attr):
                val = getattr(self._inner, attr)
                if callable(val):
                    try:
                        return int(val())
                    except Exception:
                        pass
                try:
                    return int(val)
                except Exception:
                    pass
        # fallback: try to get from inner num_qubits attr or 0
        return int(getattr(self._inner, "num_qubits", 0))

    # keep a callable version for backward compatibility if something called it
    def num_qubits_method(self):
        return self.num_qubits

    def __getattr__(self, name):
        # delegate remaining attribute access to the extension object; wrap returns when appropriate
        val = getattr(self._inner, name)
        return _wrap(val)

    def __repr__(self):
        return f"Circuit({repr(self._inner)})"


# Gate wrapper
class Gate(metaclass=ForwardMeta):
    _inner_cls = _Gate

    def __init__(self, *args, **kwargs):
        self._inner = _Gate(*args, **kwargs)

    def __str__(self):
        name = getattr(self._inner, "name", None) or getattr(self._inner, "label", None)
        qubits = getattr(self._inner, "qubits", None)
        if qubits:
            try:
                qtext = ", ".join(str(q) for q in qubits)
                return f"{name}({qtext})"
            except Exception:
                return name or str(self._inner)
        return name or str(self._inner)

    def __repr__(self):
        return f"Gate({str(self)})"

    def __getattr__(self, name):
        return _wrap(getattr(self._inner, name))


# ExecutionResult wrapper (unchanged, keep .values)
class ExecutionResult:
    def __init__(self, *args, **kwargs):
        # allow constructing from an existing extension result if passed
        if args and isinstance(args[0], _ExecutionResult):
            self._inner = args[0]
            return
        self._inner = _ExecutionResult(*args, **kwargs)

    @property
    def values(self):
        for attr in ("values", "results", "counts", "measurements", "data"):
            if hasattr(self._inner, attr):
                v = getattr(self._inner, attr)
                if callable(v):
                    try:
                        return v()
                    except Exception:
                        continue
                return v
        return self._inner

    def __getattr__(self, name):
        return _wrap(getattr(self._inner, name))

    def __repr__(self):
        return f"ExecutionResult({repr(self._inner)})"


# SimulatorBackend wrapper: forwards class-level constructors and wraps returned ExecutionResult
class SimulatorBackend(metaclass=ForwardMeta):
    _inner_cls = _SimulatorBackend

    def __init__(self, *args, **kwargs):
        self._inner = _SimulatorBackend(*args, **kwargs)

    def run(self, *args, **kwargs):
        res = self._inner.run(*args, **kwargs)
        return _wrap(res)

    def __getattr__(self, name):
        return _wrap(getattr(self._inner, name))

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
