#!/usr/bin/env python3
"""
QNS Paper Figure Generator
==========================

Generates publication-quality figures for the QNS journal paper.

Usage:
    python generate_paper_figures.py
    python generate_paper_figures.py --input results/ --output figures/
"""

import json
import csv
import argparse
from pathlib import Path
from typing import List, Dict, Tuple
import sys

try:
    import matplotlib.pyplot as plt
    import matplotlib.patches as mpatches
    import numpy as np
    HAS_MATPLOTLIB = True
except ImportError:
    HAS_MATPLOTLIB = False
    print("[WARN] matplotlib not installed. Install with: pip install matplotlib numpy")


# Publication-quality style settings
STYLE = {
    'figure.figsize': (8, 5),
    'font.size': 11,
    'font.family': 'serif',
    'axes.labelsize': 12,
    'axes.titlesize': 13,
    'xtick.labelsize': 10,
    'ytick.labelsize': 10,
    'legend.fontsize': 10,
    'axes.grid': True,
    'grid.alpha': 0.3,
    'axes.spines.top': False,
    'axes.spines.right': False,
}


def load_csv(filepath: Path) -> List[Dict]:
    """Load CSV file into list of dictionaries"""
    if not filepath.exists():
        return []
    with open(filepath, 'r') as f:
        reader = csv.DictReader(f)
        return list(reader)


def load_json(filepath: Path) -> Dict:
    """Load JSON file"""
    if not filepath.exists():
        return {}
    with open(filepath, 'r') as f:
        return json.load(f)


def plot_fidelity_comparison(data: List[Dict], output_path: Path):
    """
    Figure 1: Bar chart comparing original vs optimized fidelity
    """
    if not data:
        print("[WARN] No data for fidelity comparison plot")
        return

    plt.rcParams.update(STYLE)
    fig, ax = plt.subplots(figsize=(10, 6))

    # Extract data
    circuits = [d['circuit_name'].split('_')[0] for d in data[:8]]  # First 8 circuits
    original = [float(d['original_fidelity']) for d in data[:8]]
    optimized = [float(d['optimized_fidelity']) for d in data[:8]]

    x = np.arange(len(circuits))
    width = 0.35

    bars1 = ax.bar(x - width/2, original, width, label='Original', color='#2196F3', alpha=0.8)
    bars2 = ax.bar(x + width/2, optimized, width, label='QNS Optimized', color='#4CAF50', alpha=0.8)

    ax.set_ylabel('Estimated Fidelity')
    ax.set_xlabel('Circuit')
    ax.set_title('QNS Noise-Adaptive Optimization: Fidelity Comparison')
    ax.set_xticks(x)
    ax.set_xticklabels(circuits, rotation=45, ha='right')
    ax.legend()
    ax.set_ylim(0.7, 1.0)

    # Add value labels
    for bar in bars1:
        height = bar.get_height()
        ax.annotate(f'{height:.3f}',
                   xy=(bar.get_x() + bar.get_width() / 2, height),
                   xytext=(0, 3), textcoords="offset points",
                   ha='center', va='bottom', fontsize=8)

    plt.tight_layout()
    plt.savefig(output_path / 'fig1_fidelity_comparison.pdf', dpi=300, bbox_inches='tight')
    plt.savefig(output_path / 'fig1_fidelity_comparison.png', dpi=150, bbox_inches='tight')
    plt.close()
    print(f"Generated: {output_path / 'fig1_fidelity_comparison.pdf'}")


