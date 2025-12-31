from qiskit.circuit.library import QFT, GroverOperator, QuantumVolume
from qiskit.quantum_info import Statevector
from pytket.extensions.qiskit import qiskit_to_tk
from pytket.passes import FullPeepholeOptimise
import numpy as np

def get_gate_count(circ):
    return sum(circ.count_ops().values())

def run_tket_benchmark():
    print("Running Tket Benchmark...")
    print("-" * 40)
    print(f"{'Circuit':<15} | {'Qubits':<8} | {'Gate Count (Tket)':<15}")
    print("-" * 40)

    benchmarks = [
        ("QFT", 10),
        ("QFT", 15),
        # Grover requires oracle, simulating roughly with QFT structure complexity or using library if simple
        # For fair comparison with paper numbers, we'll use QFT as proxy or try to match exactly if possible.
        # Given paper used specific QFT and Grover implementations, Tket should optimize them similarly.
    ]
    
    # 1. QFT-10
    qc_qft10 = QFT(10).decompose().decompose() # Ensure basic gates
    tk_qft10 = qiskit_to_tk(qc_qft10)
    FullPeepholeOptimise().apply(tk_qft10)
    print(f"{'QFT':<15} | {10:<8} | {tk_qft10.n_gates:<15}")

    # 2. QFT-15
    qc_qft15 = QFT(15).decompose().decompose()
    tk_qft15 = qiskit_to_tk(qc_qft15)
    FullPeepholeOptimise().apply(tk_qft15)
    print(f"{'QFT':<15} | {15:<8} | {tk_qft15.n_gates:<15}")

    # 3. Grover-10 (Approximation)
    # Construct a simple Grover iteration for 10 qubits to get a comparable metric
    from qiskit import QuantumCircuit
    qc_grover = QuantumCircuit(10)
    # H on all
    for i in range(10):
        qc_grover.h(i)
    # Diffuser (simplified for gate count estimate)
    for i in range(10):
        qc_grover.h(i)
        qc_grover.x(i)
    qc_grover.h(9)
    qc_grover.mcx(list(range(9)), 9) # Multi-controlled X
    qc_grover.h(9)
    for i in range(10):
        qc_grover.x(i)
        qc_grover.h(i)
    
    # Decompose to ensure Tket can handle it
    qc_grover = qc_grover.decompose().decompose() 
    
    tk_grover = qiskit_to_tk(qc_grover)
    FullPeepholeOptimise().apply(tk_grover)
    print(f"{'Grover':<15} | {10:<8} | {tk_grover.n_gates:<15}")
    
    print("-" * 40)

if __name__ == "__main__":
    run_tket_benchmark()
