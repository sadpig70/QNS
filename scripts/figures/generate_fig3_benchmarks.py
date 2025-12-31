import matplotlib.pyplot as plt
import numpy as np

def generate_fig3():
    # Data from QNS_Technical_Specification_v2.5.md
    circuits = ['QFT-10', 'QFT-15', 'Grover-10']
    base_gates = [252, 591, 1227]
    qns_gates = [240, 547, 1091]
    tket_gates = [181, 421, 898]
    
    # Fidelity Data (Noisy)
    fid_circuits = ['VQE', 'GHZ-5']
    base_fid = [0.360, 0.970]
    qns_fid = [0.458, 0.970]
    
    # Plotting
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 6))
    
    # Subplot 1: Gate Count
    x = np.arange(len(circuits))
    width = 0.25 # Reduced width for 3 bars
    
    rects1 = ax1.bar(x - width, base_gates, width, label='Qiskit L3', color='gray')
    rects2 = ax1.bar(x, qns_gates, width, label='QNS', color='royalblue')
    rects3 = ax1.bar(x + width, tket_gates, width, label='Tket', color='darkorange')
    
    ax1.set_ylabel('Gate Count')
    ax1.set_title('(a) Gate Count Reduction')
    ax1.set_xticks(x)
    ax1.set_xticklabels(circuits)
    ax1.legend()
    
    # Add labels
    ax1.bar_label(rects1, padding=3)
    ax1.bar_label(rects2, padding=3)
    ax1.bar_label(rects3, padding=3)
    
    # Subplot 2: Fidelity
    x2 = np.arange(len(fid_circuits))
    width_fid = 0.35
    
    rects4 = ax2.bar(x2 - width_fid/2, base_fid, width_fid, label='Qiskit L3', color='gray')
    rects5 = ax2.bar(x2 + width_fid/2, qns_fid, width_fid, label='QNS', color='royalblue')
    
    ax2.set_ylabel('Fidelity')
    ax2.set_title('(b) Fidelity Improvement (Noisy Sim)')
    ax2.set_xticks(x2)
    ax2.set_xticklabels(fid_circuits)
    ax2.set_ylim(0, 1.1)
    ax2.legend(loc='lower left')
    
    ax2.bar_label(rects4, padding=3, fmt='%.3f')
    ax2.bar_label(rects5, padding=3, fmt='%.3f')
    
    plt.tight_layout()
    output_path = "docs/paper/figures/fig3_benchmarks.png"
    plt.savefig(output_path, dpi=300)
    print(f"Generated {output_path}")

if __name__ == "__main__":
    generate_fig3()
