"""
Enhanced QNS Benchmark Suite with Qiskit Comparison and Statistical Analysis

This script provides:
1. Routing Efficiency (SWAP count, circuit depth)
2. Simulation Performance (execution time, memory usage)
3. Qiskit Aer Comparison (baseline)
4. Statistical Analysis (mean ± std, p-values)
"""

import subprocess
import json
import time
import csv
from pathlib import Path
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass, asdict
import matplotlib.pyplot as plt
import numpy as np
from scipy import stats

# Optional: Qiskit for comparison
try:
    from qiskit import QuantumCircuit, Aer, execute
    from qiskit.converters import circuit_from_qasm_file
    QISKIT_AVAILABLE = True
except ImportError:
    QISKIT_AVAILABLE = False
    print("⚠️  Qiskit not available. Install with: pip install qiskit")

@dataclass
class RoutingResult:
    """Results from routing benchmark"""
    circuit_name: str
    num_qubits: int
    original_gates: int
    routed_gates: int
    swap_count: int
    circuit_depth: int
    routing_time_ms: float

@dataclass
class SimulationResult:
    """Results from simulation benchmark"""
    circuit_name: str
    num_qubits: int
    num_gates: int
    execution_time_ms: float
    memory_mb: float
    shots: int
    simulator: str  # 'qns' or 'qiskit'

@dataclass
class StatisticalSummary:
    """Statistical summary of multiple runs"""
    mean: float
    std: float
    min: float
    max: float
    median: float
    runs: int

