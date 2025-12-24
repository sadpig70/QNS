"""
QNS-Qiskit Bridge Module

Provides conversion between QNS CircuitGenome and Qiskit QuantumCircuit,
calibration data fetching from IBM backends, and Aer simulation integration.
"""

from typing import Dict, List, Tuple, Optional
import numpy as np
from qiskit import QuantumCircuit
from qiskit.circuit.library import HGate, XGate, YGate, ZGate, SGate, TGate, RXGate, RYGate, RZGate
from qiskit.circuit.library import CXGate, CZGate, SwapGate
from qiskit_aer import AerSimulator
from qiskit_aer.noise import NoiseModel, thermal_relaxation_error, depolarizing_error


class CircuitConverter:
    """
    Converts QNS CircuitGenome to Qiskit QuantumCircuit.
    
    Gantree: Task1_2_CircuitConverter → ImplementQNSToQiskit
    """
    
    # Gate mapping: QNS name → (Qiskit gate class, num_qubits, has_params)
    GATE_MAP = {
        'H': (HGate, 1, False),
        'X': (XGate, 1, False),
        'Y': (YGate, 1, False),
        'Z': (ZGate, 1, False),
        'S': (SGate, 1, False),
        'T': (TGate, 1, False),
        'RX': (RXGate, 1, True),
        'RY': (RYGate, 1, True),
        'RZ': (RZGate, 1, True),
        'CNOT': (CXGate, 2, False),
        'CZ': (CZGate, 2, False),
        'SWAP': (SwapGate, 2, False),
    }
    
    def __init__(self):
        """Initialize converter."""
        pass
    
    def qns_to_qiskit(self, qns_gates: List[Dict], num_qubits: int) -> QuantumCircuit:
        """
        Convert QNS circuit to Qiskit QuantumCircuit.
        
        Args:
            qns_gates: List of QNS gate dictionaries with format:
                       {'name': str, 'qubits': List[int], 'params': Optional[List[float]]}
            num_qubits: Total number of qubits
            
        Returns:
            Qiskit QuantumCircuit
            
        Gantree atomic nodes:
            - ParseQNSGates
            - MapToQiskitGates  
            - BuildQuantumCircuit
        """
        # ParseQNSGates (already parsed by Rust, just validate)
        self._validate_gates(qns_gates, num_qubits)
        
        # BuildQuantumCircuit
        qc = QuantumCircuit(num_qubits)
        
        # MapToQiskitGates and append
        for gate_dict in qns_gates:
            self._append_gate(qc, gate_dict)
        
        return qc
    
    def _validate_gates(self, gates: List[Dict], num_qubits: int) -> None:
        """
        Validate QNS gates.
        
        Gantree: ParseQNSGates atomic node
        """
        for i, gate in enumerate(gates):
            if 'name' not in gate:
                raise ValueError(f"Gate {i} missing 'name' field")
            if 'qubits' not in gate:
                raise ValueError(f"Gate {i} missing 'qubits' field")
            
            gate_name = gate['name']
            if gate_name not in self.GATE_MAP:
                raise ValueError(f"Unknown gate type: {gate_name}")
            
            qubits = gate['qubits']
            if max(qubits) >= num_qubits:
                raise ValueError(
                    f"Gate {gate_name} targets qubit {max(qubits)} "
                    f"but circuit has only {num_qubits} qubits"
                )
    
    def _append_gate(self, qc: QuantumCircuit, gate_dict: Dict) -> None:
        """
        Append single gate to QuantumCircuit.
        
        Gantree: MapToQiskitGates atomic node
        """
        gate_name = gate_dict['name']
        qubits = gate_dict['qubits']
        params = gate_dict.get('params', [])
        
        gate_class, expected_qubits, has_params = self.GATE_MAP[gate_name]
        
        # Validate qubit count
        if len(qubits) != expected_qubits:
            raise ValueError(
                f"Gate {gate_name} expects {expected_qubits} qubits, "
                f"got {len(qubits)}"
            )
        
        # Create gate instance
        if has_params:
            if not params:
                raise ValueError(f"Parametric gate {gate_name} missing parameters")
            gate = gate_class(*params)
        else:
            gate = gate_class()
        
        # Append to circuit
        qc.append(gate, qubits)


