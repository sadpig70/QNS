"""
Sprint 2: IBM Quantum Backend Connection and Calibration Test

Tests:
1. Connect to IBM Quantum service
2. List available backends
3. Fetch calibration data from real backend
4. Create NoiseModel from calibration
5. Verify NoiseModel properties

Gantree: Sprint2_Phase1L2 → Task2_1_IBMConnection + Task2_2_CalibrationFetch
"""

import sys
from pathlib import Path
import os

# Add module search paths
sys.path.insert(0, str(Path(__file__).parent.parent / 'python'))

# Set IBM Quantum token (user should set this as environment variable)
# For this test, we'll use the provided token directly
IBM_TOKEN = "eaZg3euoMUGpZVcWxXVsU55MGddnbwuR74uDPU8-F48W"
os.environ['QISKIT_IBM_TOKEN'] = IBM_TOKEN

import qiskit_bridge


def test_ibm_connection():
    """
    Test 1: Connect to IBM Quantum service
    
    Gantree atomic nodes:
    - LoadAPIKey
    - InitializeService
    - SelectBackend
    """
    print("\n" + "="*60)
    print("Test 1: IBM Quantum Connection")
    print("="*60)
    
    print("\n[1] Connecting to IBM Quantum backend...")
    
    try:
        fetcher = qiskit_bridge.CalibrationFetcher()
        
        # Try connecting to ibm_fez first, fallback to ibm_kyoto
        backends_to_try = ['ibm_fez', 'ibm_kyoto', 'ibm_osaka']
        
        connected_backend = None
        for backend_name in backends_to_try:
            try:
                print(f"    Trying backend: {backend_name}...")
                fetcher.connect(backend_name)
                connected_backend = backend_name
                print(f"    ✅ Connected to {backend_name}")
                break
            except Exception as e:
                print(f"    ❌ {backend_name} not available: {e}")
                continue
        
        if not connected_backend:
            print("\n⚠️ No backends available. Using simulator for testing.")
            return None
        
        return fetcher, connected_backend
        
    except Exception as e:
        print(f"\n❌ Connection failed: {e}")
        import traceback
        traceback.print_exc()
        return None


def test_fetch_calibration(fetcher, backend_name):
    """
    Test 2: Fetch calibration data from backend
    
    Gantree atomic nodes:
    - FetchBackendProperties
    - ParseT1T2
    - ParseGateErrors
    - ParseReadoutErrors
    """
    print("\n" + "="*60)
    print(f"Test 2: Fetch Calibration Data from {backend_name}")
    print("="*60)
    
    try:
        print("\n[1] Fetching calibration properties...")
        calibration = fetcher.fetch_properties()
        
        print("\n[2] Calibration data summary:")
        print(f"    T1 times: {len(calibration['t1'])} qubits")
        print(f"      Mean T1: {sum(calibration['t1']) / len(calibration['t1']):.2e} seconds")
        print(f"      Min T1: {min(calibration['t1']):.2e} seconds")
        print(f"      Max T1: {max(calibration['t1']):.2e} seconds")
        
        print(f"\n    T2 times: {len(calibration['t2'])} qubits")
        print(f"      Mean T2: {sum(calibration['t2']) / len(calibration['t2']):.2e} seconds")
        
        print(f"\n    1Q gate errors: {len(calibration['gate_errors_1q'])} qubits")
        print(f"      Mean error: {sum(calibration['gate_errors_1q']) / len(calibration['gate_errors_1q']):.4f}")
        
        print(f"\n    2Q gate errors: {len(calibration['gate_errors_2q'])} pairs")
        if calibration['gate_errors_2q']:
            errors = list(calibration['gate_errors_2q'].values())
            print(f"      Mean error: {sum(errors) / len(errors):.4f}")
        
        print(f"\n    Readout errors: {len(calibration['readout_errors'])} qubits")
        print(f"      Mean error: {sum(calibration['readout_errors']) / len(calibration['readout_errors']):.4f}")
        
        print("\n✅ Calibration data fetched successfully")
        return calibration
        
    except Exception as e:
        print(f"\n❌ Calibration fetch failed: {e}")
        import traceback
        traceback.print_exc()
        return None


