"""
ZNE (Zero-Noise Extrapolation) Demonstration Script
====================================================
Generates quantitative ZNE results for QNS paper.
Shows: λ-scaled expectation values and extrapolation to zero-noise limit.
"""

import numpy as np
import matplotlib.pyplot as plt
from qiskit import QuantumCircuit, transpile
from qiskit_aer import AerSimulator
from qiskit_aer.noise import NoiseModel, depolarizing_error
from qiskit.quantum_info import SparsePauliOp
import warnings
warnings.filterwarnings('ignore')

# ============================================================
# 1. Noise Model
# ============================================================
def create_noise_model(p2q: float = 0.02) -> NoiseModel:
    """Create depolarizing noise model."""
    noise_model = NoiseModel()
    error_1q = depolarizing_error(p2q / 10, 1)
    error_2q = depolarizing_error(p2q, 2)
    noise_model.add_all_qubit_quantum_error(error_1q, ['rz', 'sx', 'x'])
    noise_model.add_all_qubit_quantum_error(error_2q, ['cx'])
    return noise_model

# ============================================================
# 2. Circuit and Observable
# ============================================================
def create_vqe_circuit(n_qubits: int = 4) -> QuantumCircuit:
    """Create a simple VQE-like ansatz."""
    qc = QuantumCircuit(n_qubits)
    for i in range(n_qubits):
        qc.ry(np.pi / 4, i)
    for i in range(n_qubits - 1):
        qc.cx(i, i + 1)
    for i in range(n_qubits):
        qc.rz(np.pi / 3, i)
    return qc

def measure_expectation(qc: QuantumCircuit, observable: str, 
                        backend, shots: int = 8192) -> float:
    """Measure expectation value of a Pauli observable."""
    n = qc.num_qubits
    # Simple Z expectation on first qubit
    qc_meas = qc.copy()
    qc_meas.measure_all()
    
    result = backend.run(transpile(qc_meas, backend), shots=shots).result()
    counts = result.get_counts()
    
    # Calculate <Z_0>
    exp_val = 0
    for bitstring, count in counts.items():
        # Qiskit uses little-endian, so first qubit is rightmost
        z_val = 1 if bitstring[-1] == '0' else -1
        exp_val += z_val * count
    return exp_val / shots

# ============================================================
# 3. Unitary Folding (Noise Amplification)
# ============================================================
def fold_circuit(qc: QuantumCircuit, scale_factor: int) -> QuantumCircuit:
    """
    Apply unitary folding: G -> G (G^dag G)^n for scale factor 1 + 2n.
    Scale factor must be odd: 1, 3, 5, 7, ...
    """
    if scale_factor == 1:
        return qc.copy()
    
    n_folds = (scale_factor - 1) // 2
    folded = qc.copy()
    
    for _ in range(n_folds):
        folded = folded.compose(qc.inverse())
        folded = folded.compose(qc)
    
    return folded

# ============================================================
# 4. Extrapolation
# ============================================================
def linear_extrapolation(lambdas: list, values: list) -> float:
    """Linear extrapolation to λ=0."""
    coeffs = np.polyfit(lambdas, values, 1)
    return coeffs[1]  # Intercept

def richardson_extrapolation(lambdas: list, values: list) -> float:
    """Richardson extrapolation (quadratic fit to λ=0)."""
    coeffs = np.polyfit(lambdas, values, min(2, len(lambdas) - 1))
    return coeffs[-1]  # Intercept

# ============================================================
# 5. Main ZNE Experiment
# ============================================================
def run_zne_experiment():
    print("=" * 60)
    print("ZNE (Zero-Noise Extrapolation) Demonstration")
    print("=" * 60)
    
    # Setup
    n_qubits = 4
    qc = create_vqe_circuit(n_qubits)
    
    # Ideal value
    ideal_backend = AerSimulator(method='statevector')
    ideal_value = measure_expectation(qc, "Z", ideal_backend)
    print(f"\nIdeal <Z_0>: {ideal_value:.4f}")
    
    # Noise model
    p2q_base = 0.03  # 3% 2Q error
    noise_model = create_noise_model(p2q_base)
    noisy_backend = AerSimulator(noise_model=noise_model)
    
    # Scale factors
    scale_factors = [1, 3, 5, 7]
    noisy_values = []
    
    print("\nNoisy Measurements:")
    print("-" * 40)
    
    for lam in scale_factors:
        folded_qc = fold_circuit(qc, lam)
        # For higher λ, noise accumulates
        exp_val = measure_expectation(folded_qc, "Z", noisy_backend)
        noisy_values.append(exp_val)
        print(f"  λ={lam}: <Z_0> = {exp_val:.4f}")
    
    # Extrapolation
    zne_linear = linear_extrapolation(scale_factors, noisy_values)
    zne_richardson = richardson_extrapolation(scale_factors, noisy_values)
    
    print("\nExtrapolation Results:")
    print("-" * 40)
    print(f"  Noisy (λ=1):   {noisy_values[0]:.4f}")
    print(f"  ZNE (Linear):  {zne_linear:.4f}")
    print(f"  ZNE (Richard): {zne_richardson:.4f}")
    print(f"  Ideal:         {ideal_value:.4f}")
    
    # Error reduction
    error_noisy = abs(ideal_value - noisy_values[0])
    error_linear = abs(ideal_value - zne_linear)
    error_richardson = abs(ideal_value - zne_richardson)
    
    print("\nError Reduction:")
    print("-" * 40)
    print(f"  Noisy Error:     {error_noisy:.4f}")
    print(f"  Linear Error:    {error_linear:.4f} ({(1 - error_linear/error_noisy)*100:.1f}% reduction)")
    print(f"  Richardson Error:{error_richardson:.4f} ({(1 - error_richardson/error_noisy)*100:.1f}% reduction)")
    
    # Generate plot
    plt.figure(figsize=(10, 6))
    
    plt.scatter(scale_factors, noisy_values, s=100, c='blue', label='Measured', zorder=5)
    
    # Fit lines
    x_fit = np.linspace(0, max(scale_factors) + 0.5, 100)
    linear_fit = np.poly1d(np.polyfit(scale_factors, noisy_values, 1))
    richardson_fit = np.poly1d(np.polyfit(scale_factors, noisy_values, 2))
    
    plt.plot(x_fit, linear_fit(x_fit), '--', color='orange', label='Linear Fit')
    plt.plot(x_fit, richardson_fit(x_fit), '-.', color='green', label='Richardson Fit')
    
    # Extrapolated points
    plt.scatter([0], [zne_linear], s=150, marker='*', c='orange', label=f'ZNE Linear: {zne_linear:.3f}', zorder=6)
    plt.scatter([0], [zne_richardson], s=150, marker='*', c='green', label=f'ZNE Richardson: {zne_richardson:.3f}', zorder=6)
    
    # Ideal line
    plt.axhline(y=ideal_value, color='red', linestyle=':', label=f'Ideal: {ideal_value:.3f}')
    
    plt.xlabel('Noise Scale Factor (λ)', fontsize=12)
    plt.ylabel('Expectation Value <Z₀>', fontsize=12)
    plt.title('Zero-Noise Extrapolation for VQE Circuit', fontsize=14)
    plt.legend(loc='best')
    plt.grid(True, alpha=0.3)
    plt.xlim(-0.5, max(scale_factors) + 0.5)
    
    output_path = "docs/paper/figures/fig_zne_results.png"
    plt.savefig(output_path, dpi=300, bbox_inches='tight')
    print(f"\nFigure saved to: {output_path}")
    plt.close()

if __name__ == "__main__":
    run_zne_experiment()
