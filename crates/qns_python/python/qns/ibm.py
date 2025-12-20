"""
QNS IBM Quantum Integration Module.

Provides seamless integration with IBM Quantum hardware through Qiskit Runtime.

Example:
    >>> from qns.ibm import IBMBackend, optimize_for_hardware
    >>> 
    >>> # Using fake backend for testing
    >>> backend = IBMBackend.from_fake("manila")
    >>> result = backend.run_optimized(circuit)
    >>>
    >>> # Using real hardware (requires API token)
    >>> backend = IBMBackend.from_service("ibm_brisbane", token="...")
    >>> result = backend.run_optimized(circuit)
"""

from __future__ import annotations
from typing import Dict, List, Optional, Tuple, Any, Union
from dataclasses import dataclass, field
import time
import json

# QNS imports
from .qns import (
    Circuit, NoiseVector, NoiseModel, QnsOptimizer, 
    SimulatorBackend, HardwareProfile, OptimizationResult
)
from . import convert

# Qiskit imports
try:
    from qiskit import QuantumCircuit, transpile
    from qiskit import qasm2
    from qiskit.transpiler.preset_passmanagers import generate_preset_pass_manager
    QISKIT_AVAILABLE = True
except ImportError:
    QISKIT_AVAILABLE = False

try:
    from qiskit_aer import AerSimulator
    from qiskit_aer.noise import NoiseModel as AerNoiseModel
    AER_AVAILABLE = True
except ImportError:
    AER_AVAILABLE = False

try:
    from qiskit_ibm_runtime import QiskitRuntimeService, SamplerV2
    from qiskit_ibm_runtime.fake_provider import (
        FakeManilaV2, FakeBrisbane, FakeKyoto, FakeOsaka,
        FakeTorino, FakeSherbrooke
    )
    IBM_RUNTIME_AVAILABLE = True
except ImportError:
    IBM_RUNTIME_AVAILABLE = False


# ============================================================================
# Data Classes
# ============================================================================

@dataclass
class CalibrationInfo:
    """Calibration data extracted from IBM backend."""
    backend_name: str
    num_qubits: int
    t1_times: Dict[int, float]  # qubit_id -> T1 in μs
    t2_times: Dict[int, float]  # qubit_id -> T2 in μs
    gate_errors_1q: Dict[int, float]  # qubit_id -> error rate
    gate_errors_2q: Dict[Tuple[int, int], float]  # (q1, q2) -> error rate
    readout_errors: Dict[int, float]  # qubit_id -> error rate
    coupling_map: List[Tuple[int, int]]
    timestamp: float = field(default_factory=time.time)
    
    def to_noise_vectors(self) -> List[NoiseVector]:
        """Converts to QNS NoiseVector list."""
        vectors = []
        for qid in range(self.num_qubits):
            t1 = self.t1_times.get(qid, 100.0)
            t2 = self.t2_times.get(qid, 80.0)
            err_1q = self.gate_errors_1q.get(qid, 0.001)
            # Average 2Q error for this qubit
            err_2q_list = [e for (q1, q2), e in self.gate_errors_2q.items() if q1 == qid or q2 == qid]
            err_2q = sum(err_2q_list) / len(err_2q_list) if err_2q_list else 0.01
            readout = self.readout_errors.get(qid, 0.01)
            
            nv = NoiseVector(
                qubit_id=qid,
                t1=t1,
                t2=t2,
                gate_error_1q=err_1q,
                gate_error_2q=err_2q,
                readout_error=readout
            )
            vectors.append(nv)
        return vectors
    
    def to_noise_model(self) -> NoiseModel:
        """Creates average NoiseModel from calibration."""
        avg_t1 = sum(self.t1_times.values()) / len(self.t1_times) if self.t1_times else 100.0
        avg_t2 = sum(self.t2_times.values()) / len(self.t2_times) if self.t2_times else 80.0
        avg_err_1q = sum(self.gate_errors_1q.values()) / len(self.gate_errors_1q) if self.gate_errors_1q else 0.001
        avg_err_2q = sum(self.gate_errors_2q.values()) / len(self.gate_errors_2q) if self.gate_errors_2q else 0.01
        avg_readout = sum(self.readout_errors.values()) / len(self.readout_errors) if self.readout_errors else 0.01
        
        return NoiseModel(
            t1=avg_t1,
            t2=avg_t2,
            gate_error_1q=avg_err_1q,
            gate_error_2q=avg_err_2q,
            readout_error=avg_readout
        )
    
    def to_dict(self) -> Dict[str, Any]:
        """Serializes to dictionary."""
        return {
            "backend_name": self.backend_name,
            "num_qubits": self.num_qubits,
            "t1_times": {str(k): v for k, v in self.t1_times.items()},
            "t2_times": {str(k): v for k, v in self.t2_times.items()},
            "gate_errors_1q": {str(k): v for k, v in self.gate_errors_1q.items()},
            "gate_errors_2q": {f"{k[0]},{k[1]}": v for k, v in self.gate_errors_2q.items()},
            "readout_errors": {str(k): v for k, v in self.readout_errors.items()},
            "coupling_map": self.coupling_map,
            "timestamp": self.timestamp,
        }
    
    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> "CalibrationInfo":
        """Deserializes from dictionary."""
        return cls(
            backend_name=d["backend_name"],
            num_qubits=d["num_qubits"],
            t1_times={int(k): v for k, v in d["t1_times"].items()},
            t2_times={int(k): v for k, v in d["t2_times"].items()},
            gate_errors_1q={int(k): v for k, v in d["gate_errors_1q"].items()},
            gate_errors_2q={tuple(map(int, k.split(","))): v for k, v in d["gate_errors_2q"].items()},
            readout_errors={int(k): v for k, v in d["readout_errors"].items()},
            coupling_map=[tuple(x) for x in d["coupling_map"]],
            timestamp=d.get("timestamp", time.time()),
        )


