#!/usr/bin/env python3
"""
QNS Paper Data Collector
========================

Collects experimental data for journal paper submission.
Runs multiple benchmarks and exports results in CSV format for graphs.

Usage:
    python paper_data_collector.py
    python paper_data_collector.py --output results/
"""

import subprocess
import json
import csv
import argparse
import os
from pathlib import Path
from dataclasses import dataclass, asdict
from typing import List, Dict, Optional
import sys

@dataclass
class CircuitResult:
    """Single circuit benchmark result"""
    circuit_name: str
    num_qubits: int
    num_gates: int
    depth: int
    two_qubit_count: int
    original_fidelity: float
    optimized_fidelity: float
    improvement: float
    improvement_percent: float
    optimization_time_ms: float
    strategy: str


@dataclass
class ScalabilityResult:
    """Scalability test result"""
    num_qubits: int
    num_gates: int
    optimization_time_ms: float
    fidelity_improvement: float
    variants_evaluated: int


def run_rust_benchmark(benchmark_name: str, args: List[str] = None) -> Optional[dict]:
    """Run a Rust benchmark and return JSON results"""
    cmd = [
        "cargo", "run", "--release",
        "--example", benchmark_name,
        "--"
    ]
    if args:
        cmd.extend(args)

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            encoding='utf-8',
            errors='replace',
            cwd=Path(__file__).parent.parent,
            timeout=300
        )

        if result.returncode != 0:
            stderr_msg = result.stderr[:200] if result.stderr else "Unknown error"
            print(f"[WARN] {benchmark_name} failed: {stderr_msg}", file=sys.stderr)
            return None

        # Try to parse JSON from stdout
        stdout = result.stdout or ""

        # Try to parse complete JSON (may span multiple lines)
        try:
            # Find JSON object start
            json_start = stdout.find('{')
            if json_start >= 0:
                json_str = stdout[json_start:]
                return json.loads(json_str)
        except json.JSONDecodeError:
            pass

        # Fallback: try each line
        for line in stdout.strip().split('\n'):
            if line.startswith('{'):
                try:
                    return json.loads(line)
                except json.JSONDecodeError:
                    continue

        return None

    except (subprocess.TimeoutExpired, Exception) as e:
        print(f"[ERROR] {benchmark_name}: {e}", file=sys.stderr)
        return None


def collect_fidelity_data() -> List[CircuitResult]:
    """Collect fidelity benchmark data"""
    print("Collecting fidelity benchmark data...")

    data = run_rust_benchmark("fidelity_benchmark", ["--json"])
    if not data:
        return []

    results = []
    for r in data.get("results", []):
        results.append(CircuitResult(
            circuit_name=r.get("circuit", "unknown"),
            num_qubits=r.get("num_qubits", 0),
            num_gates=r.get("num_gates", 0),
            depth=r.get("depth", 0),
            two_qubit_count=r.get("two_qubit_count", 0),
            original_fidelity=r.get("original_fidelity", 0),
            optimized_fidelity=r.get("optimized_fidelity", 0),
            improvement=r.get("improvement", 0),
            improvement_percent=r.get("improvement_percent", 0),
            optimization_time_ms=r.get("optimization_time_ms", 0),
            strategy=r.get("strategy", "unknown"),
        ))

    return results


def collect_hardware_aware_data() -> List[Dict]:
    """Collect hardware-aware optimization data"""
    print("Collecting hardware-aware benchmark data...")

    data = run_rust_benchmark("hardware_aware_benchmark", ["--json"])
    if not data:
        return []

    return data.get("results", [])


def collect_e2e_validation_data() -> Dict:
    """Collect end-to-end validation data"""
    print("Collecting E2E validation data...")

    data = run_rust_benchmark("e2e_validation", ["--json"])
    return data or {}


def export_to_csv(results: List[Dict], filename: str, fieldnames: List[str]):
    """Export results to CSV file"""
    if not results:
        print(f"[WARN] No data to export for {filename}")
        return

    with open(filename, 'w', newline='') as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        for r in results:
            row = {k: r.get(k, '') if isinstance(r, dict) else getattr(r, k, '') for k in fieldnames}
            writer.writerow(row)

    print(f"Exported: {filename} ({len(results)} rows)")


