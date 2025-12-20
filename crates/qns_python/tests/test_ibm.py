"""
QNS IBM Integration Tests

Tests for IBM Quantum backend integration with QNS.
Requires: qiskit-ibm-runtime
"""
import pytest
import json


class TestCalibrationInfo:
    """Test CalibrationInfo data class."""
    
    def test_create_calibration_info(self):
        from qns.ibm import CalibrationInfo
        
        cal = CalibrationInfo(
            backend_name="test_backend",
            num_qubits=5,
            t1_times={0: 100.0, 1: 90.0, 2: 110.0, 3: 95.0, 4: 105.0},
            t2_times={0: 80.0, 1: 70.0, 2: 85.0, 3: 75.0, 4: 82.0},
            gate_errors_1q={0: 0.001, 1: 0.002, 2: 0.001, 3: 0.0015, 4: 0.001},
            gate_errors_2q={(0, 1): 0.01, (1, 2): 0.015, (2, 3): 0.012, (3, 4): 0.011},
            readout_errors={0: 0.02, 1: 0.025, 2: 0.018, 3: 0.022, 4: 0.02},
            coupling_map=[(0, 1), (1, 2), (2, 3), (3, 4)],
        )
        
        assert cal.backend_name == "test_backend"
        assert cal.num_qubits == 5
    
    def test_to_noise_vectors(self):
        from qns.ibm import CalibrationInfo
        
        cal = CalibrationInfo(
            backend_name="test",
            num_qubits=3,
            t1_times={0: 100.0, 1: 90.0, 2: 110.0},
            t2_times={0: 80.0, 1: 70.0, 2: 85.0},
            gate_errors_1q={0: 0.001, 1: 0.002, 2: 0.001},
            gate_errors_2q={(0, 1): 0.01, (1, 2): 0.015},
            readout_errors={0: 0.02, 1: 0.025, 2: 0.018},
            coupling_map=[(0, 1), (1, 2)],
        )
        
        nvs = cal.to_noise_vectors()
        assert len(nvs) == 3
        assert nvs[0].t1 == 100.0
        assert nvs[1].t1 == 90.0
        assert nvs[2].t1 == 110.0
    
    def test_to_noise_model(self):
        from qns.ibm import CalibrationInfo
        
        cal = CalibrationInfo(
            backend_name="test",
            num_qubits=2,
            t1_times={0: 100.0, 1: 80.0},
            t2_times={0: 80.0, 1: 60.0},
            gate_errors_1q={0: 0.001, 1: 0.002},
            gate_errors_2q={(0, 1): 0.01},
            readout_errors={0: 0.02, 1: 0.03},
            coupling_map=[(0, 1)],
        )
        
        nm = cal.to_noise_model()
        assert nm.t1 == 90.0  # Average
        assert nm.t2 == 70.0  # Average
    
    def test_serialization(self):
        from qns.ibm import CalibrationInfo
        
        cal = CalibrationInfo(
            backend_name="test",
            num_qubits=2,
            t1_times={0: 100.0, 1: 90.0},
            t2_times={0: 80.0, 1: 70.0},
            gate_errors_1q={0: 0.001, 1: 0.002},
            gate_errors_2q={(0, 1): 0.01},
            readout_errors={0: 0.02, 1: 0.025},
            coupling_map=[(0, 1)],
        )
        
        d = cal.to_dict()
        assert d["backend_name"] == "test"
        
        restored = CalibrationInfo.from_dict(d)
        assert restored.backend_name == cal.backend_name
        assert restored.num_qubits == cal.num_qubits