@dataclass
class BenchmarkResult:
    """Result of a benchmark comparison."""
    circuit_name: str
    num_qubits: int
    num_gates: int
    
    # Baseline (Qiskit transpiler only)
    baseline_counts: Dict[str, int]
    baseline_depth: int
    baseline_2q_count: int
    
    # QNS optimized
    qns_counts: Dict[str, int]
    qns_depth: int
    qns_2q_count: int
    qns_score_improvement: float
    
    # Fidelity estimates
    baseline_fidelity: float
    qns_fidelity: float
    fidelity_improvement: float
    
    # Timing
    qns_optimization_time_ms: float
    transpile_time_ms: float
    execution_time_ms: float
    
    def to_dict(self) -> Dict[str, Any]:
        """Serializes to dictionary."""
        return {
            "circuit_name": self.circuit_name,
            "num_qubits": self.num_qubits,
            "num_gates": self.num_gates,
            "baseline": {
                "counts": self.baseline_counts,
                "depth": self.baseline_depth,
                "2q_count": self.baseline_2q_count,
                "fidelity": self.baseline_fidelity,
            },
            "qns": {
                "counts": self.qns_counts,
                "depth": self.qns_depth,
                "2q_count": self.qns_2q_count,
                "fidelity": self.qns_fidelity,
                "score_improvement": self.qns_score_improvement,
                "optimization_time_ms": self.qns_optimization_time_ms,
            },
            "improvement": {
                "fidelity": self.fidelity_improvement,
                "fidelity_pct": (self.fidelity_improvement / self.baseline_fidelity * 100) if self.baseline_fidelity > 0 else 0,
            },
            "timing": {
                "transpile_ms": self.transpile_time_ms,
                "qns_opt_ms": self.qns_optimization_time_ms,
                "exec_ms": self.execution_time_ms,
            }
        }


# ============================================================================
# IBM Backend Wrapper
# ============================================================================

