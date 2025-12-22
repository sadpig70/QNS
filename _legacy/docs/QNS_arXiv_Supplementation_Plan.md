# QNS arXiv Paper Supplementation Plan

**Document Date:** 2025-12-21
**Related Document:** `paper_feasibility_analysis.md`
**Objective:** Address critical gaps identified in the feasibility analysis to meet academic standards for arXiv submission.

## 1. Mathematical Formalization Restoration

**Priority:** Critical (Cannot submit without this)
**Target File:** `docs/QNS_Mathematical_Formalization.md`

### 1.1 Fidelity Estimation Model definitions

Define the core mathematical model used by the `Scoring` module.

- **Objective Function:** Formal definition of $C^* = \arg\max \hat{F}(C', \mathbf{n}(t))$.
- **Noise Profile Vector:** Definition of $\mathbf{n}(t) = (T_1, T_2, \epsilon_{1q}, \epsilon_{2q}, \epsilon_{ro})^T$.
- **Approximation Formula:** Derivation of the heuristic cost function used in Rust code:
  $$ \hat{F} \approx \prod (1-\epsilon_{gates}) \times \exp(-t_{idle}/T_{coherence}) $$

### 1.2 Algorithm Formalization

Pseudocode representation of core algorithms in `qns_rewire`.

- **LiveRewirer:** BFS/Beam Search state transition rules.
- **NoiseAwareRouter:** Edge weight definition $w_{ij} = f(\text{fidelity}_{ij})$ for Dijkstra search.

---

## 2. Benchmark Extension (Scalability Proof)

**Priority:** High (Required for claiming "Scalability")
**Target:** 10+ Qubits

### 2.1 Mid-Scale Simulation

The current benchmarks (2-5 qubits) are insufficient to prove the benefits of heuristic search (Beam Search).

- **New Circuits:**
  - `QFT` (Quantum Fourier Transform): 5, 10, 15 qubits.
  - `Grover`: 5, 10 qubits.
  - `Bernstein-Vazirani`: 10, 20 qubits.
- **Comparison Target:** Explicitly compare against `Qiskit Transpiler (optimization_level=3)`.
- **Metrics:**
  - `Time-to-Solution`: Compilation time vs. Circuit Quality.
  - `Improvement Curve`: Fidelity gain as a function of qubit count.

### 2.2 Execution Plan

1. Create `benchmarks/scalability_benchmark.py`.
2. Configure `AerSimulator` with a synthetic "Linear Topology" (to force routing overhead, highlighting QNS router benefits).
3. Run experiments and generate `scalability_results.csv`.

---

## 3. Real Hardware Validation

**Priority:** High (Strongest proof of "Symbiosis")
**Target:** IBM Quantum (via Qiskit Runtime)

### 3.1 Experiment Design

- **Backend:** Real IBMQ backend (e.g., `ibm_brisbane`, `ibm_kyoto`, or whatever is available).
- **Circuit:** `VQE (H2)` or `QAOA (MaxCut)` (4-5 qubits).
- **Methodology:**
  1. Retrieve *current* calibration data via `CalibrationFetcher`.
  2. Optimize circuit using QNS with this specific data.
  3. Submit job to IBMQ.
  4. Compare result with a job submitted 24 hours later (using old layout) to prove "Drift Adaptation". (Optional, but powerful).

---

## 4. Execution Schedule

| Phase | Task | Estimated Duration | Output |
|-------|------|--------------------|--------|
| **Phase 1** | Restore Math Formalization | 1 Hour | `docs/QNS_Mathematical_Formalization.md` |
| **Phase 2** | Scalability Benchmarks | 2 Hours | `benchmarks/scalability_benchmark.py`, CSV Data |
| **Phase 3** | Hardware Execution | TBD (Queue dependent) | `docs/Real_Hardware_Results.md` |
| **Phase 4** | Paper Drafting | 3 Days | `docs/arXiv_Draft_v1.md` |

## 5. Risk Management

- **Intermittent Errors:** As noted in user reports, the environment may be unstable.
  - *Mitigation:* Ensure all benchmark scripts support **checkpointing** (save progress after each circuit).
  - *Mitigation:* Use `try-except` blocks aggressively in long-running Python scripts.
