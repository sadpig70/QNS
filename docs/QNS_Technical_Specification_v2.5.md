# QNS - Quantum Noise Symbiote

## Technical Specification Document

### Version 2.5 | January 2026

**Author:** Jung Wook Yang

> *"We don't fight noise. We dance with it."*

---

## Document Status

| Item | Status |
| :--- | :--- |
| **Current Version** | v2.5.0 (Simulation Enhancements: MPS & ZNE) |
| **Verification Environment** | Local Simulator (StateVector + MPS) + IBM Quantum Aer |
| **Hardware Integration** | ✅ IBM Aer Simulation + **IBM Torino (133 qubits) Execution** |
| **Benchmark Baseline** | Qiskit Transpiler L3 + Sabre |
| **Overall Completion** | 100% |

> ⚠️ **Important:** Performance metrics include both **simulation** (scalability) and **real hardware** (validation) results.

### Module Implementation Status

| Module | Status | Notes |
| :--- | :--- | :--- |
| qns_core | ✅ Stable | Core types complete (Gate, NoiseVector, CircuitGenome, HardwareProfile) |
| qns_profiler | ✅ Stable | DriftScanner complete |
| qns_rewire | ✅ Stable | LiveRewirer, GateReorder, Router, Scoring fully implemented |
| qns_simulator | ✅ Stable | StateVectorSimulator, **MpsSimulator (New)**, NoisySimulator complete |
| qns_cli | ✅ Stable | **Qiskit backend integrated**, E2E tested |
| qns_qasm | ✅ Stable | OpenQASM parser (basic gates) |
| qns_noise | ✅ Stable | Noise channels |
| qns_tensor | ✅ Stable | MPS implementation core |
| qns_zne | ✅ Stable | **ZNE module** (LocalFolding + Extrapolation) |
| qns_python | ✅ Stable | PyO3 bindings + **Qiskit Bridge** |

### 🆕 v2.5 New Features

| Feature | Status | Notes |
| :--- | :--- | :--- |
| **MPS Simulator** | ✅ Verified | Matrix Product State simulation for large circuits (30+ qubits) |
| **ZNE Module** | ✅ Verified | Error mitigation via Zero-Noise Extrapolation (Linear, Richardson) |
| **Simulator Backend** | ✅ Enhanced | Unified `SimulatorBackend` supporting Ideal/Noisy modes via CLI |
| **Crosstalk Auto** | ✅ Verified | Crosstalk-aware routing with weight optimization strategies |

---

## Table of Contents

