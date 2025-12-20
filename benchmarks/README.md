# QNS Benchmark Suite

Automated benchmarking infrastructure for publication-quality performance analysis.

## Overview

This benchmark suite measures three key aspects of QNS performance:

1. **Routing Efficiency**: SWAP gate count and circuit depth
2. **Simulation Performance**: Execution time and memory usage
3. **Noise Accuracy**: Comparison with Qiskit Aer

## Quick Start

### Prerequisites

```bash
# Install Python dependencies
pip install matplotlib numpy

# Build QNS
cd ..
cargo build --release
```

### Running Benchmarks

```bash
# Run all benchmarks
python scripts/benchmark.py --qns-binary target/release/qns.exe

# Run only routing benchmarks
python scripts/benchmark.py --mode routing

# Run only simulation benchmarks
python scripts/benchmark.py --mode simulation

# Specify custom circuit directory
python scripts/benchmark.py --circuits-dir benchmarks/qasmbench
```

## Benchmark Circuits

The `benchmarks/qasmbench/` directory contains circuits from QASMBench:

- `ghz_n4.qasm`: 4-qubit GHZ state
- `qft_n4.qasm`: 4-qubit Quantum Fourier Transform
- `wstate_n3.qasm`: 3-qubit W-state
- `adder_n4.qasm`: 4-qubit adder
- `vqe_n4.qasm`: 4-qubit VQE ansatz

## Output

Results are saved to `benchmark_results/`:

- `routing_efficiency.csv`: Raw routing data
- `simulation_performance.csv`: Raw simulation data
- `routing_efficiency.pdf`: Publication-quality figure
- `simulation_performance.pdf`: Publication-quality figure

## Metrics

### Routing Efficiency

- **SWAP Count**: Number of SWAP gates added during routing
- **Circuit Depth**: Depth of routed circuit
- **Routing Time**: Time to compute routing solution

### Simulation Performance

- **Execution Time**: Total simulation time (ms)
- **Memory Usage**: Peak memory consumption (MB)
- **Shots**: Number of measurement samples

## Adding New Circuits

1. Place QASM file in `benchmarks/qasmbench/`
2. Ensure it follows OpenQASM 2.0 format
3. Include `qelib1.inc` for standard gates
4. Run benchmark script

## Citation

If you use this benchmark suite, please cite:

```bibtex
@article{qns2025,
  title={QNS: An Integrated Quantum Network Simulator with Optimized Routing},
  author={Yang, Jung Wook},
  journal={TBD},
  year={2025}
}
```

## License

MIT
