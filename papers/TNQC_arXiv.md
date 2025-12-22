# QNS: A Noise-Adaptive Quantum Circuit Optimization Framework for NISQ Era

**Authors:** Jung Wook Yang
**Date:** December 2025
**Repository:** <https://github.com/sadpig70/QNS>

---

## Abstract

We present **QNS (Quantum Noise Symbiote)**, a noise-adaptive circuit optimization framework designed to bridge the gap between ideal quantum algorithms and the noisy reality of Near-Term Intermediate Scale Quantum (NISQ) devices. Unlike traditional compilation strategies that treat noise as a static constraint, QNS leverages real-time calibration data (T1, T2, gate errors) to dynamicially restructure quantum circuits. By exploiting the commutativity of quantum gates and employing a fidelity-based scoring model, QNS generates circuit variants optimized for the specific noise profile of the target hardware. We demonstrate QNS's efficacy through **scalability benchmarks** against Qiskit's standard transpiler (Level 3), achieving up to **11.08% gate count reduction** for 10-qubit Grover circuits. Furthermore, we report successful hardware validation on the **IBM Heron (133-qubit)** processor (Fidelity 0.85). Finally, we introduce **Crosstalk-Aware Routing** which yields over **700% fidelity improvement** in high-interference simulation scenarios, demonstrating QNS's capability to navigate complex noise landscapes.

---

## 1. Introduction

The promise of quantum advantage is currently hindered by the limitations of NISQ (Noisy Intermediate-Scale Quantum) devices. Qubits are susceptible to decoherence (T1, T2 relaxation) and gate operations suffer from significant error rates. While Quantum Error Correction (QEC) offers a long-term solution, the qubit overhead remains prohibitive for current hardware.

Current circuit compilation tools, such as Qiskit's transpiler or TKET, perform optimization based on static hardware topology and average error rates. However, they often fail to account for the temporal fluctuations in device noise (drift) and the specific interaction between gate scheduling and decoherence.

In this paper, we introduce **QNS (Quantum Noise Symbiote)**, a framework that adopts a "symbiotic" approach to noise. Instead of merely minimizing gate count, QNS optimizes the *expected fidelity* of a circuit by prioritizing gate sequences that minimize exposure to the most dominant noise channels at runtime.

QNS serves as the **Adaptive Layer** within the broader **TNQC (Temporal Noise Quantum Computing)** ecosystem [Ref]. TNQC redefines noise and time not as adversaries but as computational resources. Within this architecture, QNS functions alongside other pillars like NISO (Optimization), PROPHET (Immunity), and TQP (Spatiotemporal Core) to provide real-time noise adaptation, or "Error Navigation," specifically determining the optimal circuit configuration for fluctuating hardware conditions.

### 1.1 Key Contributions

1. **Noise-Adaptive Compilation:** A dynamic optimization pipeline that ingests real-time backend calibration data.
2. **Mathematical Fidelity Model:** A rigorous scoring model incorporating idle-time decoherence and hardware-specific gate errors.
3. **Scalability Verification:** Empirical demonstration of gate reduction advantages over industry-standard compilers for circuits up to 15 qubits.
4. **Hardware Validation:** End-to-end execution and verification on IBM Quantum's 133-qubit Heron processor (`ibm_torino`).

---

## 2. Methodology

### 2.1 Mathematical Formalization

The core optimization problem in QNS is defined as finding the circuit variant $C^*$ from a set of mathematically equivalent circuits $\mathcal{V}(C)$ that maximizes the estimated fidelity $\hat{F}$ given a time-dependent noise profile $\mathbf{n}(t)$.

$$
C^* = \arg\max_{C' \in \mathcal{V}(C)} \hat{F}(C', \mathbf{n}(t))
$$

Where the set of variants $\mathcal{V}(C)$ is generated via commutation relations ($[A, B] = 0 \implies AB = BA$) and hardware-aware transformations.

The noise profile $\mathbf{n}(t)$ is characterized by:
$$
\mathbf{n}(t) = \begin{pmatrix} T_1(t) \\ T_2(t) \\ \boldsymbol{\epsilon}_{gate}(t) \end{pmatrix}
$$

### 2.2 Fidelity Estimation Model

We employ a composite fidelity model that accounts for both coherent control errors and incoherent decoherence:

$$
\hat{F}(C, \mathbf{n}) = F_{gate}(C) \cdot F_{decoherence}(C, T_2)
$$

1. **Gate Fidelity:** Aggregated error probabilities from single ($n_{1q}$) and two-qubit ($n_{2q}$) gates.
    $$ F_{gate} \approx (1 - \epsilon_{1q})^{n_{1q}} \cdot (1 - \epsilon_{2q})^{n_{2q}} $$
2. **Decoherence Fidelity:** Decay probability as a function of total circuit duration (critical path) and the effective dephasing time $T_2$.
    $$ F_{decoherence} \approx \exp\left(-\frac{t_{total}}{T_2}\right) $$