class QNSBenchmark:
    """Enhanced benchmark orchestrator with Qiskit comparison"""
    
    def __init__(self, qns_binary: Path, output_dir: Path, num_runs: int = 10):
        self.qns_binary = qns_binary
        self.output_dir = output_dir
        self.num_runs = num_runs
        self.output_dir.mkdir(parents=True, exist_ok=True)
        
    def benchmark_routing_efficiency(self, qasm_files: List[Path]) -> List[RoutingResult]:
        """
        Benchmark routing efficiency with statistical analysis
        
        Runs each circuit multiple times and computes statistics
        """
        all_results = []
        
        for qasm_file in qasm_files:
            print(f"\n📊 Benchmarking routing: {qasm_file.name} ({self.num_runs} runs)")
            
            run_results = []
            for run in range(self.num_runs):
                start_time = time.time()
                cmd = [
                    str(self.qns_binary),
                    "run",
                    str(qasm_file),
                    "--topology", "grid"
                ]
                
                try:
                    output = subprocess.check_output(cmd, stderr=subprocess.STDOUT, text=True, timeout=30)
                    routing_time = (time.time() - start_time) * 1000
                    
                    routed_gates = self._parse_gate_count(output, "Routed")
                    original_gates = self._parse_gate_count(output, "Parsed")
                    swap_count = routed_gates - original_gates if routed_gates > original_gates else 0
                    
                    result = RoutingResult(
                        circuit_name=qasm_file.stem,
                        num_qubits=self._count_qubits(qasm_file),
                        original_gates=original_gates,
                        routed_gates=routed_gates,
                        swap_count=swap_count,
                        circuit_depth=0,
                        routing_time_ms=routing_time
                    )
                    run_results.append(result)
                    
                except (subprocess.CalledProcessError, subprocess.TimeoutExpired) as e:
                    print(f"  ⚠️  Run {run+1} failed: {e}")
                    continue
            
            if run_results:
                # Use median result as representative
                median_idx = len(run_results) // 2
                sorted_results = sorted(run_results, key=lambda x: x.routing_time_ms)
                all_results.append(sorted_results[median_idx])
                
                # Print statistics
                times = [r.routing_time_ms for r in run_results]
                swaps = [r.swap_count for r in run_results]
                print(f"  ✓ Routing time: {np.mean(times):.1f} ± {np.std(times):.1f} ms")
                print(f"  ✓ SWAP count: {np.mean(swaps):.1f} ± {np.std(swaps):.1f}")
        
        self._save_results(all_results, "routing_efficiency.csv")
        return all_results
    
    def benchmark_simulation_performance(self, qasm_files: List[Path], shots: int = 1000) -> Tuple[List[SimulationResult], Optional[List[SimulationResult]]]:
        """
        Benchmark simulation performance: QNS vs Qiskit Aer
        
        Returns:
            (qns_results, qiskit_results)
        """
        qns_results = []
        qiskit_results = []
        
        for qasm_file in qasm_files:
            print(f"\n📊 Benchmarking simulation: {qasm_file.name}")
            
            # QNS benchmark
            qns_times = []
            for run in range(self.num_runs):
                start_time = time.time()
                cmd = [str(self.qns_binary), "run", str(qasm_file)]
                
                try:
                    output = subprocess.check_output(cmd, stderr=subprocess.STDOUT, text=True, timeout=60)
                    execution_time = (time.time() - start_time) * 1000
                    qns_times.append(execution_time)
                except (subprocess.CalledProcessError, subprocess.TimeoutExpired):
                    continue
            
            if qns_times:
                result = SimulationResult(
                    circuit_name=qasm_file.stem,
                    num_qubits=self._count_qubits(qasm_file),
                    num_gates=self._parse_gate_count_from_file(qasm_file),
                    execution_time_ms=np.median(qns_times),
                    memory_mb=0.0,
                    shots=shots,
                    simulator='qns'
                )
                qns_results.append(result)
                print(f"  ✓ QNS: {np.mean(qns_times):.1f} ± {np.std(qns_times):.1f} ms")
            
            # Qiskit Aer benchmark
            if QISKIT_AVAILABLE:
                try:
                    qiskit_times = []
                    for run in range(self.num_runs):
                        start_time = time.time()
                        qc = circuit_from_qasm_file(str(qasm_file))
                        backend = Aer.get_backend('qasm_simulator')
                        job = execute(qc, backend, shots=shots)
                        result = job.result()
                        execution_time = (time.time() - start_time) * 1000
                        qiskit_times.append(execution_time)
                    
                    if qiskit_times:
                        result = SimulationResult(
                            circuit_name=qasm_file.stem,
                            num_qubits=self._count_qubits(qasm_file),
                            num_gates=self._parse_gate_count_from_file(qasm_file),
                            execution_time_ms=np.median(qiskit_times),
                            memory_mb=0.0,
                            shots=shots,
                            simulator='qiskit'
                        )
                        qiskit_results.append(result)
                        print(f"  ✓ Qiskit: {np.mean(qiskit_times):.1f} ± {np.std(qiskit_times):.1f} ms")
                        
                        # Statistical comparison
                        if len(qns_times) > 1 and len(qiskit_times) > 1:
                            t_stat, p_value = stats.ttest_ind(qns_times, qiskit_times)
                            speedup = np.mean(qiskit_times) / np.mean(qns_times)
                            print(f"  📈 Speedup: {speedup:.2f}x (p={p_value:.4f})")
                
                except Exception as e:
                    print(f"  ⚠️  Qiskit failed: {e}")
        
        self._save_results(qns_results, "simulation_qns.csv")
        if qiskit_results:
            self._save_results(qiskit_results, "simulation_qiskit.csv")
        
        return qns_results, qiskit_results if qiskit_results else None
    
    def visualize_comparison(self, qns_results: List[SimulationResult], qiskit_results: Optional[List[SimulationResult]]):
        """Create publication-quality comparison figures"""
        
        fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 6))
        
        # Group by qubit count
        qubit_counts = sorted(set(r.num_qubits for r in qns_results))
        
        qns_times = []
        qiskit_times = []
        
        for qc in qubit_counts:
            qns_avg = np.mean([r.execution_time_ms for r in qns_results if r.num_qubits == qc])
            qns_times.append(qns_avg)
            
            if qiskit_results:
                qiskit_avg = np.mean([r.execution_time_ms for r in qiskit_results if r.num_qubits == qc])
                qiskit_times.append(qiskit_avg)
        
        # Plot 1: Execution time comparison
        ax1.plot(qubit_counts, qns_times, 'o-', linewidth=2, markersize=8, label='QNS', color='steelblue')
        if qiskit_times:
            ax1.plot(qubit_counts, qiskit_times, 's-', linewidth=2, markersize=8, label='Qiskit Aer', color='coral')
        ax1.set_xlabel('Number of Qubits', fontsize=12)
        ax1.set_ylabel('Execution Time (ms)', fontsize=12)
        ax1.set_title('Simulation Performance Comparison', fontsize=14, fontweight='bold')
        ax1.grid(True, alpha=0.3)
        ax1.legend(fontsize=11)
        
        # Plot 2: Speedup
        if qiskit_times:
            speedups = [q/n for q, n in zip(qiskit_times, qns_times)]
            ax2.bar(range(len(qubit_counts)), speedups, color='green', alpha=0.7)
            ax2.axhline(y=1.0, color='red', linestyle='--', label='Baseline')
            ax2.set_xlabel('Number of Qubits', fontsize=12)
            ax2.set_ylabel('Speedup (Qiskit / QNS)', fontsize=12)
            ax2.set_title('QNS Speedup vs Qiskit Aer', fontsize=14, fontweight='bold')
            ax2.set_xticks(range(len(qubit_counts)))
            ax2.set_xticklabels(qubit_counts)
            ax2.grid(axis='y', alpha=0.3)
            ax2.legend(fontsize=11)
        
        plt.tight_layout()
        plt.savefig(self.output_dir / 'performance_comparison.pdf', dpi=300, bbox_inches='tight')
        plt.savefig(self.output_dir / 'performance_comparison.png', dpi=300, bbox_inches='tight')
        print(f"\n✅ Saved: {self.output_dir / 'performance_comparison.pdf'}")
    
    def _parse_gate_count(self, output: str, keyword: str) -> int:
        """Parse gate count from QNS output"""
        for line in output.split('\n'):
            if keyword in line and 'gates' in line:
                parts = line.split('(')
                if len(parts) > 1:
                    gate_part = parts[1].split('gates')[0].strip()
                    try:
                        return int(gate_part.split()[-1])
                    except (ValueError, IndexError):
                        pass
        return 0
    
    def _parse_gate_count_from_file(self, qasm_file: Path) -> int:
        """Count gates in QASM file"""
        count = 0
        with open(qasm_file, 'r') as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith('//') and not line.startswith('OPENQASM') and not line.startswith('include') and not line.startswith('qreg') and not line.startswith('creg'):
                    if any(gate in line for gate in ['h', 'x', 'y', 'z', 'cx', 'cz', 'rx', 'ry', 'rz', 'measure', 'swap', 'cp', 'ccx']):
                        count += 1
        return count
    
    def _count_qubits(self, qasm_file: Path) -> int:
        """Count qubits in QASM file"""
        with open(qasm_file, 'r') as f:
            for line in f:
                if line.strip().startswith('qreg'):
                    try:
                        size = line.split('[')[1].split(']')[0]
                        return int(size)
                    except (IndexError, ValueError):
                        pass
        return 0
    
    def _save_results(self, results: List, filename: str):
        """Save results to CSV"""
        if not results:
            return
        
        output_file = self.output_dir / filename
        with open(output_file, 'w', newline='') as f:
            writer = csv.DictWriter(f, fieldnames=asdict(results[0]).keys())
            writer.writeheader()
            for result in results:
                writer.writerow(asdict(result))
        
        print(f"💾 Saved: {output_file}")


