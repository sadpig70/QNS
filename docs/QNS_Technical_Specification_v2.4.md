# QNS - Quantum Noise Symbiote

## Technical Specification Document

**Version 2.4 | December 2025**

**Author:** Jung Wook Yang

> *"We don't fight noise. We dance with it."*

---

## Document Status

| Item | Status |
|------|--------|
| **Current Version** | v2.4.0 (Crosstalk-Aware Routing) |
| **Verification Environment** | Local Simulator + IBM Quantum Aer + **IBM Heron (Real QPU)** |
| **Hardware Integration** | ✅ IBM Aer Simulation + **IBM Torino (133 qubits) Execution** |
| **Benchmark Baseline** | Qiskit Transpiler L3 + Sabre |
| **Overall Completion** | 100% |

> ⚠️ **Important:** Performance metrics include both **simulation** (scalability) and **real hardware** (validation) results.

### Module Implementation Status

| Module | Status | Notes |
|--------|--------|-------|
| qns_core | ✅ Stable | Core types complete (Gate, NoiseVector, CircuitGenome, HardwareProfile) |
| qns_profiler | ✅ Stable | DriftScanner complete |
| qns_rewire | ✅ Stable | LiveRewirer, GateReorder, Router, Scoring fully implemented |
| qns_simulator | ✅ Stable | StateVectorSimulator, NoisySimulator, NoiseModel complete |
| qns_cli | ✅ Stable | **Qiskit backend integrated** (`--backend aer-ideal/aer-noisy/aer-ibm`) |
| qns_qasm | ✅ Stable | OpenQASM parser (basic gates) |
| qns_noise | ✅ Stable | Noise channels |
| qns_tensor | ✅ Stable | MPS implementation |
| qns_python | ✅ Stable | PyO3 bindings + **Qiskit Bridge** |

### 🆕 v2.3 New Features

| Feature | Status | Notes |
|---------|--------|-------|
| **Hardware Execution** | ✅ Verified | IBM Heron (`ibm_torino`) execution success (Fidelity 0.85) |
| **Scalability Bench** | ✅ Verified | QFT/Grover 5-15 qubits vs Qiskit L3 |
| **Math Formalization** | ✅ Complete | Rigorous definition of fidelity estimates and optimization |
| **Noise Model Integration** | ✅ Complete | T1/T2/Gate errors/Readout errors |

### 🆕 v2.4 New Features

| Feature | Status | Notes |
|---------|--------|-------|
| **Crosstalk Model** | ✅ Verified | Heuristic & Backend Property integration |
| **Sabre Upgrade** | ✅ Complete | Weighted cost function ($D + E + X$) |
| **Lookahead Check** | ✅ Complete | Spectator error penalty |

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

QNS (Quantum Noise Symbiote) is a noise-adaptive circuit optimization framework proposing a paradigm shift in quantum computing. While traditional Quantum Error Correction (QEC) approaches treat noise as an 'enemy to be eliminated', QNS **adapts** to noise characteristics to optimize circuits.

**Core Philosophy:** Symbiosis with Noise - Utilizing T1/T2 calibration data from quantum systems to select circuit variants optimized for current noise characteristics.

### 1.2 Key Features

| Feature | Description | Module |
|---------|-------------|--------|
| **DriftScan** | Real-time T1/T2 drift monitoring and anomaly detection | qns_profiler |
| **LiveRewirer** | Dynamic circuit reconstruction based on noise profile | qns_rewire |
| **GateReorder** | Commuting gate reordering optimization | qns_rewire |
| **PlacementOptimizer** | Hardware topology-based qubit placement optimization | qns_rewire |
| **NoiseAwareRouter** | Fidelity-based SWAP routing | qns_rewire |
| **StateVectorSimulator** | Full state vector quantum simulation | qns_simulator |
| **NoisySimulator** | Noise model applied simulation | qns_simulator |
| 🆕 **QiskitBridge** | QNS ↔ Qiskit circuit conversion and Aer simulation | qns_python |
| 🆕 **CalibrationFetcher** | IBM backend calibration data retrieval | qns_python |
| 🆕 **NoiseModelBuilder** | IBM calibration → Qiskit NoiseModel generation | qns_python |

### 1.3 Core Value Propositions

