"""
Simple E2E test that doesn't require qns module import.

Tests qiskit_bridge directly without Rust integration.
"""

import sys
from pathlib import Path

# Add module search paths
sys.path.insert(0, str(Path(__file__).parent.parent / 'python'))

import qiskit_bridge


def test_qiskit_bridge_direct():
    """
    Direct test of qiskit_bridge without Rust module.
    
    Tests:
    - CircuitConverter with manually created gate list
    - AerSimulationRunner
    - Fidelity calculation
    """
    print("\n" + "="*60)
    print("Direct Qiskit Bridge Test (No Rust Module)")
    print("="*60)
    
    # Create Bell state manually
    print("\n[1] Creating Bell state circuit (manual)...")
    qns_gates = [
        {'name': 'H', 'qubits': [0], 'params': []},
        {'name': 'CNOT', 'qubits': [0, 1], 'params': []},
    ]
    
    # Convert to Qiskit
    print("[2] Converting to Qiskit...")
    converter = qiskit_bridge.CircuitConverter()
    qiskit_circuit = converter.qns_to_qiskit(qns_gates, 2)
    
    print(f"    Circuit: {qiskit_circuit.num_qubits} qubits, "
          f"{len(qiskit_circuit.data)} gates")
    
    # Run simulation
    print("[3] Running Aer simulation...")
    runner = qiskit_bridge.AerSimulationRunner()
    counts = runner.run(qiskit_circuit, shots=1024)
    
    print(f"    Results: {counts}")
    
    # Calculate fidelity
    print("[4] Validating results...")
    fidelity = runner.calculate_fidelity(counts, '00')
    
    print(f"    Fidelity to |00⟩: {fidelity:.3f}")
    
    # Assert
    assert '00' in counts
    assert '11' in counts
    assert 0.3 <= fidelity <= 0.7  # Allow statistical fluctuation
    
    print("\n✅ Direct Bridge Test PASSED")
    print("="*60)
    return True


if __name__ == '__main__':
    try:
        test_qiskit_bridge_direct()
        print("\n✅ All tests PASSED")
        sys.exit(0)
    except Exception as e:
        print(f"\n❌ Test FAILED: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
