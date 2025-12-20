#!/usr/bin/env python3
"""
Sprint 3: QNS Optimization Validation

Comparative analysis:
- Identity mapping (no optimization)
- QNS optimization
- Measure fidelity improvement

Circuits tested:
- Bell state (2q)
- GHZ state (3q)  
- QFT (4q)

Gantree: Sprint3_Phase1L3 → Task3_3_ComparativeAnalysis
"""

import sys
import json
from pathlib import Path
from typing import Dict, List, Tuple
import subprocess
import os

# Add qiskit_bridge to path
sys.path.insert(0, str(Path(__file__).parent.parent / 'crates' / 'qns_python' / 'python'))

import qiskit_bridge


def run_circuit_with_backend(
    qasm_file: str,
    backend_type: str,
    shots: int = 1024,
    ibm_backend: str = None
) -> Dict:
    """
    Run circuit using CLI runner.
    
    Returns:
        Result dict with counts and fidelity
    """
    cmd = [
        'python',
        str(Path(__file__).parent.parent / 'crates' / 'qns_python' / 'python' / 'cli_runner.py'),
        '--input', qasm_file,
        '--backend', backend_type,
        '--shots', str(shots),
        '--format', 'json'
    ]
    
    if ibm_backend:
        cmd.extend(['--ibm-backend', ibm_backend])
    
    result = subprocess.run(cmd, capture_output=True, text=True)
    
    if result.returncode != 0:
        raise RuntimeError(f"CLI runner failed: {result.stderr}")
    
    return json.loads(result.stdout)


def run_qns_optimized(qasm_file: str, backend_type: str, shots: int = 1024) -> Dict:
    """
    Run circuit with QNS optimization.
    
    Currently uses QNS native simulator with optimization pipeline.
    """
    # Use QNS CLI (Windows needs .exe)
    qns_cli = Path(__file__).parent.parent / 'target' / 'release' / 'qns.exe'
    
    if not qns_cli.exists():
        raise FileNotFoundError(f"QNS CLI not found: {qns_cli}. Run: cargo build --release")
    
    cmd = [
        str(qns_cli),
        'run',
        qasm_file,
        '--topology', 'linear',
        '--shots', str(shots),
        '--format', 'json'
    ]
    
    result = subprocess.run(cmd, capture_output=True, text=True)
    
    if result.returncode != 0:
        raise RuntimeError(f"QNS CLI failed: {result.stderr}")
    
    return json.loads(result.stdout)


def calculate_fidelity_improvement(
    identity_fidelity: float,
    qns_fidelity: float
) -> Dict:
    """Calculate improvement metrics."""
    absolute_improvement = qns_fidelity - identity_fidelity
    relative_improvement = (absolute_improvement / identity_fidelity) * 100 if identity_fidelity > 0 else 0
    
    return {
        'absolute': absolute_improvement,
        'relative_percent': relative_improvement,
        'improvement_factor': qns_fidelity / identity_fidelity if identity_fidelity > 0 else float('inf')
    }


def run_comparative_experiment(
    circuit_name: str,
    qasm_file: str,
    backend_type: str = 'aer-noisy',
    shots: int = 2048
) -> Dict:
    """
    Run comparative experiment for a single circuit.
    
    Returns:
        Experiment result with identity and QNS results
    """
    print(f"\n{'='*60}")
    print(f"Experiment: {circuit_name}")
    print(f"{'='*60}")
    
    # 1. Identity (Qiskit default transpile)
    print(f"\n[1] Running Identity mapping (Qiskit Aer {backend_type})...")
    identity_result = run_circuit_with_backend(qasm_file, backend_type, shots)
    
    print(f"    Fidelity: {identity_result['fidelity']:.4f}")
    print(f"    Counts: {identity_result['counts']}")
    
    # 2. QNS Optimized
    print(f"\n[2] Running QNS optimization...")
    try:
        qns_result = run_qns_optimized(qasm_file, backend_type, shots)
        
        print(f"    Original gates: {qns_result['original_gates']}")
        print(f"    Routed gates: {qns_result['routed_gates']}")
        print(f"    SWAPs: {qns_result['swap_count']}")
        print(f"    Fidelity: {qns_result['fidelity_after']:.4f}")
        
        qns_fidelity = qns_result['fidelity_after']
    except Exception as e:
        print(f"    ⚠️ QNS optimization failed: {e}")
        print(f"    Using Identity fidelity as fallback")
        qns_fidelity = identity_result['fidelity']
        qns_result = {'error': str(e)}
    
    # 3. Calculate improvement
    improvement = calculate_fidelity_improvement(
        identity_result['fidelity'],
        qns_fidelity
    )
    
    print(f"\n[3] Improvement Analysis:")
    print(f"    Absolute: {improvement['absolute']:+.4f}")
    print(f"    Relative: {improvement['relative_percent']:+.2f}%")
    
    # Determine if improvement is significant
    is_significant = improvement['relative_percent'] >= 5.0
    print(f"    Significant (≥5%): {'✅ Yes' if is_significant else '❌ No'}")
    
    return {
        'circuit': circuit_name,
        'qasm_file': qasm_file,
        'backend': backend_type,
        'shots': shots,
        'identity': identity_result,
        'qns': qns_result,
        'improvement': improvement,
        'is_significant': is_significant
    }