def plot_improvement_by_noise(data: List[Dict], output_path: Path):
    """
    Figure 2: Improvement percentage by noise level
    """
    if not data:
        return

    plt.rcParams.update(STYLE)
    fig, ax = plt.subplots(figsize=(8, 5))

    # Group by noise level
    noise_levels = {'low': [], 'medium': [], 'high': []}
    for d in data:
        name = d['circuit_name']
        improvement = float(d['improvement_percent'])
        if 'low_noise' in name:
            noise_levels['low'].append(improvement)
        elif 'medium_noise' in name:
            noise_levels['medium'].append(improvement)
        elif 'high_noise' in name:
            noise_levels['high'].append(improvement)

    # Create box plot
    box_data = [noise_levels['low'], noise_levels['medium'], noise_levels['high']]
    bp = ax.boxplot(box_data, labels=['Low Noise\n(T1=300us)', 'Medium Noise\n(T1=100us)', 'High Noise\n(T1=50us)'],
                    patch_artist=True)

    colors = ['#81C784', '#FFB74D', '#E57373']
    for patch, color in zip(bp['boxes'], colors):
        patch.set_facecolor(color)
        patch.set_alpha(0.7)

    ax.set_ylabel('Fidelity Improvement (%)')
    ax.set_xlabel('Noise Configuration')
    ax.set_title('QNS Optimization Improvement vs Noise Level')
    ax.axhline(y=0, color='gray', linestyle='--', linewidth=0.8)

    plt.tight_layout()
    plt.savefig(output_path / 'fig2_improvement_by_noise.pdf', dpi=300, bbox_inches='tight')
    plt.savefig(output_path / 'fig2_improvement_by_noise.png', dpi=150, bbox_inches='tight')
    plt.close()
    print(f"Generated: {output_path / 'fig2_improvement_by_noise.pdf'}")


def plot_ablation_study(output_path: Path):
    """
    Figure 3: Ablation study pie chart
    """
    plt.rcParams.update(STYLE)
    fig, ax = plt.subplots(figsize=(8, 6))

    # Ablation study data from previous results (updated values)
    components = ['Placement\nOptimization', 'Scoring\nFunction', 'Gate\nReordering']
    contributions = [74.1, 14.3, 11.6]  # From ablation study results (sum=100%)
    colors = ['#4CAF50', '#2196F3', '#FF9800']
    explode = (0.05, 0, 0)

    wedges, texts, autotexts = ax.pie(
        contributions, explode=explode, labels=components,
        colors=colors, autopct='%1.1f%%',
        shadow=False, startangle=90
    )

    for autotext in autotexts:
        autotext.set_fontsize(12)
        autotext.set_fontweight('bold')

    ax.set_title('QNS Component Contribution Analysis\n(Ablation Study)', fontsize=14)

    plt.tight_layout()
    plt.savefig(output_path / 'fig3_ablation_study.pdf', dpi=300, bbox_inches='tight')
    plt.savefig(output_path / 'fig3_ablation_study.png', dpi=150, bbox_inches='tight')
    plt.close()
    print(f"Generated: {output_path / 'fig3_ablation_study.pdf'}")


def plot_optimization_time(data: List[Dict], output_path: Path):
    """
    Figure 4: Optimization time vs circuit size
    """
    if not data:
        return

    plt.rcParams.update(STYLE)
    fig, ax = plt.subplots(figsize=(8, 5))

    gates = [int(d['num_gates']) for d in data]
    times = [float(d['optimization_time_ms']) for d in data]

    ax.scatter(gates, times, alpha=0.6, s=60, c='#2196F3', edgecolors='white', linewidth=0.5)

    # Fit and plot trend line
    if len(gates) > 2:
        z = np.polyfit(gates, times, 1)
        p = np.poly1d(z)
        x_line = np.linspace(min(gates), max(gates), 100)
        ax.plot(x_line, p(x_line), '--', color='#E53935', alpha=0.7, label='Linear trend')

    ax.set_xlabel('Number of Gates')
    ax.set_ylabel('Optimization Time (ms)')
    ax.set_title('QNS Optimization Scalability')
    ax.legend()

    plt.tight_layout()
    plt.savefig(output_path / 'fig4_optimization_time.pdf', dpi=300, bbox_inches='tight')
    plt.savefig(output_path / 'fig4_optimization_time.png', dpi=150, bbox_inches='tight')
    plt.close()
    print(f"Generated: {output_path / 'fig4_optimization_time.pdf'}")


