# QNS - Quantum Noise Symbiote

## Technical Specification Document

**Version 2.1 | December 2025**

**Author:** Jung Wook Yang

> *"We don't fight noise. We dance with it."*

---

## Document Status

| Item | Status |
|------|--------|
| **Current Version** | v0.1.0 (Deployment Ready) |
| **Test Environment** | Local Simulator |
| **Hardware Integration** | Design Complete, Implementation Pending |
| **Benchmark Baseline** | StateVector Simulator (CPU) |
| **Overall Completion** | ~95% |

> ⚠️ **Important:** All performance metrics and fidelity estimates in this document are based on **simulator environment**. Real quantum hardware performance will be validated in future releases.

### Module Implementation Status

| Module | Status | Notes |
|--------|--------|-------|
| qns_core | ✅ Stable | Core types complete (Gate, NoiseVector, CircuitGenome, HardwareProfile) |
| qns_profiler | ✅ Stable | DriftScanner complete |
| qns_rewire | ✅ Stable | LiveRewirer, GateReorder, Router, Scoring fully implemented |
| qns_simulator | ✅ Stable | StateVectorSimulator, NoisySimulator, NoiseModel complete |
| qns_cli | ✅ Stable | QnsSystem Pipeline complete, CLI commands implemented |
| qns_qasm | ✅ Stable | OpenQASM parser |
| qns_noise | ✅ Stable | Noise channels |
| qns_tensor | ✅ Stable | MPS implementation |
| qns_python | ✅ Stable | PyO3 bindings complete |

---

## Table of Contents

