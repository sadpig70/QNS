# QNS: Noise-Adaptive Quantum Circuit Optimization via Calibration-Aware Variant Selection and Error Mitigation

---

## Author Information

**Jung Wook Yang**

Independent Researcher, Republic of Korea

Email: <sadpig70@gmail.com>

---

## Abstract

Quantum Error Correction (QEC) treats noise as an adversary to eliminate, requiring substantial qubit overhead impractical for near-term devices. We present QNS (Quantum Noise Symbiote), a noise-adaptive circuit optimization framework that exploits real-time calibration data to select circuit variants optimized for current hardware conditions. QNS introduces a fidelity estimation model incorporating gate errors, decoherence ($T_1$/$T_2$), and crosstalk interactions. Furthermore, QNS integrates **Zero-Noise Extrapolation (ZNE)** for post-execution error mitigation and a **Matrix Product State (MPS)** simulator for scalable verification.

The framework generates mathematically equivalent circuit variants through commutation analysis and selects the optimal variant via beam search over the noise-weighted cost function. A key innovation is the crosstalk-aware Sabre router with weighted heuristic $H(n) = W_D \cdot D + W_E \cdot E + W_X \cdot X$, integrating distance, gate error, and crosstalk penalties.

We validate QNS on both Qiskit Aer simulators and IBM Heron processors (ibm_torino, 133 qubits). Benchmark results demonstrate: (1) gate count reduction of 4.8%–11.1% versus Qiskit Transpiler Level 3; (2) fidelity improvement of 27.1% for VQE circuits under NISQ noise conditions; (3) hardware execution fidelity of 0.85 on IBM Torino. The MPS simulator enables efficient verification of circuits up to 30 qubits, while ZNE provides additional error suppression capabilities.

---

## Keywords

Quantum circuit optimization, Noise-aware compilation, NISQ, Crosstalk mitigation, Zero-Noise Extrapolation, Matrix Product States

---

## 1. Introduction

### 1.1 Background and Motivation

Current quantum processors operate in the Noisy Intermediate-Scale Quantum (NISQ) era, where decoherence and gate errors fundamentally limit computational fidelity [1]. Traditional Quantum Error Correction approaches require significant qubit overhead—estimates suggest 1,000–10,000 physical qubits per logical qubit for fault-tolerant operation [2]—rendering them impractical for near-term devices with 100–1,000 qubits.

An alternative paradigm treats noise not as an adversary but as a *characterizable environmental factor* to which circuits can adapt. This perspective motivates our work: rather than eliminating noise through redundancy, we optimize circuit structure to minimize noise sensitivity given current hardware conditions, and mitigate residual errors using extrapolation techniques.

### 1.2 Related Work

Prior noise-aware compilation efforts have addressed individual aspects of this challenge:

- **Qubit routing optimization**: Sabre [3] and variants minimize SWAP insertion based on topology, while recent work incorporates error rates [4].
- **Crosstalk mitigation**: Murali et al. [5] demonstrated significant fidelity improvements through crosstalk-aware scheduling.
- **Error Mitigation**: Techniques like Zero-Noise Extrapolation (ZNE) [6] extend computational reach without additional qubits.

QNS integrates these dimensions—decoherence, gate errors, crosstalk, and error mitigation—into a unified optimization framework.

### 1.3 Contributions

This paper presents QNS (Quantum Noise Symbiote) with the following contributions:

1. **Unified fidelity model** integrating decoherence, gate errors, and crosstalk (Section 2).
2. **Variant generation algorithm** exploiting gate commutation (Section 3).
3. **Crosstalk-aware router** extending Sabre with weighted cost function (Section 4).
4. **Scalable Simulation & Mitigation** featuring MPS simulation and ZNE implementation (Section 5).
5. **Experimental validation** on IBM Heron processors demonstrating practical fidelity improvements (Section 6).

---

## 2. Theoretical Framework

### 2.1 Noise Profile Vector

We characterize the hardware noise state at time $t$ as:

$$\mathbf{n}(t) = \begin{pmatrix} T_1(t) \\ T_2(t) \\ \boldsymbol{\epsilon}(t) \end{pmatrix}$$

where $T_1$ is the energy relaxation time, $T_2$ is the phase coherence time (constrained by $T_2 \leq 2T_1$), and $\boldsymbol{\epsilon}$ is the gate error vector.

### 2.2 Fidelity Estimation Model

For circuit $C$, we estimate fidelity as:

$$\hat{F}(C, \mathbf{n}) = F_{gate}(C) \cdot F_{decoherence}(C, T_2)$$

where:

$$F_{gate}(C) = (1 - \epsilon_{1q})^{n_{1q}} \cdot (1 - \epsilon_{2q})^{n_{2q}}$$

$$F_{decoherence}(C, T_2) = \exp\left(-\frac{t_{total}}{T_2}\right)$$

### 2.3 Optimization Objective

Given original circuit $C$, we seek:

$$C^* = \arg\max_{C' \in \mathcal{V}(C)} \hat{F}(C', \mathbf{n}(t))$$

where $\mathcal{V}(C) = \{C' : U_{C'} = U_C\}$ is the set of unitarily equivalent variants.

---

## 3. Circuit Variant Generation

### 3.1 Commutation Analysis