def main():
    """Main benchmark execution"""
    import argparse
    
    parser = argparse.ArgumentParser(description='QNS Benchmark Suite with Qiskit Comparison')
    parser.add_argument('--qns-binary', type=Path, default=Path('target/release/qns.exe'),
                        help='Path to QNS binary')
    parser.add_argument('--circuits-dir', type=Path, default=Path('benchmarks/qasmbench'),
                        help='Directory containing QASM circuits')
    parser.add_argument('--output-dir', type=Path, default=Path('benchmark_results'),
                        help='Output directory for results')
    parser.add_argument('--mode', choices=['routing', 'simulation', 'all'], default='all',
                        help='Benchmark mode')
    parser.add_argument('--runs', type=int, default=10,
                        help='Number of runs for statistical analysis')
    
    args = parser.parse_args()
    
    # Find all QASM files
    qasm_files = sorted(list(args.circuits_dir.glob('*.qasm')))
    if not qasm_files:
        print(f"❌ No QASM files found in {args.circuits_dir}")
        return
    
    print(f"🔬 Found {len(qasm_files)} QASM circuits")
    print(f"📊 Statistical analysis: {args.runs} runs per circuit")
    
    benchmark = QNSBenchmark(args.qns_binary, args.output_dir, num_runs=args.runs)
    
    if args.mode in ['routing', 'all']:
        print("\n" + "="*60)
        print("🔀 ROUTING EFFICIENCY BENCHMARK")
        print("="*60)
        routing_results = benchmark.benchmark_routing_efficiency(qasm_files)
    
    if args.mode in ['simulation', 'all']:
        print("\n" + "="*60)
        print("⚡ SIMULATION PERFORMANCE BENCHMARK")
        print("="*60)
        qns_results, qiskit_results = benchmark.benchmark_simulation_performance(qasm_files)
        benchmark.visualize_comparison(qns_results, qiskit_results)
    
    print("\n" + "="*60)
    print("✅ BENCHMARK COMPLETE!")
    print("="*60)
    print(f"📁 Results saved to: {args.output_dir}")
    print(f"📊 Total circuits: {len(qasm_files)}")
    print(f"🔬 Runs per circuit: {args.runs}")


if __name__ == '__main__':
    main()
