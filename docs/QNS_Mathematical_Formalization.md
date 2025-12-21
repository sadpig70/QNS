# QNS Mathematical Formalization

**Version:** 1.0  
**Date:** 2025-12-21  
**Purpose:** Formal mathematical definitions for LiveRewirer optimization

---

## 1. LiveRewirer Optimization Framework

### 1.1 Problem Definition

Given a quantum circuit $C$ with gate sequence $\{g_1, g_2, \ldots, g_n\}$ and a time-varying noise profile $\mathbf{n}(t)$, find the optimal circuit variant that maximizes expected fidelity.

### 1.2 Objective Function

$$
C^* = \arg\max_{C' \in \mathcal{V}(C)} \hat{F}(C', \mathbf{n}(t))
$$

**Where:**

| Symbol | Definition | Domain |
|--------|------------|--------|
| $C$ | Original quantum circuit | Gate sequence |
| $C^*$ | Optimized circuit | Gate sequence |
| $\mathcal{V}(C)$ | Set of mathematically equivalent circuit variants | $\|V\| \geq 1$ |
| $\mathbf{n}(t)$ | Time-dependent noise profile vector | $\mathbb{R}^3$ |
| $\hat{F}$ | Fidelity estimation function | $[0, 1]$ |

### 1.3 Variant Set Definition

The variant set $\mathcal{V}(C)$ is constructed via:

$$
\mathcal{V}(C) = \{ C' : U_{C'} = U_C \}
$$

Where $U_C$ denotes the unitary matrix representation:

$$
U_C = \prod_{i=1}^{n} U_{g_i}
$$

**Transformation Rules:**

- Gate commutation: $[g_i, g_j] = 0 \Rightarrow g_i g_j = g_j g_i$
- Gate decomposition: $U_{CNOT} = (H \otimes I) \cdot CZ \cdot (H \otimes I)$
- Gate synthesis: Multiple single-qubit gates → single $U3$ gate

---

## 2. Noise Profile Vector

### 2.1 Definition

$$
\mathbf{n}(t) = \begin{pmatrix} T_1(t) \\ T_2(t) \\ \boldsymbol{\epsilon}(t) \end{pmatrix}
$$

| Parameter | Description | Typical Range |
|-----------|-------------|---------------|
| $T_1$ | Relaxation time | 50-100 μs |
| $T_2$ | Dephasing time | 20-80 μs |
| $\boldsymbol{\epsilon}$ | Gate error vector | $10^{-4} - 10^{-2}$ |

### 2.2 Gate Error Vector

For a circuit with $m$ distinct gate types:

$$
\boldsymbol{\epsilon} = (\epsilon_{1q}, \epsilon_{2q}, \epsilon_{meas})
$$

Where:

- $\epsilon_{1q}$: Single-qubit gate error rate
- $\epsilon_{2q}$: Two-qubit gate error rate  
- $\epsilon_{meas}$: Measurement error rate

---

## 3. Fidelity Estimation Model

### 3.1 Composite Fidelity Formula

$$
\hat{F}(C, \mathbf{n}) = F_{gate}(C) \cdot F_{decoherence}(C, T_2)
$$

### 3.2 Gate Fidelity Component

$$
F_{gate}(C) = \prod_{g \in C} (1 - \epsilon_g)
$$

For a circuit with $n_{1q}$ single-qubit gates and $n_{2q}$ two-qubit gates:

$$
F_{gate}(C) = (1 - \epsilon_{1q})^{n_{1q}} \cdot (1 - \epsilon_{2q})^{n_{2q}}
$$

### 3.3 Decoherence Fidelity Component

$$
F_{decoherence}(C, T_2) = \exp\left(-\frac{t_{total}}{T_2}\right)
$$

Where total circuit execution time:

$$
t_{total} = \sum_{g \in C} t_g + t_{idle}
$$

### 3.4 Complete Fidelity Model

$$
\boxed{
\hat{F}(C, \mathbf{n}) = (1 - \epsilon_{1q})^{n_{1q}} \cdot (1 - \epsilon_{2q})^{n_{2q}} \cdot \exp\left(-\frac{t_{total}}{T_2}\right)
}
$$

---

## 4. Optimization Algorithm

### 4.1 LiveRewirer Search Strategy

```text
Algorithm: LiveRewirer Optimization
Input: C (circuit), n (noise profile)
Output: C* (optimized circuit)

1. V ← GenerateVariants(C)
2. F_max ← 0
3. C* ← C
4. for each C' in V:
5.     F' ← EstimateFidelity(C', n)
6.     if F' > F_max:
7.         F_max ← F'
8.         C* ← C'
9. return C*
```

### 4.2 Improvement Metric

$$
\Delta F = \frac{\hat{F}(C^*, \mathbf{n}) - \hat{F}(C, \mathbf{n})}{\hat{F}(C, \mathbf{n})} \times 100\%
$$

---

## 5. Validation Criteria

| Criterion | Requirement | Status |
|-----------|-------------|--------|
| Unitarity preservation | $U_{C^*} = U_C$ | ✓ |
| Fidelity improvement | $\hat{F}(C^*) \geq \hat{F}(C)$ | ✓ |
| Bounded search time | $O(\|V\|)$ polynomial | ✓ |
| Noise model consistency | $\mathbf{n}$ measured at runtime | ✓ |

---

## References

1. Nielsen & Chuang, "Quantum Computation and Quantum Information"
2. Preskill, "Quantum Computing in the NISQ era and beyond" (arXiv:1801.00862)
3. Kandala et al., "Hardware-efficient variational quantum eigensolver" (Nature 2017)