class CalibrationFetcher:
    """
    Fetches calibration data from IBM backends.
    
    Gantree: Task2_2_CalibrationFetch
    """
    
    def __init__(self):
        """Initialize calibration fetcher."""
        self._service = None
        self._backend = None
    
    def connect(self, backend_name: str = 'ibm_fez') -> None:
        """
        Connect to IBM Quantum backend.
        
        Args:
            backend_name: Name of IBM backend
            
        Gantree atomic nodes:
            - LoadAPIKey
            - InitializeService
            - SelectBackend
        """
        from qiskit_ibm_runtime import QiskitRuntimeService
        import os
        
        # LoadAPIKey
        token = os.getenv('QISKIT_IBM_TOKEN')
        if not token:
            raise ValueError(
                "QISKIT_IBM_TOKEN not found in environment. "
                "Please set it with: export QISKIT_IBM_TOKEN='your_token'"
            )
        
        # InitializeService
        # Try ibm_cloud channel first (for IBM Cloud instances)
        try:
            self._service = QiskitRuntimeService(channel="ibm_cloud", token=token)
        except Exception as e:
            # Fallback to ibm_quantum (for IBM Quantum Platform)
            try:
                self._service = QiskitRuntimeService(token=token)
            except Exception as e2:
                raise ValueError(
                    f"Failed to initialize IBM Quantum service. "
                    f"ibm_cloud error: {e}, ibm_quantum error: {e2}"
                )
        
        # SelectBackend
        self._backend = self._service.backend(backend_name)
    
    def fetch_properties(self) -> Dict:
        """
        Fetch calibration properties from connected backend.
        
        Returns:
            Dictionary with T1, T2, gate errors, readout errors
            
        Gantree atomic nodes:
            - FetchBackendProperties
            - ParseT1T2
            - ParseGateErrors
            - ParseReadoutErrors
        """
        if not self._backend:
            raise RuntimeError("Backend not connected. Call connect() first.")
        
        # FetchBackendProperties
        props = self._backend.properties()
        config = self._backend.configuration()
        
        calibration = {
            't1': self._parse_t1(props, config),
            't2': self._parse_t2(props, config),
            'gate_errors_1q': self._parse_1q_errors(props, config),
            'gate_errors_2q': self._parse_2q_errors(props),
            'readout_errors': self._parse_readout_errors(props, config),
            'crosstalk': self._parse_crosstalk(props, config),
        }
        
        return calibration
    
    def _parse_crosstalk(self, props, config) -> Dict[Tuple[int, int], float]:
        """
        Parse crosstalk interactions (atomic node).
        
        Attempts to find 'zz_interaction' or similar properties.
        Falls back to a distance-based heuristic if not found.
        """
        crosstalk = {}
        # 1. Try to find explicit crosstalk properties (uncommon in standard props)
        # Iterate all 2Q edges and check for extra params? 
        # For now, we use a connectivity-based heuristic since real data is often missing.
        
        # Heuristic: Add small crosstalk to all connected pairs to simulate
        # "residual coupling" - typically 0.1% to 1.0% depending on architecture.
        # In a real implementation, we would parse `backend.properties().gate_property(gate, 'crosstalk')` if it existed.
        
        num_qubits = config.num_qubits
        
        # Iterate over all possible connected pairs (using coupling map from configuration if available)
        # Note: config.coupling_map is a list of [q1, q2]
        if hasattr(config, 'coupling_map') and config.coupling_map:
            for edge in config.coupling_map:
                q1, q2 = edge[0], edge[1]
                # Normalize edge key
                key = tuple(sorted((q1, q2)))
                
                # Check if we already have it
                if key not in crosstalk:
                    # Assign a baseline crosstalk value (e.g. 0.005 = 0.5%)
                    # In future, this could be weighted by gate error rates
                    crosstalk[key] = 0.005 
        
        return crosstalk
    
    def _parse_t1(self, props, config) -> List[float]:
        """Parse T1 times for all qubits (atomic node)."""
        num_qubits = config.num_qubits
        t1_values = []
        for qubit in range(num_qubits):
            t1 = props.t1(qubit)
            t1_values.append(t1 if t1 is not None else 100e-6)  # default 100μs
        return t1_values
    
    def _parse_t2(self, props, config) -> List[float]:
        """Parse T2 times for all qubits (atomic node)."""
        num_qubits = config.num_qubits
        t2_values = []
        for qubit in range(num_qubits):
            t2 = props.t2(qubit)
            t2_values.append(t2 if t2 is not None else 80e-6)  # default 80μs
        return t2_values
    
    def _parse_1q_errors(self, props, config) -> List[float]:
        """Parse single-qubit gate errors (atomic node)."""
        num_qubits = config.num_qubits
        errors = []
        for qubit in range(num_qubits):
            # Get error for 'sx' gate (most common 1Q gate)
            try:
                error = props.gate_error('sx', qubit)
                errors.append(error if error is not None else 0.001)  # default 0.1%
            except:
                errors.append(0.001)
        return errors
    
    def _parse_2q_errors(self, props) -> Dict[Tuple[int, int], float]:
        """Parse two-qubit gate errors (atomic node)."""
        errors = {}
        for gate in props.gates:
            if gate.gate == 'cx':  # CNOT gate
                qubits = tuple(gate.qubits)
                error = gate.parameters[0].value  # error rate
                errors[qubits] = error if error is not None else 0.01  # default 1%
        return errors
    
    def _parse_readout_errors(self, props, config) -> List[float]:
        """Parse readout errors (atomic node)."""
        num_qubits = config.num_qubits
        errors = []
        for qubit in range(num_qubits):
            error = props.readout_error(qubit)
            errors.append(error if error is not None else 0.01)  # default 1%
        return errors


