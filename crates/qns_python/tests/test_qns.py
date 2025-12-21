"""
QNS Python Bindings - Test Suite
"""
import pytest


class TestCircuit:
    """Test Circuit class."""
    
    def test_create_circuit(self):
        import qns
        circuit = qns.Circuit(num_qubits=3)
        assert circuit.num_qubits == 3
        assert len(circuit) == 0
    
    def test_add_gates(self):
        import qns
        circuit = qns.Circuit(num_qubits=3)
        circuit.h(0)
        circuit.cnot(0, 1)
        circuit.cnot(1, 2)
        assert len(circuit) == 3
        assert circuit.two_qubit_count == 2
    
    def test_depth(self):
        import qns
        circuit = qns.Circuit(num_qubits=3)
        circuit.h(0)
        circuit.h(1)
        circuit.h(2)
        circuit.cnot(0, 1)
        assert circuit.depth >= 2
    
    def test_all_gate_types(self):
        import qns
        circuit = qns.Circuit(num_qubits=3)
        circuit.h(0)
        circuit.x(1)
        circuit.y(2)
        circuit.z(0)
        circuit.s(1)
        circuit.t(2)
        circuit.rx(0, 1.57)
        circuit.ry(1, 1.57)
        circuit.rz(2, 1.57)
        circuit.cnot(0, 1)
        circuit.cz(1, 2)
        circuit.swap(0, 2)
        circuit.measure(0)
        assert len(circuit) == 13
    
    def test_json_roundtrip(self):
        import qns
        circuit = qns.Circuit(num_qubits=3)
        circuit.h(0)
        circuit.cnot(0, 1)
        json_str = circuit.to_json()
        restored = qns.Circuit.from_json(json_str)
        assert len(restored) == len(circuit)
        assert restored.num_qubits == circuit.num_qubits


class TestGate:
    """Test Gate class."""
    
    def test_single_qubit_gates(self):
        import qns
        h = qns.Gate.h(0)
        assert h.name == "H"
        assert h.qubits == [0]
        assert not h.is_two_qubit
    
    def test_two_qubit_gates(self):
        import qns
        cnot = qns.Gate.cnot(0, 1)
        assert cnot.name == "CNOT"
        assert cnot.qubits == [0, 1]
        assert cnot.is_two_qubit
    
    def test_rotation_gates(self):
        import qns
        rx = qns.Gate.rx(0, 1.57)
        assert rx.angle == pytest.approx(1.57)
    
    def test_commutation(self):
        import qns
        h = qns.Gate.h(0)
        z = qns.Gate.z(1)
        assert h.commutes_with(z)  # Different qubits


class TestNoiseModel:
    """Test NoiseModel class."""
    
    def test_create_noise_model(self):
        import qns
        noise = qns.NoiseModel(t1=100.0, t2=80.0)
        assert noise.t1 == 100.0
        assert noise.t2 == 80.0
        assert noise.is_valid()
    
    def test_ideal_noise_model(self):
        import qns
        noise = qns.NoiseModel.ideal()
        assert noise.gate_error_1q == 0.0
        assert noise.gate_error_2q == 0.0
    
    def test_invalid_t2(self):
        import qns
        noise = qns.NoiseModel(t1=50.0, t2=120.0)  # T2 > 2*T1
        assert not noise.is_valid()


class TestNoiseVector:
    """Test NoiseVector class."""
    
    def test_create_noise_vector(self):
        import qns
        nv = qns.NoiseVector(qubit_id=0, t1=100.0, t2=80.0)
        assert nv.qubit_id == 0
        assert nv.t1 == 100.0
        assert nv.t2 == 80.0
    
    def test_t2_t1_ratio(self):
        import qns
        nv = qns.NoiseVector(qubit_id=0, t1=100.0, t2=80.0)
        assert nv.t2_t1_ratio == pytest.approx(0.8)


class TestOptimizer:
    """Test QnsOptimizer class."""
    
    def test_create_optimizer(self):
        import qns
        optimizer = qns.QnsOptimizer(num_qubits=3)
        assert optimizer.num_qubits == 3
    
    def test_optimize(self):
        import qns
        circuit = qns.Circuit(num_qubits=3)
        circuit.h(0)
        circuit.cnot(0, 1)
        circuit.cnot(1, 2)
        
        noise = qns.NoiseModel(t1=100.0, t2=80.0)
        optimizer = qns.QnsOptimizer(num_qubits=3, noise_model=noise)
        
        result = optimizer.optimize(circuit)
        assert result.optimized_score >= result.original_score * 0.99  # Allow small variation
        assert result.algorithm in ["bfs", "beam_search"]
    
    def test_score_function(self):
        import qns
        circuit = qns.Circuit(num_qubits=3)
        circuit.h(0)
        
        optimizer = qns.QnsOptimizer(num_qubits=3)
        score = optimizer.score(circuit)
        assert 0.0 <= score <= 1.0


