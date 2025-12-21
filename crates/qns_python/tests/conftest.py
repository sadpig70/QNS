import pytest
import os

@pytest.fixture(scope="module")
def fetcher():
    """Fixture to provide a CalibrationFetcher instance."""
    try:
        from qns.ibm import CalibrationFetcher
        # Use a mock or fake connection if possible, or real if token exists
        return CalibrationFetcher()
    except ImportError:
        pytest.skip("qns.ibm module not available")

@pytest.fixture(scope="module")
def backend_name():
    """Fixture to provide a backend name."""
    return "fake_manila"

@pytest.fixture(scope="module")
def calibration(fetcher):
    """Fixture to provide calibration data."""
    # Return mock calibration data
    return {
        "backend_name": "fake_manila",
        "num_qubits": 5,
        "t1": [100.0] * 5,
        "t2": [80.0] * 5,
        "gate_errors_1q": [0.001] * 5,
        "gate_errors_2q": {(0, 1): 0.01},
        "readout_errors": [0.01] * 5,
        "coupling_map": [[0, 1], [1, 2], [2, 3], [3, 4]]
    }