def plot_e2e_validation(output_path: Path):
    """
    Figure 5: E2E Validation - Analytical vs Simulated
    """
    plt.rcParams.update(STYLE)
    fig, ax = plt.subplots(figsize=(8, 5))

    # E2E validation data (from benchmark output)
    strategies = ['Identity\n(baseline)', 'Placement\nOptimized', 'Co-optimization']
    analytical = [25.0, 95.0, 95.0]
    simulated = [47.0, 95.0, 94.0]

    x = np.arange(len(strategies))
    width = 0.35

    bars1 = ax.bar(x - width/2, analytical, width, label='Analytical Estimate', color='#2196F3', alpha=0.8)
    bars2 = ax.bar(x + width/2, simulated, width, label='Monte Carlo Simulation', color='#4CAF50', alpha=0.8)

    ax.set_ylabel('Fidelity (%)')
    ax.set_xlabel('Optimization Strategy')
    ax.set_title('E2E Validation: Analytical vs Monte Carlo\n(5 CNOTs on worst edge)')
    ax.set_xticks(x)
    ax.set_xticklabels(strategies)
    ax.legend(loc='lower right')
    ax.set_ylim(0, 100)

    # Add improvement annotation
    ax.annotate('', xy=(2, 95), xytext=(0, 25),
               arrowprops=dict(arrowstyle='->', color='red', lw=2))
    ax.text(1, 60, '+70%\nimprovement', ha='center', fontsize=11, color='red', fontweight='bold')

    plt.tight_layout()
    plt.savefig(output_path / 'fig5_e2e_validation.pdf', dpi=300, bbox_inches='tight')
    plt.savefig(output_path / 'fig5_e2e_validation.png', dpi=150, bbox_inches='tight')
    plt.close()
    print(f"Generated: {output_path / 'fig5_e2e_validation.pdf'}")


def plot_placement_optimization(output_path: Path):
    """
    Figure 6: Placement Optimization - Route Through Better Edges
    Shows dramatic improvement when routing through better edges
    """
    plt.rcParams.update(STYLE)
    fig, ax = plt.subplots(figsize=(10, 6))

    # Data from placement_benchmark results (corrected values)
    scenarios = ['10 CNOTs\n(worst edge)', '5+3 CNOTs\n(mixed)', '5 CNOTs\n(best edge)']
    identity = [32.1, 61.58, 95.0]   # Identity mapping fidelity (%)
    optimized = [90.0, 61.58, 95.0]  # Optimized mapping fidelity (%)

    x = np.arange(len(scenarios))
    width = 0.35

    bars1 = ax.bar(x - width/2, identity, width, label='Identity Mapping', color='#E57373', alpha=0.8)
    bars2 = ax.bar(x + width/2, optimized, width, label='QNS Optimized', color='#4CAF50', alpha=0.8)

    ax.set_ylabel('Estimated Fidelity (%)')
    ax.set_xlabel('Circuit Scenario')
    ax.set_title('Placement Optimization: Route-Through-Better-Edges\n(Linear 4-qubit: Q0─99%─Q1─90%─Q2─95%─Q3)')
    ax.set_xticks(x)
    ax.set_xticklabels(scenarios)
    ax.legend(loc='lower right')
    ax.set_ylim(0, 105)

    # Add improvement annotation for scenario 1
    ax.annotate('+58 pp', xy=(0, 90), xytext=(0, 97),
                ha='center', va='bottom', fontsize=14, color='green', fontweight='bold')

    # Add value labels
    for bar in bars2:
        height = bar.get_height()
        ax.annotate(f'{height:.0f}%',
                   xy=(bar.get_x() + bar.get_width() / 2, height),
                   xytext=(0, 3), textcoords="offset points",
                   ha='center', va='bottom', fontsize=10, fontweight='bold')

    plt.tight_layout()
    plt.savefig(output_path / 'fig6_placement_optimization.pdf', dpi=300, bbox_inches='tight')
    plt.savefig(output_path / 'fig6_placement_optimization.png', dpi=150, bbox_inches='tight')
    plt.close()
    print(f"Generated: {output_path / 'fig6_placement_optimization.pdf'}")