class TestSimulatorBackend:
    """Test SimulatorBackend class."""
    
    def test_create_backend(self):
        import qns
        backend = qns.SimulatorBackend(num_qubits=3)
        assert backend.num_qubits == 3
        assert backend.is_available()
    
    def test_ideal_backend(self):
        import qns
        backend = qns.SimulatorBackend.ideal(num_qubits=3)
        assert backend.max_qubits == 20
    
    def test_run_circuit(self):
        import qns
        backend = qns.SimulatorBackend(num_qubits=3)
        
        circuit = qns.Circuit(num_qubits=3)
        circuit.h(0)
        circuit.cnot(0, 1)
        circuit.measure_all()
        
        counts = backend.run(circuit, shots=100)
        assert sum(counts.values()) == 100
        
        # Check gate representation
        gates = circuit.gates
        assert str(gates[0]) == 'H(0)'
        assert str(gates[1]) == 'CNOT(0, 1)'
    
    def test_noisy_simulation(self):
        import qns
        circuit = qns.Circuit(num_qubits=2)
        circuit.h(0)
        circuit.cnot(0, 1)
        circuit.measure_all()
        
        noise = qns.NoiseModel(t1=50.0, t2=40.0, gate_error_1q=0.01, gate_error_2q=0.05)
        backend = qns.SimulatorBackend(num_qubits=2, noise_model=noise)
        result = backend.run(circuit, shots=1000)
        
        # With noise, we expect some error states
        total = sum(result.counts.values())
        assert total == 1000
    
    def test_calibration(self):
        import qns
        backend = qns.SimulatorBackend(num_qubits=3)
        cal = backend.calibration()
        assert cal.num_qubits == 3
        assert cal.source == "simulator"
    
    def test_topology(self):
        import qns
        backend = qns.SimulatorBackend(num_qubits=5)
        hw = backend.topology()
        assert hw.num_qubits == 5
        # Linear topology
        assert hw.are_connected(0, 1)
        assert not hw.are_connected(0, 2)


class TestHardwareProfile:
    """Test HardwareProfile class."""
    
    def test_linear_topology(self):
        import qns
        hw = qns.HardwareProfile.linear("test", 5)
        assert hw.num_qubits == 5
        assert hw.are_connected(0, 1)
        assert hw.are_connected(1, 2)
        assert not hw.are_connected(0, 2)
    
    def test_ring_topology(self):
        import qns
        hw = qns.HardwareProfile.ring("test", 4)
        assert hw.are_connected(0, 1)
        assert hw.are_connected(3, 0)  # Ring wraps
    
    def test_fully_connected(self):
        import qns
        hw = qns.HardwareProfile.fully_connected("test", 4)
        assert hw.are_connected(0, 2)
        assert hw.are_connected(1, 3)
    
    def test_coupling_map(self):
        import qns
        hw = qns.HardwareProfile.linear("test", 3)
        coupling = hw.coupling_map()
        assert (0, 1) in coupling or (1, 0) in coupling


class TestConvert:
    """Test convert module."""
    
    def test_qasm_roundtrip(self):
        import qns
        circuit = qns.Circuit(num_qubits=3)
        circuit.h(0)
        circuit.cnot(0, 1)
        circuit.measure(0)
        
        qasm = qns.convert.circuit_to_qasm(circuit)
        restored = qns.convert.circuit_from_qasm(qasm)
        
        assert restored.num_qubits == circuit.num_qubits
        assert len(restored) == len(circuit)
    
    def test_dict_roundtrip(self):
        import qns
        circuit = qns.Circuit(num_qubits=2)
        circuit.h(0)
        circuit.cnot(0, 1)
        
        d = qns.convert.circuit_to_dict(circuit)
        assert d["num_qubits"] == 2
        assert len(d["gates"]) == 2
        
        restored = qns.convert.circuit_from_dict(d)
        assert len(restored) == len(circuit)


class TestUtilityFunctions:
    """Test utility functions."""
    
    def test_estimate_fidelity(self):
        import qns
        circuit = qns.Circuit(num_qubits=3)
        circuit.h(0)
        circuit.cnot(0, 1)
        
        noise = qns.NoiseVector(qubit_id=0, t1=100.0, t2=80.0)
        fidelity = qns.estimate_circuit_fidelity(circuit, noise)
        assert 0.0 <= fidelity <= 1.0
    
    def test_score_circuit(self):
        import qns
        circuit = qns.Circuit(num_qubits=3)
        circuit.h(0)
        
        noise = qns.NoiseVector(qubit_id=0, t1=100.0, t2=80.0)
        score = qns.score_circuit(circuit, noise)
        assert 0.0 <= score <= 1.0


class TestQiskitIntegration:
    """Test Qiskit integration."""
    
    def test_qiskit_qasm_conversion(self):
        try:
            from qiskit import QuantumCircuit
            from qiskit import qasm2
        except ImportError:
            pytest.skip("Qiskit not installed")
        
        import qns
        
        qc = QuantumCircuit(3)
        qc.h(0)
        qc.cx(0, 1)
        qc.cx(1, 2)
        
        qasm_str = qasm2.dumps(qc)
        circuit = qns.convert.circuit_from_qasm(qasm_str)
        
        assert circuit.num_qubits == 3
        assert circuit.two_qubit_count == 2


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
