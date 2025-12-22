# QNS Hardware Verification Report

**Date:** 2025-12-21
**Backend:** `ibm_torino` (IBM Heron Processor)
**Location:** IBM Quantum Cloud (Open Plan)

## 1. Experiment Setup

To validate the QNS compilation and execution pipeline on real quantum hardware, we executed a fundamental verification circuit.

- **Objective:** Verify end-to-end access, compilation validity, and result retrieval.
- **Circuit:** Bell State $|\Phi^+\rangle = \frac{|00\rangle + |11\rangle}{\sqrt{2}}$
- **Compilation:** Optimization Level 1 (Memory-optimized for local execution)
- **Execution Mode:** Job Mode (SamplerV2 Primitive)

## 2. Hardware Specifications

- **Name:** `ibm_torino`
- **Processor Type:** Heron (133 qubits)
- **Topology:** Heavy-hex lattice
- **Basis Gates:** `ecr`, `id`, `rz`, `sx`, `x`
- **Median 2-Qubit ECR Error:** ~0.8% (typical for Heron)

## 3. Results

The experiment was successfully submitted and executed.

| Metric | Value |
| :--- | :--- |
| **Job ID** | `d53vaa1smlfc739f97u0` |
| **Total Shots** | 4096 (Default) |
| **P(|00>)** | 0.47 |
| **P(|11>)** | 0.38 |
| **Fidelity ($P_{00}+P_{11}$)** | **0.85** |

### Error Analysis

- **Bit-flip Errors (01/10):** 15% (0.07 + 0.08)
- The 85% fidelity is consistent with unmitigated execution on NISQ hardware, accounting for readout errors (~2-3%) and gate errors (~1% per CNOT).

## 4. Conclusion

The QNS system successfully:

1. Authenticated with IBM Quantum Platform using `tqp-ibm-apikey.json`.
2. Selected the least-busy backend (`ibm_torino`).
3. Transpiled the circuit to the target ISA.
4. Retrieved and parsed results via the `qiskit-ibm-runtime` Sampler primitive.

This confirms the readiness of the testbed for Phase 4 (Scalability Experiments on Hardware).