| Value | Description | Target | Status |
|-------|-------------|--------|--------|
| Noise Adaptation | Calibration data-based circuit optimization | Noise profile reflection | ✅ Implemented |
| Local Pipeline | Simulator-based optimization speed | <100ms (5q, 20gates) | ✅ Achieved |
| Hardware Integration | Support for real hardware like IBM Quantum | Qiskit Runtime integration | ✅ **Complete** |
| Fidelity Improvement | Simulator-based quality improvement | 5-15% improvement | ✅ Verified |
| 🆕 Aer Simulation | IBM noise model-based simulation | 156-qubit noise model | ✅ **Complete** |

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

### 2.3 Circuit Rewiring

The same quantum algorithm can be affected differently by noise depending on gate ordering. QNS's LiveRewirer:

- Identifies commuting gate pairs (Commutation Analysis)
- Generates circuit variants using BFS/Beam Search
- Selects the optimal variant for current noise profile
- Reflects hardware connectivity constraints (Coupling Map)
- Fidelity-based SWAP routing (NoiseAwareRouter)

### 2.4 Fidelity Estimation Model

#### 2.4.1 Optimization Objective Function

$$
C^* = \arg\max_{C' \in \mathcal{V}(C)} \hat{F}(C', \mathbf{n}(t))
$$

| Symbol | Definition | Domain |
|--------|------------|--------|
| $C$ | Original quantum circuit | Gate sequence |
| $C^*$ | Optimized circuit | Gate sequence |
| $\mathcal{V}(C)$ | Set of mathematically equivalent circuit variants | $\|V\| \geq 1$ |
| $\mathbf{n}(t)$ | Time-dependent noise profile vector | $\mathbb{R}^3$ |
| $\hat{F}$ | Fidelity estimation function | $[0, 1]$ |

#### 2.4.2 Variant Set Definition

$$
\mathcal{V}(C) = \{ C' : U_{C'} = U_C \}
$$

Where $U_C$ is the unitary matrix representation:

$$
U_C = \prod_{i=1}^{n} U_{g_i}
$$

**Transformation Rules:**

- Gate commutation: $[g_i, g_j] = 0 \Rightarrow g_i g_j = g_j g_i$
- Gate decomposition: $U_{CNOT} = (H \otimes I) \cdot CZ \cdot (H \otimes I)$
- Gate synthesis: Multiple single-qubit gates → single $U3$ gate

#### 2.4.3 Noise Profile Vector

$$
\mathbf{n}(t) = \begin{pmatrix} T_1(t) \\ T_2(t) \\ \boldsymbol{\epsilon}(t) \end{pmatrix}
$$

| Parameter | Description | Typical Range |
|-----------|-------------|---------------|
| $T_1$ | Relaxation time | 50-100 μs |
| $T_2$ | Dephasing time | 20-80 μs |
| $\boldsymbol{\epsilon}$ | Gate error vector | $10^{-4} - 10^{-2}$ |

#### 2.4.4 Complete Fidelity Model

$$
\boxed{
\hat{F}(C, \mathbf{n}) = (1 - \epsilon_{1q})^{n_{1q}} \cdot (1 - \epsilon_{2q})^{n_{2q}} \cdot \exp\left(-\frac{t_{total}}{T_2}\right)
}
$$

**Components:**

1. **Gate Fidelity**: $F_{gate}(C) = (1 - \epsilon_{1q})^{n_{1q}} \cdot (1 - \epsilon_{2q})^{n_{2q}}$
2. **Decoherence Fidelity**: $F_{decoherence}(C, T_2) = \exp\left(-\frac{t_{total}}{T_2}\right)$

Where:

- $\epsilon_{1q}$: Single-qubit gate error rate
- $\epsilon_{2q}$: Two-qubit gate error rate
- $n_{1q}$: Number of single-qubit gates
- $n_{2q}$: Number of two-qubit gates
- $t_{total} = \sum_{g \in C} t_g + t_{idle}$: Total circuit execution time

> **Note:** The above formula represents a critical-path approximation. QNS implementation employs **Idle Time Tracking** for per-qubit coherence modeling.
>
> 📘 **Detailed Mathematical Formalization:** See [QNS_Mathematical_Formalization.md](QNS_Mathematical_Formalization.md) for the complete derivation including the idle time survival probability $S_q(t_{idle})$.

---

## 3. System Architecture

### 3.1 Module Structure

