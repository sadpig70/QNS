"""
Generate Scalability Figure from Collected Benchmark Data
"""
import matplotlib.pyplot as plt
import numpy as np

# Data from benchmark (partial results, 15Q QFT excluded due to timeout)
data = {
    "GHZ": {
        "qubits": [5, 8, 10, 12, 15],
        "baseline": [0.949, 0.906, 0.874, 0.856, 0.814],
        "baseline_std": [0.002, 0.003, 0.004, 0.001, 0.002],
        "qns": [0.947, 0.906, 0.878, 0.855, 0.816],
        "qns_std": [0.002, 0.005, 0.005, 0.001, 0.008]
    },
    "QFT": {
        "qubits": [5, 8, 10, 12],  # 15Q excluded
        "baseline": [0.998, 0.984, 0.936, 0.640],
        "baseline_std": [0.000, 0.001, 0.002, 0.008],
        "qns": [0.998, 0.985, 0.931, 0.647],
        "qns_std": [0.001, 0.001, 0.003, 0.004]
    }
}

fig, axes = plt.subplots(1, 2, figsize=(14, 5))

for idx, (circuit_type, d) in enumerate(data.items()):
    ax = axes[idx]
    
    qubits = d["qubits"]
    x = np.arange(len(qubits))
    width = 0.35
    
    bars1 = ax.bar(x - width/2, d["baseline"], width, yerr=d["baseline_std"],
                   label='Qiskit L3', color='steelblue', capsize=3, alpha=0.8)
    bars2 = ax.bar(x + width/2, d["qns"], width, yerr=d["qns_std"],
                   label='QNS', color='darkorange', capsize=3, alpha=0.8)
    
    ax.set_xlabel('Number of Qubits', fontsize=12)
    ax.set_ylabel('Hellinger Fidelity', fontsize=12)
    ax.set_title(f'{circuit_type} Circuit Scalability\n(IBM Heron-class Noise Model)', fontsize=13)
    ax.set_xticks(x)
    ax.set_xticklabels(qubits)
    ax.legend(loc='upper right')
    ax.set_ylim(0, 1.1)
    ax.grid(True, alpha=0.3, axis='y')
    
    # Add value labels
    for bar, val in zip(bars1, d["baseline"]):
        ax.text(bar.get_x() + bar.get_width()/2, bar.get_height() + 0.02, 
                f'{val:.3f}', ha='center', va='bottom', fontsize=8)
    for bar, val in zip(bars2, d["qns"]):
        ax.text(bar.get_x() + bar.get_width()/2, bar.get_height() + 0.02, 
                f'{val:.3f}', ha='center', va='bottom', fontsize=8)

plt.tight_layout()
output_path = "docs/paper/figures/fig_scalability.png"
plt.savefig(output_path, dpi=300, bbox_inches='tight')
print(f"Scalability figure saved to: {output_path}")
plt.close()

# Print summary table
print("\n" + "=" * 70)
print("Summary Table: Scalability Validation Results")
print("=" * 70)
print(f"{'Qubits':<8} | {'Circuit':<6} | {'Baseline':<14} | {'QNS':<14} | {'Δ':<8}")
print("-" * 70)
for circuit_type, d in data.items():
    for i, q in enumerate(d["qubits"]):
        b = d["baseline"][i]
        b_std = d["baseline_std"][i]
        n = d["qns"][i]
        n_std = d["qns_std"][i]
        delta = ((n - b) / b) * 100 if b > 0 else 0
        print(f"{q:<8} | {circuit_type:<6} | {b:.3f}±{b_std:.3f}   | {n:.3f}±{n_std:.3f}   | {delta:+.1f}%")