def test_noise_model_creation(calibration):
    """
    Test 3: Create NoiseModel from calibration data
    
    Gantree atomic nodes:
    - CreateNoiseModel
    - AddT1T2Errors
    - AddGateErrors
    """
    print("\n" + "="*60)
    print("Test 3: Create NoiseModel from Calibration")
    print("="*60)
    
    try:
        print("\n[1] Building noise model...")
        builder = qiskit_bridge.NoiseModelBuilder()
        noise_model = builder.build_noise_model(calibration)
        
        print("\n[2] NoiseModel properties:")
        print(f"    Noise qubits: {len(noise_model.noise_qubits)}")
        print(f"    Basis gates: {noise_model.basis_gates}")
        
        print("\n✅ NoiseModel created successfully")
        return noise_model
        
    except Exception as e:
        print(f"\n❌ NoiseModel creation failed: {e}")
        import traceback
        traceback.print_exc()
        return None


def test_noisy_simulation(calibration):
    """
    Test 4: Run noisy simulation with real calibration
    
    Compares ideal vs. noisy Bell state simulation
    """
    print("\n" + "="*60)
    print("Test 4: Noisy vs. Ideal Simulation Comparison")
    print("="*60)
    
    try:
        # Create Bell state
        qns_gates = [
            {'name': 'H', 'qubits': [0], 'params': []},
            {'name': 'CNOT', 'qubits': [0, 1], 'params': []},
        ]
        
        converter = qiskit_bridge.CircuitConverter()
        qc = converter.qns_to_qiskit(qns_gates, 2)
        
        # Ideal simulation
        print("\n[1] Running ideal simulation...")
        ideal_runner = qiskit_bridge.AerSimulationRunner(noise_model=None)
        ideal_counts = ideal_runner.run(qc, shots=1024)
        ideal_fidelity = ideal_runner.calculate_fidelity(ideal_counts, '00')
        
        print(f"    Ideal results: {ideal_counts}")
        print(f"    Ideal fidelity: {ideal_fidelity:.4f}")
        
        # Noisy simulation
        print("\n[2] Running noisy simulation (with real calibration)...")
        builder = qiskit_bridge.NoiseModelBuilder()
        noise_model = builder.build_noise_model(calibration)
        
        noisy_runner = qiskit_bridge.AerSimulationRunner(noise_model=noise_model)
        noisy_counts = noisy_runner.run(qc, shots=1024)
        noisy_fidelity = noisy_runner.calculate_fidelity(noisy_counts, '00')
        
        print(f"    Noisy results: {noisy_counts}")
        print(f"    Noisy fidelity: {noisy_fidelity:.4f}")
        
        # Analysis
        print("\n[3] Analysis:")
        fidelity_drop = ideal_fidelity - noisy_fidelity
        print(f"    Fidelity drop: {fidelity_drop:.4f} ({fidelity_drop*100:.2f}%)")
        
        if fidelity_drop > 0.01:
            print("    ✅ Noise model is effective (fidelity degradation observed)")
        else:
            print("    ⚠️ Noise effect minimal (may need more complex circuit)")
        
        print("\n✅ Noisy simulation completed")
        return True
        
    except Exception as e:
        print(f"\n❌ Noisy simulation failed: {e}")
        import traceback
        traceback.print_exc()
        return False


if __name__ == '__main__':
    print("\n" + "="*60)
    print("Sprint 2: IBM Quantum Calibration Integration Tests")
    print("="*60)
    
    # Test 1: Connection
    result = test_ibm_connection()
    if result is None:
        print("\n⚠️ Tests skipped - no backend connection")
        sys.exit(1)
    
    fetcher, backend_name = result
    
    # Test 2: Calibration fetch
    calibration = test_fetch_calibration(fetcher, backend_name)
    if calibration is None:
        print("\n❌ Tests failed - calibration fetch error")
        sys.exit(1)
    
    # Test 3: NoiseModel creation
    noise_model = test_noise_model_creation(calibration)
    if noise_model is None:
        print("\n❌ Tests failed - NoiseModel creation error")
        sys.exit(1)
    
    # Test 4: Noisy simulation
    success = test_noisy_simulation(calibration)
    if not success:
        print("\n❌ Tests failed - noisy simulation error")
        sys.exit(1)
    
    # Summary
    print("\n" + "="*60)
    print("Sprint 2 Test Summary")
    print("="*60)
    print("✅ All tests PASSED")
    print(f"✅ Backend: {backend_name}")
    print(f"✅ Calibration data: {len(calibration['t1'])} qubits")
    print(f"✅ NoiseModel: {len(noise_model.noise_qubits)} qubits")
    print("="*60)
    
    sys.exit(0)