class IBMBackend:
    """
    Wrapper for IBM Quantum backends with QNS integration.
    
    Supports both real IBM Quantum hardware and fake backends for testing.
    """
    
    def __init__(
        self,
        backend: Any,
        name: str,
        is_fake: bool = False,
        calibration: Optional[CalibrationInfo] = None,
    ):
        """
        Initialize IBMBackend.
        
        Args:
            backend: Qiskit backend instance
            name: Backend name
            is_fake: Whether this is a fake/simulator backend
            calibration: Pre-extracted calibration data
        """
        self._backend = backend
        self._name = name
        self._is_fake = is_fake
        self._calibration = calibration or self._extract_calibration()
        self._qns_optimizer: Optional[QnsOptimizer] = None
    
    @classmethod
    def from_fake(cls, name: str) -> "IBMBackend":
        """
        Creates an IBMBackend from a fake backend.
        
        Available fake backends: manila, brisbane, kyoto, osaka, torino, sherbrooke
        
        Args:
            name: Fake backend name (case-insensitive)
        
        Returns:
            IBMBackend instance
        """
        if not IBM_RUNTIME_AVAILABLE:
            raise ImportError("qiskit-ibm-runtime not installed. Run: pip install qiskit-ibm-runtime")
        
        fake_backends = {
            "manila": FakeManilaV2,
            "brisbane": FakeBrisbane,
            "kyoto": FakeKyoto,
            "osaka": FakeOsaka,
            "torino": FakeTorino,
            "sherbrooke": FakeSherbrooke,
        }
        
        name_lower = name.lower()
        if name_lower not in fake_backends:
            raise ValueError(f"Unknown fake backend: {name}. Available: {list(fake_backends.keys())}")
        
        backend = fake_backends[name_lower]()
        return cls(backend, f"fake_{name_lower}", is_fake=True)
    
    @classmethod
    def from_service(
        cls,
        backend_name: str,
        token: Optional[str] = None,
        channel: str = "ibm_quantum",
        instance: Optional[str] = None,
    ) -> "IBMBackend":
        """
        Creates an IBMBackend from IBM Quantum service.
        
        Args:
            backend_name: IBM backend name (e.g., "ibm_brisbane")
            token: IBM Quantum API token (uses saved credentials if None)
            channel: Service channel ("ibm_quantum" or "ibm_cloud")
            instance: Service instance (e.g., "ibm-q/open/main")
        
        Returns:
            IBMBackend instance
        """
        if not IBM_RUNTIME_AVAILABLE:
            raise ImportError("qiskit-ibm-runtime not installed. Run: pip install qiskit-ibm-runtime")
        
        service_kwargs = {"channel": channel}
        if token:
            service_kwargs["token"] = token
        if instance:
            service_kwargs["instance"] = instance
        
        service = QiskitRuntimeService(**service_kwargs)
        backend = service.backend(backend_name)
        
        return cls(backend, backend_name, is_fake=False)
    
    def _extract_calibration(self) -> CalibrationInfo:
        """Extracts calibration data from backend."""
        try:
            target = self._backend.target
            num_qubits = target.num_qubits
            
            t1_times = {}
            t2_times = {}
            gate_errors_1q = {}
            gate_errors_2q = {}
            readout_errors = {}
            
            # Extract qubit properties
            for qid in range(num_qubits):
                props = target.qubit_properties
                if props and qid < len(props) and props[qid]:
                    qp = props[qid]
                    if hasattr(qp, 't1') and qp.t1 is not None:
                        t1_times[qid] = qp.t1 * 1e6  # Convert s to μs
                    if hasattr(qp, 't2') and qp.t2 is not None:
                        t2_times[qid] = qp.t2 * 1e6
            
            # Extract gate errors
            for op_name in target.operation_names:
                qargs_iter = target.qargs_for_operation_name(op_name)
                if qargs_iter is None:
                    continue
                for qargs in qargs_iter:
                    if qargs is None:
                        continue
                    inst_props = target[op_name].get(qargs)
                    if inst_props and inst_props.error is not None:
                        if len(qargs) == 1:
                            gate_errors_1q[qargs[0]] = inst_props.error
                        elif len(qargs) == 2:
                            gate_errors_2q[qargs] = inst_props.error
            
            # Extract readout errors (measure operation)
            if 'measure' in target.operation_names:
                measure_qargs = target.qargs_for_operation_name('measure')
                if measure_qargs:
                    for qargs in measure_qargs:
                        if qargs is None:
                            continue
                        inst_props = target['measure'].get(qargs)
                        if inst_props and inst_props.error is not None:
                            readout_errors[qargs[0]] = inst_props.error
            
            # Build coupling map from 2Q gate qargs
            coupling_map = []
            two_qubit_ops = ['cx', 'ecr', 'cz', 'rzx', 'iswap', 'cr']
            for op_name in two_qubit_ops:
                if op_name in target.operation_names:
                    try:
                        # Get qargs from target[op_name].keys()
                        op_qargs = target[op_name]
                        if op_qargs:
                            for qargs in op_qargs.keys():
                                if qargs and len(qargs) == 2:
                                    coupling_map.append(tuple(qargs))
                    except (KeyError, TypeError):
                        pass
            
            # Fallback: try backend.coupling_map
            if not coupling_map and hasattr(self._backend, 'coupling_map'):
                cm = self._backend.coupling_map
                if cm:
                    coupling_map = [tuple(edge) for edge in cm]
            
            # Remove duplicates
            coupling_map = list(set(coupling_map))
            
            # Fill missing values with defaults
            for qid in range(num_qubits):
                if qid not in t1_times:
                    t1_times[qid] = 100.0
                if qid not in t2_times:
                    t2_times[qid] = min(t1_times[qid] * 1.5, 80.0)
                if qid not in gate_errors_1q:
                    gate_errors_1q[qid] = 0.001
                if qid not in readout_errors:
                    readout_errors[qid] = 0.01
            
            return CalibrationInfo(
                backend_name=self._name,
                num_qubits=num_qubits,
                t1_times=t1_times,
                t2_times=t2_times,
                gate_errors_1q=gate_errors_1q,
                gate_errors_2q=gate_errors_2q,
                readout_errors=readout_errors,
                coupling_map=coupling_map,
            )
            
        except Exception as e:
            # Fallback for backends without full target support
            num_qubits = getattr(self._backend, 'num_qubits', 5)
            return CalibrationInfo(
                backend_name=self._name,
                num_qubits=num_qubits,
                t1_times={i: 100.0 for i in range(num_qubits)},
                t2_times={i: 80.0 for i in range(num_qubits)},
                gate_errors_1q={i: 0.001 for i in range(num_qubits)},
                gate_errors_2q={},
                readout_errors={i: 0.01 for i in range(num_qubits)},
                coupling_map=[],
            )
    
    @property
    def name(self) -> str:
        return self._name
    
    @property
    def num_qubits(self) -> int:
        return self._calibration.num_qubits
    
    @property
    def calibration(self) -> CalibrationInfo:
        return self._calibration
    
    @property
    def coupling_map(self) -> List[Tuple[int, int]]:
        return self._calibration.coupling_map
    
    def refresh_calibration(self) -> CalibrationInfo:
        """Refreshes calibration data from backend."""
        self._calibration = self._extract_calibration()
        self._qns_optimizer = None  # Reset optimizer
        return self._calibration
    
    def get_qns_optimizer(self, num_qubits: Optional[int] = None) -> QnsOptimizer:
        """
        Gets a QNS optimizer configured with this backend's calibration.
        
        Args:
            num_qubits: Number of qubits (defaults to backend's num_qubits)
        
        Returns:
            Configured QnsOptimizer
        """
        n = num_qubits or self._calibration.num_qubits
        noise_model = self._calibration.to_noise_model()
        optimizer = QnsOptimizer(num_qubits=n, noise_model=noise_model)
        
        # Set per-qubit noise
        for nv in self._calibration.to_noise_vectors()[:n]:
            optimizer.set_noise(nv)
        
        return optimizer
    
    def transpile(
        self,
        circuit: Union[QuantumCircuit, Circuit],
        optimization_level: int = 1,
    ) -> QuantumCircuit:
        """
        Transpiles a circuit for this backend.
        
        Args:
            circuit: Qiskit or QNS circuit
            optimization_level: Qiskit optimization level (0-3)
        
        Returns:
            Transpiled Qiskit circuit
        """
        if isinstance(circuit, Circuit):
            qasm_str = convert.circuit_to_qasm(circuit)
            qc = qasm2.loads(qasm_str)
        else:
            qc = circuit
        
        return transpile(qc, self._backend, optimization_level=optimization_level)
    
    def optimize_with_qns(
        self,
        circuit: Union[QuantumCircuit, Circuit],
        use_beam_search: Optional[bool] = None,
    ) -> Tuple[Circuit, OptimizationResult]:
        """
        Optimizes a circuit using QNS noise-aware optimization.
        
        Args:
            circuit: Qiskit or QNS circuit
            use_beam_search: Force beam search algorithm
        
        Returns:
            Tuple of (optimized QNS Circuit, OptimizationResult)
        """
        # Convert to QNS circuit if needed
        if isinstance(circuit, QuantumCircuit):
            qasm_str = qasm2.dumps(circuit)
            qns_circuit = convert.circuit_from_qasm(qasm_str)
        else:
            qns_circuit = circuit
        
        # Get optimizer
        optimizer = self.get_qns_optimizer(qns_circuit.num_qubits)
        
        # Optimize
        result = optimizer.optimize(qns_circuit, use_beam_search=use_beam_search)
        
        return result.optimized_circuit, result
    
    def run(
        self,
        circuit: Union[QuantumCircuit, Circuit],
        shots: int = 1024,
        transpile_options: Optional[Dict[str, Any]] = None,
    ) -> Dict[str, int]:
        """
        Runs a circuit on this backend.
        
        Args:
            circuit: Qiskit or QNS circuit
            shots: Number of measurement shots
            transpile_options: Options for transpilation
        
        Returns:
            Measurement counts
        """
        if isinstance(circuit, Circuit):
            qasm_str = convert.circuit_to_qasm(circuit)
            qc = qasm2.loads(qasm_str)
        else:
            qc = circuit
        
        # Transpile
        opt_level = (transpile_options or {}).get("optimization_level", 1)
        transpiled = transpile(qc, self._backend, optimization_level=opt_level)
        
        # Run using SamplerV2 for fake backends
        if self._is_fake:
            sampler = SamplerV2(self._backend)
            job = sampler.run([transpiled], shots=shots)
            result = job.result()
            
            # Extract counts from PrimitiveResult
            pub_result = result[0]
            counts_dict = {}
            if hasattr(pub_result.data, 'meas'):
                bitarray = pub_result.data.meas
                counts = bitarray.get_counts()
                counts_dict = dict(counts)
            elif hasattr(pub_result.data, 'c'):
                bitarray = pub_result.data.c
                counts = bitarray.get_counts()
                counts_dict = dict(counts)
            else:
                # Try to find any classical register
                for attr_name in dir(pub_result.data):
                    if not attr_name.startswith('_'):
                        attr = getattr(pub_result.data, attr_name)
                        if hasattr(attr, 'get_counts'):
                            counts_dict = dict(attr.get_counts())
                            break
            
            return counts_dict
        else:
            # For real hardware, use SamplerV2 through Runtime
            sampler = SamplerV2(self._backend)
            job = sampler.run([transpiled], shots=shots)
            result = job.result()
            pub_result = result[0]
            
            counts_dict = {}
            for attr_name in dir(pub_result.data):
                if not attr_name.startswith('_'):
                    attr = getattr(pub_result.data, attr_name)
                    if hasattr(attr, 'get_counts'):
                        counts_dict = dict(attr.get_counts())
                        break
            
            return counts_dict
    
    def run_optimized(
        self,
        circuit: Union[QuantumCircuit, Circuit],
        shots: int = 1024,
        use_beam_search: Optional[bool] = None,
        optimization_level: int = 1,
    ) -> Tuple[Dict[str, int], OptimizationResult]:
        """
        Runs QNS-optimized circuit on this backend.
        
        Args:
            circuit: Qiskit or QNS circuit
            shots: Number of measurement shots
            use_beam_search: Force beam search algorithm
            optimization_level: Qiskit transpiler optimization level
        
        Returns:
            Tuple of (measurement counts, OptimizationResult)
        """
        # QNS optimization
        optimized_circuit, opt_result = self.optimize_with_qns(circuit, use_beam_search)
        
        # Run
        counts = self.run(optimized_circuit, shots, {"optimization_level": optimization_level})
        
        return counts, opt_result
    
    def benchmark(
        self,
        circuit: Union[QuantumCircuit, Circuit],
        circuit_name: str = "circuit",
        shots: int = 1024,
        optimization_level: int = 1,
    ) -> BenchmarkResult:
        """
        Benchmarks QNS optimization against baseline.
        
        Args:
            circuit: Circuit to benchmark
            circuit_name: Name for identification
            shots: Number of shots
            optimization_level: Qiskit transpiler level
        
        Returns:
            BenchmarkResult with comparison data
        """
        # Convert to QNS circuit
        if isinstance(circuit, QuantumCircuit):
            qasm_str = qasm2.dumps(circuit)
            qns_circuit = convert.circuit_from_qasm(qasm_str)
            qc = circuit
        else:
            qns_circuit = circuit
            qasm_str = convert.circuit_to_qasm(circuit)
            qc = qasm2.loads(qasm_str)
        
        # Get noise model for fidelity estimation
        noise_model = self._calibration.to_noise_model()
        noise_vectors = self._calibration.to_noise_vectors()
        nv0 = noise_vectors[0] if noise_vectors else NoiseVector()
        
        # ===== Baseline (Qiskit transpiler only) =====
        t0 = time.time()
        baseline_transpiled = transpile(qc, self._backend, optimization_level=optimization_level)
        transpile_time = (time.time() - t0) * 1000
        
        baseline_depth = baseline_transpiled.depth()
        baseline_2q = sum(1 for inst in baseline_transpiled.data if inst.operation.num_qubits == 2)
        
        # Run baseline
        t0 = time.time()
        baseline_counts = self.run(baseline_transpiled, shots)
        exec_time = (time.time() - t0) * 1000
        
        # Estimate baseline fidelity
        from .qns import estimate_circuit_fidelity
        baseline_fidelity = estimate_circuit_fidelity(qns_circuit, nv0)
        
        # ===== QNS Optimized =====
        t0 = time.time()
        opt_circuit, opt_result = self.optimize_with_qns(qns_circuit)
        qns_opt_time = opt_result.optimization_time_ms
        
        # Convert and transpile
        qns_qasm = convert.circuit_to_qasm(opt_circuit)
        qns_qc = qasm2.loads(qns_qasm)
        qns_transpiled = transpile(qns_qc, self._backend, optimization_level=optimization_level)
        
        qns_depth = qns_transpiled.depth()
        qns_2q = sum(1 for inst in qns_transpiled.data if inst.operation.num_qubits == 2)
        
        # Run QNS optimized
        qns_counts = self.run(qns_transpiled, shots)
        
        # Estimate QNS fidelity
        qns_fidelity = estimate_circuit_fidelity(opt_circuit, nv0)
        
        return BenchmarkResult(
            circuit_name=circuit_name,
            num_qubits=qns_circuit.num_qubits,
            num_gates=len(qns_circuit),
            baseline_counts=baseline_counts,
            baseline_depth=baseline_depth,
            baseline_2q_count=baseline_2q,
            qns_counts=qns_counts,
            qns_depth=qns_depth,
            qns_2q_count=qns_2q,
            qns_score_improvement=opt_result.score_improvement,
            baseline_fidelity=baseline_fidelity,
            qns_fidelity=qns_fidelity,
            fidelity_improvement=qns_fidelity - baseline_fidelity,
            qns_optimization_time_ms=qns_opt_time,
            transpile_time_ms=transpile_time,
            execution_time_ms=exec_time,
        )


