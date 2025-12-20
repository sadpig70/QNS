# QNS - Quantum Noise Symbiote

[![PyPI](https://img.shields.io/pypi/v/qns)](https://pypi.org/project/qns/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE)

**Noise-aware quantum circuit optimization library written in Rust with Python bindings.**

QNS optimizes quantum circuits by intelligently reordering gates to minimize the impact of hardware noise, improving execution fidelity on real quantum devices.

## Installation

```bash
pip install qns
```

### From source

```bash
git clone https://github.com/qns-ai/qns-mvp
cd qns-mvp/crates/qns_python
pip install maturin
maturin develop
```

## Quick Start

```python
from qns import Circuit, QnsOptimizer, NoiseModel

# Create a GHZ-3 circuit
circuit = Circuit(num_qubits=3)
circuit.h(0)
circuit.cnot(0, 1)
circuit.cnot(1, 2)
circuit.measure_all()

# Create optimizer with noise model
noise = NoiseModel(t1=100.0, t2=80.0, gate_error_1q=0.001, gate_error_2q=0.01)
optimizer = QnsOptimizer(num_qubits=3, noise_model=noise)

# Optimize
result = optimizer.optimize(circuit)
print(f"Original score: {result.original_score:.4f}")
print(f"Optimized score: {result.optimized_score:.4f}")
print(f"Improvement: {result.score_improvement:.2%}")
print(f"Algorithm: {result.algorithm}")
```

## Simulation

```python
from qns import Circuit, SimulatorBackend, NoiseModel

# Create circuit
circuit = Circuit(num_qubits=2)
circuit.h(0)
circuit.cnot(0, 1)
circuit.measure_all()

# Run on ideal simulator
backend = SimulatorBackend.ideal(num_qubits=2)
result = backend.run(circuit, shots=1000)
print(result.counts)  # {'00': ~500, '11': ~500}

# Run on noisy simulator
noise = NoiseModel(t1=50.0, t2=40.0, gate_error_1q=0.01, gate_error_2q=0.05)
noisy_backend = SimulatorBackend(num_qubits=2, noise_model=noise)
result = noisy_backend.run(circuit, shots=1000)
print(result.counts)  # {'00': ~450, '11': ~450, '01': ~50, '10': ~50}
```

## Qiskit Integration

```python
from qiskit import QuantumCircuit
from qns import Circuit
from qns.convert import circuit_from_qasm, circuit_to_qasm

# Convert from Qiskit
qc = QuantumCircuit(3)
qc.h(0)
qc.cx(0, 1)
qc.cx(1, 2)

# QNS uses OpenQASM for conversion
qasm = qc.qasm()
circuit = circuit_from_qasm(qasm)

# Optimize and convert back
optimizer = QnsOptimizer(num_qubits=3)
result = optimizer.optimize(circuit)

# Get optimized QASM
optimized_qasm = circuit_to_qasm(result.optimized_circuit)
qc_optimized = QuantumCircuit.from_qasm_str(optimized_qasm)
```

## API Reference

### Core Types

| Class | Description |
|-------|-------------|
| `Gate` | Quantum gate (H, X, Y, Z, S, T, Rx, Ry, Rz, CNOT, CZ, SWAP, Measure) |
| `Circuit` | Quantum circuit container |
| `NoiseVector` | Per-qubit noise parameters |
| `NoiseModel` | Simulator noise configuration |
| `HardwareProfile` | Hardware topology |

### Optimizer

| Class | Description |
|-------|-------------|
| `QnsOptimizer` | Noise-aware circuit optimizer |
| `OptimizationResult` | Optimization result with metrics |

### Backend

| Class | Description |
|-------|-------------|
| `SimulatorBackend` | Quantum circuit simulator |
| `ExecutionResult` | Execution result with counts |
| `CalibrationData` | Backend calibration data |

### Convert Module

| Function | Description |
|----------|-------------|
| `circuit_to_qasm(circuit)` | Convert to OpenQASM 2.0 |
| `circuit_from_qasm(qasm)` | Parse from OpenQASM 2.0 |
| `circuit_to_dict(circuit)` | Convert to dictionary |
| `circuit_from_dict(dict)` | Create from dictionary |

## Performance

QNS is written in Rust for maximum performance:

- **~95μs** pipeline latency (5 qubits, 20 gates)
- **Beam Search** for large circuits (50+ gates)
- **Parallel execution** with Rayon

## License

MIT OR Apache-2.0