1. [Overview](#1-overview)
2. [Core Concepts](#2-core-concepts)
3. [System Architecture](#3-system-architecture)
4. [Algorithm Details](#4-algorithm-details)
5. [Performance Benchmarks](#5-performance-benchmarks)
6. [Roadmap](#6-roadmap)
7. [Appendix](#appendix)

---

## 1. Overview

### 1.1 What is QNS?

QNS (Quantum Noise Symbiote) is a noise-adaptive circuit optimization framework that proposes a paradigm shift in quantum computing. While traditional Quantum Error Correction (QEC) approaches treat noise as 'an enemy to eliminate', QNS **adapts** to noise characteristics to optimize circuits.

**Core Philosophy:** Symbiosis with Noise - Utilizing T1/T2 calibration data from quantum systems to select circuit variants optimized for current noise characteristics.

### 1.2 Key Features

| Feature | Description | Module |
|---------|-------------|--------|
| **DriftScan** | Real-time T1/T2 drift monitoring and anomaly detection | qns_profiler |
| **LiveRewirer** | Dynamic circuit rewiring based on noise profiles | qns_rewire |
| **GateReorder** | Commutative gate reordering optimization | qns_rewire |
| **PlacementOptimizer** | Hardware topology-aware qubit placement optimization | qns_rewire |
| **NoiseAwareRouter** | Fidelity-based SWAP routing | qns_rewire |
| **StateVectorSimulator** | Full state vector quantum simulation | qns_simulator |
| **NoisySimulator** | Noise model applied simulation | qns_simulator |

### 1.3 Value Proposition

| Value | Description | Target | Status |
|-------|-------------|--------|--------|
| Noise Adaptation | Circuit optimization based on calibration data | Noise profile reflection | ✅ Implemented |
| Local Pipeline | Simulator-based optimization speed | <100ms (5q, 20gates) | ✅ Achieved |
| Hardware Integration | IBM Quantum and other real hardware support | Qiskit Runtime integration | 🔄 Design Complete |
| Fidelity Improvement | Quality improvement on simulator basis | 5-15% improvement (simulator) | ✅ Verified |

---

## 2. Core Concepts

### 2.1 Noise Adaptation

QNS's "noise symbiosis" means:

1. **Noise Profiling:** Collect calibration data (T1, T2, gate error rates)
2. **Variant Generation:** Generate equivalent circuits through commutative gate reordering
3. **Optimal Selection:** Select the variant with highest fidelity for current noise profile

### 2.2 T1/T2 Profiling

Two key time constants of quantum qubits:

- **T1 (Energy Relaxation Time):** Characteristic time for |1⟩ state to decay to |0⟩
- **T2 (Phase Coherence Time):** Time for phase information of superposition state to be lost
- **Physical Constraint:** T2 ≤ 2T1

### 2.3 Circuit Rewiring

The same quantum algorithm exhibits different noise impacts depending on gate ordering. QNS's LiveRewirer:

- Identifies commutative gate pairs (Commutation Analysis)
- Generates circuit variants using BFS/Beam Search
- Selects optimal variant for current noise profile
- Reflects hardware connectivity constraints (Coupling Map)
- Fidelity-based SWAP routing (NoiseAwareRouter)

### 2.4 Fidelity Estimation Model

$$F_{circuit} \approx \prod_g (1 - \varepsilon_g) \times \exp\left(-\sum_g \frac{t_g}{T_1}\right) \times \exp\left(-\sum_g \frac{t_g}{T_\phi}\right) \times (1 - \varepsilon_{ro})^{n_m}$$

Where:

- $\varepsilon_g$: Gate error rate (1Q/2Q differentiated)
- $t_g$: Gate execution time
- $T_1$: Energy relaxation time
- $T_\phi$: Pure dephasing time
- $\varepsilon_{ro}$: Readout error rate
- $n_m$: Number of measurement qubits

> **Note:** This model is a **heuristic for relative comparison**.

---

## 3. System Architecture

### 3.1 Module Structure

```
qns-mvp/
├── crates/
│   ├── qns_core        # Core types: Gate, NoiseVector, CircuitGenome, HardwareProfile
│   ├── qns_profiler    # Noise profiling: DriftScanner
│   ├── qns_rewire      # Circuit optimization: GateReorder, LiveRewirer, Router, Scoring
│   ├── qns_simulator   # Quantum simulation: StateVectorSimulator, NoisySimulator
│   ├── qns_cli         # CLI and integration: Pipeline, QnsSystem
│   ├── qns_qasm        # OpenQASM parser: Parser, AST, Builder
│   ├── qns_noise       # Noise channels: NoiseChannel, NoiseModel
│   ├── qns_tensor      # Tensor network: TensorNetwork, MPS
│   └── qns_python      # Python bindings: PyO3 module
├── docs/               # Documentation
├── scripts/            # Benchmark/analysis scripts
├── benchmarks/         # Benchmark circuits
└── .github/            # CI/CD
```

### 3.2 Current Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    QNS Architecture v2.1                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │  qns_core   │    │qns_profiler │    │ qns_rewire  │     │
│  │  [✅ Done]  │    │  [✅ Done]  │    │  [✅ Done]  │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
│         │                  │                  │            │
│         └──────────────────┴──────────────────┘            │
│                           │                                │
│  ┌─────────────┐    ┌─────┴─────┐    ┌─────────────┐      │
│  │  qns_qasm   │    │qns_simulator│   │ qns_tensor  │      │
│  │  [✅ Done]  │    │  [✅ Done]  │   │  [✅ Done]  │      │
│  └─────────────┘    └───────────┘    └─────────────┘      │
│         │                  │                  │            │
│         └──────────────────┴──────────────────┘            │
│                           │                                │
│              ┌────────────┼────────────┐                   │
│              │            │            │                   │
│         ┌────┴────┐ ┌─────┴─────┐ ┌────┴────┐             │
│         │qns_cli  │ │qns_python │ │qns_noise│             │
│         │[✅ Done]│ │ [✅ Done] │ │[✅ Done]│             │
│         └─────────┘ └───────────┘ └─────────┘             │
└─────────────────────────────────────────────────────────────┘
```

### 3.3 Data Flow

```
Circuit Input → DriftScanner → NoiseVector → LiveRewirer → Optimized Circuit
                                    ↓
                              [Hardware Topology]
                                    ↓
                     PlacementOptimizer + NoiseAwareRouter
                                    ↓
                            StateVectorSimulator → Execution Result
```

---

## 4. Algorithm Details

### 4.1 GateReorder Algorithm

**BFS-based Variant Generation:**

```
INPUT: circuit, max_depth, max_variants
OUTPUT: List<CircuitVariant>

1. queue = [circuit], visited = {}
2. WHILE queue.not_empty AND variants.len < max_variants:
   a. current = queue.pop_front()
   b. pairs = find_adjacent_commuting_pairs(current)
   c. FOR each (i, j) in pairs:
      new_circuit = swap_gates(current, i, j)
      IF new_circuit NOT IN visited:
         variants.push(new_circuit)
3. RETURN variants
```

**Complexity Analysis:**

| Algorithm | Time Complexity | Space Complexity | Suitable Circuits |
|-----------|-----------------|------------------|-------------------|
| BFS | O(V × E) | O(V) | <50 gates |
| Beam Search | O(k × n × b) | O(b) | <500 gates |

### 4.2 LiveRewirer Optimization

```rust
// Scoring function
fn score_variant(circuit, noise, hardware) -> f64 {
    let fidelity = estimate_fidelity_with_hardware(circuit, noise, hardware);
    let violations = count_connectivity_violations(circuit, hardware);
    fidelity * (0.9_f64.powi(violations as i32))
}
```

### 4.3 PlacementOptimizer

Qubit placement optimized for hardware topology:

- Random search-based initialization
- Local search improvement
- Fidelity-based evaluation

### 4.4 NoiseAwareRouter

Dijkstra variant algorithm for fidelity-optimal path finding:

```
Cost = α × distance + β × (1 - edge_fidelity)
```

---

## 5. Performance Benchmarks

### 5.1 Test Environment

| Item | Value |
|------|-------|
| **CPU** | AMD Ryzen 9 / Intel i7 equivalent |
| **Memory** | 16GB DDR4 |
| **Rust** | 1.75+ (release build) |
| **Optimization** | `-O3`, LTO enabled |

### 5.2 Performance Results (Simulator)

| Component | Condition | Measured | Notes |
|-----------|-----------|----------|-------|
| Full Pipeline | 5q, 20gates | ~95 μs | Simulator |
| DriftScanner | 5 qubits | ~21 μs | Parameter reference |
| LiveRewirer | 50 variants | ~62 μs | BFS |
| Simulator Execute | 5q, 20gates | ~1.4 μs | StateVector |
| Measure | 5q, 1000shots | ~180 μs | Probability sampling |

### 5.3 Scaling

| Qubits | State Vector Size | Memory | Execute (20g) |
|--------|-------------------|--------|---------------|
| 5 | 32 | 512 B | ~1.4 μs |
| 10 | 1,024 | 16 KB | ~45 μs |
| 15 | 32,768 | 512 KB | ~1.5 ms |
| 20 | 1,048,576 | 16 MB | ~50 ms |
| 25 | 33,554,432 | 512 MB | ~2 s |

---

## 6. Roadmap

### 6.1 v0.1.0 (Current) - Deployment Ready ✅

- ✅ Core types and circuit representation (qns_core)
- ✅ DriftScanner noise profiling (qns_profiler)
- ✅ LiveRewirer/GateReorder algorithms (qns_rewire)
- ✅ PlacementOptimizer/NoiseAwareRouter (qns_rewire)
- ✅ StateVector/Noisy simulator (qns_simulator)
- ✅ CLI pipeline (qns_cli)
- ✅ OpenQASM parser (qns_qasm)
- ✅ Noise channels (qns_noise)
- ✅ Tensor network MPS (qns_tensor)
- ✅ PyO3 Python bindings (qns_python)
- ✅ CI/CD pipeline

### 6.2 v1.0.0 (Next Target) - Hardware Integration

- 📋 HardwareBackend trait implementation
- 📋 IBM Quantum integration (Qiskit Runtime)
- 📋 Real hardware benchmarks
- 📋 Job-to-Job optimization

### 6.3 v2.0.0 (Long-term) - Extension

- 📋 Crosstalk model
- 📋 ZNE (Zero-Noise Extrapolation) integration
- 📋 Multi-backend support (IonQ, Rigetti)
- 📋 Cloud service

---

## Appendix

### A. Technology Stack

| Category | Technology | Reason |
|----------|------------|--------|
| Language | Rust 1.75+ | Memory safety, performance |
| Build | Cargo Workspace | Monorepo multi-crate |
| Math | num-complex, ndarray | Complex numbers, N-dim arrays |
| Parallelism | rayon | Data parallel processing |
| CLI | clap | Command-line interface |
| Serialization | serde, serde_json | Config/result storage |
| Python | PyO3 | Python bindings |

### B. Test Status

| Crate | Unit | Doc | Integration | Total |
|-------|------|-----|-------------|-------|
| qns_core | 46+ | 4+ | - | 50+ |
| qns_profiler | 29+ | 1+ | - | 30+ |
| qns_rewire | 60+ | 3+ | - | 63+ |
| qns_simulator | 39+ | 5+ | - | 44+ |
| qns_cli | 7+ | 2+ | 17+ | 26+ |
| **Total** | **180+** | **15+** | **17+** | **213+** |

### C. License

QNS is provided under the MIT License.

Commercial use, modification, and distribution are freely permitted.

### D. Change History

| Version | Date | Major Changes |
|---------|------|---------------|
| v1.0 | 2025-11-27 | Initial version |
| v2.0 | 2025-11-27 | AI evaluation reflection, expression fixes |
| v2.1 | 2025-12-17 | Implementation status update (all modules complete), MIT license only |

**Major Changes (v2.1):**

- Module status "skeleton" → "✅ Stable" update
- Overall completion 40-50% → 95%
- License MIT/Apache dual → MIT only
- PlacementOptimizer, NoiseAwareRouter documentation added
- Test count update (213+ tests)
- Roadmap updated (v0.1.0 deployment ready)

---

*— End of Document —*

*Copyright © 2025 Jung Wook Yang. Licensed under MIT.*