Crucially, QNS enforces the physical constraint $T_2 \le 2T_1$ during model calibration to prevent unphysical optimization targets from anomalous calibration data.

### 2.3 Crosstalk-Aware Routing

To mitigate coherent errors arising from simultaneous gate operations, QNS extends the SABRE heuristic with a crosstalk penalty term:

$$
H(n) = W_{dist} \cdot D + W_{err} \cdot E + W_{xtalk} \cdot X
$$

Where $X$ represents the cumulative crosstalk strength of all active edges in the front layer. $C_{ij}$ values are derived from backend property reports (e.g., `zz_interaction`) or heuristically inferred from topology. This lookahead mechanism allows the router to sacrifice distance (SWAP count) to avoid high-interference parallel operations.

---

## 3. System Architecture

QNS is implemented in **Rust** for performance and safety, with **Python** bindings for seamless integration with the Qiskit ecosystem.

* **DriftScanner:** Monitors backend properties to detect significant noise drift.
* **LiveRewirer:** The core engine that explores the search space of equivalent circuits using Breadth-First Search (BFS) for small circuits and Beam Search for larger instances.
* **Qiskit Bridge:** A bidirectional interface facilitating the retrieval of IBM calibration data and the submission of optimized circuits to IBM Runtime.

---

## 4. Experiments and Results

### 4.1 Scalability Benchmarks

We compared QNS against Qiskit's `Transpiler` (optimization_level=3) using Quantum Fourier Transform (QFT) and Grover's Algorithm circuits.

**Table 1: Scalability Benchmark Results (Gate Count)**

| Circuit | Qubits | Baseline (Qiskit L3) | QNS Optimized | Reduction |
|:-------:|:------:|:--------------------:|:-------------:|:---------:|
| **QFT** | 10     | 252                  | 240           | **4.76%** |
| **QFT** | 15     | 591                  | 547           | **7.45%** |
| **Grover**| 10   | 1227                 | 1091          | **11.08%**|

*Note: For Grover's algorithm (10 qubits), QNS not only reduced gate count by over 11% but also completed compilation significantly faster (27ms vs 219ms), highlighting the efficiency of the Rust-based implementation.*

### 4.2 Hardware Validation: IBM Heron

To validate the framework on physical hardware, we executed experiments on the **IBM Torino** backend (Heron processor, 133 qubits).

* **Target:** Bell State Preparation $\frac{|00\rangle + |11\rangle}{\sqrt{2}}$
* **Backend:** `ibm_torino`
* **Protocol:**
    1. Fetch real-time calibration for `ibm_torino`.
    2. Optimize Bell circuit using QNS LiveRewirer.
    3. Execute via `qiskit-ibm-runtime` (Job ID: `cva...`).
    4. Compute Fidelity vs. Ideal State.

**Results:**

* **Measured Fidelity:** **0.85**
* **Execution Status:** Success (Job completed in <5s queue time)

This successful execution confirms that the QNS pipeline—from calibration fetching to Rust-based optimization and final hardware dispatch—is fully operational and compatible with the latest generation of IBM quantum processors.

### 4.3 Crosstalk Resilience

We evaluated the efficacy of the crosstalk-aware routing on a mock backend with strong ZZ-interaction coupling ($C_{ij}=0.1$) using a 5-qubit GHZ state.

**Table 2: GHZ-5 Fidelity vs. Crosstalk Weight**

| Weight ($W_{xtalk}$) | Estimated Fidelity | Improvement |
|:--------------------:|:------------------:|:-----------:|
| 0.0 (Baseline)       | 0.1094             | -           |
| 0.25 (Optimized)     | 0.8816             | **+705.8%** |

The results demonstrate that introducing a non-zero crosstalk penalty effectively steers the router away from high-interference parallel operations, recovering usable fidelity from a failing circuit.

---

## 5. Conclusion

QNS demonstrates that adapting circuit compilation to real-time noise characteristics is a viable strategy for improving performance in the NISQ era. Our results show that this "symbiotic" approach yields measurable gains in circuit compactness (up to 11% reduction), robust hardware execution (Fidelity 0.85), and significant resilience against crosstalk (700% gain).

As a core pillar of the **TNQC** paradigm, QNS contributes to evolving fragile quantum processors into "Antifragile" systems that gain reliability through active noise navigation. Future work will focus on deeper integration with the PROPHET immunity platform for predictive noise resilience and expanding support for ion-trap architectures.

---

## References

1. Qiskit Contributors, "Qiskit: An Open-source Framework for Quantum Computing," 2023.
2. IBM Quantum, "IBM Quantum Heron Processor Specification," 2024.
3. Cross, A. W., et al., "OpenQASM 3: A broader and deeper quantum assembly language," ACM Transactions on Quantum Computing, 2022.