```
qns/
├── crates/
│   ├── qns_core        # Core types: Gate, NoiseVector, CircuitGenome, HardwareProfile
│   ├── qns_profiler    # Noise profiling: DriftScanner
│   ├── qns_rewire      # Circuit optimization: GateReorder, LiveRewirer, Router, Scoring
│   ├── qns_simulator   # Quantum simulation: StateVectorSimulator, NoisySimulator
│   ├── qns_cli         # CLI and integration: Pipeline, QnsSystem, QiskitRunner
│   ├── qns_qasm        # OpenQASM parser: Parser, AST, Builder
│   ├── qns_noise       # Noise channels: NoiseChannel, NoiseModel
│   ├── qns_tensor      # Tensor networks: TensorNetwork, MPS
│   └── qns_python/     # Python bindings + Qiskit Bridge
│       ├── src/lib.rs      # PyO3 bindings
│       └── python/         # 🆕 Qiskit integration
│           ├── qiskit_bridge.py   # CircuitConverter, NoiseModelBuilder
│           └── cli_runner.py      # CLI backend runner
├── docs/               # Documentation
├── scripts/            # Benchmark/analysis scripts
├── benchmarks/         # Benchmark circuits + validation scripts
└── .github/            # CI/CD
```

### 3.2 Current Architecture (v2.3)

```
┌─────────────────────────────────────────────────────────────────────┐
│                    QNS Architecture v2.3                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐             │
│  │  qns_core   │    │qns_profiler │    │ qns_rewire  │             │
│  │ [✅ Complete]│    │[✅ Complete]│    │[✅ Complete]│             │
│  └─────────────┘    └─────────────┘    └─────────────┘             │
│         │                  │                  │                    │
│         └──────────────────┴──────────────────┘                    │
│                           │                                        │
│  ┌─────────────┐    ┌─────┴─────┐    ┌─────────────┐              │
│  │  qns_qasm   │    │qns_simulator│   │ qns_tensor  │              │
│  │[✅ Complete]│    │[✅ Complete]│   │[✅ Complete]│              │
│  └─────────────┘    └───────────┘    └─────────────┘              │
│         │                  │                  │                    │
│         └──────────────────┴──────────────────┘                    │
│                           │                                        │
│              ┌────────────┼────────────┐                           │
│              │            │            │                           │
│         ┌────┴────┐ ┌─────┴─────┐ ┌────┴────┐                     │
│         │qns_cli  │ │qns_python │ │qns_noise│                     │
│         │[✅ Done]│ │ [✅ Done] │ │[✅ Done]│                      │
│         └─────────┘ └─────┬─────┘ └─────────┘                     │
│                           │                                        │
│              ┌────────────┴────────────┐                           │
│              │   🆕 Qiskit Bridge      │                           │
│              │     [✅ Complete]       │                           │
│              ├─────────────────────────┤                           │
│              │ • CircuitConverter      │                           │
│              │ • CalibrationFetcher    │                           │
│              │ • NoiseModelBuilder     │                           │
│              │ • AerSimulationRunner   │                           │
│              └────────────┬────────────┘                           │
│                           │                                        │
│              ┌────────────┴────────────┐                           │
│              │    IBM Quantum          │                           │
│              │  ibm_torino (133q)      │                           │
│              │  [✅ Verified]          │                           │
│              └─────────────────────────┘                           │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.3 Data Flow (v2.3)

```
Circuit Input → DriftScanner → NoiseVector → LiveRewirer → Optimized Circuit
                                    ↓
                            [Hardware Topology]
                                    ↓
                     PlacementOptimizer + NoiseAwareRouter
                                    ↓
                    ┌───────────────┴───────────────┐
                    │                               │
             ┌──────┴──────┐              ┌─────────┴─────────┐
             │ QNS Native  │              │   Qiskit Aer      │
             │ Simulator   │              │  (Noisy/IBM)      │
             └─────────────┘              └───────────────────┘
                    │                               │
                    └───────────────┬───────────────┘
                                    ↓
                            Execution Result
```

---

## 4. Qiskit Integration

### 4.1 Integration Overview

QNS v2.3 is fully integrated with the IBM Qiskit ecosystem, supporting real IBM Quantum hardware simulation.

**Integration Strategy:** Simulation-First Validation → Hardware Execution

### 4.2 Qiskit Bridge Architecture

```python
# Core Classes (qiskit_bridge.py)

class CircuitConverter:
    """QNS CircuitGenome ↔ Qiskit QuantumCircuit conversion"""
    # Supported gates: H, X, Y, Z, S, T, RX, RY, RZ, CNOT, CZ, SWAP (12 types)

class CalibrationFetcher:
    """IBM backend calibration data retrieval"""
    # Integration verified: ibm_fez (156 qubits)
    # Extracted data: T1, T2, gate_errors_1q, gate_errors_2q, readout_errors