# ============================================================================
# Utility Functions
# ============================================================================

def create_ghz_circuit(num_qubits: int) -> QuantumCircuit:
    """Creates a GHZ state preparation circuit."""
    qc = QuantumCircuit(num_qubits)
    qc.h(0)
    for i in range(num_qubits - 1):
        qc.cx(i, i + 1)
    qc.measure_all()
    return qc


def create_qft_circuit(num_qubits: int) -> QuantumCircuit:
    """Creates a Quantum Fourier Transform circuit."""
    import math
    qc = QuantumCircuit(num_qubits)
    
    for i in range(num_qubits):
        qc.h(i)
        for j in range(i + 1, num_qubits):
            angle = math.pi / (2 ** (j - i))
            qc.cp(angle, j, i)
    
    # Swap qubits using CNOT decomposition (SWAP = CNOT-CNOT-CNOT)
    for i in range(num_qubits // 2):
        j = num_qubits - i - 1
        # SWAP(i, j) = CNOT(i,j) CNOT(j,i) CNOT(i,j)
        qc.cx(i, j)
        qc.cx(j, i)
        qc.cx(i, j)
    
    qc.measure_all()
    return qc


def create_random_circuit(num_qubits: int, depth: int, seed: int = 42) -> QuantumCircuit:
    """Creates a random circuit for testing."""
    import random
    random.seed(seed)
    
    qc = QuantumCircuit(num_qubits)
    
    single_gates = ['h', 'x', 'y', 'z', 's', 't']
    
    for _ in range(depth):
        # Random single-qubit gates
        for q in range(num_qubits):
            if random.random() < 0.5:
                gate = random.choice(single_gates)
                getattr(qc, gate)(q)
        
        # Random two-qubit gates
        if num_qubits >= 2:
            q1 = random.randint(0, num_qubits - 2)
            q2 = q1 + 1
            qc.cx(q1, q2)
    
    qc.measure_all()
    return qc


def run_benchmark_suite(
    backend: IBMBackend,
    num_qubits_list: List[int] = [3, 5],
    shots: int = 1024,
) -> List[BenchmarkResult]:
    """
    Runs a suite of benchmarks.
    
    Args:
        backend: IBMBackend to benchmark
        num_qubits_list: List of qubit counts to test
        shots: Number of shots per circuit
    
    Returns:
        List of BenchmarkResult
    """
    results = []
    
    for n in num_qubits_list:
        if n > backend.num_qubits:
            continue
        
        # GHZ
        print(f"Benchmarking GHZ-{n}...")
        ghz = create_ghz_circuit(n)
        results.append(backend.benchmark(ghz, f"GHZ-{n}", shots))
        
        # QFT
        print(f"Benchmarking QFT-{n}...")
        qft = create_qft_circuit(n)
        results.append(backend.benchmark(qft, f"QFT-{n}", shots))
        
        # Random
        print(f"Benchmarking Random-{n}...")
        rand = create_random_circuit(n, n * 2)
        results.append(backend.benchmark(rand, f"Random-{n}", shots))
    
    return results


def print_benchmark_summary(results: List[BenchmarkResult]) -> None:
    """Prints a summary table of benchmark results."""
    print("\n" + "=" * 80)
    print("QNS BENCHMARK SUMMARY")
    print("=" * 80)
    print(f"{'Circuit':<15} {'Qubits':<7} {'Baseline':<12} {'QNS':<12} {'Δ Fidelity':<12} {'Time(ms)':<10}")
    print("-" * 80)
    
    for r in results:
        delta_pct = (r.fidelity_improvement / r.baseline_fidelity * 100) if r.baseline_fidelity > 0 else 0
        print(f"{r.circuit_name:<15} {r.num_qubits:<7} {r.baseline_fidelity:<12.4f} {r.qns_fidelity:<12.4f} {delta_pct:>+10.2f}% {r.qns_optimization_time_ms:<10.2f}")
    
    print("-" * 80)
    
    # Average improvement
    avg_improvement = sum(r.fidelity_improvement for r in results) / len(results) if results else 0
    avg_baseline = sum(r.baseline_fidelity for r in results) / len(results) if results else 0
    avg_pct = (avg_improvement / avg_baseline * 100) if avg_baseline > 0 else 0
    
    print(f"{'AVERAGE':<15} {'':<7} {avg_baseline:<12.4f} {avg_baseline + avg_improvement:<12.4f} {avg_pct:>+10.2f}%")
    print("=" * 80)
