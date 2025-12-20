#!/usr/bin/env python3
"""
QNS CLI Runner for Qiskit Backends

Executes quantum circuits using Qiskit backends:
- aer-ideal: Ideal statevector simulation
- aer-noisy: Noisy simulation with calibration data
- aer-ibm: Noisy simulation with IBM backend calibration

Gantree: Sprint4_Phase1L3 → Task4_2_CLIRunner
"""

import argparse
import json
import sys
from pathlib import Path

# Add qiskit_bridge to path
sys.path.insert(0, str(Path(__file__).parent))

import qiskit_bridge


def parse_qasm_file(filepath: str):
    """Parse QASM file (simple parser for basic circuits)."""
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Extract number of qubits
    num_qubits = 0
    gates = []
    
    for line in content.splitlines():
        line = line.strip()
        
        # Skip comments and empty lines
        if not line or line.startswith('//'):
            continue
        
        # Parse qubit declaration
        if line.startswith('qreg'):
            # Example: qreg q[2];
            parts = line.split('[')
            if len(parts) > 1:
                num_qubits = int(parts[1].split(']')[0])
        
        # Parse gates (simplified)
        elif any(gate in line.lower() for gate in ['h', 'x', 'y', 'z', 'rx', 'ry', 'rz', 'cx', 'cnot']):
            # Example: h q[0];
            # Example: cx q[0], q[1];
            gate_name = line.split()[0].upper()
            
            # Map CX to CNOT
            if gate_name == 'CX':
                gate_name = 'CNOT'
            
            # Extract qubit indices
            qubit_str = line.split('[')[1:]
            qubits = [int(s.split(']')[0]) for s in qubit_str]
            
            # Extract parameters for rotations
            params = []
            if any(g in gate_name for g in ['RX', 'RY', 'RZ']):
                param_str = line.split('(')[1].split(')')[0]
                params = [float(param_str)]
            
            gates.append({
                'name': gate_name,
                'qubits': qubits,
                'params': params
            })
    
    return num_qubits, gates


def run_backend(backend_type: str, gates: list, num_qubits: int, shots: int, ibm_backend: str = None):
    """Run circuit on specified backend."""
    
    converter = qiskit_bridge.CircuitConverter()
    
    if backend_type == 'aer-ideal':
        # Ideal Aer simulation
        runner = qiskit_bridge.AerSimulationRunner(noise_model=None)
        qc = converter.qns_to_qiskit(gates, num_qubits)
        counts = runner.run(qc, shots=shots)
        
        result = {
            'backend': 'aer-ideal',
            'num_qubits': num_qubits,
            'num_gates': len(gates),
            'shots': shots,
            'counts': counts,
            'fidelity': runner.calculate_fidelity(counts, '0' * num_qubits),
        }
        
    elif backend_type == 'aer-noisy':
        # Noisy Aer with default calibration
        # Use mock calibration data
        mock_calibration = {
            't1': [100e-6] * num_qubits,
            't2': [80e-6] * num_qubits,
            'gate_errors_1q': [0.001] * num_qubits,
            'gate_errors_2q': {},  # No 2Q gates for simplicity
            'readout_errors': [0.01] * num_qubits,
        }
        
        builder = qiskit_bridge.NoiseModelBuilder()
        noise_model = builder.build_noise_model(mock_calibration)
        
        runner = qiskit_bridge.AerSimulationRunner(noise_model=noise_model)
        qc = converter.qns_to_qiskit(gates, num_qubits)
        counts = runner.run(qc, shots=shots)
        
        result = {
            'backend': 'aer-noisy',
            'num_qubits': num_qubits,
            'num_gates': len(gates),
            'shots': shots,
            'counts': counts,
            'fidelity': runner.calculate_fidelity(counts, '0' * num_qubits),
            'noise_model': 'mock_calibration',
        }
        
    elif backend_type == 'aer-ibm':
        if not ibm_backend:
            raise ValueError("--ibm-backend required for aer-ibm backend type")
        
        # Fetch real IBM calibration
        import os
        fetcher = qiskit_bridge.CalibrationFetcher()
        fetcher.connect(ibm_backend)
        calibration = fetcher.fetch_properties()
        
        # Build noise model
        builder = qiskit_bridge.NoiseModelBuilder()
        noise_model = builder.build_noise_model(calibration)
        
        # Run simulation
        runner = qiskit_bridge.AerSimulationRunner(noise_model=noise_model)
        qc = converter.qns_to_qiskit(gates, num_qubits)
        counts = runner.run(qc, shots=shots)
        
        result = {
            'backend': 'aer-ibm',
            'ibm_backend': ibm_backend,
            'num_qubits': num_qubits,
            'num_gates': len(gates),
            'shots': shots,
            'counts': counts,
            'fidelity': runner.calculate_fidelity(counts, '0' * num_qubits),
            'calibration_summary': {
                't1_mean': sum(calibration['t1']) / len(calibration['t1']),
                't2_mean': sum(calibration['t2']) / len(calibration['t2']),
                'gate_error_mean': sum(calibration['gate_errors_1q']) / len(calibration['gate_errors_1q']),
            }
        }
    
    else:
        raise ValueError(f"Unknown backend type: {backend_type}")
    
    return result


def main():
    parser = argparse.ArgumentParser(description='QNS Qiskit Backend Runner')
    parser.add_argument('--input', required=True, help='Path to QASM file')
    parser.add_argument('--backend', required=True, choices=['aer-ideal', 'aer-noisy', 'aer-ibm'],
                        help='Backend type')
    parser.add_argument('--ibm-backend', help='IBM backend name (for aer-ibm)')
    parser.add_argument('--shots', type=int, default=1024, help='Number of shots')
    parser.add_argument('--format', choices=['text', 'json'], default='json')
    
    args = parser.parse_args()
    
    try:
        # Parse QASM
        num_qubits, gates = parse_qasm_file(args.input)
        
        # Run on backend
        result = run_backend(args.backend, gates, num_qubits, args.shots, args.ibm_backend)
        
        # Output result
        if args.format == 'json':
            print(json.dumps(result, indent=2))
        else:
            print(f"\n=== QNS Qiskit Backend Result ===")
            print(f"Backend:    {result['backend']}")
            if 'ibm_backend' in result:
                print(f"IBM Backend: {result['ibm_backend']}")
            print(f"Qubits:     {result['num_qubits']}")
            print(f"Gates:      {result['num_gates']}")
            print(f"Shots:      {result['shots']}")
            print(f"\nCounts:     {result['counts']}")
            print(f"Fidelity:   {result['fidelity']:.4f}")
            
            if 'calibration_summary' in result:
                cal = result['calibration_summary']
                print(f"\nCalibration:")
                print(f"  T1 mean:    {cal['t1_mean']:.2e} s")
                print(f"  T2 mean:    {cal['t2_mean']:.2e} s")
                print(f"  Gate error: {cal['gate_error_mean']:.4f}")
        
        sys.exit(0)
        
    except Exception as e:
        print(f"ERROR: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc(file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
