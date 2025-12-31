import matplotlib.pyplot as plt
import networkx as nx

def generate_fig1():
    # Define the directed graph
    G = nx.DiGraph()
    
    # Define nodes with labels
    nodes = {
        'Calib': 'Calibration\n(IBM Backend)',
        'Profile': 'DriftScanner\n(Profiler)',
        'Compiler': 'Variant\nGeneration',
        'Cost': 'Cost Eval\n(Fidelity Model)',
        'Opt': 'Optimal\nCircuit',
        'Exec': 'Execution\n(Torino)',
        'ZNE': 'ZNE\nMitigation'
    }
    
    G.add_nodes_from(nodes.keys())
    
    # Define edges (Flow)
    edges = [
        ('Calib', 'Profile'),
        ('Profile', 'Cost'),
        ('Compiler', 'Cost'),
        ('Cost', 'Opt'),
        ('Opt', 'Exec'),
        ('Exec', 'ZNE'),
        ('ZNE', 'Compiler') # Feedback loop conceptual
    ]
    
    G.add_edges_from(edges)
    
    # Manual Layout for Schematic
    pos = {
        'Calib':    (0, 1),
        'Profile':  (1, 1),
        'Compiler': (0, 0),
        'Cost':     (1, 0),
        'Opt':      (2, 0),
        'Exec':     (3, 1),
        'ZNE':      (3, 0)
    }

    plt.figure(figsize=(10, 5))
    
    # Draw Nodes
    nx.draw_networkx_nodes(G, pos, node_size=3000, node_color='lightblue', edgecolors='black')
    
    # Draw Edges
    nx.draw_networkx_edges(G, pos, edge_color='black', width=2, arrowsize=20, arrowstyle='->')
    
    # Draw Labels
    labels = {k: v for k, v in nodes.items()}
    nx.draw_networkx_labels(G, pos, labels, font_size=9, font_weight='bold')
    
    plt.title("Figure 1: QNS Optimization Pipeline", fontsize=14)
    plt.axis('off')
    plt.tight_layout()
    
    # Save
    output_path = "docs/paper/figures/fig1_pipeline.png"
    plt.savefig(output_path, dpi=300)
    print(f"Generated {output_path}")

if __name__ == "__main__":
    generate_fig1()
