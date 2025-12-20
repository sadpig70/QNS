// Simon's Algorithm - 4 qubits (2 data + 2 ancilla)
// Finding hidden period s = 11
OPENQASM 2.0;
include "qelib1.inc";

qreg q[4];
creg c[2];

// Initialize
h q[0];
h q[1];

// Oracle for s = 11
cx q[0],q[2];
cx q[1],q[3];
cx q[0],q[3];
cx q[1],q[2];

// Hadamard on data qubits
h q[0];
h q[1];

measure q[0] -> c[0];
measure q[1] -> c[1];
