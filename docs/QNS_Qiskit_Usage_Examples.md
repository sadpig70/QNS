# QNS-Qiskit Integration - Usage Examples

## Quick Start

### 1. Installation

```bash
# Install Python dependencies
cd crates/qns_python
pip install -r requirements.txt

# Build QNS CLI
cd ../..
cargo build --release
```

### 2. Set IBM Quantum API Token (Optional)

```bash
export QISKIT_IBM_TOKEN='your_token_here'
```

## Example Circuits

### Bell State (2 qubits)

**File**: `examples/bell_state.qasm`

```qasm
OPENQASM 2.0;
include "qelib1.inc";

qreg q[2];
creg c[2];

h q[0];
cx q[0], q[1];

measure q[0] -> c[0];
measure q[1] -> c[1];
```

**Run with different backends**:

```bash
# QNS native simulator (fastest)
qns run examples/bell_state.qasm

# Qiskit Aer ideal
qns run examples/bell_state.qasm --backend aer-ideal --shots 1024

# Qiskit Aer noisy (mock calibration)
qns run examples/bell_state.qasm --backend aer-noisy --shots 2048

# Qiskit Aer with IBM backend calibration
qns run examples/bell_state.qasm --backend aer-ibm --ibm-backend ibm_fez
```

### GHZ State (3 qubits)

**File**: `examples/ghz_state.qasm`

```qasm
OPENQASM 2.0;
include "qelib1.inc";

qreg q[3];
creg c[3];

h q[0];
cx q[0], q[1];
cx q[0], q[2];

measure q -> c;
```

## Backend Comparison

### Ideal vs. Noisy Simulation

```bash
# Run ideal simulation
qns run bell_state.qasm --backend aer-ideal --shots 1024 --format json > ideal.json

# Run noisy simulation
qns run bell_state.qasm --backend aer-noisy --shots 1024 --format json > noisy.json

# Compare fidelity (Python)
python -c "
import json
ideal = json.load(open('ideal.json'))
noisy = json.load(open('noisy.json'))
print(f'Fidelity drop: {ideal[\"fidelity\"] - noisy[\"fidelity\"]:.4f}')
"
```

## Python API Usage

### Direct Qiskit Bridge

```python
import sys
sys.path.insert(0, 'crates/qns_python/python')

from qiskit_bridge import CircuitConverter, AerSimulationRunner

# Define circuit
qns_gates = [
    {'name': 'H', 'qubits': [0], 'params': []},
    {'name': 'CNOT', 'qubits': [0, 1], 'params': []},
]

# Convert and run
converter = CircuitConverter()
qc = converter.qns_to_qiskit(qns_gates, num_qubits=2)

runner = AerSimulationRunner()
counts = runner.run(qc, shots=1024)

print(f"Results: {counts}")
```

### IBM Calibration Fetching

```python
import os
os.environ['QISKIT_IBM_TOKEN'] = 'your_token'

from qiskit_bridge import CalibrationFetcher, NoiseModelBuilder

# Fetch calibration
fetcher = CalibrationFetcher()
fetcher.connect('ibm_fez')
calibration = fetcher.fetch_properties()

print(f"T1 mean: {sum(calibration['t1'])/len(calibration['t1']):.2e} s")
print(f"T2 mean: {sum(calibration['t2'])/len(calibration['t2']):.2e} s")

# Build noise model
builder = NoiseModelBuilder()
noise_model = builder.build_noise_model(calibration)

print(f"NoiseModel qubits: {len(noise_model.noise_qubits)}")
```

## Troubleshooting

### Error: "qns module not found"

```bash
# Install Rust module with maturin
cd crates/qns_python
maturin develop --release
```

### Error: "QISKIT_IBM_TOKEN not found"

```bash
# Set environment variable
export QISKIT_IBM_TOKEN='your_token_here'

# Or create .env file (gitignored)
echo "QISKIT_IBM_TOKEN=your_token_here" > .env
```

### Error: "Backend not available"

Try different IBM backends:

- `ibm_fez` (156 qubits, open plan)
- `ibm_kyoto` (127 qubits)
- `ibm_osaka` (127 qubits)

## Performance Tips

1. **Use JSON output for automation**:

   ```bash
   qns run circuit.qasm --backend aer-ideal --format json | jq '.fidelity'
   ```

2. **Reduce shots for quick tests**:

   ```bash
   qns run circuit.qasm --backend aer-noisy --shots 100
   ```

3. **Use aer-ideal for circuit validation**:

   ```bash
   qns run circuit.qasm --backend aer-ideal --shots 1024
   ```

4. **Reserve aer-ibm for final validation** (slower due to API calls):

   ```bash
   qns run circuit.qasm --backend aer-ibm --ibm-backend ibm_fez
   ```