class NoiseModelBuilder:
    """
    Builds Qiskit NoiseModel from calibration data.
    
    Gantree: Task2_3_NoiseModelBuilder
    """
    
    def build_noise_model(self, calibration: Dict) -> NoiseModel:
        """
        Create NoiseModel from calibration data.
        
        Args:
            calibration: Dict with t1, t2, gate_errors, readout_errors
            
        Returns:
            Qiskit NoiseModel
            
        Gantree atomic nodes:
            - CreateNoiseModel
            - AddT1T2Errors
            - AddGateErrors
            - AddReadoutErrors
        """
        noise_model = NoiseModel()
        
        t1_list = calibration['t1']
        t2_list = calibration['t2']
        gate_errors_1q = calibration['gate_errors_1q']
        gate_errors_2q = calibration['gate_errors_2q']
        
        num_qubits = len(t1_list)
        
        #  T2 constraint: T2 must be <= 2*T1 (physical constraint)
        t2_list_validated = []
        for i in range(num_qubits):
            t1 = t1_list[i]
            t2 = t2_list[i]
            # Clamp T2 to 2*T1 if it exceeds the constraint
            t2_validated = min(t2, 2 * t1)
            t2_list_validated.append(t2_validated)
        
        # AddT1T2Errors
        gate_time_1q = 35e-9  # 35ns for single-qubit gates
        gate_time_2q = 660e-9  # 660ns for two-qubit gates
        
        for qubit in range(num_qubits):
            # Thermal relaxation for 1Q gates
            t1_error = thermal_relaxation_error(
                t1_list[qubit], t2_list_validated[qubit], gate_time_1q
            )
            noise_model.add_quantum_error(t1_error, ['sx', 'x'], [qubit])
        
        # AddGateErrors (depolarizing on top of T1/T2)
        for qubit in range(num_qubits):
            dep_error = depolarizing_error(gate_errors_1q[qubit], 1)
            noise_model.add_quantum_error(dep_error, ['sx', 'x'], [qubit])
        
        for (q0, q1), error_rate in gate_errors_2q.items():
            # T1/T2 error for 2Q gates
            t1_error = thermal_relaxation_error(
                t1_list[q0], t2_list_validated[q0], gate_time_2q
            ).tensor(thermal_relaxation_error(
                t1_list[q1], t2_list_validated[q1], gate_time_2q
            ))
            noise_model.add_quantum_error(t1_error, ['cx'], [q0, q1])
            
            # Depolarizing error
            dep_error = depolarizing_error(error_rate, 2)
            noise_model.add_quantum_error(dep_error, ['cx'], [q0, q1])
        
        # AddReadoutErrors - simplified (TODO: use actual assignment matrix)
        # For now, just informational - Aer handles readout errors separately
        
        return noise_model


class AerSimulationRunner:
    """
    Runs Aer noisy simulation.
    
    Gantree: Task3_2_CircuitExecution
    """
    
    def __init__(self, noise_model: Optional[NoiseModel] = None):
        """Initialize Aer simulator with optional noise model."""
        self.noise_model = noise_model
        self.simulator = AerSimulator(noise_model=noise_model)
    
    def run(self, circuit: QuantumCircuit, shots: int = 1024) -> Dict[str, int]:
        """
        Run circuit on Aer simulator.
        
        Args:
            circuit: QuantumCircuit to execute
            shots: Number of shots
            
        Returns:
            Measurement counts dictionary
            
        Gantree atomic nodes:
            - PrepareCircuit
            - ExecuteWithShots
            - ExtractCounts
        """
        # PrepareCircuit - add measurements if not present
        if not circuit.num_clbits:
            circuit.measure_all()
        
        # ExecuteWithShots
        job = self.simulator.run(circuit, shots=shots)
        result = job.result()
        
        # ExtractCounts
        counts = result.get_counts()
        
        return counts
    
    def calculate_fidelity(
        self, 
        counts: Dict[str, int], 
        expected_state: str
    ) -> float:
        """
        Calculate fidelity as probability of expected state.
        
        Args:
            counts: Measurement counts
            expected_state: Expected bitstring (e.g., '00')
            
        Returns:
            Fidelity (0-1)
            
        Gantree: CalculateFidelity atomic node
        """
        total_shots = sum(counts.values())
        correct_counts = counts.get(expected_state, 0)
        fidelity = correct_counts / total_shots
        return fidelity
