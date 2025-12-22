#!/usr/bin/env python3
"""
QNS Scalability Benchmark Suite
Target: arXiv Paper Supplement (Scalability Analysis)

Scope:
- Circuits: QFT, Grover (5-20 qubits)
- Comparison: QNS (Rust CLI) vs Qiskit Transpiler (Optimization Level 3)
- Metrics: Fidelity (Estimated), Compilation Time, Gate Count

Features:
- Checkpointing (Append to CSV)
- Error Handling (Skip failed circuits)
"""

import sys
import csv
import json
import time
import uuid
import subprocess
from pathlib import Path
from dataclasses import dataclass, asdict
from typing import List, Optional, Tuple

# Attempt Qiskit imports
try:
    from qiskit import QuantumCircuit, transpile
    from qiskit.circuit.library import QFT, GroverOperator
    from qiskit.quantum_info import Operator
    from qiskit_aer import AerSimulator
    from qiskit_aer.noise import NoiseModel, depolarizing_error
    QISKIT_AVAILABLE = True
except ImportError:
    QISKIT_AVAILABLE = False
    print("⚠️  Qiskit not available.")

# Configuration
RANDOM_SEED = 42
TIMEOUT_SECONDS = 300  # 5 minutes per circuit
RESULTS_DIR = Path(__file__).parent / "results"
CSV_PATH = RESULTS_DIR / "scalability_results.csv"

@dataclass
class ScalabilityResult:
    timestamp: str
    circuit_type: str
    qubits: int
    original_depth: int
    original_gates: int
    
    # Baseline (Qiskit L3)
    baseline_fidelity: float
    baseline_depth: int
    baseline_gates: int
    baseline_time_ms: float
    
    # QNS
    qns_fidelity: float
    qns_depth: int
    qns_gates: int
    qns_time_ms: float
    
    # Improvement
    fidelity_improvement: float  # %
    gate_reduction: float        # %