def generate_latex_table(results: List[CircuitResult], filename: str):
    """Generate LaTeX table for paper"""
    if not results:
        return

    with open(filename, 'w') as f:
        f.write("\\begin{table}[ht]\n")
        f.write("\\centering\n")
        f.write("\\caption{QNS Optimization Results on Standard Benchmark Circuits}\n")
        f.write("\\label{tab:benchmark_results}\n")
        f.write("\\begin{tabular}{lrrrrr}\n")
        f.write("\\toprule\n")
        f.write("Circuit & Qubits & Gates & Original $F$ & Optimized $F$ & Improvement \\\\\n")
        f.write("\\midrule\n")

        for r in results:
            escaped_name = r.circuit_name.replace('_', r'\_')
            f.write(f"{escaped_name} & {r.num_qubits} & {r.num_gates} & ")
            f.write(f"{r.original_fidelity:.4f} & {r.optimized_fidelity:.4f} & ")
            f.write(f"+{r.improvement_percent:.2f}" + r"\% \\" + "\n")

        f.write("\\bottomrule\n")
        f.write("\\end{tabular}\n")
        f.write("\\end{table}\n")

    print(f"Generated LaTeX table: {filename}")


def generate_summary_stats(results: List[CircuitResult]) -> Dict:
    """Generate summary statistics"""
    if not results:
        return {}

    improvements = [r.improvement_percent for r in results]
    times = [r.optimization_time_ms for r in results]

    return {
        "total_circuits": len(results),
        "avg_improvement_percent": sum(improvements) / len(improvements),
        "max_improvement_percent": max(improvements),
        "min_improvement_percent": min(improvements),
        "avg_optimization_time_ms": sum(times) / len(times),
        "circuits_improved": sum(1 for r in results if r.improvement > 0),
    }


def main():
    parser = argparse.ArgumentParser(description='QNS Paper Data Collector')
    parser.add_argument('--output', type=Path, default=Path('results'),
                       help='Output directory for results')
    args = parser.parse_args()

    # Create output directory
    args.output.mkdir(parents=True, exist_ok=True)

    print("=" * 60)
    print("QNS Paper Data Collector")
    print("=" * 60)

    # 1. Collect fidelity benchmark data
    fidelity_results = collect_fidelity_data()
    if fidelity_results:
        export_to_csv(
            [asdict(r) for r in fidelity_results],
            args.output / "fidelity_benchmark.csv",
            ["circuit_name", "num_qubits", "num_gates", "depth", "two_qubit_count",
             "original_fidelity", "optimized_fidelity", "improvement",
             "improvement_percent", "optimization_time_ms", "strategy"]
        )
        generate_latex_table(fidelity_results, args.output / "table_benchmark.tex")

    # 2. Collect hardware-aware data
    hw_results = collect_hardware_aware_data()
    if hw_results:
        export_to_csv(
            hw_results,
            args.output / "hardware_aware.csv",
            ["circuit", "topology", "original_fidelity", "optimized_fidelity",
             "improvement", "swaps_inserted", "strategy"]
        )

    # 3. Collect E2E validation data
    e2e_data = collect_e2e_validation_data()
    if e2e_data:
        with open(args.output / "e2e_validation.json", 'w') as f:
            json.dump(e2e_data, f, indent=2)
        print(f"Exported: {args.output / 'e2e_validation.json'}")

    # 4. Generate summary
    summary = generate_summary_stats(fidelity_results)
    if summary:
        print("\n" + "=" * 60)
        print("Summary Statistics")
        print("=" * 60)
        print(f"  Total circuits tested: {summary['total_circuits']}")
        print(f"  Circuits improved: {summary['circuits_improved']}")
        print(f"  Average improvement: {summary['avg_improvement_percent']:.2f}%")
        print(f"  Max improvement: {summary['max_improvement_percent']:.2f}%")
        print(f"  Average optimization time: {summary['avg_optimization_time_ms']:.2f}ms")

        with open(args.output / "summary.json", 'w') as f:
            json.dump(summary, f, indent=2)
        print(f"\nExported: {args.output / 'summary.json'}")

    print("\n" + "=" * 60)
    print("Data collection complete!")
    print(f"Results saved to: {args.output.absolute()}")
    print("=" * 60)


if __name__ == '__main__':
    main()
