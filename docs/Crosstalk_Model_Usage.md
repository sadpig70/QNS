# Crosstalk Model Usage Guide

## Overview

The QNS v2.4 Crosstalk Symbiosis Model allows users to simulate and mitigate crosstalk errors in quantum circuits. This guide explains how to configure and use these features via the Python SDK.

## 1. Configuring Crosstalk in Hardware Profiles

You can define crosstalk interactions in the `HardwareProfile`. The `crosstalk_weight` parameter controls how aggressively the router avoids simultaneous operations on interacting qubits.

### Python Example

```python
import qns

# 1. Create or load a hardware profile
# For simulation, you can create a custom profile or load from a backend
profile = qns.HardwareProfile.heavy_hex("ibm_heron", 133)

# 2. Define crosstalk interactions
# Interaction between Qubit 0 and Qubit 1 with 5% error rate
# This means if Q0 is active, Q1 (if idle) suffers Z-error, and vice versa.
profile.crosstalk.set_interaction(0, 1, 0.05)
```

## 2. Optimizing with Crosstalk Awareness

Use the `QnsOptimizer` to compile circuits while accounting for crosstalk.

```python
# Initialize Optimizer with crosstalk_weight
# weight > 0.0 enables crosstalk-aware routing
# weight = 0.5 is a good default
optimizer = qns.QnsOptimizer(
    num_qubits=20,
    crosstalk_weight=0.5
)

# Set the hardware profile containing crosstalk data
optimizer.set_hardware(profile)

# Optimize your circuit
circuit = qns.Circuit(5)
# ... add gates ...
result = optimizer.optimize(circuit)

print(f"Optimized Fidelity: {result.optimized_fidelity}")
```

## 3. Tuning the Crosstalk Weight

- **`crosstalk_weight = 0.0`**: Ignored. Router focuses solely on SWAP count and gate errors.
- **`crosstalk_weight = 0.1 - 0.5`**: Balanced. Penalizes high-crosstalk parallelism but permits it if the alternative is too many SWAPs.
- **`crosstalk_weight > 1.0`**: Aggressive. Router will serialize operations or take long detours to avoid ALL crosstalk. Use with caution on large circuits.

## 4. CLI Usage (v2.4)

You can mostly access crosstalk features directly from the CLI without writing Python code.

```bash
# Standard run (Crosstalk ignored, defaults to 0.0)
qns run circuit.qasm

# Enable Crosstalk-Aware Routing
# This activates SabreRouter with weighted cost function (Dist + Error + Crosstalk)
qns run circuit.qasm --crosstalk-weight 0.5
```

## 5. Loading Real Backend Data

(Planned Feature)
Future updates will allow loading `backend_properties.json` directly from IBM Quantum to populate the crosstalk matrix automatically.