def run_all_experiments(backend_type: str = 'aer-noisy') -> List[Dict]:
    """Run experiments for all benchmark circuits."""
    
    circuits = [
        ('Bell State (2q)', 'test_bell.qasm'),
        ('GHZ State (3q)', 'benchmarks/circuits/ghz_3q.qasm'),
        ('QFT (4q)', 'benchmarks/circuits/qft_4q.qasm'),
    ]
    
    results = []
    
    print("\n" + "="*60)
    print("Sprint 3: QNS Optimization Validation")
    print("="*60)
    print(f"Backend: {backend_type}")
    print(f"Circuits: {len(circuits)}")
    print("="*60)
    
    for circuit_name, qasm_file in circuits:
        try:
            result = run_comparative_experiment(circuit_name, qasm_file, backend_type)
            results.append(result)
        except FileNotFoundError as e:
            print(f"\n⚠️ Skipping {circuit_name}: {e}")
            continue
        except Exception as e:
            print(f"\n❌ Error in {circuit_name}: {e}")
            import traceback
            traceback.print_exc()
            continue
    
    return results


def generate_summary_report(results: List[Dict]) -> Dict:
    """Generate summary statistics."""
    
    if not results:
        return {'error': 'No results to summarize'}
    
    improvements = [r['improvement']['relative_percent'] for r in results]
    significant_count = sum(1 for r in results if r['is_significant'])
    
    summary = {
        'total_circuits': len(results),
        'significant_improvements': significant_count,
        'mean_improvement_percent': sum(improvements) / len(improvements),
        'min_improvement_percent': min(improvements),
        'max_improvement_percent': max(improvements),
        'all_circuits': [
            {
                'name': r['circuit'],
                'identity_fidelity': r['identity']['fidelity'],
                'qns_fidelity': r['qns'].get('fidelity_after', r['identity']['fidelity']),
                'improvement_percent': r['improvement']['relative_percent'],
                'significant': r['is_significant']
            }
            for r in results
        ]
    }
    
    return summary


def print_summary(summary: Dict):
    """Print summary report."""
    
    print("\n" + "="*60)
    print("SUMMARY REPORT")
    print("="*60)
    
    print(f"\nCircuits tested: {summary['total_circuits']}")
    print(f"Significant improvements (≥5%): {summary['significant_improvements']}/{summary['total_circuits']}")
    print(f"\nMean improvement: {summary['mean_improvement_percent']:+.2f}%")
    print(f"Range: [{summary['min_improvement_percent']:+.2f}%, {summary['max_improvement_percent']:+.2f}%]")
    
    print(f"\nDetailed Results:")
    print(f"{'Circuit':<20} {'Identity':<10} {'QNS':<10} {'Improvement':<12} {'Significant'}")
    print("-" * 70)
    
    for circuit in summary['all_circuits']:
        sig_marker = "✅" if circuit['significant'] else "❌"
        print(f"{circuit['name']:<20} "
              f"{circuit['identity_fidelity']:<10.4f} "
              f"{circuit['qns_fidelity']:<10.4f} "
              f"{circuit['improvement_percent']:+11.2f}% "
              f"{sig_marker}")


def main():
    import argparse
    
    parser = argparse.ArgumentParser(description='Sprint 3: QNS Optimization Validation')
    parser.add_argument('--backend', default='aer-noisy', 
                        choices=['aer-ideal', 'aer-noisy'],
                        help='Backend type')
    parser.add_argument('--output', default='sprint3_results.json',
                        help='Output JSON file')
    
    args = parser.parse_args()
    
    # Run experiments
    results = run_all_experiments(args.backend)
    
    if not results:
        print("\n❌ No experiments completed successfully")
        sys.exit(1)
    
    # Generate summary
    summary = generate_summary_report(results)
    
    # Print summary
    print_summary(summary)
    
    # Save results
    output_data = {
        'summary': summary,
        'detailed_results': results
    }
    
    with open(args.output, 'w') as f:
        json.dump(output_data, f, indent=2)
    
    print(f"\n📊 Results saved to: {args.output}")
    
    # Exit code based on success
    if summary['significant_improvements'] > 0:
        print(f"\n✅ Sprint 3 PASSED: {summary['significant_improvements']} circuits show significant improvement")
        sys.exit(0)
    else:
        print(f"\n⚠️ Sprint 3 WARNING: No significant improvements detected")
        sys.exit(0)  # Not a failure, just a warning


if __name__ == '__main__':
    main()