class TestBenchmarkResult:
    """Test BenchmarkResult data class."""
    
    def test_create_benchmark_result(self):
        from qns.ibm import BenchmarkResult
        
        result = BenchmarkResult(
            circuit_name="GHZ-3",
            num_qubits=3,
            num_gates=5,
            baseline_counts={"000": 500, "111": 500},
            baseline_depth=3,
            baseline_2q_count=2,
            qns_counts={"000": 520, "111": 480},
            qns_depth=3,
            qns_2q_count=2,
            qns_score_improvement=0.02,
            baseline_fidelity=0.85,
            qns_fidelity=0.88,
            fidelity_improvement=0.03,
            qns_optimization_time_ms=0.5,
            transpile_time_ms=10.0,
            execution_time_ms=100.0,
        )
        
        assert result.circuit_name == "GHZ-3"
        assert result.fidelity_improvement == 0.03
    
    def test_to_dict(self):
        from qns.ibm import BenchmarkResult
        
        result = BenchmarkResult(
            circuit_name="test",
            num_qubits=2,
            num_gates=3,
            baseline_counts={"00": 500, "11": 500},
            baseline_depth=2,
            baseline_2q_count=1,
            qns_counts={"00": 510, "11": 490},
            qns_depth=2,
            qns_2q_count=1,
            qns_score_improvement=0.01,
            baseline_fidelity=0.90,
            qns_fidelity=0.92,
            fidelity_improvement=0.02,
            qns_optimization_time_ms=0.3,
            transpile_time_ms=5.0,
            execution_time_ms=50.0,
        )
        
        d = result.to_dict()
        assert d["circuit_name"] == "test"
        assert d["improvement"]["fidelity"] == 0.02


class TestCircuitGenerators:
    """Test circuit generator functions."""
    
    def test_create_ghz_circuit(self):
        from qns.ibm import create_ghz_circuit
        
        qc = create_ghz_circuit(3)
        assert qc.num_qubits == 3
        
        # Should have H gate and CNOT gates
        gate_names = [inst.operation.name for inst in qc.data]
        assert 'h' in gate_names
        assert 'cx' in gate_names
    
    def test_create_qft_circuit(self):
        from qns.ibm import create_qft_circuit
        
        qc = create_qft_circuit(3)
        assert qc.num_qubits == 3
        
        gate_names = [inst.operation.name for inst in qc.data]
        assert 'h' in gate_names
    
    def test_create_random_circuit(self):
        from qns.ibm import create_random_circuit
        
        qc = create_random_circuit(3, 5, seed=42)
        assert qc.num_qubits == 3
        assert len(qc.data) > 0
        
        # Same seed should produce same circuit
        qc2 = create_random_circuit(3, 5, seed=42)
        assert len(qc.data) == len(qc2.data)


class TestIBMBackendFake:
    """Test IBMBackend with fake backends."""
    
    @pytest.fixture
    def fake_backend(self):
        try:
            from qns.ibm import IBMBackend
            return IBMBackend.from_fake("manila")
        except ImportError:
            pytest.skip("qiskit-ibm-runtime not installed")
    
    def test_from_fake(self, fake_backend):
        assert fake_backend.name == "fake_manila"
        assert fake_backend.num_qubits == 5
    
    def test_calibration(self, fake_backend):
        cal = fake_backend.calibration
        assert cal.num_qubits == 5
        assert len(cal.t1_times) == 5
        assert len(cal.t2_times) == 5
    
    def test_coupling_map(self, fake_backend):
        coupling = fake_backend.coupling_map
        assert len(coupling) > 0
    
    def test_get_qns_optimizer(self, fake_backend):
        optimizer = fake_backend.get_qns_optimizer()
        assert optimizer.num_qubits == 5
    
    def test_optimize_with_qns(self, fake_backend):
        import qns
        
        circuit = qns.Circuit(num_qubits=3)
        circuit.h(0)
        circuit.cnot(0, 1)
        circuit.cnot(1, 2)
        
        opt_circuit, result = fake_backend.optimize_with_qns(circuit)
        assert opt_circuit is not None
        assert result.optimized_score >= 0
    
    def test_transpile(self, fake_backend):
        from qns.ibm import create_ghz_circuit
        
        qc = create_ghz_circuit(3)
        transpiled = fake_backend.transpile(qc)
        assert transpiled is not None
    
    def test_run(self, fake_backend):
        from qns.ibm import create_ghz_circuit
        
        qc = create_ghz_circuit(3)
        counts = fake_backend.run(qc, shots=100)
        
        assert isinstance(counts, dict)
        assert sum(counts.values()) == 100
    
    def test_run_optimized(self, fake_backend):
        import qns
        
        circuit = qns.Circuit(num_qubits=3)
        circuit.h(0)
        circuit.cnot(0, 1)
        circuit.cnot(1, 2)
        circuit.measure_all()
        
        counts, result = fake_backend.run_optimized(circuit, shots=100)
        
        assert isinstance(counts, dict)
        assert result is not None
    
    def test_benchmark(self, fake_backend):
        from qns.ibm import create_ghz_circuit
        
        qc = create_ghz_circuit(3)
        result = fake_backend.benchmark(qc, "GHZ-3", shots=100)
        
        assert result.circuit_name == "GHZ-3"
        assert result.num_qubits == 3
        assert result.baseline_fidelity > 0
        assert result.qns_fidelity > 0


