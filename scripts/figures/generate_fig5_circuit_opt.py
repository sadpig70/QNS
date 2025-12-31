from qiskit import QuantumCircuit
import matplotlib.pyplot as plt

def generate_fig5():
    # 1. Original Circuit (Inefficient routing)
    qc_orig = QuantumCircuit(3)
    qc_orig.h(0)
    qc_orig.cx(0, 2) # Hardware may not have 0-2 connectivity
    qc_orig.cx(1, 2)
    qc_orig.measure_all()
    
    # 2. Optimized Circuit (QNS mapped to linear 0-1-2)
    qc_opt = QuantumCircuit(3)
    qc_opt.h(0)
    qc_opt.cx(0, 1) # Mapped 2 -> 1
    qc_opt.swap(1, 2) # SWAP inserted or logical mapping
    qc_opt.cx(1, 2)
    qc_opt.measure_all()
    
    # Plotting
    # Note: Qiskit matplotlib drawer can return a matplotlib Figure
    fig = plt.figure(figsize=(12, 6))
    
    # We can't easily put two qiskit drawers in one subplot without complex hacking.
    # So we will generate two separate images and combine them, or just save one "Before/After" if we could.
    # Easier strategy: Generate two separate files and the user can combine in LaTeX or we use subplots if qiskit supports passing 'ax'.
    # Qiskit `draw(output='mpl', ax=ax)` is supported.
    
    ax1 = fig.add_subplot(121)
    qc_orig.draw(output='mpl', ax=ax1, style='iqp')
    ax1.set_title("(a) Original Circuit (Logical)", fontsize=12)
    
    ax2 = fig.add_subplot(122)
    qc_opt.draw(output='mpl', ax=ax2, style='iqp')
    ax2.set_title("(b) QNS Optimized (Physical)", fontsize=12)
    
    plt.tight_layout()
    output_path = "docs/paper/figures/fig5_circuit_opt.png"
    plt.savefig(output_path, dpi=300)
    print(f"Generated {output_path}")

if __name__ == "__main__":
    generate_fig5()