class NoiseModelBuilder:
    """Calibration data → Qiskit NoiseModel generation"""
    # Applied errors: Thermal relaxation, Depolarizing, Readout
    # T2 constraint validation: Automatic clamping T2 ≤ 2*T1

class AerSimulationRunner:
    """Qiskit Aer simulation execution and result analysis"""
    # Fidelity calculation: Comparison of measurement results to theoretical expectations
```

### 4.3 CLI Backend Options

```bash
# QNS native simulator (default)
qns run circuit.qasm --backend simulator

# Qiskit Aer ideal simulation
qns run circuit.qasm --backend aer-ideal --shots 1024

# Qiskit Aer noisy simulation (mock calibration)
qns run circuit.qasm --backend aer-noisy --shots 2048

# Qiskit Aer + IBM backend calibration
qns run circuit.qasm --backend aer-ibm --ibm-backend ibm_fez --shots 1024
```

### 4.4 IBM Quantum Integration Results

| Backend | Qubits | T1 Average | T2 Average | 1Q Error | Readout |
|---------|--------|------------|------------|----------|---------|
| ibm_fez | 156 | 145 μs | 105 μs | 0.68% | 1.97% |

**Verification Results:**

- ✅ Calibration data retrieval successful
- ✅ NoiseModel creation successful (156-qubit)
- ✅ Noisy simulation executed: Fidelity 0.493 (vs ideal 0.501)
- 🆕 **Hardware Execution**: `ibm_torino` execution successful (Bell Fidelity 0.85)

### 4.5 PyO3 Qiskit Bridge Functions

```rust
// lib.rs exports

#[pyfunction]
fn convert_circuit_to_qiskit(circuit: &PyCircuit) 
    -> PyResult<Vec<HashMap<String, Py<PyAny>>>>;

#[pyfunction]  
fn run_aer_simulation(py: Python, circuit: &PyCircuit, shots: usize) 
    -> PyResult<HashMap<String, usize>>;

#[pyfunction]
fn fetch_ibm_calibration(py: Python, backend_name: &str) 
    -> PyResult<HashMap<String, Py<PyAny>>>;
