"""
End-to-End Integration Test for QNS-Qiskit Bridge.

Tests the full pipeline: Rust → Python → Qiskit → Aer → Python → Rust

Gantree: Task1_4 E2E Integration Test
"""

import sys
from pathlib import Path

# Add module search paths
sys.path.insert(0, str(Path(__file__).parent.parent / 'python'))

# Import Rust bindings (built with maturin)
try:
    import qns
except ImportError:
    print("ERROR: qns module not found. Please run: maturin develop")
    sys.exit(1)

import qiskit_bridge


def test_e2e_bell_state_pipeline():
    """
    E2E Test: Rust→Python→Qiskit→Aer→Python→Rust
    
    Gantree atomic nodes:
        - CreateTestCircuitInRust
        - ConvertToPython
        - RunQiskitSimulator
        - ValidateResults
    """
    print("\n" + "="*60)
    print("E2E Test: Bell State Pipeline")
    print("="*60)
    
    # Step 1: Create circuit in Rust (CreateTestCircuitInRust)
    print("\n[1] Creating Bell state circuit in Rust...")
    circuit = qns.Circuit(2)
    circuit.add_gate(qns.Gate.h(0))
    circuit.add_gate(qns.Gate.cnot(0, 1))
    
    print(f"    Circuit created: {circuit.num_qubits()} qubits, "
          f"{circuit.num_gates()} gates")
    
    # Step 2: Convert to Qiskit (ConvertToPython)
    print("\n[2] Converting to Qiskit via Python bridge...")
    qns_gates = qns.convert_circuit_to_qiskit(circuit)
    
    print(f"    Converted {len(qns_gates)} gates")
    for i, gate in enumerate(qns_gates):
        print(f"      Gate {i}: {gate['name']} on qubits {gate['qubits']}")
    
    # Step 3: Verify circuit converter worked
    converter = qiskit_bridge.CircuitConverter()
    qiskit_circuit = converter.qns_to_qiskit(qns_gates, 2)
    
    print(f"    Qiskit circuit: {qiskit_circuit.num_qubits} qubits")
    
    # Step 4: Run Aer simulation (RunQiskitSimulator)
    print("\n[3] Running Aer simulation...")
    counts = qns.run_aer_simulation(circuit, shots=1024)
    
    print(f"    Measurement results: {counts}")
    
    # Step 5: Validate results (ValidateResults)
    print("\n[4] Validating results...")
    
    total_shots = sum(counts.values())
    assert total_shots == 1024, f"Expected 1024 shots, got {total_shots}"
    
    # Bell state should give ~50% |00⟩ and ~50% |11⟩
    assert '00' in counts, "Missing |00⟩ state"
    assert '11' in counts, "Missing |11⟩ state"
    
    prob_00 = counts['00'] / total_shots
    prob_11 = counts['11'] / total_shots
    
    print(f"    P(|00⟩) = {prob_00:.3f} (expected ~0.5)")
    print(f"    P(|11⟩) = {prob_11:.3f} (expected ~0.5)")
    
    # Allow 15% deviation for statistical fluctuation
    assert 0.35 <= prob_00 <= 0.65, f"P(|00⟩) out of range: {prob_00}"
    assert 0.35 <= prob_11 <= 0.65, f"P(|11⟩) out of range: {prob_11}"
    
    print("\n✅ E2E Test PASSED")
    print("="*60)