class BenchmarkRunner:
    def __init__(self, noisy: bool = True):
        self.noisy = noisy
        self.noise_model = self._create_noise_model() if noisy else None
        
        # Ensure results dir exists
        RESULTS_DIR.mkdir(exist_ok=True, parents=True)
        
        # Initialize CSV if not exists
        if not CSV_PATH.exists():
            self._init_csv()

    def _create_noise_model(self) -> Optional['NoiseModel']:
        if not QISKIT_AVAILABLE: return None
        nm = NoiseModel()
        # Typical superconducting errors
        error_1q = depolarizing_error(0.001, 1)
        error_2q = depolarizing_error(0.01, 2)
        nm.add_all_qubit_quantum_error(error_1q, ['u1', 'u2', 'u3', 'x', 'y', 'z', 'h', 's', 't', 'rx', 'ry', 'rz'])
        nm.add_all_qubit_quantum_error(error_2q, ['cx', 'cz', 'swap'])
        return nm

    def _init_csv(self):
        with open(CSV_PATH, 'w', newline='', encoding='utf-8') as f:
            writer = csv.writer(f)
            headers = [
                "Timestamp", "Circuit", "Qubits", "Orig_Depth", "Orig_Gates",
                "Base_Fid", "Base_Depth", "Base_Gates", "Base_Time_ms",
                "QNS_Fid", "QNS_Depth", "QNS_Gates", "QNS_Time_ms",
                "Fid_Imp_Pct", "Gate_Red_Pct"
            ]
            writer.writerow(headers)

    def _save_result(self, r: ScalabilityResult):
        with open(CSV_PATH, 'a', newline='', encoding='utf-8') as f:
            writer = csv.writer(f)
            writer.writerow([
                r.timestamp, r.circuit_type, r.qubits, r.original_depth, r.original_gates,
                f"{r.baseline_fidelity:.4f}", r.baseline_depth, r.baseline_gates, f"{r.baseline_time_ms:.2f}",
                f"{r.qns_fidelity:.4f}", r.qns_depth, r.qns_gates, f"{r.qns_time_ms:.2f}",
                f"{r.fidelity_improvement:.2f}", f"{r.gate_reduction:.2f}"
            ])
        print(f"  📝 Saved checkpoint for {r.circuit_type}-{r.qubits}")

    # --- Circuit Generators ---
    
    def generate_qft(self, n: int) -> QuantumCircuit:
        return QFT(n, do_swaps=True).decompose()

    def generate_grover(self, n: int) -> QuantumCircuit:
        if n < 2: return QuantumCircuit(n)
        # Simple Grover: Oracle |11...1>
        oracle = QuantumCircuit(n)
        oracle.cp(3.14159, 0, n-1) # Dummy phase oracle for benchmark
        
        grover_op = GroverOperator(oracle)
        qc = QuantumCircuit(n)
        for i in range(n): qc.h(i)
        qc = qc.compose(grover_op)
        return qc.decompose().decompose()

    # --- Execution Logic ---

    def run_baseline_qiskit(self, circuit: QuantumCircuit) -> Tuple[float, int, int, float]:
        """Run Qiskit Transpiler Level 3"""
        start_t = time.perf_counter()
        
        # Transpile with high optimization
        # Utilizing a linear topology to force routing effort (simulating NISQ constraints)
        coupling_map = [[i, i+1] for i in range(circuit.num_qubits - 1)]
        coupling_map += [[i+1, i] for i in range(circuit.num_qubits - 1)]
        
        backend = AerSimulator(noise_model=self.noise_model)
        
        transpiled = transpile(
            circuit, 
            backend, 
            optimization_level=3, 
            coupling_map=coupling_map,
            seed_transpiler=RANDOM_SEED
        )
        
        elapsed_ms = (time.perf_counter() - start_t) * 1000
        
        # Estimate fidelity (heuristic: generic product of errors)
        # Note: We use a simple estimation here because simulating 20q noisy circuits is too slow for typical CI/benchmark
        # Fidelity ~ (1-e)^N_gates
        count_ops = transpiled.count_ops()
        n_2q = count_ops.get('cx', 0) + count_ops.get('cz', 0) + count_ops.get('swap', 0)
        n_1q = sum(count_ops.values()) - n_2q - count_ops.get('measure', 0) - count_ops.get('barrier', 0)
        
        # Simple estimation model for comparison
        est_fidelity = (0.999 ** n_1q) * (0.99 ** n_2q)
        
        return est_fidelity, transpiled.depth(), sum(count_ops.values()), elapsed_ms

    def run_qns_cli(self, circuit: QuantumCircuit, circuit_name: str) -> Tuple[float, int, int, float]:
        """Run QNS CLI"""
        
        # 1. Export Circuit to QASM
        # Save in benchmarks/ root to access qelib1.inc
        temp_qasm = RESULTS_DIR.parent / f"temp_{uuid.uuid4().hex}.qasm"
        
        # Transpile to QNS-compatible basis set
        # QNS supports ONLY these explicit gates (no u1/u2/u3)
        basis_gates = ['h', 'x', 'y', 'z', 's', 't', 'rx', 'ry', 'rz', 'cx', 'cz', 'swap', 'id']
        try:
            transpiled_for_qns = transpile(circuit, basis_gates=basis_gates, optimization_level=1)
        except Exception as e:
            print(f"  ⚠️ Transpilation failed: {e}")
            transpiled_for_qns = circuit
            
        try:
            from qiskit import qasm2
            qasm_str = qasm2.dumps(transpiled_for_qns)
        except:
            qasm_str = transpiled_for_qns.qasm()
            
        # SANITIZE QASM: Replace symbolic expressions (pi/2) with evaluated floats
        # QNS parser only supports float literals.
        import re
        import math
        
        def eval_math(match):
            args_str = match.group(1)
            # Split args by comma
            args = [a.strip() for a in args_str.split(',')]
            eval_args = []
            for arg in args:
                try:
                    # distinct pi handling
                    val_str = arg.replace('pi', str(math.pi))
                    val = eval(val_str, {"__builtins__": None}, {})
                    eval_args.append(f"{val:.10f}")
                except:
                    eval_args.append(arg)
            return "(" + ",".join(eval_args) + ")"
            
        # Regex to find (arg1, arg2...) patterns attached to gates
        # Matches literal parenthesis with content inside
        qasm_str = re.sub(r'\(([^)]+)\)', eval_math, qasm_str)
            
        with open(temp_qasm, 'w') as f:
            f.write(qasm_str)
            
        # 2. Find CLI
        cli_path = Path(__file__).parent.parent / 'target' / 'release' / 'qns.exe'
        if not cli_path.exists():
            # Fallback path logic
            cli_path = Path("qns") # Hope it's in PATH
            
        # 3. Execute
        cmd = [
            str(cli_path), 'run', str(temp_qasm),
            '--topology', 'linear',
            '--format', 'json',
            '--shots', '0' # Skip simulation, just compile & estimate
        ]
        
        start_t = time.perf_counter()
        try:
            res = subprocess.run(cmd, capture_output=True, text=True, timeout=TIMEOUT_SECONDS)
        except subprocess.TimeoutExpired:
            print("  ❌ QNS Timeout")
            return 0.0, 0, 0, 0.0
        finally:
            if temp_qasm.exists(): temp_qasm.unlink()
            
        if res.returncode != 0:
            print(f"  ❌ QNS Fatal: {res.stderr[:200]}")
            return 0.0, 0, 0, 0.0
            
        # 4. Parse JSON
        # Filter non-JSON log lines (search for first { and last })
        try:
            start_idx = res.stdout.find('{')
            end_idx = res.stdout.rfind('}') + 1
            if start_idx == -1 or end_idx == 0:
                print("  ❌ No JSON block found in output")
                print(f"     Full Output: {res.stdout[:200]}...")
                return 0.0, 0, 0, 0.0
                
            json_str = res.stdout[start_idx:end_idx]
            data = json.loads(json_str)
            
            # QNS result structure keys: fidelity_after, final_depth, total_gates, total_time_ms
            # If keys differ, adjust here based on qns_python/test output or lib.rs logic
            # Observed output shows "routed_gates"
            fid = data.get('fidelity_after', 0.0)
            depth = data.get('circuit_depth', 0) # Also fix depth key if needed (was final_depth, example showed circuit_depth)
            gates = data.get('routed_gates', 0)
            t_ms = data.get('total_time_ms', 0.0)
            
            return fid, depth, gates, t_ms
            
        except json.JSONDecodeError as e:
            print(f"  ❌ Invalid JSON from QNS: {e}")
            print(f"     String being parsed: {json_str[:100]}...")
            return 0.0, 0, 0, 0.0

    def run_suite(self):
        print("🚀 QNS Scalability Benchmark Suite Started")
        print(f"   Mode: {'Noisy (estimated)' if self.noisy else 'Ideal'}")
        
        configs = []
        # QFT: 5 to 15
        for n in [5, 10, 15]:
            configs.append(('QFT', n, self.generate_qft(n)))
        
        # Grover: 5 to 10
        for n in [5, 10]:
            configs.append(('Grover', n, self.generate_grover(n)))
            
        for name, qubits, qc in configs:
            print(f"\n🔹 Benchmarking {name}-{qubits}q...")
            
            try:
                # 1. Baseline
                base_fid, base_depth, base_gates, base_time = self.run_baseline_qiskit(qc)
                print(f"   Base L3 | Fid: {base_fid:.4f} | Gates: {base_gates} | Time: {base_time:.1f}ms")
                
                # 2. QNS
                qns_fid, qns_depth, qns_gates, qns_time = self.run_qns_cli(qc, name)
                if qns_fid == 0.0:
                    print("   Skipping due to QNS failure")
                    continue
                    
                print(f"   QNS     | Fid: {qns_fid:.4f} | Gates: {qns_gates} | Time: {qns_time:.1f}ms")
                
                # 3. Comparison
                imp_fid = (qns_fid - base_fid) / base_fid * 100 if base_fid > 0 else 0
                imp_gate = (base_gates - qns_gates) / base_gates * 100 if base_gates > 0 else 0
                
                res = ScalabilityResult(
                    timestamp=time.strftime("%Y-%m-%d %H:%M:%S"),
                    circuit_type=name,
                    qubits=qubits,
                    original_depth=qc.depth(),
                    original_gates=sum(qc.count_ops().values()),
                    baseline_fidelity=base_fid,
                    baseline_depth=base_depth,
                    baseline_gates=base_gates,
                    baseline_time_ms=base_time,
                    qns_fidelity=qns_fid,
                    qns_depth=qns_depth,
                    qns_gates=qns_gates,
                    qns_time_ms=qns_time,
                    fidelity_improvement=imp_fid,
                    gate_reduction=imp_gate
                )
                
                self._save_result(res)
                
            except Exception as e:
                print(f"   ❌ Unexpected Error: {e}")
                import traceback
                traceback.print_exc()

if __name__ == "__main__":
    if not QISKIT_AVAILABLE:
        sys.exit(1)
        
    runner = BenchmarkRunner(noisy=True)
    runner.run_suite()
