OPENQASM 2.0;
include "qelib1.inc";

// 3-qubit GHZ state: (|000⟩ + |111⟩) / √2
// Tests: Entanglement, multi-qubit gates, CNOT chains

qreg q[3];
creg c[3];

// Create GHZ state
h q[0];
cx q[0], q[1];
cx q[1], q[2];

// Measure all
measure q[0] -> c[0];
measure q[1] -> c[1];
measure q[2] -> c[2];