def plot_e2e_comparison(output_path: Path):
    """
    Figure 7: E2E Validation - All test cases
    """
    plt.rcParams.update(STYLE)
    fig, axes = plt.subplots(1, 2, figsize=(12, 5))

    # Test case 1: 5 CNOTs on worst edge (main result)
    ax1 = axes[0]
    strategies = ['Identity', 'Placement', 'Co-opt']
    analytical = [25.0, 95.0, 95.0]
    simulated = [33.0, 93.0, 94.0]

    x = np.arange(len(strategies))
    width = 0.35

    ax1.bar(x - width/2, analytical, width, label='Analytical', color='#2196F3', alpha=0.8)
    ax1.bar(x + width/2, simulated, width, label='Monte Carlo', color='#FF9800', alpha=0.8)
    ax1.set_ylabel('Fidelity (%)')
    ax1.set_title('Test 1: 5 CNOTs on Worst Edge')
    ax1.set_xticks(x)
    ax1.set_xticklabels(strategies)
    ax1.legend()
    ax1.set_ylim(0, 105)
    ax1.axhline(y=90, color='green', linestyle='--', alpha=0.5, label='90% threshold')

    # Add improvement arrow
    ax1.annotate('', xy=(1, 94), xytext=(0, 29),
                arrowprops=dict(arrowstyle='->', color='red', lw=2))
    ax1.text(0.5, 60, '+62%', ha='center', fontsize=12, color='red', fontweight='bold')

    # Test case 2: All cases summary
    ax2 = axes[1]
    test_cases = ['5 CNOTs\nworst', 'Bell\nstate', 'GHZ\nchain', 'Complex']
    identity_sim = [33, 99.99, 64, 70]
    optimized_sim = [93, 99.99, 69, 73]

    x = np.arange(len(test_cases))
    ax2.bar(x - width/2, identity_sim, width, label='Identity', color='#E57373', alpha=0.8)
    ax2.bar(x + width/2, optimized_sim, width, label='Optimized', color='#4CAF50', alpha=0.8)
    ax2.set_ylabel('Simulated Fidelity (%)')
    ax2.set_title('Monte Carlo Simulation Results')
    ax2.set_xticks(x)
    ax2.set_xticklabels(test_cases)
    ax2.legend()
    ax2.set_ylim(0, 105)

    plt.tight_layout()
    plt.savefig(output_path / 'fig7_e2e_all_cases.pdf', dpi=300, bbox_inches='tight')
    plt.savefig(output_path / 'fig7_e2e_all_cases.png', dpi=150, bbox_inches='tight')
    plt.close()
    print(f"Generated: {output_path / 'fig7_e2e_all_cases.pdf'}")


def generate_all_figures(input_dir: Path, output_dir: Path):
    """Generate all paper figures"""
    if not HAS_MATPLOTLIB:
        print("[ERROR] matplotlib required for figure generation")
        return

    output_dir.mkdir(parents=True, exist_ok=True)

    print("=" * 60)
    print("QNS Paper Figure Generator")
    print("=" * 60)

    # Load data
    fidelity_data = load_csv(input_dir / 'fidelity_benchmark.csv')
    placement_data = load_csv(input_dir / 'placement_results.csv')
    e2e_data = load_csv(input_dir / 'e2e_validation_results.csv')

    # Generate figures
    print("\nGenerating figures...")
    plot_fidelity_comparison(fidelity_data, output_dir)
    plot_improvement_by_noise(fidelity_data, output_dir)
    plot_ablation_study(output_dir)
    plot_optimization_time(fidelity_data, output_dir)
    plot_e2e_validation(output_dir)
    plot_placement_optimization(output_dir)
    plot_e2e_comparison(output_dir)

    print("\n" + "=" * 60)
    print(f"Figures saved to: {output_dir.absolute()}")
    print("=" * 60)


def main():
    parser = argparse.ArgumentParser(description='Generate QNS paper figures')
    parser.add_argument('--input', type=Path, default=Path('results'),
                       help='Input directory with CSV data')
    parser.add_argument('--output', type=Path, default=Path('figures'),
                       help='Output directory for figures')
    args = parser.parse_args()

    generate_all_figures(args.input, args.output)


if __name__ == '__main__':
    main()
