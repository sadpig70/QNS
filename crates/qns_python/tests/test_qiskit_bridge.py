"""
Unit tests for qiskit_bridge module.

Tests circuit conversion, calibration fetching, and Aer simulation.
"""

import pytest
import sys
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent / 'python'))

from qiskit_bridge import CircuitConverter, NoiseModelBuilder, AerSimulationRunner
from qiskit import QuantumCircuit


class TestCircuitConverter:
    """Test CircuitConverter class."""
    
    def setup_method(self):
        """Setup converter for each test."""
        self.converter = CircuitConverter()
    
    def test_bell_state_conversion(self):
        """
        Test Bell state conversion: QNS → Qiskit
        
        Gantree: TestBellState atomic node
        """
        # QNS representation of Bell state
        qns_gates = [
            {'name': 'H', 'qubits': [0], 'params': []},
            {'name': 'CNOT', 'qubits': [0, 1], 'params': []},
        ]
        num_qubits = 2
        
        # Convert
        qc = self.converter.qns_to_qiskit(qns_gates, num_qubits)
        
        # Assertions
        assert qc.num_qubits == 2
        assert qc.num_clbits == 0  # No measurements yet
        assert len(qc.data) == 2  # 2 gates
        
        # Check gate types
        gates = [instruction.operation.name for instruction in qc.data]
        assert gates == ['h', 'cx']
    
    def test_ghz_state_conversion(self):
        """
        Test GHZ state conversion.
        
        Gantree: TestGHZState atomic node
        """
        # QNS representation of 3-qubit GHZ
        qns_gates = [
            {'name': 'H', 'qubits': [0], 'params': []},
            {'name': 'CNOT', 'qubits': [0, 1], 'params': []},
            {'name': 'CNOT', 'qubits': [0, 2], 'params': []},
        ]
        num_qubits = 3
        
        # Convert
        qc = self.converter.qns_to_qiskit(qns_gates, num_qubits)
        
        # Assertions
        assert qc.num_qubits == 3
        assert len(qc.data) == 3
        
        gates = [instruction.operation.name for instruction in qc.data]
        assert gates == ['h', 'cx', 'cx']
    
    def test_all_gate_types(self):
        """
        Test all supported gate types.
        
        Gantree: TestAllGateTypes atomic node
        """
        qns_gates = [
            {'name': 'H', 'qubits': [0], 'params': []},
            {'name': 'X', 'qubits': [1], 'params': []},
            {'name': 'Y', 'qubits': [2], 'params': []},
            {'name': 'Z', 'qubits': [3], 'params': []},
            {'name': 'RX', 'qubits': [0], 'params': [0.5]},
            {'name': 'RY', 'qubits': [1], 'params': [1.0]},
            {'name': 'RZ', 'qubits': [2], 'params': [1.5]},
            {'name': 'CNOT', 'qubits': [0, 1], 'params': []},
            {'name': 'CZ', 'qubits': [2, 3], 'params': []},
            {'name': 'SWAP', 'qubits': [0, 3], 'params': []},
        ]
        num_qubits = 4
        
        # Convert
        qc = self.converter.qns_to_qiskit(qns_gates, num_qubits)
        
        # Assertions
        assert qc.num_qubits == 4
        assert len(qc.data) == 10
        
        expected_gates = ['h', 'x', 'y', 'z', 'rx', 'ry', 'rz', 'cx', 'cz', 'swap']
        gates = [instruction.operation.name for instruction in qc.data]
        assert gates == expected_gates
    
    def test_invalid_gate_name(self):
        """Test error handling for unknown gate."""
        qns_gates = [
            {'name': 'UNKNOWN_GATE', 'qubits': [0], 'params': []},
        ]
        
        with pytest.raises(ValueError, match="Unknown gate type"):
            self.converter.qns_to_qiskit(qns_gates, 2)
    
    def test_qubit_out_of_range(self):
        """Test error handling for qubit index out of range."""
        qns_gates = [
            {'name': 'H', 'qubits': [5], 'params': []},  # qubit 5 but only 2 qubits
        ]
        
        with pytest.raises(ValueError, match="circuit has only 2 qubits"):
            self.converter.qns_to_qiskit(qns_gates, 2)
    
    def test_parametric_gate_missing_params(self):
        """Test error handling for parametric gate without parameters."""
        qns_gates = [
            {'name': 'RX', 'qubits': [0], 'params': []},  # RX needs parameter
        ]
        
        with pytest.raises(ValueError, match="missing parameters"):
            self.converter.qns_to_qiskit(qns_gates, 2)


class TestNoiseModelBuilder:
    """Test NoiseModelBuilder class."""
    
    def setup_method(self):
        """Setup builder for each test."""
        self.builder = NoiseModelBuilder()
    
    def test_noise_model_creation(self):
        """Test NoiseModel creation from calibration data."""
        calibration = {
            't1': [100e-6, 120e-6],  # 100μs, 120μs
            't2': [80e-6, 90e-6],     # 80μs, 90μs
            'gate_errors_1q': [0.001, 0.0015],  # 0.1%, 0.15%
            'gate_errors_2q': {(0, 1): 0.01},   # 1%
            'readout_errors': [0.01, 0.015],    # 1%, 1.5%
        }
        
        noise_model = self.builder.build_noise_model(calibration)
        
        # Assert noise model is not None
        assert noise_model is not None
        assert len(noise_model.noise_qubits) > 0


class TestAerSimulationRunner:
    """Test AerSimulationRunner class."""
    
    def test_bell_state_simulation(self):
        """Test Bell state simulation with ideal (no noise) backend."""
        # Create Bell state circuit
        qc = QuantumCircuit(2)
        qc.h(0)
        qc.cx(0, 1)
        qc.measure_all()
        
        # Run simulation
        runner = AerSimulationRunner(noise_model=None)
        counts = runner.run(qc, shots=1024)
        
        # Bell state should give 50% |00⟩ and 50% |11⟩
        assert '00' in counts
        assert '11' in counts
        
        # Total counts should be 1024
        assert sum(counts.values()) == 1024
        
        # Check approximately 50-50 split (allow 10% deviation)
        prob_00 = counts['00'] / 1024
        assert 0.4 <= prob_00 <= 0.6
    
    def test_fidelity_calculation(self):
        """Test fidelity calculation."""
        counts = {'00': 900, '01': 50, '10': 50, '11': 24}
        runner = AerSimulationRunner()
        
        fidelity = runner.calculate_fidelity(counts, '00')
        
        # Fidelity should be 900/1024 ≈ 0.879
        assert abs(fidelity - 0.879) < 0.01


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