```

---

## 5. Algorithm Details

### 5.1 GateReorder Algorithm

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

**Beam Search (for large circuits):**

| Algorithm | Time Complexity | Space Complexity | Suitable Circuits |
|-----------|-----------------|------------------|-------------------|
| BFS | O(V × E) | O(V) | <50 gates |
| Beam Search | O(k × n × b) | O(b) | <500 gates |

### 5.2 LiveRewirer Optimization

```rust
// Scoring function
fn score_variant(circuit, noise, hardware) -> f64 {
    let fidelity = estimate_fidelity_with_hardware(circuit, noise, hardware);
    let violations = count_connectivity_violations(circuit, hardware);
    fidelity * (0.9_f64.powi(violations as i32))
}
```

### 5.3 PlacementOptimizer

Hardware topology-optimized qubit placement:

- Random search-based initialization
- Local search improvement
- Fidelity-based evaluation

### 5.4 NoiseAwareRouter

Dijkstra variant algorithm for fidelity-optimal path search:

```
Cost = α × distance + β × (1 - edge_fidelity)
```

---

## 6. Performance Benchmarks

### 6.1 Measurement Environment

| Item | Value |
|------|-------|
| **CPU** | AMD Ryzen 9 / Intel i7 equivalent |
| **Memory** | 16GB DDR4 |
| **Rust** | 1.75+ (release build) |
| **Python** | 3.11+ (Qiskit 1.0+) |
| **Optimization** | `-O3`, LTO enabled |

### 6.2 QNS Native Performance (Simulator Baseline)

| Component | Conditions | Measurement | Notes |
|-----------|------------|-------------|-------|
| Full Pipeline | 5q, 20gates | ~95 μs | Simulator |
| DriftScanner | 5 qubits | ~21 μs | Parameter reference |
| LiveRewirer | 50 variants | ~62 μs | BFS |
| Simulator Execute | 5q, 20gates | ~1.4 μs | StateVector |
| Measure | 5q, 1000shots | ~180 μs | Probability sampling |

### 6.3 🆕 Qiskit Aer Performance

| Simulation Type | Conditions | Measurement | Notes |
|-----------------|------------|-------------|-------|
| Aer Ideal | 2q, Bell state, 1024 shots | ~50 ms | No noise |
| Aer Noisy | 2q, Bell state, 1024 shots | ~100 ms | mock calibration |
| Aer IBM | 2q, Bell state, 1024 shots | ~150 ms | ibm_fez calibration |

### 6.4 🆕 arXiv Benchmark Results

#### 6.4.1 Scalability: QNS vs Qiskit L3 (Gate Count)

| Circuit | Qubits | Baseline Gates | QNS Gates | Reduction | Time (ms) |
|---------|:------:|:--------------:|:---------:|:---------:|:---------:|
| **QFT** | 10 | 252 | 240 | **4.8%** | 9.7 (QNS) vs 101 |
| **QFT** | 15 | 591 | 547 | **7.5%** | 109 (QNS) vs 134 |
| **Grover**| 10 | 1227 | 1091 | **11.1%**| 27 (QNS) vs 219 |

#### 6.4.2 Ideal Environment (Noiseless)

| Circuit | Baseline | QNS | Improvement |
|---------|----------|-----|-------------|
| Bell | 1.0000 | 1.0000 | +0.0% |
| GHZ-5 | 1.0000 | 0.9700 | -3.0% |
| **VQE** | 0.4000 | **0.4576** | **+14.4%** |

#### 6.4.3 NISQ Environment (Noisy) ⭐

| Circuit | Baseline | QNS | Improvement |
|---------|----------|-----|-------------|
| Bell | 1.0000 | 1.0000 | +0.0% |
| GHZ-5 | 0.9700 | 0.9700 | +0.0% |
| **VQE** | 0.3600 | **0.4576** | **+27.1%** ✅ |

> 📊 Detailed results: See [QNS_Benchmark_Results.md](QNS_Benchmark_Results.md)
>
> 📘 Mathematical formalization: See [QNS_Mathematical_Formalization.md](QNS_Mathematical_Formalization.md)

### 6.5 Scaling

| Qubits | State Vector Size | Memory | Execute (20g) |
|--------|-------------------|--------|---------------|
| 5 | 32 | 512 B | ~1.4 μs |
| 10 | 1,024 | 16 KB | ~45 μs |
| 15 | 32,768 | 512 KB | ~1.5 ms |
| 20 | 1,048,576 | 16 MB | ~50 ms |
| 25 | 33,554,432 | 512 MB | ~2 s |

---

## 5. Crosstalk Model v2.4

QNS v2.4 integrates a sophisticated crosstalk-aware routing mechanism.

### 5.1 Hardware Profile Expansion

- **CrosstalkMatrix**: Stores interaction strengths $C_{ij}$ between physical qubits $i$ and $j$.
- **Data Source**: Fetched from Qiskit backend properties (e.g. `zz_interaction`) or heuristically derived from topology.

### 5.2 Sabre Router Upgrade

The standard Sabre heuristic is expanded to a weighted cost function:
$$ H(n) = W_{dist} \cdot D + W_{err} \cdot E + W_{xtalk} \cdot X $$

- $D$: Sum of distances (classic Sabre).
- $E$: Gate error penalty for chosen edges.
- $X$: Crosstalk penalty for interactions between active edges in the front layer.

### 5.3 CLI Integration

- New Argument: `--crosstalk-weight <FLOAT>`
- Behavior:
  - If $> 0.0$, enables `SabreRouter` and sets the crosstalk weight.
  - Typical range: $0.1 - 1.0$.

---

## 7. Roadmap

### 7.1 v0.1.0 - Release Ready ✅

- ✅ Core types and circuit representation (qns_core)
- ✅ DriftScanner noise profiling (qns_profiler)
- ✅ LiveRewirer/GateReorder algorithms (qns_rewire)
- ✅ PlacementOptimizer/NoiseAwareRouter (qns_rewire)
- ✅ StateVector/Noisy simulators (qns_simulator)
- ✅ CLI pipeline (qns_cli)
- ✅ OpenQASM parser (qns_qasm)
- ✅ Noise channels (qns_noise)
- ✅ Tensor network MPS (qns_tensor)
- ✅ PyO3 Python bindings (qns_python)
- ✅ CI/CD pipeline

### 7.2 v0.2.0 (Current) - Qiskit Integration Complete ✅

- ✅ Qiskit Bridge (CircuitConverter, NoiseModelBuilder)
- ✅ IBM Calibration Fetcher (ibm_fez 156q verified)
- ✅ Aer Simulation Runner (ideal, noisy)
- ✅ CLI Backend Selection (--backend aer-ideal/aer-noisy/aer-ibm)
- ✅ PyO3 Qiskit Functions (3 exported functions)
- ✅ Clean build state (193 tests, 0 warnings)

### 7.3 v1.0.0 (Latest) - Hardware Verification ✅

- ✅ IBM Runtime real QPU job submission (`ibm_torino` verified)
- ✅ Queue monitoring and result retrieval
- ✅ QNS vs. Qiskit Transpiler statistical comparison (Scalability)
- ✅ 5+ circuit benchmarks (Bell, GHZ, QFT, VQE, etc.)

### 7.4 v2.0.0 (Long-term) - Extension

- ✅ Crosstalk model
- 📋 ZNE (Zero-Noise Extrapolation) integration
- 📋 Multi-backend support (IonQ, Rigetti)
- 📋 Cloud service

---

## Appendix

### A. Technology Stack

| Category | Technology | Selection Rationale |
|----------|------------|---------------------|
| Language | Rust 1.75+ | Memory safety, performance |
| Build | Cargo Workspace | Monorepo multi-crate |
| Math | num-complex, ndarray | Complex numbers, N-dimensional arrays |
| Parallelization | rayon | Data parallel processing |
| CLI | clap | Command line interface |
| Serialization | serde, serde_json | Config/result storage |
| Python | PyO3 | Python bindings |
| 🆕 Qiskit | qiskit 1.0+, qiskit-aer 0.13+ | IBM Quantum integration |
| 🆕 IBM Runtime | qiskit-ibm-runtime 0.17+ | Calibration retrieval |

### B. Test Coverage

| Crate | Unit | Doc | Integration | Total |
|-------|------|-----|-------------|-------|
| qns_core | 46+ | 4+ | - | 50+ |
| qns_profiler | 29+ | 1+ | - | 30+ |
| qns_rewire | 60+ | 3+ | - | 63+ |
| qns_simulator | 39+ | 5+ | - | 44+ |
| qns_cli | 7+ | 2+ | 17+ | 26+ |
| qns_python (Qiskit) | 9+ | - | 3+ | 12+ |
| **Total** | **190+** | **15+** | **20+** | **225+** |

### C. Qiskit Dependencies

```
# requirements.txt (crates/qns_python/)
qiskit>=1.0.0
qiskit-aer>=0.13.0
qiskit-ibm-runtime>=0.17.0
numpy>=1.24.0
scipy>=1.10.0
pytest>=7.0.0
python-dotenv>=1.0.0
```

### D. License

QNS is provided under the MIT License.

Commercial use, modification, and distribution are permitted.

### E. Change History

| Version | Date | Major Changes |
|---------|------|---------------|
| v1.0 | 2025-11-27 | Initial version |
| v2.0 | 2025-11-27 | AI evaluation reflected, expression corrections |
| v2.1 | 2025-12-17 | Implementation status update (all modules complete), MIT license unification |
| v2.2 | 2025-12-20 | Qiskit integration complete (Sprint 1-4) |
| **v2.3** | **2025-12-21** | **Mathematical formalization integration, Scalability benchmarks, Hardware validation** |
| **v2.4** | **2025-12-22** | **Crosstalk model integration, Sabre router upgrade, CLI integration** |

**Major Changes (v2.3):**

- ✅ **Hardware Execution Verified**: IBM Heron (`ibm_torino`) execution success (Fidelity 0.85).
- 📊 **Scalability Benchmarks**: QFT/Grover demonstrated up to 11% gate reduction vs Qiskit L3.
- 📘 **Mathematical Formalization**: Expanded fidelity models and optimization definitions.

**Major Changes (v2.4):**

- 🆕 **Crosstalk-Aware Routing**: Integrated crosstalk penalty into Sabre router.
- 🆕 **LiveRewirer Upgrade**: Support for weighted cost functions.
- 🆕 **CLI Integration**: Added `--crosstalk-weight` flag.

**Major Changes (v2.2):**

- 🆕 Qiskit Bridge added (CircuitConverter, NoiseModelBuilder, AerSimulationRunner)
- 🆕 IBM Calibration Fetcher completed (ibm_fez 156 qubits verified)
- 🆕 CLI Backend Selection (simulator, aer-ideal, aer-noisy, aer-ibm)
- 🆕 3 PyO3 Qiskit functions exported
- 🆕 T2 ≤ 2*T1 physical constraint automatic validation/clamping
- Clean build state achieved (0 warnings)
- Test count updated (225+ tests)
- Architecture diagram updated to v2.2

---

*— End of Document —*

*Copyright © 2025 Jung Wook Yang. Licensed under MIT.*
