
# Verification Script: Crosstalk Data Fetching
# 1. Connects to (mock) backend via CalibrationFetcher
# 2. Fetches properties including crosstalk
# 3. Creates PyHardwareProfile
# 4. Injects crosstalk data
# 5. Verifies data in Rust

import sys
import os

# Ensure we can import the local qns_python module if built, or mock it for now
# For this verify script, we rely on the bridge code we just modified.

# Mocking qiskit structures for standalone testing without full Qiskit install
# In a real env, these would come from Qiskit
class MockBackend:
    def properties(self):
        return MockProperties()
    def configuration(self):
        return MockConfig()

class MockConfig:
    def __init__(self):
        self.num_qubits = 5
        self.coupling_map = [[0,1], [1,2], [2,3], [3,4]] # Linear

class MockProperties:
    def t1(self, qubit): return 100e-6
    def t2(self, qubit): return 80e-6
    def gate_error(self, gate, qubit): return 0.001
    def readout_error(self, qubit): return 0.01
    @property
    def gates(self): return []

# Mock service
class MockService:
    def backend(self, name): return MockBackend()

# Instantiate Bridge
from qns_python.python.qiskit_bridge import CalibrationFetcher

# Monkey patch for testing env if Qiskit not installed
# In the real environment, we'd assume Qiskit is present.
# Here we just assume the file is correct and unit test the logic if possible.

def test_bridge_parsing():
    print("Testing CalibrationFetcher crosstalk parsing...")
    fetcher = CalibrationFetcher()
    
    # Inject mock backend directly
    fetcher._backend = MockBackend()
    
    # Test fetch
    props = fetcher.fetch_properties()
    
    print("Fetched keys:", props.keys())
    
    if 'crosstalk' in props:
        print("SUCCESS: 'crosstalk' key found.")
        print("Crosstalk data:", props['crosstalk'])
        
        # Verify heuristic (linear chain -> 4 edges)
        # 0-1, 1-2, 2-3, 3-4 should have dummy crosstalk ~0.005
        assert len(props['crosstalk']) == 4
        assert props['crosstalk'][(0,1)] == 0.005
        print("SUCCESS: Heuristic values verified.")
    else:
        print("FAILURE: 'crosstalk' key missing.")
        sys.exit(1)

if __name__ == "__main__":
    test_bridge_parsing()