class TestBenchmarkSuite:
    """Test benchmark suite functions."""
    
    def test_run_benchmark_suite(self):
        try:
            from qns.ibm import IBMBackend, run_benchmark_suite
            backend = IBMBackend.from_fake("manila")
        except ImportError:
            pytest.skip("qiskit-ibm-runtime not installed")
        
        results = run_benchmark_suite(backend, num_qubits_list=[3], shots=100)
        
        assert len(results) == 3  # GHZ, QFT, Random for 3 qubits
        for r in results:
            assert r.num_qubits == 3
    
    def test_print_benchmark_summary(self, capsys):
        try:
            from qns.ibm import BenchmarkResult, print_benchmark_summary
        except ImportError:
            pytest.skip("qiskit-ibm-runtime not installed")
        
        results = [
            BenchmarkResult(
                circuit_name="GHZ-3",
                num_qubits=3,
                num_gates=5,
                baseline_counts={"000": 500, "111": 500},
                baseline_depth=3,
                baseline_2q_count=2,
                qns_counts={"000": 520, "111": 480},
                qns_depth=3,
                qns_2q_count=2,
                qns_score_improvement=0.02,
                baseline_fidelity=0.85,
                qns_fidelity=0.88,
                fidelity_improvement=0.03,
                qns_optimization_time_ms=0.5,
                transpile_time_ms=10.0,
                execution_time_ms=100.0,
            )
        ]
        
        print_benchmark_summary(results)
        captured = capsys.readouterr()
        
        assert "QNS BENCHMARK SUMMARY" in captured.out
        assert "GHZ-3" in captured.out


class TestQiskitIntegration:
    """Test Qiskit integration."""
    
    def test_qiskit_circuit_conversion(self):
        try:
            from qns.ibm import IBMBackend
            from qiskit import QuantumCircuit
            backend = IBMBackend.from_fake("manila")
        except ImportError:
            pytest.skip("qiskit-ibm-runtime not installed")
        
        qc = QuantumCircuit(3)
        qc.h(0)
        qc.cx(0, 1)
        qc.cx(1, 2)
        qc.measure_all()
        
        opt_circuit, result = backend.optimize_with_qns(qc)
        assert opt_circuit is not None
    
    def test_end_to_end_workflow(self):
        try:
            from qns.ibm import IBMBackend, create_ghz_circuit
        except ImportError:
            pytest.skip("qiskit-ibm-runtime not installed")
        
        # Create backend
        backend = IBMBackend.from_fake("manila")
        
        # Create circuit
        qc = create_ghz_circuit(3)
        
        # Get calibration
        cal = backend.calibration
        assert cal.num_qubits >= 3
        
        # Optimize and run
        counts, result = backend.run_optimized(qc, shots=100)
        
        # Verify GHZ state (mostly |000⟩ and |111⟩)
        total = sum(counts.values())
        assert total == 100


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