1. [Overview](#1-overview)
2. [Core Concepts](#2-core-concepts)
3. [System Architecture](#3-system-architecture)
4. [Qiskit Integration](#4-qiskit-integration)
5. [Algorithm Details](#5-algorithm-details)
6. [Performance Benchmarks](#6-performance-benchmarks)
7. [Roadmap](#7-roadmap)
8. [Appendix](#appendix)

---

## 1. Overview

### 1.1 What is QNS?

QNS (Quantum Noise Symbiote) is a noise-adaptive circuit optimization framework proposing a paradigm shift in quantum computing. While traditional Quantum Error Correction (QEC) approaches treat noise as an 'enemy to be eliminated', QNS **adapts** to noise characteristics to optimize circuits, and mitigates unavoidable errors using techniques like ZNE.

**Core Philosophy:** Symbiosis with Noise - Utilizing T1/T2 calibration data from quantum systems to select circuit variants optimized for current noise characteristics.

### 1.2 Key Features

| Feature | Description | Module |
| :--- | :--- | :--- |
| **DriftScan** | Real-time T1/T2 drift monitoring and anomaly detection | qns_profiler |
| **LiveRewirer** | Dynamic circuit reconstruction based on noise profile | qns_rewire |
| **GateReorder** | Commuting gate reordering optimization | qns_rewire |
| **PlacementOptimizer** | Hardware topology-based qubit placement optimization | qns_rewire |
| **NoiseAwareRouter** | Fidelity-based SWAP routing | qns_rewire |
| **StateVectorSimulator** | Full state vector quantum simulation | qns_simulator |
| **MpsSimulator** (v2.5) | Memory-efficient simulation using Matrix Product States | qns_simulator |
| **ZNE** (v2.5) | Zero-Noise Extrapolation for error mitigation | qns_zne |
| 🆕 **QiskitBridge** | QNS ↔ Qiskit circuit conversion and Aer simulation | qns_python |

### 1.3 Core Value Propositions

| Value | Description | Target | Status |
| :--- | :--- | :--- | :--- |
| Noise Adaptation | Calibration data-based circuit optimization | Noise profile reflection | ✅ Implemented |
| Local Pipeline | Simulator-based optimization speed | <100ms (5q, 20gates) | ✅ Achieved |
| Hardware Integration | Support for real hardware like IBM Quantum | Qiskit Runtime integration | ✅ **Complete** |
| Fidelity Improvement | Simulator-based quality improvement | 5-15% improvement | ✅ Verified |
| Aer Simulation | IBM noise model-based simulation | 156-qubit noise model | ✅ **Complete** |
| Scalability | Large-scale simulation via MPS | 30+ qubits | ✅ **Complete** |

---

## 2. Core Concepts

### 2.1 Noise Adaptation

QNS's "noise symbiosis" means:

1. **Noise Characterization:** Collect calibration data (T1, T2, gate error rates)
2. **Circuit Variant Generation:** Create equivalent circuits through commuting gate reordering
3. **Optimal Variant Selection:** Select the variant with highest fidelity for current noise profile

### 2.2 T1/T2 Profiling

Two key time constants for quantum qubits:

- **T1 (Energy Relaxation Time):** Characteristic time for |1⟩ state to decay to |0⟩ state
- **T2 (Phase Coherence Time):** Time for phase information of superposition state to be lost
- **Physical Constraint:** T2 ≤ 2T1

> 🆕 **v2.2 Update:** Automatic clamping applied when T2 > 2T1 cases are detected in IBM calibration data (`T2 = min(T2, 2*T1)`)

### 2.3 Error Mitigation (v2.5)

In addition to optimization (Noise Adaptation), QNS employs **Zero-Noise Extrapolation (ZNE)** to mitigate errors in execution results:

1. **Noise Amplification:** Intentionally increasing noise levels (e.g., scaling factor $\lambda = 1, 3, 5...$) using unitary folding ($G \to G G^\dagger G$).
2. **Expectation Measurement:** Measuring expectation values at each noise level.
3. **Extrapolation:** Using Linear, Richardson, or Exponential extrapolation to estimate the zero-noise value ($\lambda = 0$).

### 2.4 Fidelity Estimation Model

#### 2.4.1 Optimization Objective Function

$$
C^* = \arg\max_{C' \in \mathcal{V}(C)} \hat{F}(C', \mathbf{n}(t))
$$

| Symbol | Definition | Domain |
| :--- | :--- | :--- |
| $C$ | Original quantum circuit | Gate sequence |
| $C^*$ | Optimized circuit | Gate sequence |
| $\mathcal{V}(C)$ | Set of mathematically equivalent circuit variants | $\|V\| \geq 1$ |
| $\mathbf{n}(t)$ | Time-dependent noise profile vector | $\mathbb{R}^3$ |
| $\hat{F}$ | Fidelity estimation function | $[0, 1]$ |

#### 2.4.2 Complete Fidelity Model

$$
\boxed{
\hat{F}(C, \mathbf{n}) = (1 - \epsilon_{1q})^{n_{1q}} \cdot (1 - \epsilon_{2q})^{n_{2q}} \cdot \exp\left(-\frac{t_{total}}{T_2}\right)
}
$$

> **Note:** The above formula represents a critical-path approximation. QNS implementation employs **Idle Time Tracking** for per-qubit coherence modeling.
>
> 📘 **Detailed Mathematical Formalization:** See [QNS_Mathematical_Formalization.md](QNS_Mathematical_Formalization.md).

---

## 3. System Architecture

### 3.1 Module Structure

```text
qns/
├── crates/
│   ├── qns_core        # Core types: Gate, NoiseVector, CircuitGenome, HardwareProfile
│   ├── qns_profiler    # Noise profiling: DriftScanner
│   ├── qns_rewire      # Circuit optimization: GateReorder, LiveRewirer, Router, Scoring
│   ├── qns_simulator   # Quantum simulation: StateVector, MPS, NoisySimulator
│   ├── qns_zne         # 🆕 Zero-Noise Extrapolation: Amplifier, Extrapolator
│   ├── qns_cli         # CLI and integration: Pipeline, QnsSystem, QiskitRunner
│   ├── qns_qasm        # OpenQASM parser: Parser, AST, Builder
│   ├── qns_noise       # Noise channels: NoiseChannel, NoiseModel
│   ├── qns_tensor      # Tensor networks: TensorNetwork, MPS
│   └── qns_python/     # Python bindings + Qiskit Bridge
│       ├── src/lib.rs      # PyO3 bindings
│       └── python/         # Qiskit integration
├── docs/               # Documentation
├── scripts/            # Benchmark/analysis scripts
├── benchmarks/         # Benchmark circuits + validation scripts
└── .github/            # CI/CD
```

### 3.2 Current Architecture (v2.5)

```text
┌─────────────────────────────────────────────────────────────────────┐
│                    QNS Architecture v2.5                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐             │
│  │  qns_core   │    │qns_profiler │    │ qns_rewire  │             │
│  │ [✅ Complete]│    │[✅ Complete]│    │[✅ Complete]│             │
│  └─────────────┘    └─────────────┘    └─────────────┘             │
│         │                  │                  │                    │
│         └──────────────────┼──────────────────┘                    │
│                            │                                       │
│          ┌─────────────────┴───────────────────┐                   │
│          │                                     │                   │
│  ┌───────▼──────┐    ┌─────▼───────┐    ┌──────▼──────┐           │
│  │ qns_simulator│    │   qns_zne   │    │  qns_tensor │           │
│  │[✅+MPS/Noisy]│    │[✅ Complete] │    │[✅ Complete] │           │
│  └──────────────┘    └─────────────┘    └─────────────┘           │
│                                                                     │
│              ┌────────────┼────────────┐                           │
│              │            │            │                           │
│         ┌────┴────┐ ┌─────┴─────┐ ┌────┴────┐                     │
│         │qns_cli  │ │qns_python │ │qns_noise│                     │
│         │[✅ Done]│ │ [✅ Done] │ │[✅ Done]│                      │
│         └─────────┘ └─────┬─────┘ └─────────┘                     │
│                           │                                        │
│              ┌────────────┴────────────┐                           │
│              │   Qiskit Bridge         │                           │
│              │     [✅ Complete]       │                           │
│              └────────────┬────────────┘                           │
│                           │                                        │
│              ┌────────────┴────────────┐                           │
│              │    IBM Quantum          │                           │
│              │  ibm_torino (133q)      │                           │
│              │  [✅ Verified]          │                           │
│              └─────────────────────────┘                           │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 4. Qiskit Integration

### 4.1 Integration Overview

QNS v2.5 maintains full integration with the IBM Qiskit ecosystem.

- **Frontend**: QNS optimized circuits can be sent to Qiskit backends.
- **Backend**: Qiskit noise models and calibration data inform QNS optimizations.

### 4.2 CLI Backend Options

```bash
# QNS native simulator (default)
qns run circuit.qasm

# MPS Simulator (Memory Efficient) (v2.5)
qns run circuit.qasm --backend mps

# Qiskit Aer ideal simulation
qns run circuit.qasm --backend aer-ideal --shots 1024

# Qiskit Aer noisy simulation (mock calibration)
qns run circuit.qasm --backend aer-noisy --shots 2048

# Qiskit Aer + IBM backend calibration
qns run circuit.qasm --backend aer-ibm --ibm-backend ibm_fez --shots 1024
```

---

## 5. Algorithm Details

### 5.1 MPS Simulation (v2.5)

Matrix Product States (MPS) represent the quantum state as a chain of tensors, allowing efficient simulation of states with low entanglement (low bond dimension $\chi$).

- **Tensor Contraction**: Gates are applied by contracting local tensors.
- **SVD Truncation**: After each 2-qubit gate, SVD is performed and singular values are truncated to maintain maximum bond dimension $\chi_{max}$.
- **Complexity**: $O(n \cdot \chi^3)$ vs $O(2^n)$ for state vector.

### 5.2 Zero-Noise Extrapolation (v2.5)

- **Noise Scaling**: Unitary folding $G \to G (G^\dagger G)^k$. Scale factor $\lambda = 1 + 2k$.
- **Extrapolation**:
  - **Linear**: $E(\lambda) = E_0 + a\lambda$. Uses 2 points (e.g., $\lambda=1, 3$).
  - **Richardson**: Polynomial fit to eliminate higher-order error terms.
  - **Exponential**: $E(\lambda) = E_0 + A e^{-B\lambda}$. Better for decoherence-dominated noise.

### 5.3 Crosstalk-Aware Routing (v2.4)

$$ H(n) = W_{dist} \cdot D + W_{err} \cdot E + W_{xtalk} \cdot X $$

- **$X$ penalty**: Adds cost if parallel SWAPs activate edges with high crosstalk interaction ($zz\_interaction$).
- **Config**: `--crosstalk-weight <FLOAT>` allows tuning the importance of crosstalk avoidance.

---

## 6. Performance Benchmarks

### 6.1 Scaling

| Qubits | State Vector Size | Memory | Execute (20g) | MPS (Bond 16) |
| :--- | :--- | :--- | :--- | :--- |
| 5 | 32 | 512 B | ~1.4 μs | ~2.0 μs |
| 10 | 1,024 | 16 KB | ~45 μs | ~50 μs |
| 15 | 32,768 | 512 KB | ~1.5 ms | ~200 μs |
| 20 | 1,048,576 | 16 MB | ~50 ms | ~1 ms |
| 25 | 33,554,432 | 512 MB | ~2 s | ~5 ms |
| 30 | 1B | 16 GB | ~1 min | ~20 ms |

### 6.2 Crosstalk Resilience (Mock Backend)

| Circuit | Crosstalk Weight | Fidelity | Improvement |
| :--- | :--- | :--- | :--- |
| GHZ-5   | 0.0 (Baseline)   | 0.1094   | -           |
| **GHZ-5** | **0.25+**      | **0.8816** | **+705.8%** |

---

## 7. Roadmap

### 7.1 v1.0.0 - Hardware Verification ✅

- ✅ IBM Runtime real QPU job submission (`ibm_torino` verified)
- ✅ QNS vs. Qiskit Transpiler statistical comparison

### 7.2 v2.4.0 - Crosstalk & Routing ✅

- ✅ Crosstalk model and Sabre router upgrade (`qns_rewire`)
- ✅ CLI `--crosstalk-weight` support

### 7.3 v2.5.0 (Current) - Simulation & Mitigation ✅

- ✅ **MPS Simulator**: Scalable to 30+ qubits for low-entanglement circuits.
- ✅ **ZNE**: Error mitigation integrated into CLI/Library.
- ✅ **Enhanced CLI Testing**: E2E pipeline verification complete.
- ✅ **Refined Documentation**: Updated specs and benchmarks.

### 7.4 v3.0.0 (Future) - Cloud & Distributed

- 📋 Cloud service API (REST/gRPC)
- 📋 Distributed Tensor Network simulation
- 📋 Multi-backend support (IonQ, Rigetti)

---

## Appendix

### A. Technology Stack

| Category | Technology |
| :--- | :--- |
| Language | Rust 1.75+ |
| Math | num-complex, ndarray |
| Tensor | qns_tensor (custom) |
| CLI | clap |
| Python | PyO3, Qiskit 1.0+ |

### B. Test Coverage (v2.5)

| Crate | Unit | Integration | Status |
| :--- | :--- | :--- | :--- |
| qns_core | 50+ | - | ✅ |
| qns_simulator | 45+ | - | ✅ |
| qns_zne | 15+ | - | ✅ |
| qns_cli | 10+ | 17+ | ✅ |
| **Total** | **250+** | **20+** | **Stable** |

### C. License

QNS is provided under the MIT License.

---

*— End of Document —*

*Copyright © 2026 Jung Wook Yang. Licensed under MIT.*
