# QNS MVP - Quantum Noise Symbiote

![infographic](docs/qns_infographic_v2.5.png)

**QNS (Quantum Noise Symbiote)** is a revolutionary quantum computing platform that transforms the traditional noise elimination paradigm into **noise symbiosis**.

> "We don't fight noise. We dance with it."

## 🎯 Key Features

- **DriftScan**: Real-time T1/T2 drift monitoring and anomaly detection
- **LiveRewirer**: Dynamic circuit rewiring based on noise profiles
- **GateReorder**: Commutative gate reordering for optimization
- **StateVectorSimulator**: Full state vector quantum simulation
- 🆕 **MPS Simulator**: Efficient simulation of large quantum circuits with low entanglement
- 🆕 **Crosstalk-Aware Routing**: Mitigate local interference (ZZ) errors via weighted routing
- 🆕 **Zero-Noise Extrapolation (ZNE)**: Error mitigation by scaling noise and extrapolating to zero

## 📦 Crates

| Crate | Description |
| :--- | :--- |
| `qns_core` | Core types: Gate, NoiseVector, CircuitGenome |
| `qns_profiler` | Noise profiling and drift scanning |
| `qns_rewire` | Circuit rewiring and optimization |
| `qns_simulator` | Quantum simulators (StateVector, MPS) |
| `qns_zne` | Zero-Noise Extrapolation module |
| `qns_cli` | Command-line interface |

## 🚀 Quick Start

```rust
use qns_core::prelude::*;
use qns_profiler::DriftScanner;
use qns_rewire::LiveRewirer;

// Create a circuit
let mut circuit = CircuitGenome::new(3);
circuit.add_gate(Gate::H(0))?;
circuit.add_gate(Gate::CNOT(0, 1))?;
circuit.add_gate(Gate::CNOT(1, 2))?;

// Profile noise
let mut scanner = DriftScanner::with_defaults();
let noise = scanner.scan(0)?;

// Optimize circuit
let mut rewirer = LiveRewirer::new();
rewirer.load(circuit)?;
let optimized = rewirer.optimize(&noise, 10)?;
```

## 🔧 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
qns_core = "0.2"
qns_profiler = "0.2"
qns_rewire = "0.2"
qns_zne = "0.1"
```

## 💻 CLI Usage

### Native QNS Simulation

```bash
# Build the CLI
cargo build --release -p qns_cli

# Run a circuit with default backend (QNS simulator)
./target/release/qns run circuit.qasm

# Specify topology
./target/release/qns run circuit.qasm --topology grid
```

### Qiskit Backend Integration ⭐ NEW

```bash
# Ideal Aer simulation
qns run bell_state.qasm --backend aer-ideal --shots 1024

# Noisy simulation with mock calibration
qns run bell_state.qasm --backend aer-noisy --shots 2048

# Aer with real IBM backend calibration
export QISKIT_IBM_TOKEN='your_token'
qns run bell_state.qasm --backend aer-ibm --ibm-backend ibm_fez

# Output formats (text or json)
qns run circuit.qasm --backend aer-ideal --format json

# 🆕 Crosstalk-Aware Routing
qns run circuit.qasm --crosstalk-weight 0.5
```

### Advanced Features (v2.5) ⭐

```bash
# 🆕 Zero-Noise Extrapolation (ZNE)
qns run circuit.qasm --zne linear

# 🆕 MPS Simulator (for large, shallow circuits)
qns run circuit.qasm --backend mps
```

**Supported Backends**: `simulator` (default), `mps`, `aer-ideal`, `aer-noisy`, `aer-ibm`

### Other Commands

```bash
./target/release/qns benchmark -q 5 -g 20 -i 100

# Show system info
./target/release/qns info
```

## 📊 Performance Targets

| Module | Target |
| :--- | :--- |
| DriftScan | <10ms |
| GateReorder | <20ms |
| LiveRewirer | <100ms |
| StateVectorSim | <50ms (10 qubits) |
| MpsSim | <100ms (20 qubits, low entanglement) |

## 📈 Benchmark Results

QNS LiveRewirer optimization (SABRE + optimization_level=3) vs Baseline (level=1):

### Ideal Environment

| Circuit | Baseline | QNS | Improvement |
| :--- | :--- | :--- | :--- |
| Bell | 1.0000 | 1.0000 | +0.0% |
| GHZ-5 | 1.0000 | 0.9700 | -3.0% |
| **VQE** | 0.4000 | **0.4576** | **+14.4%** |

### NISQ Environment (Noisy) ⭐

| Circuit | Baseline | QNS | Improvement |
| :--- | :--- | :--- | :--- |
| GHZ-5 | 0.9700 | 0.9700 | +0.0% |
| **VQE** | 0.3600 | **0.4576** | **+27.1%** ✅ |

### Crosstalk Resilience (Mock Backend) ⭐ NEW v2.4

| Circuit | Crosstalk Weight | Fidelity | Improvement |
| :--- | :--- | :--- | :--- |
| GHZ-5   | 0.0 (Baseline)   | 0.1094   | -           |
| **GHZ-5** | **0.25+**      | **0.8816** | **+705.8%** |

> 📄 See [Benchmark Results](docs/QNS_Benchmark_Results.md) for full analysis.

```bash
# Ideal benchmark
python benchmarks/arxiv_benchmark.py --output benchmarks/results

# NISQ noisy benchmark
python benchmarks/arxiv_benchmark.py --output benchmarks/results --noisy
```

## 📖 Documentation

- [Technical Specification (English)](docs/QNS_Technical_Specification_v2.5.md)
- [Technical Specification (Korean)](_legacy/docs/QNS_Technical_Specification_v2.3_kr.md)
- [Qiskit Usage Examples](docs/QNS_Qiskit_Usage_Examples.md)
- [Benchmark Results](docs/QNS_Benchmark_Results.md)
- [Mathematical Formalization](docs/QNS_Mathematical_Formalization.md)

## 🛠️ Development

```bash
# Build
cargo build

# Test
cargo test --all

# Format
cargo fmt --all

# Lint
cargo clippy --all-targets

# Documentation
cargo doc --open
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 👤 Author

**Jung Wook Yang** (양정욱)

- Email: <sadpig70@gmail.com>
- GitHub: [@sadpig70](https://github.com/sadpig70/QNS)

---

*Copyright © 2025 QNS. All rights reserved.*