def test_e2e_ghz_state_pipeline():
    """
    E2E Test: GHZ state (3 qubits)
    """
    print("\n" + "="*60)
    print("E2E Test: GHZ State Pipeline")
    print("="*60)
    
    # Create GHZ circuit
    print("\n[1] Creating 3-qubit GHZ circuit in Rust...")
    circuit = qns.Circuit(3)
    circuit.add_gate(qns.Gate.h(0))
    circuit.add_gate(qns.Gate.cnot(0, 1))
    circuit.add_gate(qns.Gate.cnot(0, 2))
    
    print(f"    Circuit: {circuit.num_qubits()} qubits, {circuit.num_gates()} gates")
    
    # Run simulation
    print("\n[2] Running Aer simulation...")
    counts = qns.run_aer_simulation(circuit, shots=2048)
    
    print(f"    Measurement results: {counts}")
    
    # Validate
    print("\n[3] Validating results...")
    
    total_shots = sum(counts.values())
    assert total_shots == 2048
    
    # GHZ should give ~50% |000⟩ and ~50% |111⟩
    assert '000' in counts
    assert '111' in counts
    
    prob_000 = counts['000'] / total_shots
    prob_111 = counts['111'] / total_shots
    
    print(f"    P(|000⟩) = {prob_000:.3f} (expected ~0.5)")
    print(f"    P(|111⟩) = {prob_111:.3f} (expected ~0.5)")
    
    assert 0.35 <= prob_000 <= 0.65
    assert 0.35 <= prob_111 <= 0.65
    
    print("\n✅ GHZ Test PASSED")
    print("="*60)


def test_circuit_conversion_roundtrip():
    """
    Test roundtrip conversion: Rust → Python dict → Qiskit QuantumCircuit
    """
    print("\n" + "="*60)
    print("Roundtrip Test: Circuit Conversion")
    print("="*60)
    
    # Test all supported gate types
    print("\n[1] Creating circuit with all gate types...")
    circuit = qns.Circuit(4)
    
    # 1-qubit gates
    circuit.add_gate(qns.Gate.h(0))
    circuit.add_gate(qns.Gate.x(1))
    circuit.add_gate(qns.Gate.y(2))
    circuit.add_gate(qns.Gate.z(3))
    circuit.add_gate(qns.Gate.s(0))
    circuit.add_gate(qns.Gate.t(1))
    
    # Parametric gates
    circuit.add_gate(qns.Gate.rx(0, 0.5))
    circuit.add_gate(qns.Gate.ry(1, 1.0))
    circuit.add_gate(qns.Gate.rz(2, 1.5))
    
    # 2-qubit gates
    circuit.add_gate(qns.Gate.cnot(0, 1))
    circuit.add_gate(qns.Gate.cz(2, 3))
    circuit.add_gate(qns.Gate.swap(0, 3))
    
    print(f"    Created circuit with {circuit.num_gates()} gates")
    
    # Convert
    print("\n[2] Converting to Qiskit...")
    qns_gates = qns.convert_circuit_to_qiskit(circuit)
    
    converter = qiskit_bridge.CircuitConverter()
    qiskit_circuit = converter.qns_to_qiskit(qns_gates, 4)
    
    print(f"    Qiskit circuit: {qiskit_circuit.num_qubits} qubits, "
          f"{len(qiskit_circuit.data)} gates")
    
    # Validate gate count
    assert len(qiskit_circuit.data) == circuit.num_gates()
    
    print("\n✅ Roundtrip Test PASSED")
    print("="*60)


if __name__ == '__main__':
    import traceback
    
    tests = [
        ("Bell State E2E", test_e2e_bell_state_pipeline),
        ("GHZ State E2E", test_e2e_ghz_state_pipeline),
        ("Roundtrip Conversion", test_circuit_conversion_roundtrip),
    ]
    
    passed = 0
    failed = 0
    
    for test_name, test_func in tests:
        try:
            test_func()
            passed += 1
        except Exception as e:
            print(f"\n❌ {test_name} FAILED")
            print(f"Error: {e}")
            traceback.print_exc()
            failed += 1
    
    print("\n" + "="*60)
    print(f"E2E Integration Test Summary")
    print(f"PASSED: {passed}/{len(tests)}")
    print(f"FAILED: {failed}/{len(tests)}")
    print("="*60)
    
    sys.exit(0 if failed == 0 else 1)