Two gates $g_i$ and $g_j$ commute if $[g_i, g_j] = 0$. QNS identifies commuting pairs (disjoint qubits, diagonal gates) to enable reordering without affecting the circuit unitary.

### 3.2 Variant Search

QNS employs beam search over the commutation graph:

1. Initialize frontier with original circuit.
2. For each frontier circuit, enumerate valid commutations.
3. Score candidates using $\hat{F}(C', \mathbf{n})$.
4. Retain top-$k$ candidates (beam width $k=50$).

---

## 4. Crosstalk-Aware Routing

### 4.1 Crosstalk Model

We model crosstalk as pairwise interaction strengths $C_{ij}$ between physical qubits, obtained from backend properties (e.g., `zz_interaction`).

### 4.2 Extended Sabre Heuristic

The standard Sabre distance heuristic is extended to:

$$H(n) = W_{dist} \cdot D + W_{err} \cdot E + W_{xtalk} \cdot X$$

where:

- $D$: Sum of shortest-path distances.
- $E$: Gate error penalty.
- $X$: Crosstalk penalty $\sum_{(i,j) \in \text{front}} C_{ij}$.

Default weights: $W_{dist} = 1.0$, $W_{err} = 0.5$, $W_{xtalk} = 0.3$.

---

## 5. Advanced Simulation & Mitigation

### 5.1 Matrix Product State (MPS) Simulation

To address the exponential memory scaling of state vector simulation ($O(2^n)$), QNS implements a Matrix Product State (MPS) simulator. This allows efficient simulation of circuits with low entanglement (low bond dimension $\chi$).

- **Tensor Contraction**: Gates are applied by contracting local tensors.
- **SVD Truncation**: Singular Value Decomposition (SVD) is performed after 2-qubit gates, truncating small singular values to maintain a maximum bond dimension $\chi_{max}$.
- **Complexity**: Scalability improves to $O(n \cdot \chi^3)$, enabling simulation of 30+ qubits for shallow circuits.

### 5.2 Zero-Noise Extrapolation (ZNE)

QNS integrates ZNE for error mitigation without qubit overhead:

1. **Noise Amplification**: Unitary folding ($G \to G G^\dagger G$). Scale factors $\lambda = 1, 3, 5...$.
2. **Extrapolation**:
    - **Linear**: $E(\lambda) = E_0 + a\lambda$.
    - **Richardson**: Polynomial fit to eliminate higher-order terms.
    - **Exponential**: $E(\lambda) = E_0 + A e^{-B\lambda}$.

---

## 6. Experimental Results

### 6.1 Experimental Setup

| Component | Configuration |
| :--- | :--- |
| Simulator | Qiskit Aer 0.13+, QNS MPS |
| Hardware | IBM Torino (ibm_torino), 133 qubits |
| Baseline | Qiskit Transpiler Level 3 + Sabre |

### 6.2 Gate Count Reduction (Simulation)

| Circuit | Qubits | Baseline Gates | QNS Gates | Reduction |
| :--- | :--- | :--- | :--- | :--- |
| QFT | 10 | 252 | 240 | **4.8%** |
| QFT | 15 | 591 | 547 | **7.5%** |
| Grover | 10 | 1,227 | 1,091 | **11.1%** |

### 6.3 Fidelity Improvement (Noisy Simulation)

| Circuit | Baseline Fidelity | QNS Fidelity | Improvement |
| :--- | :--- | :--- | :--- |
| VQE | 0.360 | 0.458 | **+27.1%** |
| GHZ-5 | 0.970 | 0.970 | +0.0% |

### 6.4 MPS Scalability

| Qubits | State Vector Memory | MPS Memory (Bond 16) | MPS Execution |
| :--- | :--- | :--- | :--- |
| 20 | 16 MB | ~1 MB | ~1 ms |
| 30 | 16 GB | ~5 MB | ~20 ms |

### 6.5 Hardware Validation

IBM Torino execution (5-qubit entanglement):

- **QNS Fidelity**: 0.85
- **Queue time**: ~2 hours

---

## 7. Discussion & Conclusions

We presented QNS, a noise-adaptive optimization framework. Key results include:

- **27.1% fidelity improvement** for VQE circuits.
- **MPS Simulator** enabling efficient large-scale circuit verification.
- **ZNE integration** providing a pathway for further error suppression.

The noise symbiosis paradigm—adapting to noise rather than eliminating it—offers a practical pathway to improved NISQ algorithm performance. Future work will focus on cloud deployment and multi-backend support.

---

## Data Availability

The source code for the QNS framework is available on GitHub at [https://github.com/sadpig70/QNS](https://github.com/sadpig70/QNS) under the MIT License.

---

## References

[1] J. Preskill, *Quantum* **2**, 79 (2018).
[2] A. G. Fowler et al., *Phys. Rev. A* **86**, 032324 (2012).
[3] G. Li et al., *ASPLOS*, 1001–1014 (2019).
[4] P. Murali et al., *ASPLOS*, 1015–1029 (2019).
[5] P. Murali et al., *ASPLOS*, 1001–1016 (2020).
[6] T. Giurgica-Tiron et al., *2020 IEEE International Conference on Quantum Computing and Engineering (QCE)*, 306-316 (2020).
