import sys
import os

# Ensure qns module is importable (assuming it's installed or in PYTHONPATH)
try:
    import qns
except ImportError:
    print("Error: qns module not found. Make sure it is installed or in PYTHONPATH.")
    # For dev environment, we might try to find the shared library, but usually
    # verification runs in an environment where it's installed.
    sys.exit(1)

def verify_binding():
    print(f"QNS Version: {qns.__version__}")
    
    # Check QnsOptimizer signature support
    try:
        # Initializing with crosstalk_weight
        optimizer = qns.QnsOptimizer(
            num_qubits=5,
            beam_width=10,
            max_iterations=20,
            crosstalk_weight=1.5
        )
        print("Successfully instantiated QnsOptimizer with crosstalk_weight.")
        print(f"Optimizer: {optimizer}")
    except TypeError as e:
        print(f"Failed to instantiate QnsOptimizer: {e}")
        sys.exit(1)
        
    # Check if we can run optimization (mock circuit)
    circuit = qns.Circuit(5)
    circuit.h(0)
    circuit.cnot(0, 1)
    
    try:
        result = optimizer.optimize(circuit)
        print("Optimization ran successfully.")
        print(f"Result: {result}")
    except Exception as e:
        print(f"Optimization failed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    verify_binding()
