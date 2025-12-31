import numpy as np
import matplotlib.pyplot as plt
from matplotlib import cm

def generate_fig2():
    # Data Generation
    # X: T2 time (microseconds)
    t2_vals = np.linspace(20, 100, 50) 
    # Y: Circuit Depth (number of gates, assuming 100ns per gate)
    depth_vals = np.linspace(10, 200, 50)
    
    X, Y = np.meshgrid(t2_vals, depth_vals)
    
    # Constants
    gate_time_us = 0.1 # 100ns
    gate_error = 0.001 # 0.1% error per gate
    
    # Fidelity Model: F = (1 - epsilon)^depth * exp(-t_total / T2)
    # t_total = depth * gate_time
    Z = np.power(1 - gate_error, Y) * np.exp(-(Y * gate_time_us) / X)
    
    # Plotting
    fig = plt.figure(figsize=(10, 7))
    ax = fig.add_subplot(111, projection='3d')
    
    surf = ax.plot_surface(X, Y, Z, cmap=cm.viridis, linewidth=0, antialiased=False, alpha=0.9)
    
    ax.set_xlabel('T2 Coherence Time (μs)')
    ax.set_ylabel('Circuit Depth (Gates)')
    ax.set_zlabel('Estimated Fidelity')
    ax.set_title('Figure 2: Fidelity Estimation Model Landscape', fontsize=12)
    
    # Add a color bar which maps values to colors.
    fig.colorbar(surf, shrink=0.5, aspect=5, label='Fidelity')
    
    # View angle
    ax.view_init(elev=30, azim=135)
    
    plt.tight_layout()
    
    # Save
    output_path = "docs/paper/figures/fig2_fidelity_model.png"
    plt.savefig(output_path, dpi=300)
    print(f"Generated {output_path}")

if __name__ == "__main__":
    generate_fig2()
