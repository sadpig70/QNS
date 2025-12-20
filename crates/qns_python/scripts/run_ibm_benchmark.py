#!/usr/bin/env python3
"""
QNS IBM Quantum Real Hardware Benchmark Script

This script runs QNS optimization benchmarks on real IBM Quantum hardware.
Requires an IBM Quantum account and API token.

Usage:
    # Set up credentials first:
    python -c "from qiskit_ibm_runtime import QiskitRuntimeService; QiskitRuntimeService.save_account(channel='ibm_quantum', token='YOUR_TOKEN')"
    
    # Then run:
    python run_ibm_benchmark.py --backend ibm_brisbane --shots 4000
    
    # Or with token directly:
    python run_ibm_benchmark.py --backend ibm_brisbane --token YOUR_TOKEN --shots 4000
"""

import argparse
import json
import time
from datetime import datetime
from pathlib import Path

def main():
    parser = argparse.ArgumentParser(description="QNS IBM Quantum Benchmark")
    parser.add_argument("--backend", default="ibm_brisbane", help="IBM backend name")
    parser.add_argument("--token", help="IBM Quantum API token")
    parser.add_argument("--shots", type=int, default=4000, help="Number of shots")
    parser.add_argument("--qubits", type=int, nargs="+", default=[3, 5], help="Qubit counts to test")
    parser.add_argument("--output", default="benchmark_results.json", help="Output file")
    parser.add_argument("--fake", action="store_true", help="Use fake backend instead")
    args = parser.parse_args()
    
    from qns.ibm import (
        IBMBackend, 
        run_benchmark_suite, 
        print_benchmark_summary,
        create_ghz_circuit,
        create_qft_circuit,
        create_random_circuit,
    )
    
    print("=" * 70)
    print("QNS IBM Quantum Real Hardware Benchmark")
    print("=" * 70)
    print(f"Timestamp: {datetime.now().isoformat()}")
    print(f"Backend: {args.backend}")
    print(f"Shots: {args.shots}")
    print(f"Qubit counts: {args.qubits}")
    print()
    
    # Create backend
    if args.fake:
        print("Using FAKE backend for testing...")
        backend_name = args.backend.replace("ibm_", "").replace("fake_", "")
        backend = IBMBackend.from_fake(backend_name)
    else:
        print("Connecting to IBM Quantum...")
        if args.token:
            backend = IBMBackend.from_service(args.backend, token=args.token)
        else:
            backend = IBMBackend.from_service(args.backend)
    
    print(f"Backend connected: {backend.name}")
    print(f"Number of qubits: {backend.num_qubits}")
    
    # Get calibration
    cal = backend.calibration
    print(f"\nCalibration Data (timestamp: {cal.timestamp}):")
    print(f"  T1 range: {min(cal.t1_times.values()):.1f} - {max(cal.t1_times.values()):.1f} μs")
    print(f"  T2 range: {min(cal.t2_times.values()):.1f} - {max(cal.t2_times.values()):.1f} μs")
    if cal.gate_errors_1q:
        print(f"  1Q error range: {min(cal.gate_errors_1q.values()):.6f} - {max(cal.gate_errors_1q.values()):.6f}")
    if cal.gate_errors_2q:
        print(f"  2Q error range: {min(cal.gate_errors_2q.values()):.6f} - {max(cal.gate_errors_2q.values()):.6f}")
    print(f"  Readout error range: {min(cal.readout_errors.values()):.6f} - {max(cal.readout_errors.values()):.6f}")
    print(f"  Coupling map edges: {len(cal.coupling_map)}")
    
    # Filter qubit counts
    valid_qubits = [n for n in args.qubits if n <= backend.num_qubits]
    if not valid_qubits:
        print(f"ERROR: No valid qubit counts. Backend has {backend.num_qubits} qubits.")
        return
    
    print(f"\nRunning benchmarks for {valid_qubits} qubits...")
    
    # Run benchmarks
    results = run_benchmark_suite(backend, num_qubits_list=valid_qubits, shots=args.shots)
    
    # Print summary
    print_benchmark_summary(results)
    
    # Save results
    output_data = {
        "timestamp": datetime.now().isoformat(),
        "backend": backend.name,
        "num_qubits": backend.num_qubits,
        "shots": args.shots,
        "calibration": cal.to_dict(),
        "results": [r.to_dict() for r in results],
    }
    
    output_path = Path(args.output)
    with open(output_path, "w") as f:
        json.dump(output_data, f, indent=2, default=str)
    
    print(f"\nResults saved to: {output_path}")
    
    # Analysis
    print("\n" + "=" * 70)
    print("ANALYSIS")
    print("=" * 70)
    
    total_improvement = sum(r.fidelity_improvement for r in results)
    avg_improvement = total_improvement / len(results) if results else 0
    avg_baseline = sum(r.baseline_fidelity for r in results) / len(results) if results else 0
    avg_qns = sum(r.qns_fidelity for r in results) / len(results) if results else 0
    
    print(f"Average baseline fidelity: {avg_baseline:.4f}")
    print(f"Average QNS fidelity: {avg_qns:.4f}")
    print(f"Average improvement: {avg_improvement:.4f} ({avg_improvement/avg_baseline*100 if avg_baseline > 0 else 0:.2f}%)")
    
    # Check if any improvement
    improved = [r for r in results if r.fidelity_improvement > 0]
    if improved:
        print(f"\nCircuits with improvement: {len(improved)}/{len(results)}")
        for r in improved:
            pct = r.fidelity_improvement / r.baseline_fidelity * 100 if r.baseline_fidelity > 0 else 0
            print(f"  {r.circuit_name}: +{pct:.2f}%")
    else:
        print("\nNo fidelity improvement detected.")
        print("This is expected for simple circuits where gate ordering is already optimal.")
        print("QNS optimization shows more benefit for:")
        print("  - Circuits with commutable gate sequences")
        print("  - Higher noise environments")
        print("  - Circuits where timing matters (decoherence-sensitive)")


if __name__ == "__main__":
    main()
